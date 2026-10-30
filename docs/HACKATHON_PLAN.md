# RAI-OS hackathon plan

Updated October 29, 2026. Local times: America/Guatemala. This is a proposed
roadmap; completed foundations are identified below. One real Meta-to-guest operation is verified; a continuous autonomous coordinator and a general guest service runtime remain pending.

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
booting QEMU. This completes the first integration path, not the autonomous demo.
The main delivery risk is extending infrastructure without completing the model
and useful workload path. These are targets, not completed-work claims.

| Date | Target | Exit condition |
| --- | --- | --- |
| Oct 29 | Bounded guest service; resolve Meta access | Guest service and authenticated Meta access verified; two live bridge outcomes captured. |
| Oct 30 | First actual Meta decision | First path verified Oct 29; extend it into a bounded coordinator with observed feedback. |
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
