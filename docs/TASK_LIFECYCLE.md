# Kernel task records and bounded lifecycle

Implemented October 28, 2026. Two built-in EL0 tasks now have explicit states.
A task can finish or fault without terminating its ready peer. This is a small,
fixed-capacity runtime on one CPU, not a general process/service system.

## Where the kernel remembers A and B

`src/schedule/runtime.rs` creates a two-element Record array on the private
kernel stack. Slot 0 identifies A, slot 1 identifies B. Each record contains:

| Field | Purpose |
| --- | --- |
| Frame (288 bytes) | x0-x30, saved PC, status, user SP and exception diagnostics. |
| Stack bounds | Assigned lower/upper addresses inside the existing user RAM area. |
| Lifecycle | Ready/running/terminal state and count of timer preemptions. |
| Reason, exit code, value | Small outcome metadata. |

This bookkeeping consumes RAM in addition to program code and data. The programs'
code and stacks already reside in RAM; switching tasks does not copy a whole
program out of the CPU. It saves the CPU register state in the selected record
and later restores that state, including the instruction address and stack pointer.

The kernel knows the array through its ordinary local variables and stack frame.
EL0 uses SP_EL0, while exception handling and the fixed return path preserve the
separate kernel stack/continuation. There is no infinite chain of lookup tables:
the kernel itself has code, a known stack and memory for its bookkeeping.

The array exists for the command's duration and cannot be accessed by EL0 under
the current mappings. A fresh command creates fresh records. There is no heap,
persistent task registry or dynamic task ID. Results copy only compact outcome
fields out of the records; full saved registers are not carried through the
command reply enum.

## Try it

```sh
cargo run --locked
```

At the UART prompt:

```text
schedule.run
schedule.fault
echo still running
```

The first command runs finite Rust calculations and returns two exited states.
A computes 1,024 wrapping multiply/add iterations starting at 7; B computes
4,096 starting at 11. Their returned u64 values are 14091860730339196423 and
14607574309510313995. These are test results, not performance measurements.
The default tasks are short enough to finish before a first timer tick in the
captured runs; normal completion alone does not demonstrate preemption.

The second command replaces A with an explicit read at 0x40080000. The MMU
rejects that access from EL0; A becomes faulted, then B returns its normal result.
The console remains usable. An outer protocol ok means the batch completed its
handling, not that every task succeeded.

## States and selection

| Current state/event | Next state |
| --- | --- |
| Ready selected | Running |
| Running, timer preemption 1 through 5 | Ready |
| Running, sixth timer preemption | LimitReached |
| Running, valid exit syscall | Exited |
| Running, user exception or invalid syscall/context | Faulted |

Only Ready records are selected. The cursor first considers the other slot, then
the current one if its peer is terminal. Exited, Faulted and LimitReached records
never restart within the batch. The command finishes when no ready record remains.
`src/schedule/lifecycle.rs` holds these pure rules; it knows no UART, pointers,
registers or timer devices.

Each turn uses a fresh 20 ms physical-counter deadline. At most six preemptions
per task keep even deliberate infinite-loop fixtures bounded when timer delivery
works. There are at most twelve recorded dispatch returns. This is a temporary
diagnostic cap, not an application lifetime design. The earlier user.run/user.spin
100 ms invocation deadline remains separate. These bounds are not hard real-time
guarantees, CPU-time accounting or energy measurements.

Reports encode A/B plus P (preempted), E (exited), F (faulted), L (limit reached).
For example, AE,BE is normal short completion; AF,BE is the deliberate fault.
With the spin test feature, B exits while A continues receiving turns until AL.
The previous schedule.check still verifies twelve strict alternating IRQ returns
and register/stack integrity independently of this task runtime.

## Limited lifecycle syscall ABI

Only SVC #0 with x8=2 is accepted. x0 is an exit code from 0 through 255; x1 is
an opaque numeric result. The kernel copies scalar values and never dereferences
x1. Unknown syscall numbers or invalid exit codes terminate that task with
bad_syscall while its peer continues. There is no write, wait, IPC or allocation
syscall in this runtime. The legacy user.run write/exit ABI is unchanged.

The kernel does not prove that returned numeric results are semantically correct;
the host verifier independently computes expected results. Public commands accept
no executable bytes, user-supplied context, stack address or task entry point.

## Ownership and unsafe contracts

- One CPU; kernel Rust runs with IRQ masked. Legacy user execution and scheduling
  cannot own the user region concurrently.
- Each stored frame is kernel-owned. Before entry, the runtime checks executable
  user PC range/alignment, EL0 mode and interrupt masks, and assigned stack bounds
  with 16-byte alignment. An invalid stored frame aborts the batch before ERET.
- The shared run_turn adapter publishes a separate initialized Observation for
  exception capture. The handler does not mutate the caller's Record through an
  alias. It returns to the fixed kernel continuation, then the runtime copies the
  captured state into the selected record.
- Timer setup/acknowledgment/cleanup uses the existing exclusive adapter. Every
  completed turn stops the timer and restores the masked kernel state before
  interpreting the event. Timer or internal errors abort the whole batch.
- The linker-reserved 8 KiB user stack is zeroed before initial entry and split
  into two 4 KiB halves. Kernel references are not exposed to user code.
- Rust task arithmetic wraps explicitly; inline helpers remain in user text.
  The small exit assembly declares its input registers and never returns.
  The fault fixture uses assembly for an intentional forbidden access rather
  than constructing an invalid Rust reference.

The later [private-stack increment](TASK_MEMORY.md) now selects a distinct task
root per turn. Each task can access only its own writable stack page at EL0;
code/constants remain shared read-only. Checking SP alone would not supply this
protection: it is enforced by page permissions and translation-cache maintenance.
No FPU/SIMD state, SMP, dynamic loader, persistent task creation, priorities, IPC,
agent execution or Meta integration is added. The UART command blocks until its
batch completes. Broken timer delivery and corrupted kernel stacks remain outside
the demonstrated guarantees. Direct cross-stack reads/writes are now tested by
schedule.isolate; this is not a general malicious-program security proof.

## Verification

```sh
python3 scripts/verify_lifecycle.py
python3 scripts/verify_lifecycle.py --release
```

Both profiles passed in actual QEMU. Each runs three pure lifecycle tests and
five images: normal, one spinning task, two spinning tasks, invalid exit code
256, and unknown syscall 99. Test features are mutually exclusive and use
separate target directories.

The verifier independently replays each returned trace, checks which task should
run next, terminal-task skipping and the exact six-preemption boundary. It checks
both Rust results against Python calculations, plus the forbidden read's actual
ESR (0x9200000f), PC at the ELF fault instruction, FAR (0x40080000), saved status
and stack bounds. Repeated invocations verify cleanup and fresh state. There are
22 lifecycle invocations per profile across all five images, including the two
normal shell commands. Invalid arguments are rejected.

Each image also checks subsequent legacy user, context, timer, schedule.check,
MMU, memory-pool and echo operations. Regression suites for those foundations,
cooperative tasks, the terminal console and host panel also passed; details are
in [WORKLOG.md](WORKLOG.md).

Actual captured output: [debug](evidence/2026-10-28-lifecycle-debug.txt) and
[release](evidence/2026-10-28-lifecycle-release.txt). Timing-dependent traces are
observations, not fixed expected output or comparative efficiency evidence.
