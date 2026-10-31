# RAI-OS hackathon plan

Updated October 29, 2026. Local times: America/Guatemala. This is a proposed
roadmap; completed foundations are identified below. One real Meta-to-guest operation is verified; a bounded revision-event coordinator is also verified. General autonomous OS coordination and a general guest service runtime remain pending.

## Sources and eligibility

The user supplied the Participant Guide in chat on October 26. Its source link
and version are still missing. Public pages consulted on the same date:

- https://dev.meta.ai/events/global-hackathon
- https://cerebralvalley.ai/e/meta-ai-developer-hackathon

The guide lists kickoff on Oct 26 and building on Oct 27-Nov 4. Public pages say
the event starts on Oct 26, with workshops from Oct 27. Clarify eligibility of
Oct 26 work before presenting it as eligible. Deadline: Nov 4 at 21:00 ET/EST,
or 20:00 Guatemala. Internal submission target: 18:00 Guatemala. Recheck updates.

The guide explicitly permits startups to build on existing products while
identifying new contributions. Permission for prior Indie work is not clear.
Proposed organizer question (not sent): Can an Indie entrant use an explicitly
declared Oct 13 repository and Oct 26 kernel, and showcase only the agent
features built Oct 27-Nov 4? Is kickoff-day work eligible? Save the answer.
Never backdate commits or recreate work to disguise its origin.

## Current direction

Proposed theme: Push the Frontier. Proposed track: Indie Developer for a solo
entrant; confirm the actual registration, acceptance, and team composition.
Requesting Meta API access is not proof of hackathon acceptance.

RAI-OS explores coordinating jobs around goals, events, capabilities, and resource
budgets. A coordinator proposes plans; deterministic code enforces operations,
permissions, and limits. Expensive model reasoning is not on every CPU operation.
Adaptation and commercial usefulness are hypotheses to test, including AI overhead.
There is no automatic claim of fewer layers, better security, or higher efficiency.

The earlier status/self-test demo was exploratory. The agreed direction is a
side-by-side comparison of two policies using the same RAI-OS execution engine,
workloads, inputs, and limits: a reasonable deterministic baseline and a Meta
coordinator. Define a concrete task and success conditions before implementing it.
The candidate comparison is specified in [DEMO_EXPERIMENT.md](DEMO_EXPERIMENT.md).
The first correctness dataset and verifier implementation sequence are in
[DEMO_FIXTURE.md](DEMO_FIXTURE.md); candidate verification is still pending.
The [graphical host panel](PANEL.md) displays actual kernel results; agent
decisions remain future work. A [deterministic host coordinator](COORDINATOR.md)
now sequences word-count jobs by priority, using observed completion. It shares the transport used by the
[terminal console](CONSOLE.md).

Current priority, revised at the user's request on October 26: advance the kernel
foundation and visible controls before continuing the demo verifier. The kernel
now exposes measured uptime and [fatal EL1 diagnostics](EXCEPTIONS.md). Exception
entry and a [bounded physical-page allocator](MEMORY.md) are implemented foundations
for future guest services. [EL1 page tables and permissions](MMU.md) are now active;
A [controlled EL1 context return](CONTEXT.md) is now verified.
One [built-in Rust program at EL0](USER_PROGRAM.md) now has bounded write/exit
calls and synchronous fault containment. A [controlled EL1 timer IRQ diagnostic](TIMER.md)
now verifies asynchronous entry and return; a [fixed EL0 deadline](USER_TIMEOUT.md)
can terminate a non-cooperative invocation. A [two-context diagnostic](SCHEDULING.md) now verifies
suspension/resumption on one CPU. A [bounded task lifecycle](TASK_LIFECYCLE.md)
now lets a task finish or fault while its peer continues. [Private stack maps](TASK_MEMORY.md)
now prevent these tasks from reading/writing each other's stack. General service
address-space ownership and task creation remain pending. Two fixed tasks now
exchange [bounded scalar messages](IPC.md) and [wait for messages](IPC_WAIT.md)
without being scheduled while blocked. External event delivery, persistent
service lifetime and a general service runtime remain pending. A first
[bounded record service](RECORD_SERVICE.md) now accepts a host-selected record,
executes counting at EL0 and returns an independently checked result. See the [kernel-to-service roadmap](ROADMAP.md). A [graphical host control panel](PANEL.md) now exposes real
QEMU boot, info, uptime, echo, and a [cooperative word-count task](TASKS.md). This is a host UI, not a guest desktop. The
correctness fixture is retained; verifier implementation resumes with demo jobs.

Implementation order: review the working foundation; freeze the demo contract;
design the host UI; add observable guest state/events; implement bounded jobs;
enforce limits; establish the baseline; integrate Meta; test and present results.
Storage, alternative-plan previews, and reversible operations were candidate
examples, not additional promised features.

## Architecture

User goal/events -> host coordinator -> Meta proposal -> host validation -> UART
-> guest validation/execution -> observed results -> coordinator/UI.

The host uses std and HTTPS; the bare-metal guest uses no_std. Secrets remain in
the host environment. The UI renders observations rather than owning policy.
Both sides validate requests. Timeouts and API errors are errors, never successful
execution. Real service isolation remains future work.

## Submission checklist from the supplied guide

- Public repository and accessible demo video; all contributors listed.
- One project per team, at most four participants, with a designated representative.
- Clearly identify work created during the eligible event period. The video must
  highlight those contributions, not present pre-existing work as new.
- Real Meta model use, with provider/model identified. A development mock does
  not satisfy final integration. API keys never go into chat or Git.
- Three-minute video, working demonstration, and reproducible instructions.
- Rights and attribution for code, data, assets, and model/provider usage.
- Confirm CV submission fields and retain the actual submission confirmation.

The rubric weights supplied in the guide are impact 30%, Meta tooling 25%,
technical depth/execution 20%, and demo 25%. Show a concrete user task, substantive
Meta decisions, sound limits/tests, and a reproducible visible result. Do not
expand scope merely to pursue every bonus award.

## Original proposed schedule (superseded October 29)

| Date | Deliverable | Completion condition |
| --- | --- | --- |
| Oct 26 | Rules, provenance, learning, demo design | Understand the foundation; record eligibility questions and experiment goals. |
| Oct 27 | Eligibility and scope | Record organizer guidance; define task operations, constraints, and outputs. |
| Oct 28 | Guest work and host observability | Manual requests produce verified real results over UART. |
| Oct 29 | Meta integration | One actual model decision completes an allowed operation with observed output. |
| Oct 30-31 | Autonomous loop and failures | Events trigger decisions; invalid actions/timeouts are handled visibly. |
| Nov 1 | Feature freeze | Repeatable demo and clean-checkout instructions. |
| Nov 2 | Documentation and rehearsal | Contributions, problem, and Meta's role are clear. |
| Nov 3 | Final video and submission review | Three-minute video, accessible links, complete fields; last full troubleshooting day. |
| Nov 4 | Final check and submission | Submit before 18:00 Guatemala and retain confirmation. |

Schedule is a target, not a claim of completion. Access and eligibility are external
dependencies. If access is delayed, a mock may support transport development only;
resolve an eligible real provider before calling the final demo integrated.

## Revised delivery targets — October 29

The kernel foundations and one useful protected service are implemented. The first
real Meta operation was subsequently verified on October 29 through the
[host bridge](META_BRIDGE.md): Muse Spark 1.3 selected notes@2, and the guest returned
44 bytes/9 tokens. A second live request declined an unavailable operation without
booting QEMU. A subsequent [bounded event loop](META_COORDINATOR.md) also completed
two revisions using real Meta calls and retained observations, without duplicate
work. Scripted events and offline stale-response tests are labeled explicitly.
This establishes coordination mechanics, not the complete autonomous demo.
The main delivery risk is extending infrastructure without completing the model
and useful workload path. These are targets, not completed-work claims.

| Date | Target | Exit condition |
| --- | --- | --- |
| Oct 29 | Bounded guest service; resolve Meta access | Guest service and authenticated Meta access verified; two live bridge outcomes captured. |
| Oct 30 | First actual Meta decision | First path and bounded revision loop verified Oct 29; next extend the useful workload and constraints. |
| Oct 31-Nov 1 | Useful workload, events and failures; feature freeze | Repeatable cases, independent result checks and fair baseline/comparison. |
| Nov 2 | Corrections, evidence, reproducibility | Clean-checkout run and reviewed results/limitations. |
| Nov 3 | Video and submission preparation | Three-minute video, contribution attribution and accessible links. |
| Nov 4 | Final check and submission | Internal target 18:00 Guatemala; official deadline 20:00. |

Protect the complete delivery path before adding GUI polish, persistent guest
services, more cores or phone support. The small record-count checker is not the
full independent review-pack verifier. Real Meta usage is now observed; useful adaptation and
comparative benefit remain pending; do not infer them from passing kernel tests.

## Non-goals and teaching

No phone drivers, Android/Linux application compatibility, native desktop, kernel
HTTPS, local inference, general agent framework, or production microkernel this
milestone. QEMU host portability does not mean guest Linux app compatibility.

The user wants short Spanish explanations, one small verified step at a time.
Project artifacts and all new commit text are English. Follow AGENTS.md for
architecture boundaries, honest testing, and commit/push per completed increment.

## Scope adjustment — October 30

The user explicitly prioritized a kernel increment before more AI/workload work.
Implemented one [persistent guest worker](PERSISTENT_WORKER.md): retained EL0
state between console requests, exclusive user-RAM ownership, bounded execution
and stop/failure recovery. The host Meta coordinator remains unchanged and still
uses closed record batches. This is a concrete service-lifetime foundation, not
completion of the kernel or of the submission demo. The remaining useful workload,
independent artifact verifier, fair comparison and three-minute video remain
pending; the November 4 deadline and internal target are unchanged.

## Guest coordinator increment — October 30

Implemented a persistent EL0 coordinator plus worker, with typed/versioned manual
proposal admission and result retention. This fulfills the next guest-service
boundary requested by the user. The existing live Meta path is unchanged; the
new guest path still needs a host transport mode forwarding its pending goal to
Meta and submitting the response. This is a separate integration task, not an
additional claim of live Meta execution, autonomous optimization or demo completion.
Keep the remaining workload/verifier/comparison/video tasks and deadline visible.

## Current delivery sequence — October 30

The explicit --guest transport mode now connects the EL0 coordinator's pending
goal/ticket to the Meta provider, with guest-side stale rejection. The older host
coordinator remains available as a distinct mode. Use [DEMO_PLAN.md](DEMO_PLAN.md)
as the current sequence: close real integration on October 30; deliver a useful
artifact/checker October 31; freeze the bounded workload/baseline November 1;
review evidence and presentation November 2; prepare the three-minute submission
November 3; submit November 4 before the 18:00 internal target.

The tiny count workload verifies the integration boundary. It does not establish
useful scheduling advantage or implement the full review-pack contract. Freeze
additional kernel mechanisms unless an actual demo output requires a small change.


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

## October 30: retained index slice delivered

The exact index now passes through the guest coordinator/worker, bounded scalar
IPC and validated revision storage. The native display exposes actual pending,
created and reused states. The Meta bridge continues to count; extend it to this
useful artifact next, then combine revision-aware results and measure a fair
comparison. This increment makes no AI-performance claim and does not change
the stated event dates or deadline. See [RETAINED_INDEX.md](RETAINED_INDEX.md).

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
