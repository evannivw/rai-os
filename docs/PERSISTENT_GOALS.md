# Persistent goals: demonstration contract

Design agreed October 30, 2026, after the [measured policy pilot](POLICY_PILOT_RESULTS.md).
This is a plan, not a new working feature or a frozen benchmark. It supersedes
per-record AI ordering as the next demonstration hypothesis. Preserve the original
pilot, settings and all failed outcomes.

## Product hypothesis and novelty

Maintain an approved project review pack for offline use as inputs change,
without changing originals or exceeding explicit permissions and capacity.
The user establishes the objective once. Meta interprets changes requiring a new
plan; deterministic services execute, reuse, enforce limits and verify results.

The proposed distinction is observable continuity: keep the last verified result
inspectable while preparing its replacement; show why it is out of date; reject
obsolete plans; publish only a checked replacement. These are research aims, not
measured advantages. Reconciliation, transactional publication and model planning
are established ideas. Combining them here does not establish a world-first.

Official sources consulted October 30, 2026 describe adjacent capabilities:
[Apple App Schemas](https://developer.apple.com/videos/play/wwdc2026/240/) covers
semantic search and cross-app actions;
[Android AppFunctions](https://developer.android.com/reference/android/app/appfunctions/package-summary)
exposes app functionality for agent orchestration;
[AndroidWorld](https://research.google/pubs/androidworld-an-open-world-for-autonomous-agents/)
evaluates autonomous agents with programmatically checked tasks.
These sources do not establish an absence of persistent goals or recovery in
other products. We have not established that our proposed combination is unique
or rare. A custom kernel is an experimental foundation, not a prerequisite or
a demonstrated advantage over building the same feature on an existing OS.

## First bounded slice: keep the result correct

Use one objective, the existing two guest services, immutable public fixture
records and at most four required revisions. Keep the source-plus-index format
and independent checks from [INDEX_SETS.md](INDEX_SETS.md). Events are scripted
host inputs; caps are simulations. No real downloads, PDFs or phone filesystem.

1. Complete and verify brief@1, empty@1 and notes@1. Retain the exact artifact and
   its original goal generation as the last verified publication.
2. Replace the required notes revision with notes@2. Immediately label the older
   publication historical: still inspectable, but not current for the new goal.
3. Reuse brief@1 and empty@1, and prepare only notes@2. No model call is needed
   to decide whether identical immutable work can be reused.
4. Deliver an old-generation proposal while updating. Observe rejection and zero
   worker dispatch attributable to that proposal.
5. Check the replacement independently, recheck the current generation, then
   publish it. Show the actual changed entry and unchanged entries.
6. In a separate failure run, fail preparation or validation. Keep the earlier
   bytes unchanged and mark the active objective blocked, not complete.

This slice tests lifecycle mechanics, not semantic intelligence. It is the first
implementation gate; scripted decisions cannot represent live Meta decisions.

## Publication rules and ownership

The active target and last verified publication are separate state. A historical
artifact may remain readable while failing the current objective. READY requires
exact authorized membership, source bytes and indexes, plus a still-current goal
generation. Recheck freshness after validation and before committing publication:
a goal change during checking must invalidate the candidate.

Failed/partial work cannot replace the last valid artifact. Reuse requires the
same source revision, operation parameters and applicable permissions. A later
access revocation must also prevent access to restricted historical output;
preservation is not permission. Cancellation/revocation are later test cases,
not existing capabilities. Session retention does not imply reboot durability.

Existing: guest EL0 coordinator/worker, exact cache, generation admission,
complete-set output, native display and independent host checker. Previously
exported bundles already remain inspectable on the host; that is not a guest
publication-history service. Planned: explicit lifecycle, previous-publication
handle, validation/commit handshake and current-versus-historical native view.

The guest must own target admission and publication identity. The host carries
events, provider calls and the independent checker. Its validation acknowledgment
is trusted host input, not protection against a malicious host. At implementation,
document the actual owner and storage bound for every artifact and handle. A first
slice retaining bytes only on the host must say so. Keep pure lifecycle rules
separate from UART/provider/display adapters; reuse the current execution engine.

## Second slice: give Meta meaningful decisions

Create a separate compact public fixture requiring interpretation of a user goal
and authorized content: resolve the intended revision or an implicit dependency,
then propose a bounded set and a batch of allowed operations. Author expected
memberships, acceptable alternatives and ambiguous cases independently of model
answers. Ambiguity may require clarification rather than an invented fact.

Initial interpretation or a materially changed meaning can trigger planning.
Known revision replacements, completions, cache hits and identical events use
fast deterministic rules. Coalesce changes while a request is pending; reject
its superseded answer and plan from the newest snapshot. Do not run continuous
inference or put remote reasoning in the CPU scheduler.

Bind plans to goal, catalog and permission versions. Validate dependencies and
operations before admission, then recheck at execution and publication. Document
contents are untrusted data, never authority to change permissions or goals.
No arbitrary generated code, source deletion or model-controlled verifier.

First distinguish provider rejection categories and preserve available usage
safely. A provider error must be visible; no hidden fixed fallback earns a Meta
success. Count inference, transport, validation and any declared retries in total
time. Already admitted independent work may continue during a provider outage;
new offline model planning is not available. Batching benefit remains unmeasured.

## Evaluation and implementation sequence

Separate execution/recovery from interpretation. For execution, give both
policies the same explicit goal, events, batching, cache and verifier. For
interpretation, give a documented deterministic resolver and Meta identical
content and instructions; report the resolver's supported scope. Also measure
execution with the correct structured plan supplied, clearly labeling that
extra knowledge rather than presenting it as an end-to-end language competitor.

Freeze inputs, independent expected results, baseline capabilities, provider
settings, retries and metrics before comparing. Include quiet cases, routine
changes, semantic changes, ambiguity, unavailable resources and delayed/failed
answers; neither side sees future events. Retain failures. The existing tiny
index fixture cannot establish useful semantic-planning advantage.

Measure completed objectives, missing/wrong dependencies, recovery time to a
verified current result, availability/staleness of the old result, reused/new
jobs, user clarifications, model latency, usage coverage and limit violations.
Old readable data is not current correctness; simulated credits are not battery
or storage savings. If fixed rules are equally successful and cheaper, retain them.

Each following gate is a separate verified increment:

1. Implement publication lifecycle with the existing fixture. Verify stale
   proposals, reuse, failed updates and a goal change during checking in QEMU.
2. Expose observed current/historical/error states in the native guest view.
   Checked data controls state; replay must be labeled.
3. Add provider diagnostics and one validated batch proposal; capture a real
   Meta run, preserving the old per-action pilot unchanged.
4. Author semantic fixtures and independent acceptance checks, then freeze and
   run a new comparison including cases where deterministic rules are sufficient.
5. Rehearse the three-minute story: objective, initial result, change, obsolete
   plan blocked, verified replacement, measurements and limitations.

Follow [DEMO_PLAN.md](DEMO_PLAN.md), including the November 1 feature-freeze target
and November 4 submission deadline. If semantic planning is not demonstrably useful
at freeze, present the narrower recovery prototype and its actual Meta role;
do not claim the intended innovation is delivered. No phone drivers, additional
cores or generic agent framework are needed. Video, held-out evidence, organizer
clarification and submission remain pending.

## 2026-10-31: first lifecycle slice implemented

[Retained publication](PUBLICATION.md) now supplies guest-owned historical output,
fresh prepare/commit/reject tickets, failed-update retention and QEMU verification.
Storage runs in EL1 alongside the existing coordinator adapter; the checker is
trusted host code. Native lifecycle rendering and semantic batch planning remain
pending. The design above keeps its original date and broader planned scope.

## October 31: native lifecycle slice implemented

The [native publication view](PUBLICATION_DISPLAY.md) now renders real lifecycle
state and earlier-result availability. A Rust walkthrough exercises changes,
independent checks and a clearly labeled rejected check. Semantic planning and
comparative AI benefit remain pending; the design above is not a novelty claim.
