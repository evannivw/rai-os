# RAI-OS documentation

For a first visit, use [Try RAI-OS](TRY_RAI_OS.md), the [video and project overview](../README.md)
and the [current architecture](ARCHITECTURE.md). Jarrol reported submission on November 4;
[SUBMISSION_PACKAGE.md](SUBMISSION_PACKAGE.md) records delivery status.

This index separates the current app demo from kernel experiments and planning history.
A dated result remains evidence of that run; it does not claim that every later source
revision was tested in the same environment. The supplied guide's October 26/27
eligibility discrepancy is retained in the [project provenance](SUBMISSION.md#development-provenance).

## Start here

Current evaluator and contributor entry points.

- [Try RAI-OS](TRY_RAI_OS.md)
- [Build and run RAI-OS](GETTING_STARTED.md)
- [Run RAI-OS from Windows, Linux or macOS](PORTABLE_RUN.md)
- [RAI-OS — Submission description](SUBMISSION.md)
- [Submission record — November 4, 2026](SUBMISSION_PACKAGE.md)
- [RAI-OS demo kit](DEMO_READY.md)
- [Final three-minute recording runbook](DEMO_FINAL_RUNBOOK.md)
- [RAI-OS architecture](ARCHITECTURE.md)
- [Building with the RAI-OS app toolbox](APP_TOOLBOX.md)
- [What limits generated applications](APP_LIMITS.md)

## Current app contracts

Implemented bounded capabilities. Read their limits alongside examples; these are not arbitrary native applications.

- [Prompt-generated microapps](MICROAPPS.md)
- [Declarative app capabilities](APP_CAPABILITIES.md)
- [Dynamic calculated fields](APP_FORMULAS.md)
- [Generated apps with checked processes](APP_WORKFLOWS.md)
- [Composable application behavior](APP_BEHAVIORS.md)
- [Composable whole-table actions](APP_BATCHES.md)
- [Atomic transfers between app records](APP_TRANSFERS.md)
- [Composable visual pages](APP_PAGES.md)
- [Saved app sessions](APP_STORAGE.md)
- [User-mediated app data exchange](APP_EXCHANGE.md)
- [Git tools for generated applications](APP_GIT.md)
- [Bounded application correction](APP_REPAIR.md)
- [An app that controls the guest display](DISPLAY_CONTROL.md)

## Kernel foundations and lower-privilege experiments

Implemented mechanisms or explicitly bounded diagnostics. These EL0 experiments are not the generated-app isolation boundary.

- [Why an own kernel, and what it does not prove](KERNEL_DECISION.md)
- [UART protocol v1](PROTOCOL.md)
- [First physical-page allocator](MEMORY.md)
- [EL1 translation tables and permissions](MMU.md)
- [EL1 fatal exception diagnostics](EXCEPTIONS.md)
- [One resumable EL1 context](CONTEXT.md)
- [Kernel clock and uptime](CLOCK.md)
- [Controlled EL1 timer interrupts](TIMER.md)
- [First cooperative kernel task](TASKS.md)
- [First lower-privilege Rust program](USER_PROGRAM.md)
- [A deadline for one EL0 invocation](USER_TIMEOUT.md)
- [Two resumable EL0 contexts on one CPU](SCHEDULING.md)
- [Kernel task records and bounded lifecycle](TASK_LIFECYCLE.md)
- [Private task stack mappings](TASK_MEMORY.md)
- [Bounded scalar IPC between tasks](IPC.md)
- [Blocking receive, wakeup and bounded deadlock handling](IPC_WAIT.md)
- [First bounded guest record service](RECORD_SERVICE.md)
- [One persistent guest worker](PERSISTENT_WORKER.md)
- [First coordinator inside RAI-OS](GUEST_COORDINATOR.md)
- [RAI-OS native display — first screen](GUEST_DISPLAY.md)

## Retained host, files and model experiments

Earlier runnable slices remain available for learning and evidence. Their fixtures, entry points and limits differ from the current app demo.

- [Host console](CONSOLE.md)
- [Device Lab: graphical host panel](PANEL.md)
- [Small host coordinator](COORDINATOR.md)
- [First real Meta-to-guest operation](META_BRIDGE.md)
- [Bounded event-driven Meta coordinator](META_COORDINATOR.md)
- [Meta bridge for the guest coordinator](GUEST_META_BRIDGE.md)
- [First writable guest file store](GUEST_FILES.md)
- [Prompt-driven guest workspace](PROMPT_WORKSPACE.md)
- [First useful artifact: an exact record index](RECORD_INDEX.md)
- [Retained indexes inside RAI-OS](RETAINED_INDEX.md)
- [Meta proposals for retained guest indexes](META_INDEX_BRIDGE.md)
- [Complete versioned index sets](INDEX_SETS.md)
- [Meta planning for complete guest index sets](META_SET_BRIDGE.md)
- [Provider diagnostics before batch planning](META_DIAGNOSTICS.md)
- [Bounded Meta batch proposals](META_BATCH.md)
- [Retained publication lifecycle](PUBLICATION.md)
- [Native publication states](PUBLICATION_DISPLAY.md)
- [Content-based requirement selection](SEMANTIC_SELECTION.md)
- [New semantic decisions in one retained RAI-OS session](SEMANTIC_SESSION.md)
- [One objective, changing context](CONTEXT_SESSION.md)
- [Offline presentation mission: first useful host artifact](OFFLINE_MISSION.md)

## Results, history and future plans

Preserved measurements and evolving plans. Dated pending items describe their original checkpoint; follow the current entry points above for delivery status.

- [Policy comparison pilot v1](POLICY_PILOT.md)
- [First policy pilot: fixed rules outperform this per-action Meta configuration](POLICY_PILOT_RESULTS.md)
- [Frozen semantic development comparison](SEMANTIC_COMPARISON.md)
- [Persistent goals: demonstration contract](PERSISTENT_GOALS.md)
- [RAI-OS hackathon planning history](HACKATHON_PLAN.md)
- [Delivery plan — October 30, 2026](DEMO_PLAN.md)
- [Demo experiment: an adaptive project review pack](DEMO_EXPERIMENT.md)
- [Required result contract: project review pack v1](DEMO_ACCEPTANCE.md)
- [Review-pack correctness fixture v1](DEMO_FIXTURE.md)
- [Three-minute demo: an app that changes with your work](DEMO_JOURNEY.md)
- [Native guest display: original scope and completed gates](GUEST_GRAPHICS_PLAN.md)
- [Native interface directions for the final demo](DESIGN_DIRECTION.md)
- [Kernel-to-service roadmap and next app steps](ROADMAP.md)
- [Work log and provenance](WORKLOG.md)

## Verification evidence and compatibility links

- [Visible update review: actual QEMU captures and debug/release/Linux checks](evidence/2026-11-04-update-review/README.md)

- [Repository-wide audit: documentation, controller stress and portable startup](evidence/2026-11-04-repository-audit/README.md)

- [Recorded two-request Meta demo](evidence/2026-11-04-submission/README.md)
- [Final code review, 114 Rust tests and QEMU/Linux checks](evidence/2026-11-04-final-review/README.md)
- [Evidence directory](evidence/): original run outputs, failures and dates remain unchanged.
- [Old description URL](SUBMISSION_DRAFT.md): compatibility pointer to the finalized description.

Check relative paths and Markdown heading links with `python3 -B scripts/check_docs.py`
from the repository root. This offline check does not validate external web access
or factual claims; those still require review against code and recorded outcomes.
