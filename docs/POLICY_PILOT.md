# Policy comparison pilot v1

This October 30 development pilot compares real guest index work with a shared
host experiment controller. It is a narrower experiment than DEMO_EXPERIMENT.md,
not completion of the full review-pack contract or the planned held-out study.
Freeze this specification, fixtures, policies and provider prompt in Git before
any live pilot measurement. Do not adjust them after seeing which policy wins.

## Question and common capabilities

Can Meta choose useful job order under changing structured priorities, revision
requirements and limits, after paying its actual inference cost? We do not assume
an LLM wins. All policies must produce the identical exact final required set.

Every trial boots a fresh copy of the same release kernel with one QEMU CPU,
128 MiB, TCG and no display. The guest executes all indexes at EL0, retains exact
revisions and validates goal generations. Both fixed policies and Meta use the
same driver, operations, cache, current observations and independent checker.
Neither receives future events. No concurrent QEMU trials or graphical rendering
are used for timing. A visual replay may later show this evidence with labels.

Policies choose only one currently missing affordable record, or decline. There
is no arbitrary code, goal editing, optional work, batching, extra model tool or
permission difference. The controller never substitutes a fixed choice when the
model fails. If an inference is pending, this pilot waits for it while real time
and event delivery continue; no background fallback is implemented. Existing
known work is retained. This deliberately tests the current single-action model
boundary, not all possible hybrid architectures.

## Priorities, deadlines and simulated quota

Scenario time starts immediately before QEMU startup, so boot, host validation,
UART, policy computation, inference and independent publication checking count.
Every current record has a target due_ms and positive weight. The final scenario
deadline is hard: only a complete, independently checked final-generation bundle
by that deadline passes. A correct older generation never counts as final.

Among passing trials, report the sum of final-goal weights whose exact revisions
were retrieved/checked by their target times. Previously completed revisions keep
their observed readiness timestamps. Scores use the final activated goal's weights
and targets, identically for all policies. Failure scores zero, regardless of
partial progress. This is an early-readiness measure, not optional preview value.

Each dispatched index costs max(1, ceil(public source bytes / 32)) simulated work
credits. This known accounting rule is independent of the policy. It does not
measure cycles, joules, battery or actual RAM use; no artificial execution delay
is inserted. Cache reuse costs no new credit. A nonrefundable debit occurs just
before a worker proposal; guest failure retains that attempted-work charge.

An event replaces the current requirement, weights, targets and cumulative cap.
Completed revisions remain cached. Spent credits are never reset. A lower cap
applies to future admissions; it cannot retroactively invalidate prior legal
spending or interrupt an already dispatched synchronous job. No new job may start
if spent + cost exceeds the current cap. Unaffordable missing work waits for an
event or deadline; it does not prompt the model repeatedly. These quota checks
belong to the common HOST experiment controller. The kernel still owns freshness,
execution and publication, but does not enforce this pilot's credits or deadlines.
Do not present the host constraint simulator as new guest power-management code.

Due events are applied at serialized request boundaries and their actual delivery
times are logged. All policies use this same rule; a synchronous UART call cannot
be preempted by the host. Recheck after selection and before debit/dispatch. A
priority-only event also commits a fresh guest goal generation, invalidating an
in-flight proposal for the previous rules. An already-running index may finish
after a target/deadline and earns no false on-time success.

## Fixed policies and model

Both fixed rules recompute on every observed event/result, share affordability
filtering and retain valid caches. They prioritize targets not yet expired.

- EDF: earliest target first, then higher weight, then lower credits, then key.
- Value: highest weight/credits ratio first, then earlier target, then key.
  Ratios use exact integer cross products. Required work is never optional.

These are two declared deterministic heuristics, not an exhaustive optimum or a
claim to represent Android's scheduler. Both are reported; compare Meta with the
stronger outcome per scenario. FIFO is not the headline comparator. No side gets
special future knowledge or an artificial delay handicap.

Meta uses the existing Muse Spark 1.3 endpoint/schema/response bounds with a
pilot-specific prompt in host/meta/src/provider.rs. Its context includes the
entire current set, readiness, elapsed time, deadline, cap, spend and job weights.
Only affordable missing keys are schema-authorized. state_version is the guest
goal generation, checked against the original request and again in the guest.
One inference may be pending; at most eight choices per trial for every policy.
An invalid choice, current decline, provider failure or exhausted choice budget
fails visibly, without retry/fallback. Pending provider work is drained after a
deadline only for cleanup/usage; its late result cannot execute. Total drain and
cleanup time is reported separately from qualifying completion time.

## Frozen development scenarios

The three JSON inputs in fixtures/policy_pilot/v1 are deliberately small:

- steady: three records, four credits, an early high-weight notes target.
- priority-change: same records, but empty@1 becomes urgent at 2500 ms.
- revision-quota: notes@2 replaces notes@1 at 3500 ms and the cap drops 8 -> 6.

The revision case still allows an eager baseline's legally completed notes@1
plus all final work. A model that happens not to execute an obsolete revision
may spend less; report that as revision timing, not automatically intelligence.
Tiny real indexes may make fixed policies overwhelmingly faster. Include ordinary
cases and all failures. We are measuring this design, not optimizing a staged win.

Run two repetitions of each of three policies on each scenario: 18 sequential
trials. Rotate policy order by (scenario_index + repetition) modulo three. Save
all raw logs, scenario/kernel/bridge hashes, source commit and exact commands.
The independent verifier reconstructs policy inputs, deadlines, quota admissions,
EL0 results, exact final bytes/indexes and final metrics; the model never scores.
Report per-trial outcomes, per-scenario distributions and model usage. Two runs
are a development pilot, not statistical evidence or the planned 20-scenario,
three-repetition held-out evaluation. A representative demo and stronger claims
must follow a separate frozen evaluation, not selection of a lucky winning run.

## Reproduce and inspect

From a clean committed checkout with Rust/QEMU and Python 3 installed:

```sh
. "$HOME/.cargo/env"
python3 scripts/verify_pilot.py
python3 scripts/verify_pilot.py --release
# Fixed policies only; no key/network usage:
python3 scripts/compare_policies.py --output target/pilot-fixed
# Include actual Meta calls (each clone supplies its own ignored .env.local key):
python3 scripts/compare_policies.py --live --output target/pilot-live
python3 scripts/compare_policies.py --audit target/pilot-live
```

Output directories must be new. The driver builds the same release binaries for
all trials, records hashes and rotates policy order. REPORT.md includes every
trial, distributions and comparison with the stronger fixed result. results.json
also contains per-trial timelines for later visual replay. Raw logs, provider
usage, exact publications and failures are retained. Infeasible/deadline outcomes
are valid failed trials; a broken trace/transport audit is explicitly unauditable,
kept in the report and never silently excluded as a successful comparison.

For a single trial, the host binary accepts
`<kernel> --pilot edf|value|meta <scenario.json>` or
`<kernel> --pilot replay <scenario.json> <replay.json>`. Replay is test evidence,
not Meta inference. Timed runs intentionally do not offer a graphical flag.
The publication timestamp includes the same independent Python source/index
checker for every policy; the later whole-log audit is outside trial time.
Per-record readiness is observed retrieval with structural checks, subsequently
validated semantically by the publication/audit before earning any score.
A debit records its event-cutoff timestamp: events due at that checked boundary
must be delivered first; arrivals after that boundary apply to the next admission.

Hosted inference energy and monetary cost are not measured by the guest credit
counter. Report available response token usage separately; do not treat remote
compute as free energy or infer a phone battery saving from this experiment.


## First measured results

The [October 30 results and interpretation](POLICY_PILOT_RESULTS.md) retain all
18 trials from the frozen source. Both fixed policies passed 6/6; the current
Meta configuration passed 1/6, with five provider responses rejected by the
adapter. No rule/prompt/fixture was changed in response to these outcomes.
This is evidence against the current per-microtask remote-model boundary in this
small workload, not a general verdict on AI planning. See the report's diagnostic
and partial-usage limits before interpreting model scores or timing totals.
