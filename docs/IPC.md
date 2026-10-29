# Bounded scalar IPC between tasks

Implemented October 28, 2026. IPC means inter-process communication; this first
implementation connects two fixed lower-privilege tasks, not arbitrary loaded
processes. Rust task A sends a request to B through a kernel-owned inbox. B reads
it, computes a response and sends it back. Their writable stacks remain private.

## Try and verify

```sh
sh scripts/console.sh
```

At the console:

```text
ipc.check
schedule.isolate
echo still running
```

A (ID 1) sends kind 1, value 41 to B (ID 2). B verifies the kernel-reported sender,
kind and value, computes value+1, and sends kind 2, value 42 back. A validates the
response. Both exit with code 0; a_value=42 and b_value=41 report their actual
received values. sent=2 and received=2 count the successful operations. This is a
transport/permission demonstration, not AI, performance optimization or useful
application workload. The host merely displays the guest's result.

## Identity, permission and inbox ownership

A message contains a kernel-assigned sender ID, an 8-bit nonzero kind, and one
u64 value. The send operation also supplies a destination ID. Values are opaque
scalars: even an integer that resembles an address is never dereferenced.
The kernel copies registers into its own mailbox storage and copies a received
message into the destination's saved registers. No shared writable user buffer,
user-memory pointer or raw memory-copy syscall is involved.

TaskId accepts IDs 1 through TASK_COUNT (currently 2). The runtime derives the
caller ID from the selected task record; there is no user-supplied sender field.
Receive always selects that caller's inbox, ignoring its input register values.
The linked clients deliberately put 999 in x3 before calls; it cannot spoof a
sender and is cleared on errors. The rejection fixture also requests receive
with x0=2 while A runs and B's inbox has data: A still receives only from A's inbox.

The kernel configures explicit routes: ID 1 may send to ID 2, and ID 2 to ID 1.
Self-send is denied. A future task is not automatically authorized to contact
all existing tasks. Routes are private kernel policy, not syscall input.

Each inbox holds two messages in FIFO order. Full returns an error before any
mutation; accepted messages are never silently overwritten. Empty is a normal
nonblocking result. Each handled send or receive yields a scheduler turn to the
next ready task, including errors. It does not reset the accumulated timer count.

Exit, user fault, timer limit or call limit closes that task's endpoint and clears
its pending incoming messages. Subsequent sends to it return Closed. Messages it
already sent to a live peer remain deliverable. Every command creates fresh
records/inboxes and destroys them on return. IDs are scoped to that batch; there
is no persistent task creation, reuse across live sessions or global PID registry.

## Syscall ABI for ipc.check only

All calls use SVC #0 at EL0, with x8 selecting the operation. Other saved registers
and the task's private stack/context are preserved across rescheduling.

| Operation | Inputs | Outputs |
| --- | --- | --- |
| 10: send | x0 destination, x1 kind, x2 value | x0 status; x1-x3 zero |
| 11: receive | None; x0-x3 input ignored | x0 status; on success x1 sender, x2 kind, x3 value; on error x1-x3 zero |
| 2: exit | x0 code 0..255, x1 result | Terminal, never returns |

| Status | Meaning |
| --- | --- |
| 0 | Success |
| 1 | InvalidTask: destination outside 1..2 |
| 2 | Denied: no configured route |
| 3 | Full: destination inbox already has two messages |
| 4 | Empty: caller's inbox has no messages |
| 5 | Closed: endpoint is no longer live |
| 6 | InvalidKind: kind outside 1..255 |

Send validates destination, liveness, route, kind and capacity in that order.
Failures leave queue contents unchanged. A valid task cannot receive another
task's messages by changing operands. An unknown syscall or invalid exit code
retains the runtime's terminal bad_syscall policy. The legacy user.run ABI and
the exit-only schedule.run/fault modes remain unchanged.

This first version polls on Empty. Blocking receive, a waiting state, wakeups,
request timeouts and peer-death notifications are not implemented. Each task has
a fixed limit of sixteen attempted send/receive calls: attempts 1-15 are handled;
attempt 16 terminates the task before servicing it, with state limit_reached.
The trace marks that terminal attempt L, and calls includes it; message/error
counters do not. The sixth timer preemption also remains terminal. This prevents
an empty-receive or invalid-send loop from keeping the command alive indefinitely
when the hardware timer works. These are diagnostic bounds, not a permanent
application lifetime policy or a hard real-time guarantee.

## Structure and safety

- src/ipc.rs: pure checked IDs, explicit routes, two-slot FIFO storage, validation
  and endpoint closure. No unsafe, hardware registers or UART dependency.
- src/schedule/runtime.rs: owns fresh inboxes for the batch, binds caller identity,
  interprets scalar call operands, writes only x0-x3 results, counts calls and
  closes endpoints on every terminal state.
- src/schedule/lifecycle.rs: syscall yield returns Running to Ready; CallLimit is
  terminal. Neither event grants more timer preemptions.
- src/schedule/ipc_programs.rs: linked Rust clients and negative fixtures. Small
  inline assembly boundaries declare syscall registers. Scalar comparisons stay
  within user text; there is no user Rust runtime/standard-library loader.
- Existing exception capture and MMU selection return to masked EL1 with the base
  root before mailbox handling. Kernel-only arrays cannot be accessed by EL0.

The shared runtime reserves 46 trace slots for the bounded returns of two tasks.
Counters fit in u8 because at most 30 calls are serviced per batch. Storage is on
the kernel stack; no heap, allocation syscall or physical-page mapping is added.
The pool remains at twelve table pages and 244 free pages. One CPU, soft-float
integer context and exclusive user-RAM ownership remain required. Kernel modules
share privilege; scalar IPC is not proof of full service isolation or robustness
against every hostile program, kernel bug or hardware failure.

## Growing beyond A and B

The public message concept uses numeric task IDs, not special A-to-B operations.
The current implementation intentionally fixes two entries, stacks, MMU roots
and route rows. Adding C/D/E requires coordinated changes to task ownership,
private memory/maps, scheduler selection, route policy, storage and limits, plus
new tests. Merely increasing TASK_COUNT would not make the system support them.
The next small foundation is waiting/wakeup so a receiver can stop polling.
Dynamic task creation, reusable identities/capabilities and a service registry
come later; there is no generic agent framework in this increment.

## Actual tests

```sh
python3 scripts/verify_ipc.py
python3 scripts/verify_ipc.py --release
```

Both profiles passed three pure mailbox tests and four lifecycle tests. FIFO
order, ring wrap, full rejection, invalid IDs/kinds, denied routes, close/discard,
already-sent delivery after sender exit and fresh inboxes are checked directly.

Each profile also runs five real QEMU images: normal request/reply, invalid/full
requests, empty-call flood, peer exit and peer fault. The normal image runs four
IPC batches (including a shell invocation); each other image runs three, totaling
sixteen IPC batches per profile. The host independently replays ready-task
selection, terminal skipping, syscall/timer bounds and counters; it checks task
exit codes and received values. It verifies exact fault PC/address/syndrome in
the peer-fault image. Full/denied/closed/error results and zeroed unused return
registers are checked by the real Rust tasks. The sixteenth attempt does not
produce another successful receive/send or returned error.

Afterward, real scheduling, private-memory access checks, legacy user calls,
memory diagnostics and echo must still work; MMU snapshots must match. Repetition
checks that no prior message or endpoint state survives. Parser extra arguments
are rejected. Regression suite details are in [WORKLOG.md](WORKLOG.md).

Captured output: [debug](evidence/2026-10-28-ipc-debug.txt) and
[release](evidence/2026-10-28-ipc-release.txt). These are real QEMU observations,
not evidence of mobile hardware support, AI integration or efficiency gains.
