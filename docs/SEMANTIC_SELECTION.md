# Content-based requirement selection

November 1, 2026. First bounded semantic integration, not a completed complex
workload, held-out benchmark, general agent or demonstrated performance advantage.

Previously the host supplied an exact required set and Meta chose execution order.
Here Meta receives a natural-language goal, shared handoff rules and the exact
contents of six authorized fixture documents. It proposes the required membership
or requests clarification/declines. Deterministic code executes an approved set
through the existing EL0 worker and independent publication checker, with no
second model call per file.

## Concrete example and independent expected answers

“Prepare the Aurora pump handoff for the unit used before the switch to pump 8”
requires interpreting notes@1 (Pump A / pump 7) versus notes@2 (Pump B / pump 8).
Shared policy adds brief@1 and checklist@1 to the selected operational note.
Current/newest explicitly selects notes@2; an unspecified “right note” requires
clarification. Pump C has no authorized note, so decline rather than substitute.
A separate request about the eastern gate after wet weather selects appendix@1.

The six [public development cases](../fixtures/semantic/v1/README.md), exact
memberships and rationales were authored before model calls. The independent
expected.json is used by the Python demo harness only. It is not in the Rust
provider context or imported by Rust selection/execution code. The oracle checks
exact membership, decision and request identity BEFORE any QEMU execution. Wrong
meaning, omitted dependencies or inappropriate certainty fail the case, even if
the proposed operation would otherwise be permitted. No hidden fallback repairs
model answers. A refusal or format failure remains a failed observed outcome.

These are deliberately small development inputs (279 bytes of public source).
They demonstrate wiring and a few interpretation behaviors, not the highly
complex real-world scenario originally envisioned. The dependencies are a shared
explicit rule, not dependencies discovered in arbitrary documents. A future
baseline must receive exactly the same content and policy; it may implement these
rules efficiently. Do not claim uniqueness or superiority from passing these cases.

## Run the complete demo

From the repository root, with the [prerequisites](GETTING_STARTED.md) installed:

```sh
. "$HOME/.cargo/env"
# Oracle-supplied replay, no network/key; real QEMU work for accepted selections.
python3 -B scripts/semantic_demo.py --source replay --case all --output target/semantic-replay-new
# Real Meta; configure your OWN ignored .env.local per META_BRIDGE.md first.
python3 -B scripts/semantic_demo.py --source meta --case historical --graphics --output target/semantic-live-new
# All six development cases, at most one call per case; no retries/fallback.
python3 -B scripts/semantic_demo.py --source meta --case all --output target/semantic-suite-new
```

Output directories must be new. Defaults to release; --debug changes the build.
The native display renders existing actual publication states and closes after
completion/cleanup; this is not a paused visual tour. Clarify/decline boots no QEMU.
No production data is sent: only the goal and the six compiled public sources.
Unlike old metadata-only modes, this new interpretation mode sends source content.

Each case stores interpretation stdout/stderr, accepted selection, execution
stdout/stderr if admitted, and a report. The root manifest records source revision,
dirty state, binary/kernel and fixture hashes, settings and observation timestamp.
Compilation happens before measurement. Case end_to_end_ms includes interpretation,
independent intent gate, local file handoff, QEMU boot/execution/publication and
cleanup. It is not pure inference. No latency budget or comparative speedup is
asserted. Replays supply the correct answer from the oracle and must never be
presented as model intelligence. All six live outcomes are retained on failure.

## Boundaries and safety

The strict selection returns request_goal (exact original text), request_version,
catalog_version, permissions_version, decision, records and reason. Current catalog
and permissions versions are both 1; they describe static compiled public data,
not a dynamic catalog or revocation service. Content/permission changes require
version updates before accepting old proposals. Exact goal binding prevents reuse
across different goals even when callers reuse a request version.

Selections are at most 2048 bytes; reasons 1–240 bytes without control characters;
select allows 1–4 public records with no duplicate/conflicting document revisions.
Clarify/decline requires no records. Unknown fields, versions and operations fail
closed. Content is evidence and cannot grant file/shell/write permissions. There
is no generated executable code. Only the existing record.index work is available.

There are two separate correctness gates:

1. Rust semantic::validate checks identity, shape, bounds and permissions. It cannot
   prove arbitrary intent correctness. semantic::POLICY is the shared model policy;
   it is not an independent verifier or a claim of model reliability.
2. The demo's independent fixed-case oracle checks the intended selection; the
   existing independent artifact checker then verifies exact bytes/indexes before
   the guest accepts the publication acknowledgment. Neither oracle runs in the
   kernel, and the acknowledgment trusts the host as before.

Low-level --interpret proposes without executing. --interpret-replay accepts a
local selection fixture. --execute-selection takes a trusted host-approved
selection plus its original request and does NOT run the semantic oracle itself.
It logs this explicitly. The supported demo command above runs the oracle first.
Direct callers must obtain their own approval/meaning check; a CURRENT guest
publication alone means “checked output for the admitted set,” not “proved user
intent.” A malicious host remains outside this trust model.

The selection is bound to its host request first. Admission creates a separate
fresh guest goal generation; every job uses that generation and a fresh per-job
ticket. Execution reuses the batch worker/checker/publication functions and has
no further inference. Each case boots a fresh session. Continuous semantic
replanning and retained publications across semantic goal changes are not yet
connected; the existing explicit-goal batch loop remains a separate tested path.
This increment adds no kernel, syscall, unsafe or graphical code.

## Frozen probe and checks

New semantic mode only: muse-spark-1.3, reasoning_effort minimal, strict rai_selection
schema, max_tokens 1024, existing endpoint, one request per case, no retry/fallback.
Existing single/batch prompts and settings remain unchanged. Envelope diagnostics,
timeouts and secret handling are reused. Provider diagnostic and semantic_response
refer to the same call; count their usage once. Unknown usage stays unknown.

```sh
python3 -B scripts/verify_semantic.py --debug
python3 -B scripts/verify_semantic.py
python3 -B scripts/verify_semantic.py --debug --graphics
python3 -B scripts/verify_semantic.py --graphics
```

Offline checks include the six oracle-supplied cases, 20 pre-boot admission
rejections, four dummy-key provider cases per profile, and deliberate corruption
of semantic expectations, trace verdicts and artifacts. A permitted-but-wrong
selection is rejected by the oracle before execution. The native historical case
runs in both profiles. Forty-eight host unit tests and all-target Clippy pass in
both profiles. The existing 20 release batch cases remain a regression gate.

After committing these cases and source, run exactly six live requests (one per
case), preserve all outcomes and append results. This is a small fixed development
probe, not a performance comparison. Next freeze a meaningful fair resolver
comparison and connect a bounded goal change only if the remaining delivery time
permits. Protect November 2 evidence/rehearsal and November 3 video preparation;
November 4 deadline remains 20:00 Guatemala (internal target 18:00).


## November 1: observed six-call development probe

Ran the frozen suite once after pushing source 732051d34db0c388c7944538f831e0c01719b32f.
All six independently specified cases passed, with one live request each and no
retry, fallback, prompt tuning or expected-answer change. Four select outcomes
executed ten total real guest jobs across four fresh sessions and published
independently checked artifacts. Clarify/decline started no QEMU and did no work.
Every call returned HTTP 200, matching model, one assistant choice, finish stop,
no refusal/tools. All six reported usage; total 5335 tokens, counted once.

| Case | Decision | Selected records | Provider adapter ms | End-to-end ms |
| --- | --- | --- | ---: | ---: |
| explicit | select | brief@1, notes@2, checklist@1 | 4764 | 5342 |
| historical | select | brief@1, notes@1, checklist@1 | 4543 | 4684 |
| latest | select | brief@1, notes@2, checklist@1 | 5506 | 5644 |
| ambiguous | clarify | None | 2180 | 2191 |
| unavailable | decline | None | 3170 | 3183 |
| field | select | appendix@1 | 3135 | 3260 |

These are six public development successes, not a held-out accuracy estimate or
proof of superiority over a good deterministic resolver. The end-to-end intervals
include the independent intent gate, file handoff and execution/cleanup where
applicable; builds are excluded. Interpretation adds measurable delay. No fair
performance comparison has been run and no complex real-world workload is solved.
The historical prompt resolved notes@1 and handoff dependencies; the ambiguous
prompt asked for a revision choice rather than inventing one.

The exact manifest, selections, reports and complete stdout/stderr are preserved
under evidence/2026-11-01-semantic/live. Independent intent and execution audits
were rerun against the archived data; provider diagnostics and response summaries
agree on usage/latency. Native graphics were verified offline in both profiles;
this live suite was headless. Continuous semantic event recovery remains pending.
