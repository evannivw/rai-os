# A deadline for one EL0 invocation

Implemented October 28, 2026. The kernel can now terminate a built-in EL0 program
that never calls back or exits. Each invocation gets a fixed 100 ms guest-counter
deadline. This is one CPU and one invocation at a time, with no suspended-task
queue, context switching between programs, separate process address spaces or SMP.

## Try it

From the repository root:

```sh
. "$HOME/.cargo/env"
sh scripts/console.sh
```

Enter these commands separately:

```text
user.spin
user.run
timer.check
echo still running
```

`user.spin` selects a linked assembly program containing only a branch to itself.
There is no syscall, polling check or voluntary return in that loop. The kernel
receives the timer IRQ and returns `state=timed_out`, `budget_ms=100`, measured
`elapsed_ms`, vector 9, and the actual interrupted PC/status/SP. `user.run` still
selects the Rust hello program and normally returns `state=exited; code=0`.
Both commands reject arguments and arbitrary entry addresses/binaries.

An outer UART `ok` means that a report is available; it does not mean the program
completed its work. Timeout and fault are distinct terminal outcomes. Buffered
user output is not published on timeout. Normal exit/fault payload formats remain
unchanged, except saved EL0 state now reflects IRQ being enabled.

## What changed

- `timer/budget.rs` calculates a checked absolute deadline from the counter and
  frequency. Fractional ticks round up; zero frequency and deadline overflow are
  errors. `timer/hardware.rs` programs CNTP_CVAL_EL0 once per invocation. Syscalls
  do not reset or extend it. Timer ownership stays with the existing GICv2 adapter.
- `user/mod.rs` clears the user stack, obtains a budget, creates a fully initialized
  stack-owned Session and publishes its pointer. The kernel selects Hello or Spin.
  The assembly entry saves its private EL1 caller, sets the selected user PC/SP,
  clears registers and enters EL0t with saved status 0x340: IRQ enabled, D/A/F masked.
- Vector 9 saves the user integer frame, including SP_EL0, on the private kernel
  stack and enters the dedicated exception stack with interrupts masked. CNTP's
  level is deasserted before EOI. A valid active session, expected EL0 status and
  an expired budget permit termination through the fixed kernel trampoline.
  The expired instruction is never resumed. ESR/FAR are not interpreted for IRQs.
- Synchronous entry (vector 8) checks that same absolute deadline before dispatching
  a syscall or reporting a fault. An elapsed deadline takes precedence. Repeated
  unsupported syscalls therefore cannot obtain a new time slice each time. The
  expired call is neither executed nor counted. Otherwise existing bounded syscall
  and fault rules apply. All kernel handlers stay IRQ-masked and non-reentrant.
- Normal exit, synchronous fault and timeout return to the same cleanup: clear the
  session pointer, stop CNTP/GIC delivery and verify actual timer/GIC enables,
  active state and DAIF. Failure returns `timer_cleanup_failed`, not a success
  report. A fatal kernel exception still halts rather than unwinding/cleaning up.
- MMU activation explicitly clears SCTLR_EL1.UMA. Timer initialization sets
  CNTKCTL_EL1 to zero and checks readback. EL0 cannot mask IRQs or modify physical
  timer registers; GIC pages remain EL1-only. Register-access attempts are contained
  synchronous user faults. Timer/IRQ access is not granted by an `_EL0` register name.

## Ownership and timing limits

One EL1 CPU and masked kernel execution are required. No other caller can publish
a Session between the active check and store. The Session is fully initialized
before publication; its caller does not touch it while user execution/dispatch is
active. Only non-nested vector 8/9 dispatch borrows it during that interval. The
pointer is cleared before moving the report or returning. AtomicPtr does not
make the design safe for SMP, parallel handlers or arbitrary asynchronous Rust.

The hardware timer is armed while EL1 IRQs remain masked; only ERET enables IRQs
at EL0. Every termination reaches the kernel caller with all DAIF masks set.
Kernel code does not call UART, allocate or enter other services from a successful
IRQ handler. User SP is recorded, never used as a kernel return stack. Saved user
PC/SP are diagnostic data, not chosen kernel continuation addresses.

The 100 ms value is elapsed time in the QEMU counter domain, starting just before
timer programming and including entry setup and syscall handling. It is not exact
CPU accounting, host wall-clock timing or a hard real-time guarantee. IRQ delivery
and finite handler work can add delay. A bounded syscall started before expiry
can finish after it; the next EL0 IRQ/synchronous entry observes expiry. A stuck
kernel handler or failed timer delivery can still hang the guest. The verifier's
host timeout detects that failure and reaps QEMU; it is not guest recovery.

The console and cooperative word-count job do not advance while this invocation
runs. Timeout terminates it rather than saving it for later. Memory permissions
are unchanged: six table pages, 250 pool pages free. No efficiency, battery or
multi-service isolation claim follows. Single-core scheduling is the next area;
see [ROADMAP.md](ROADMAP.md). Host coordinator and GUI placement are unchanged.

## Executed verification

Rust 1.90.0 and QEMU 11.1.2, macOS ARM64, debug and release:

```sh
python3 scripts/verify_user_timeout.py
python3 scripts/verify_user_timeout.py --release
python3 scripts/verify_user.py
python3 scripts/verify_user.py --release
```

The deadline verifier checks the exact ELF instruction/size for the infinite
loop and the interrupted PC, vector, saved EL0 status and stack. The normal image
runs four spin timeouts, interleaving successful hello calls and timer/context
checks. It verifies repeat use, unchanged mappings/page ownership, malformed
arguments and console continuity after waiting past the old deadline.

Two separate test-only images extend coverage: `user-test-syscall-loop` repeatedly
issues unsupported calls and must expire with a nonzero call count, via either
vector 8 or 9. `user-test-expired-syscall` deliberately holds the first bounded
kernel handler until expiry to exercise vector 8 deterministically; that delay
is absent from normal builds. Each runs twice, followed by a direct-shell spin
and continuity checks. A pure budget test covers exact expiry, rounding and overflow.

The user suite runs ten images twice each: hello, eight invalid calls, and eight
hardware fault probes. New probes attempt DAIFSet and a CNTP_CTL_EL0 write; both
trap with exception class 0x18 at the exact ELF instruction and preserve kernel
usability. Every image also runs timer/context/memory checks after user termination.
The normal image additionally runs hello through the direct shell.

Captured output: [deadline debug](evidence/2026-10-28-user-timeout-debug.txt),
[deadline release](evidence/2026-10-28-user-timeout-release.txt),
[access-guard debug](evidence/2026-10-28-user-timeout-guards-debug.txt),
[access-guard release](evidence/2026-10-28-user-timeout-guards-release.txt).
These are observations, not promised latencies. Earlier evidence remains intact.
Broader checks are recorded in [WORKLOG.md](WORKLOG.md).

Not injected: spurious or unexpected lower-EL IRQs, broken timer delivery, cleanup
readback failure, counter reset/wrap, corrupted/nested kernel stacks, SIMD/FPU,
multiple cores, other boards or physical phones. Deadlines are fixed rather than
user/model configurable; the Meta integration is still pending.

## References

- [Arm Generic Timer](https://documentation-service.arm.com/static/633d612a4c59b30b51771183): absolute compare registers and timer delivery.
- [Arm architecture registers](https://documentation-service.arm.com/static/60df1943677cf7536a55ddef): CNTKCTL_EL1, SCTLR_EL1 and exception syndromes.
- [Arm AArch64 exception model](https://documentation-service.arm.com/static/63a065c41d698c4dc521cb1c): lower-EL IRQ entry and saved execution state.
