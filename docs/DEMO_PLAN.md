# Delivery plan — October 30, 2026

**Current next increment:** bounded batch proposals, followed by separately
specified semantic planning. [Provider diagnostics](META_DIAGNOSTICS.md) now
distinguish response failures and retain available usage. The implemented
[publication lifecycle](PUBLICATION.md) and [native view](PUBLICATION_DISPLAY.md)
now expose retained output, current versus historical state and failed updates.
Historical plans and submission dates below are preserved.

This is the current delivery sequence. Earlier design contracts and evidence keep
their original dates. Deadline: November 4, 20:00 Guatemala; internal submission
target: 18:00. The guide's October 26 kickoff versus October 27 build start remains
an eligibility question for organizers, not permission to rewrite history.

## What the demonstration must establish

RAI-OS runs its own AArch64 Rust kernel and two retained EL0 services in QEMU.
A Meta model proposes a bounded operation through a host bridge. The guest
coordinator owns the active goal, rejects obsolete/invalid proposals, dispatches
to the worker and retains actual completion. A late AI response must not override
newer work. Faults must return control rather than fabricate a successful result.

The new --guest path closes that integration. Counting six tiny public fixture
records is an integration check, not the final useful workload or proof that an
LLM schedules better than conventional code. The existing bridge and kernel are
not an Android replacement, native phone build, general microkernel or local model.

## Scope to complete next

Keep the [review-pack result contract](DEMO_ACCEPTANCE.md) as the useful target:
required records at exact revisions, an exact searchable index, and a consistent
manifest. A final READY state must be independently verified against the current
goal; superseded or partial work cannot qualify. Source preservation and explicit
permissions remain mandatory. The full artifact/verifier is not implemented yet.

The first [exact per-record index](RECORD_INDEX.md) now produces an EL0 artifact
and checks it independently against immutable records. It is a synchronous command,
not yet a retained coordinator job. The bounded [native guest display](GUEST_DISPLAY.md)
now shows real index/error outcomes. Freeze this first visual slice and extend the guest coordinator
from one record goal to the bounded dependency list needed for that artifact.
Permit model choices only where they affect a real plan (ready-job order and
valid result reuse), with identical capabilities available to the baseline.
The October 30 scope adjustment permits one minimal guest display adapter and
observed-result view. Do not expand it into phone drivers, a window system, more
cores, general networking or a generic agent framework.

## Completion gates and dates

| Target | Deliverable | Exit evidence |
| --- | --- | --- |
| Oct 30 | Guest coordinator -> host Meta -> guest worker | One live trace, offline stale/invalid/duplicate tests, clear per-clone credentials, documented limits. |
| Oct 31 | First useful artifact and independent checker | Per-record index/checker completed Oct 30; native display and observed index also completed Oct 30. Multi-record pack still pending. |
| Nov 1 | Complete bounded workload; feature freeze | Current-goal artifact, interrupted/revised-goal case, deterministic baseline using the same operations. Freeze scenario, prompts and metrics before comparison. |
| Nov 2 | Evidence and presentation | Repeated runs including inference/transport time; visible actual events/results; clean-checkout rehearsal. |
| Nov 3 | Three-minute video and submission package | Accessible video/repository, original contributions and team listed, reproducible commands and known limitations. |
| Nov 4 | Final review and submit | Submit before 18:00 Guatemala and retain confirmation; official cutoff 20:00. |

These are targets, not completion claims. If the useful workload is not complete
by feature freeze, explicitly revise the demo scope and claims; do not label the
counting proof as the full review-pack implementation. Keep all failed outcomes
in the evaluation. No numerical speed, battery or security superiority is claimed
until the corresponding evidence exists.

## What judges should see

1. Boot our kernel and identify the guest coordinator/worker versus the host bridge.
2. State the current goal and show Meta's real structured proposal and measured
   latency. The bridge carries requests; it does not own the guest goal state.
3. Change the goal while a proposal is pending. Show the guest rejecting its old
   ticket and completing the current task. Label replay-only fault cases clearly.
4. Show the useful artifact and independent acceptance result once implemented.
   A model explanation or a green UI alone is not correctness evidence.
5. Show a fair comparison only with the same output contract, operations, events
   and budgets. Report full wall time, model overhead and failures. A two-panel
   view is presentation of evidence, not an assumption that the AI side wins.

Keep the existing terminal/host tools for debugging. The implemented
[native guest display](GUEST_DISPLAY.md) is explicitly distinct from the Mac panel.
Use it to show actual index results; further visual features wait for useful-work integration. Scripted revision events are synthetic inputs; no phone battery,
GPU telemetry or physical sensor behavior has been implemented.

## Suggested three-minute structure

- 0:00-0:25: problem and architectural idea, with a concrete useful output.
- 0:25-0:50: real boot, guest services and Meta/host boundary.
- 0:50-1:50: actual task, interruption, stale rejection and verified new result.
- 1:50-2:25: independent check and comparison, if completed and measured.
- 2:25-3:00: contribution dates, how to reproduce, limitations and next direction.

The public repository, eligible Meta integration, three-minute video and clear
identification of event-built work are submission requirements from the supplied
guide. Preserve earlier work and the unresolved eligibility distinction. This
plan does not declare compliance or eligibility on the organizers' behalf.

## October 30: retained useful output

The [retained index](RETAINED_INDEX.md) supplies a reproducible notes@1 -> notes@2
-> notes@1 cycle: two real EL0 worker jobs and an exact cache hit, with native
created/reused views. This manual deterministic example is not an AI advantage
benchmark. Meta-selected indexing, combined outputs, the fair baseline, final
three-minute video and submission remain outstanding.

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


## October 30: frozen development policy pilot

The [policy pilot](POLICY_PILOT.md) adds a shared host experiment controller for
EDF, weighted value/credit ordering and Meta, using the same existing EL0 index
worker, exact cache and guest generation gate. Priorities, real deadlines and
simulated credit admissions now have declared rules and an independent log/output
audit. The simulation is on the host; this does not add kernel power management
or complete the full review-pack contract. Three development scenarios and two
repetitions per policy are frozen before live measurement. Report all outcomes
and model overhead; no efficiency claim follows from implementing the experiment.
The larger held-out study, final presentation and submission remain pending.


## October 30 measurement decision

The [first frozen policy pilot](POLICY_PILOT_RESULTS.md) found no AI advantage:
EDF and value each passed six trials; per-action Meta passed one of six, with
five rejected provider responses. Preserve all evidence. Do not present this as
an AI efficiency win in the video. Next diagnose provider rejection categories
and evaluate a separately frozen higher-level/batched-planning hypothesis,
keeping deterministic guest execution and equally capable baselines. This does
not change the submission dates or establish general model/OS superiority.


## November 1: bounded batch integration

[Batch proposals](META_BATCH.md) now cover all 1–4 currently missing operations
in one response. Pure all-before-dispatch admission, original-generation checks,
per-job guest tickets and the independent publication handshake constrain the
plan. Events during inference, between jobs and during validation supersede old
authority. Current catalog/permissions remain immutable fixtures. The host holds
the pending plan; the guest retains goals, execution admission and publication.
No new kernel mechanism or GUI scope is added. Single-operation modes and the
original pilot remain separate and unchanged in policy/settings.

This is a delivery gate, not completion of semantic planning or a new efficiency
comparison. November 1 was the feature-freeze target; the independently authored
semantic workload/fair baseline, final video and submission are still pending.
Prioritize one bounded demonstrable case, evidence and rehearsal over additional
infrastructure. If no semantic advantage is established, present the measured
recovery prototype with its actual Meta role. November 4 remains the deadline
(18:00 internal target, 20:00 official Guatemala); preserve contribution dates and
the unresolved October 26 kickoff / October 27 build-period eligibility question.


## November 1: first content-based selection gate

[Semantic selection](SEMANTIC_SELECTION.md) now separates interpretation from
execution: natural-language goal plus six public document contents -> bounded
Meta selection/clarification/decline -> independent fixed-case intent check ->
existing deterministic guest batch and verified publication. Expected answers
are authored before inference and excluded from provider context. Host request
text/version and static catalog/permission versions bind selection; guest goal
and per-job tickets bind execution. No new kernel or UI mechanisms.

Each case uses a fresh guest session. Continuous semantic replanning, a genuinely
complex workload and fair deterministic comparison remain pending. These six
public development cases do not establish general correctness or superiority.
Freeze additional infrastructure; prioritize measured outcomes, reproducibility
and the three-minute video. November 4 dates and provenance questions are unchanged.


## November 1: scoped resolver, frozen comparison and same-session replay

[The comparison](SEMANTIC_COMPARISON.md) adds a deterministic content/reference
resolver with explicit language coverage, sharing permissions, dependency rules,
intent oracle and guest execution with Meta. Twelve development cases and two
rotated repetitions per policy are frozen before live measurement (24 calls max).
No held-out or general efficiency claim is made. A separate recorded-decision
replay uses two archived Meta selections to show historical/current publication,
exact reuse and one new job in a single real guest session. It performs no live
inference; graphic pauses are presentation-only. Continuous live semantic event
recovery remains pending. No kernel or graphical implementation changes.


## November 1 comparison conclusion and bounded video story

The frozen semantic development comparison passed 24/24 for each policy; rules
were faster in every pair. Selecting trials had median 138.5 ms rules / 3778.5 ms
Meta. Do not present this as AI performance superiority. Full evidence and limits
are in SEMANTIC_COMPARISON.md. Record the working prototype honestly:

- 0:00–0:25: goal and architecture: our AArch64 guest, host model bridge, bounded
  operations. Identify actual contributions and outstanding eligibility questions.
- 0:25–1:00: one recorded real Meta interpretation, exact selected documents and
  independently checked output. Label recording/probe latency; do not imply live
  inference if showing archived decisions.
- 1:00–1:45: native same-session replay from historical to current requirement.
  Show old output retained, two indexes reused, one new job, verified replacement.
  Explicitly identify prerecorded approved decisions and presentation pauses.
- 1:45–2:10: ambiguity/absent-data cases and an offline stale/invalid rejection.
  Label injected faults/replay. No arbitrary model-controlled execution.
- 2:10–2:40: actual comparison: equal development correctness, deterministic
  rules faster; model flexibility outside this domain is not yet measured.
- 2:40–3:00: repository/run instructions and limits: experimental mobile direction,
  QEMU prototype, no physical-phone support or completed autonomous microkernel.

This is a proposed recording sequence, not a finished video or submission. No
further kernel/UI expansion is needed for this story. Clean-checkout rehearsal,
three-minute video, accessible links, organizer clarification and submission
confirmation remain outstanding; preserve the November 4 internal/official targets.


## November 2: useful offline artifact pivot

The user approved prioritizing a useful result after no semantic speed advantage
was measured. [OFFLINE_MISSION.md](OFFLINE_MISSION.md) defines a larger fictional
project corpus with approved versus draft releases, compatible dependencies,
permission exclusion and unresolved human actions. The Rust host bridge produces
a portable reader plus exact source files/manifest, with independent checking
before publication. This new corpus does NOT execute in QEMU. The kernel remains
unchanged; do not relabel host packaging as a guest filesystem or new native UI.

This first slice is a one-shot checked host publication. Automatic semantic event
recovery, incremental updates and general file ingestion remain pending. Prior
negative performance evidence is retained. Protect final verification, browser
review, video and submission time; the original November 4 targets remain intact.


## November 2: fresh semantic decisions in a retained guest

[SEMANTIC_SESSION.md](SEMANTIC_SESSION.md) connects fresh host-side interpretation
to ONE retained guest session, with externally checked streamed approvals. Guest
EL0 jobs/cache/publication and native presentation remain the execution boundary.
Controlled sequential request events replace the previous preloaded-decision-only
walkthrough; the recorded replay remains available separately. Ambiguity schedules
no work. During interpretation the screen still represents the last accepted guest
target; new request/clarification text is host-side only.

This is an integration increment on the existing six tiny guest records, not a
new decision-quality win. Freeze three known events, both rules and Meta, and three
Meta calls maximum before measurement. No kernel expansion, concurrent event
handling, file discovery or resource-efficiency claims. The host-only offline
mission remains separate. Prioritize final evidence/rehearsal/video/submission;
preserve original deadlines and unresolved contribution eligibility.
