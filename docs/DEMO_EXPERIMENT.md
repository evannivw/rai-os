# Demo experiment: an adaptive project review pack

Status: proposed design, October 26, 2026. No jobs, scheduler, AI coordinator,
graphical comparison, or benchmark results described here are implemented yet.

## Question and honest claim

Can a Meta coordinator select a better work plan than competitive deterministic
policies when priorities, dependencies, memory limits, and execution costs change?
Better means completing the SAME required outputs within constraints, or obtaining
more independently verified optional value under the SAME deadline and budget.

We do not assume that an LLM is a better scheduler. For fully specified numerical
scheduling, classical algorithms may win. The potential opportunity is composing
and revising a cross-task plan under varied goals and incomplete cost estimates.
Any gain must exceed inference/coordination latency. If the policy does not win,
report that outcome rather than weaken the baseline or hide model overhead.

Required artifacts and validation rules are defined in
[DEMO_ACCEPTANCE.md](DEMO_ACCEPTANCE.md).

## User-visible task

Prepare a project review pack before a deadline, under an explicit resource budget.
Required: validate selected source records, build an exact searchable index, and
publish a manifest that references all required outputs. Optional: previews of
additional relevant records. Source records must be preserved; unauthorized
objects must remain inaccessible. Optional previews cannot replace required work.

First guest implementation: small, deterministic in-memory records and cooperative
jobs, not PDF processing, a persistent filesystem, or isolated processes. A preview
can be a bounded excerpt, not model-generated prose. Checksums/reference outputs
validate transformations; a model explanation does not prove correctness.

The UI shows the pack's required checklist, optional verified outputs, progress,
allocated job-buffer bytes, operations, rejections, and model call durations.
Changes to the goal are shown in both panels at the same event boundary.

## Controlled events

- A lower work budget, visually labeled simulated energy/battery budget.
- A lower guest-managed job-buffer limit: allocation decisions must respect it.
  This is not hot-changing QEMU physical RAM or measuring the entire guest RSS.
- Newly required records or a changed deadline/priority, defined by the scenario.
- Revised observed job costs, obtained from actual executor progress.

Use a scripted, versioned event trace for comparisons; manual controls are for
exploration, not evidence of reproducible superiority. Rules must define whether
a reduced buffer budget affects only new allocations or requires cooperative
release before further work. Never silently discard output or exceed a hard cap.
Artificial energy units are accounting coefficients, not joules or phone battery
life. Do not claim CPU/GPU/battery hardware support based on these controls.

## Common engine and allowed actions

Both policies use the same guest binary, dataset, initial state, available actions,
observations, cost estimates, target requirements, and permissions. Candidate
actions: choose ready job, set a bounded batch size, pause/resume cooperative work,
and reuse an already-validated result. All policies have the same reuse ability.
No policy may invent successful output, skip required records, remove sources,
raise its own permissions, or alter scoring weights.

The engine checks prerequisites, budgets, and state versions before applying a
plan. A stale response is rejected or revalidated, never applied blindly. While
Meta is thinking, the last valid bounded policy remains active; time continues.
Timeout/disconnection uses the same documented fallback, counted in results.

## Baselines

FIFO may be shown for teaching but is not the headline competitor. Implement:

1. Dependency-aware earliest-deadline scheduling with required work prioritized.
2. Dependency-aware priority/value-per-estimated-work scheduling, with required
   outputs protected, memory backpressure, batching, and valid-result reuse.

Tune on development scenarios only. Compare against both and report the stronger
result for each evaluation scenario; do not conceal the better classical policy.
Both receive the same structured goals, including updates and deadlines. If Meta
also interprets natural-language goals, evaluate that separately as usability:
giving one side extra knowledge would confound the scheduling comparison.

Freeze shared capabilities, validators, weights, baseline settings, prompts, model
identifier, and event generator before the held-out evaluation. A small exhaustive
planner may provide an optional offline bound for tiny cases, but must be labeled
clairvoyant if it knows future events; live policies must not receive that future.

## Measures

Hard gates: correct required outputs, source integrity, authorized operations,
no budget violations. A fast invalid result is a failed run.

For successful runs report:

- End-to-end wall time to required completion, including the first planning call,
  all subsequent inference, host transport, validation, and retries.
- Verified optional value at a fixed deadline; weights defined before evaluation.
- Actual executed work units and peak tracked job-buffer allocation.
- Model calls, request/response latency, available token/cost accounting, and
  rejected/stale actions. Hosted inference energy is unknown, not zero.

Distinguish real measured timings and allocations from modeled energy. Faster,
less work, lower memory, and better optional coverage are separate outcomes.
Do not trade away correctness or change output quality without declaring it.
A deterministic verifier scores outputs; Meta never grades its own performance.

## Evaluation and visual presentation

Start with a cheap small pilot of the policy loop and shared deterministic work
model before building the full guest/UI feature set. Treat that as a host
simulation, not proof of a working kernel feature. It tests the core hypothesis.

After the pilot, reserve at least 20 reproducible scenarios across ordinary loads,
resource pressure, goal changes, and combined changes; run 3 repetitions per
policy to expose model variability. Keep held-out seeds separate from development.
Record all results and failures, not only a winning demo. Give dispersion and
per-scenario outcomes; a small exploratory sample is not a universal superiority
claim. Publish raw traces with secrets removed, model configuration, and versions.

For timing, run each policy separately on the same host with reset state and
alternating order, not two contending QEMU instances. Side-by-side visual replay
must be labeled as replay. A simultaneous interactive view can illustrate behavior
but cannot establish an uncontended speedup. Each policy sees only events revealed
so far on the same elapsed-time trace; model waiting does not pause the deadline.

The three-minute video should show one representative held-out case, then the
aggregate results and a brief deliberately injected invalid-action rejection.
A staged failure test must not be described as a spontaneous model failure.
Provide a script for judges to replay the trace and a separate live mode.

## Decision gate

Proceed with an efficiency claim only if the pilot/evaluation shows a repeatable,
useful advantage against strong baselines under declared constraints, including
model overhead. Do not choose a claimed speedup threshold after observing results.
If the model loses, report it and reconsider the role of AI: goal interpretation
or cross-service planning may be useful even when a deterministic scheduler should
retain low-level execution control. Those are different claims and experiments.

Required/optional outputs now have a design contract in DEMO_ACCEPTANCE.md.
Next: freeze a small fixture and trace, then implement an independent verifier
and baseline rules. Do not build an elaborate dashboard around an assumed result.


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
