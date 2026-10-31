# Delivery plan — October 30, 2026

**Current next increment:** [persistent goals](PERSISTENT_GOALS.md), agreed after
the measured pilot. Build current-versus-historical publication and recovery
first, then bounded semantic/batch planning. This supersedes earlier next-step
suggestions about optimizing per-record AI ordering. Historical status below
and the delivery dates are preserved.

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
