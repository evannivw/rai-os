# From a booting kernel to guest services

Status: October 30, 2026. Each step should produce a small, independently
observable result before the next. The kernel supplies mechanisms and enforces
access; a future coordinator chooses policy above it. Kernel Rust modules are
not isolated processes. Fixed linked programs now run with lower privilege. The existing coordinator still runs on the host.

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
| One bounded guest service | Done for immutable record token counts | Host request -> EL0 client/worker -> checked result, with private stacks and scalar IPC. See [RECORD_SERVICE.md](RECORD_SERVICE.md). Persistent lifetime remains separate. |
| One persistent guest worker | Done for one fixed EL0 service | Retains private stack/register state between host requests, with exclusive RAM ownership and explicit stop/failure cleanup. See [PERSISTENT_WORKER.md](PERSISTENT_WORKER.md). |
| Guest coordinator and executor | Initial EL0 pair implemented | Retains a goal, rejects stale/manual proposals, dispatches by IPC and remembers the result. Real Meta transport is available through --guest; admission stays at EL0. See [GUEST_COORDINATOR.md](GUEST_COORDINATOR.md). |
| Meta-driven policy | First single proposal verified on the host | Muse Spark 1.3 selects a permitted operation; host/guest validation precedes execution. A bounded two-revision event loop now retains observations and rejects stale/duplicate work. See [META_COORDINATOR.md](META_COORDINATOR.md). |

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

The current implementation uses the waiting/wakeup foundation for a closed,
bounded two-task record-count service. The next integration should stay equally small.
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

## Complete project view — October 29 checkpoint

This groups completed work for learning; the work log retains actual dates and
individual increments. Done means the stated bounded behavior, not general OS support.

| Order | Delivered piece | What it contributes |
| --- | --- | --- |
| 1 | Repository, Rust/Cargo, pinned target and QEMU setup | Repeatable build and preserved original history. |
| 2 | Own no_std/no_main Rust kernel and assembly entry | Boot on the emulated AArch64 board. |
| 3 | UART, shell and versioned structured protocol | Human and host requests, bounded parsing and real responses. |
| 4 | Rules/provenance, demo contract and immutable fixtures | Submission constraints, desired outputs and reference data; full demo remains planned. |
| 5 | Clock and host terminal | Measured guest uptime and a host-to-guest control path. |
| 6 | Graphical host Device Lab | Visible real guest results; not a guest graphical desktop. |
| 7 | Cooperative word-count job and cancellation | First observable bounded work in kernel context. |
| 8 | Host coordinator, then high/normal/low priorities | Sequence jobs according to observed completion. |
| 9 | Exception diagnostics | Identify controlled execution faults. |
| 10 | Physical-page ownership | Reserve, zero, test and release memory safely under current assumptions. |
| 11 | MMU tables and access permissions | Hardware-enforced executable/read-only/writable boundaries. |
| 12 | Saved exception context | Resume register/stack state after a controlled exception. |
| 13 | EL0 program and checked syscalls | Less-privileged code requests kernel operations. |
| 14 | Timer interrupts and invocation deadline | Regain control even if a user program does not cooperate. |
| 15 | Two resumable tasks and bounded lifecycle | Share one CPU; keep the peer progressing after completion/fault/limit. |
| 16 | Private writable stacks | Deny each task access to its peer's stack. |
| 17 | Bounded scalar IPC | Authorized task-to-task request/reply with FIFO inboxes. |
| 18 | Blocking receive and wakeup | Waiting tasks receive no CPU turns until completion/closure. |
| 19 | Bounded record service | Host chooses a record; an EL0 worker scans it and returns an independently checked count. |
| 20 | First real Meta proposal | Host bridge validates the model choice and executes the guest service; a live unsupported goal is declined. |
| 21 | Bounded event-driven coordinator | Retain results, react to released revisions, ignore duplicate events and discard stale proposals. Runs on the host. |

Remaining delivery path: extend the bounded revision coordinator into a useful
multi-step workload with explicit constraints; independent demo artifact verification;
fair baseline/comparison including model overhead; presentation, reproducibility,
three-minute video and submission. See the revised [schedule](HACKATHON_PLAN.md).
Do not continue expanding kernel mechanisms without a concrete demo dependency.

## October 30 kernel consolidation

At the user's request, delivered step 22: a persistent EL0 worker, independent
of further model experiments. The same linked program waits/resumes over multiple
requests, and every EL0 entry now respects an exclusive fixed-RAM lease. This
provides a lifetime boundary for a later guest service/coordinator; it does not
move the current host coordinator or complete a general kernel. Keep the remaining
demo/verifier/baseline/video work visible when deciding the next increment.

## October 30 guest coordinator boundary

Step 23 now runs coordinator A and worker B as retained EL0 programs. A owns the
current goal and proposal admission policy; the kernel supplies transport,
private context switching, limits and cleanup. This first path uses manual typed
proposals. Next connect the existing host Meta provider to the guest's pending
goal/ticket, without duplicating goal authority in the host. Full workload,
independent demo artifact verification and comparative evidence remain pending.

## October 30 guest Meta connection

Step 24 closes the integration described in step 23: the --guest bridge requests
Meta proposals from actual guest reports and sends the original ticket back for
EL0 admission. It transports observations while inference is pending; stale
responses are rejected inside RAI-OS. The kernel/ABI are unchanged. See
[GUEST_META_BRIDGE.md](GUEST_META_BRIDGE.md). The next delivery slice is a useful
artifact plus independent checker, following [DEMO_PLAN.md](DEMO_PLAN.md).


## October 30: useful artifact and native graphics scope

The first [exact record index](RECORD_INDEX.md) is now produced by a bounded EL0
program and independently checked against fixture bytes. It is one immutable
record per synchronous invocation, not a full review pack or Meta-selected job.
The user requested a native guest screen next; [GUEST_GRAPHICS_PLAN.md](GUEST_GRAPHICS_PLAN.md)
supersedes earlier blanket GUI exclusions with two small gates: actual guest
pixels, then observed index/error rendering. No finished kernel is required to
begin that work. Native graphics remain unimplemented. Return afterward to
retained jobs, multi-record/revision-aware results and fair comparison; the
[delivery plan](DEMO_PLAN.md) and submission deadline remain in force.


## October 30: native first screen delivered

The [native guest display](GUEST_DISPLAY.md) now draws actual RAI-OS pixels and
observed index/error results in QEMU, with optional protected framebuffer/device
mappings and verified failure containment. The first visual slice is complete;
freeze its appearance and return to useful guest-service execution. Next migrate
the exact index into a retained coordinator job with bounded output transfer,
then combine revision-aware results and compare policies fairly. No touch/window
system, phone support or AI performance advantage is implied by the new screen.

## October 30: retained index delivered

The exact content index now runs as a retained EL0 coordinator/worker job, with
bounded scalar output transfer, six immutable revision slots, explicit reuse
without B execution and native observed-state rendering. See
[RETAINED_INDEX.md](RETAINED_INDEX.md). Next connect the existing Meta transport
to this mode, then build combined revision-aware outputs and compare fairly.

## October 30: Meta index proposal path

The [Meta index bridge](META_INDEX_BRIDGE.md) now connects the model to retained
EL0 indexes, returns independently checkable artifacts and observes guest reuse
without another inference request. It optionally drives the native QEMU screen.
This remains a single-action integration, not an AI-performance comparison.
Next build a combined revision-aware artifact with explicit completeness and
invalidation rules, then evaluate useful model choices against a fair baseline.
The final video/submission and original event dates/deadline remain unchanged.

## October 30: complete index-set foundation

The [index-set increment](INDEX_SETS.md) now binds up to four required revisions
in the guest and publishes their exact source bytes/indexes only when complete.
The three reference requirements need 3, then 1, then 1 new worker jobs; the native
view exposes missing/current entries. The Rust walkthrough and independent checker
make this reproducible without model calls. Next bind Meta planning to the full
set generation and missing list; full review-pack event/budget semantics, fair
comparison, video and submission remain pending. Original dates are unchanged.


## October 30: whole-set model proposals

The [whole-set Meta bridge](META_SET_BRIDGE.md) now exposes the current complete
requirement and guest-reported missing list to the model. The explicit set mode
binds proposals to a goal generation; `coord.select` rejects obsolete authority
before issuing a per-record ticket. Existing EL0 admission/execution, exact cache
and complete-set publication remain the guest's responsibility. This permits
choices of missing-job order, not a demonstrated AI performance advantage.
Next establish a useful constrained choice and fair comparison; full review-pack
budget/release semantics, final video and submission remain pending. Original
event dates and the November 4 deadline are unchanged.
