# Required result contract: project review pack v1

Status: design contract, October 26, 2026. This specifies what the forthcoming
pilot must produce; none of the jobs, artifacts, permissions, or validators below
are implemented by the current help/info/echo kernel. No benchmark results exist.
See [DEMO_EXPERIMENT.md](DEMO_EXPERIMENT.md) for the comparison protocol.

## Mission

Before the scenario deadline, publish a usable project review pack containing
all currently required, authorized records at their required revisions, plus an
exact index and manifest. Preserve the sources and obey declared resource limits.
The policy may change execution order and batch sizes, not the required result.

Start with a small in-memory dataset, not real PDF files, a filesystem, downloads,
or isolated application processes. Dataset bytes, sizes, event times, budgets,
and scoring weights will be fixed in the next scenario-definition step. Do not
choose them after seeing which policy wins.

## Input contract

Each record has a stable record_id, immutable revision, project_id, ASCII text,
and declared read permission. A source revision is never modified in place;
an update adds a new revision. A goal has a goal_version, explicit required
(record_id, revision) pairs, optional authorized pairs, and a deadline.

Both policies receive identical structured goals and the same observations as
events occur. Required pairs are supplied by the scenario, not inferred differently
by each policy. Natural-language goal interpretation is a separate experiment.
The host must not include restricted record contents in model prompts; the guest
must independently reject unauthorized access through its execution interface.
These are narrow prototype guarantees, not protection against a malicious host.

## Mandatory deliverables and acceptance checks

| ID | Required result | Independent acceptance check |
| --- | --- | --- |
| R1 | Complete required record set | Exact equality with the active goal's required ID/revision pairs: no missing, duplicate, stale, extra, or unauthorized entries in the required section. |
| R2 | Exact record contents | Each included required record matches the fixture's bytes and byte length. Checksums may aid reporting, but the verifier compares bytes as well. |
| R3 | Exact searchable index | Index every required record using the fixed tokenization below; compare the full map to the reference, including counts and membership. |
| R4 | Consistent manifest | Identify goal_version, pack_version, required ID/revision pairs and output references. Every reference resolves to an actual validated result from this same pack. |
| R5 | Safe, complete publication | Expose a new current pack only after all required checks pass for the current goal. Partial/stale work cannot be reported as READY. |
| R6 | Preserved inputs and respected limits | Original source revisions remain byte-identical, no forbidden operation succeeds, and the execution log shows no resource-limit violation under the declared budget-transition rules. |

Required results must not depend on optional previews. A policy cannot report a
summary as a substitute for original bytes or omit difficult required records.
The same deterministic verifier evaluates every policy; Meta does not grade itself.

## Index definition

Use maximal ASCII letter/digit sequences as tokens, case-fold A-Z to a-z, and
treat all other bytes as separators. Do not stem words, remove stopwords, or use
semantic similarity. Map each token to its per-record occurrence count using
(record_id, revision) keys. Empty records have zero tokens but remain required.
Sort tokens and record keys for stable serialization. Index only required records
in v1, so optional work cannot change required-result correctness.

Illustrative input: a required record containing `Pump A: pump 7.` yields
`pump: 2`, `a: 1`, and `7: 1` for that record. This is a specification example,
not an executed kernel result or a benchmark fixture.

## Publication and changes during execution

The pack lifecycle is PENDING -> BUILDING -> VALIDATING -> READY, with explicit
BLOCKED or FAILED states when appropriate. These are future UI/domain states,
not current protocol commands.

Every proposed operation is tied to a goal/state version. Revalidate or reject
stale plans at execution. The common engine, not the model, owns the publication
check. In-memory publication can swap a validated pack reference; no disk
transaction, persistence, or crash-durability guarantee is implied.

A new requirement creates a new goal_version. A previously published pack can
remain inspectable as an earlier version, but must be shown as stale for the new
goal until the new required result is valid. Cache reuse requires matching source
revision, operation parameters, and applicable permissions. Both policies can reuse.
At final evaluation, only a pack valid for the final activated goal qualifies.
No policy sees future goals before their scheduled event.

If a required record is inaccessible, a dependency cannot complete, or the deadline
passes without a valid final pack, report BLOCKED/FAILED and the cause. Never
silently weaken permissions, skip a requirement, or fabricate success. Such runs
are failures or declared infeasible cases, not excluded from the published results.
A rejected forbidden request demonstrates enforcement; it is not the same as a
successful forbidden access, and its latency/attempt count is still reported.

## Optional value, after correctness

Candidate optional work: bounded previews of additional authorized records. An
excerpt is exactly the first min(64, byte_length) bytes of the selected revision,
with its ID/revision and length. This remains deterministic, not an AI summary.
Optional data is stored in a separate section, excluded from the required index.

An optional output earns its predeclared weight only once, if its bytes validate
and its revision/permission remains valid at the evaluation deadline. It cannot
compensate for any failed mandatory gate. Freeze the eligible optional set and
weights before held-out evaluation; neither policy may assign its own score.

## Completion time and comparison

A run passes only if R1-R6 hold for the final goal by the declared deadline.
Measure required completion from scenario start to the earliest valid publication
for that final goal after its activation. Record the final-goal activation time
as well. A fast publication for an earlier superseded goal is not final completion.
Include model calls, transport, validation, rework, and fallback time in wall time.

Report separately: pass/fail, completion time for passing runs, verified optional
value at the common deadline, work units, tracked job-buffer peak, rejected actions,
and model overhead. Never replace total time with guest-only time to hide inference.
When comparing speed, require equal required outputs; when comparing optional
value, require the same deadline/budget and all required gates passed.

## Controlled complexity

The forthcoming trace will combine changes such as a new required record,
a new revision, a dependency failure, and a lower resource budget. Apply the
same trace and execution rules to every policy. Ordinary no-interruption cases
must also be retained. An event may invalidate previous work but must not
specifically disable a baseline capability that remains available to Meta.

Budget-transition semantics are still a scenario-design prerequisite: specify
when a lower limit becomes effective, what can finish/release before then, and
how an impossible target is recorded. Simulated energy is not measured battery
life. A lower job-buffer cap is not hot-removing QEMU physical RAM.

## Graphical acceptance view

Both panels should show the same checklist: required records, content integrity,
index validity, manifest validity, permission/budget status, and current goal version.
READY comes from verifier/engine evidence, not from a model-written message.
Show remaining time and optional verified value separately. Indicate clearly
whether the view is live or a replay and whether resource values are simulated.

## Next step

Create a small versioned fixture and event trace, then implement the independent
host reference verifier. Test rejection of a missing record, wrong revision,
changed byte, incorrect index, dangling reference, and incomplete publication.
That establishes an objective definition of success before either policy runs.
The host pilot is not a substitute for later real-QEMU integration evidence.
