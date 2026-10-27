# RAI-OS hackathon plan

Updated October 27, 2026. Local times: America/Guatemala. This is a proposed
roadmap; completed foundations are identified below. AI and kernel scheduling remain pending.

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
entry is the next implemented foundation for future guest services; memory
protection, user-mode execution, scheduling, and IPC remain pending. A [graphical host control panel](PANEL.md) now exposes real
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

## Proposed schedule

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

## Non-goals and teaching

No phone drivers, Android/Linux application compatibility, native desktop, kernel
HTTPS, local inference, general agent framework, or production microkernel this
milestone. QEMU host portability does not mean guest Linux app compatibility.

The user wants short Spanish explanations, one small verified step at a time.
Project artifacts and all new commit text are English. Follow AGENTS.md for
architecture boundaries, honest testing, and commit/push per completed increment.
