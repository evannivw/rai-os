# From a booting kernel to guest services

Status: October 28, 2026. Each step should produce a small, independently
observable result before the next. The kernel supplies mechanisms and enforces
access; a future coordinator chooses policy above it. Kernel Rust modules are
not isolated processes. One built-in program now runs with lower privilege. The existing coordinator still runs on the host.

| Step | Status | What it enables / how to observe it |
| --- | --- | --- |
| Boot, UART, bounded commands | Done | QEMU boots our Rust kernel and returns real results. |
| Clock and cooperative job | Done | Measured uptime, word-count progress and cancellation. |
| Fatal exception diagnostics | Done | A deliberate fault reports its cause and stops safely within the tested conditions. |
| Physical-page ownership | Done | Allocate, zero, test and release a reserved pool without overlapping kernel regions. |
| Translation tables and permissions | Done | MMU rejects forbidden writes, instruction fetches and unmapped reads. |
| Resumable EL1 exception context | Done for one controlled SVC | Save/restore integer state and resume; unrecognized synchronous events remain fatal. See [CONTEXT.md](CONTEXT.md). |
| One lower-privilege program and bounded syscall | Done for one built-in program | Rust at EL0 writes through a checked syscall and exits; its synchronous faults are contained. See [USER_PROGRAM.md](USER_PROGRAM.md). |
| Controlled EL1 timer interrupt | Done | GICv2/CNTP delivers six IRQs per check; integer context survives. See [TIMER.md](TIMER.md). |
| Timer while running EL0 | Done for one invocation | A fixed elapsed deadline terminates a non-cooperative program and returns to the console. See [USER_TIMEOUT.md](USER_TIMEOUT.md). |
| Two resumable contexts | Done as a bounded diagnostic | Alternate two EL0 loops with distinct saved state and stack storage. See [SCHEDULING.md](SCHEDULING.md). |
| Task completion and fault lifecycle | Done for two bounded tasks, one CPU | A task exits, faults or reaches its slice cap; its ready peer continues. See [TASK_LIFECYCLE.md](TASK_LIFECYCLE.md). |
| Separate task memory | Done for two fixed stack pages | A/B task maps deny peer-stack reads/writes; shared code remains read-only. See [TASK_MEMORY.md](TASK_MEMORY.md). |
| Bounded task messages | Done for two fixed tasks | Checked IDs/routes, two-slot FIFO inboxes, request/reply, closed/full/empty errors and a call cap. See [IPC.md](IPC.md). |
| Waiting and wakeup | Done for the two-task batch | A receiver stops getting CPU turns and resumes with a message or Closed; mutual waiting is reported as deadlocked. See [IPC_WAIT.md](IPC_WAIT.md). |
| One bounded guest service | Next small step | Define a small request/result contract and a worker that waits, validates, executes and replies; add external event delivery and persistent lifetime as separate increments. |
| Guest coordinator and executors | Planned | Move decision/execution services into RAI-OS with enforced permissions. Adapt host std/storage assumptions. |
| Meta-driven policy | Planned | Real model proposals go through validated operations and produce observed outcomes. |

This is a dependency roadmap, not a promise to finish every row before the
hackathon. EL0/scheduling/IPC require careful validation; a working demo can
integrate Meta through the host bridge before all service infrastructure exists.
The API credentials and HTTPS stay on the host for the initial demo. Revisit
scope against [HACKATHON_PLAN.md](HACKATHON_PLAN.md) before expanding a phase.

## Why this order

This is a dependency sequence for the RAI-OS prototype, not a universal OS recipe
or a promise to implement every possible kernel subsystem before the demo.
The immediate target is a small guest service that can receive a request, perform
an allowed operation and return a result. The longer-term coordinator can build
on that service boundary instead of owning hardware or arbitrary memory access.

| Foundation | Why it comes before the next step |
| --- | --- |
| Boot, UART and exception reports | Make real guest behavior observable and failures diagnosable. |
| Memory ownership and permissions | Give kernel/user data explicit boundaries before running less-trusted code. |
| User entry, saved context and timer | Let code run at lower privilege and let the kernel regain control. |
| Scheduling and task lifecycle | Keep several computations progressing and retire finished/failed tasks. |
| Private task memory | Prevent a task from reading or overwriting its peer's writable data. |
| Validated messages | Let protected tasks cooperate without exposing each other's memory. |
| Waiting and wakeup | Let a receiver retain its state without repeatedly asking for work. |
| A small guest service | Turn these mechanisms into a request/operation/result behavior with explicit limits. |
| Coordinator and Meta policy | Choose among allowed operations; keep enforcement and observed results in deterministic code. |

The current implementation has completed the waiting/wakeup foundation for a
closed, bounded two-task batch. The next service step should stay equally small.
Long-lived sessions need external event delivery, cancellation/timeouts and
explicit lifetime rules; C/D/E need additional records, maps and permissions.
Neither is obtained simply by renaming a module. A native GUI, phone drivers,
network stack, multicore scheduling and a general application loader are separate
projects, not prerequisites for proving this first request/result demo.

Each increment follows the same loop: explain the purpose, implement the smallest
observable behavior, verify it in real QEMU (including relevant failures), record
limits/evidence, then commit and push. The work log records what actually happened.
The hackathon plan and demo contract remain the delivery checkpoints; this roadmap
must not become an excuse to postpone a useful workload or real Meta integration.

## What runs where today

- QEMU emulates the machine. Our own kernel runs inside it; this is not a Linux
  kernel or an Android derivative. It currently targets this virtual board only.
- The terminal and Device Lab panel run on the Mac. They send UART requests and
  display guest results. Neither is necessary for direct guest UART operation.
- The host coordinator chooses the next bounded word-count job. It is a temporary
  development service, not a guest process and not connected to Meta yet.
- A guest-native graphical shell, physical-phone drivers, Android app compatibility,
  storage, networking and measured performance gains remain outside this increment.

## How this reaches the demo

Keep the kernel route and demo route distinguishable. Once the needed execution
operations exist, resume the [demo result contract](DEMO_ACCEPTANCE.md), implement
its independent verifier, establish the deterministic baseline, integrate an
actual eligible Meta model, and compare the same workload and constraints.
Record model/network overhead and unsuccessful outcomes as well as successes.
The visual panel can show that evidence; it does not manufacture it or establish
that AI is superior. Retain dates and eligible contributions, then prepare the
three-minute video and reproducible checkout instructions.
