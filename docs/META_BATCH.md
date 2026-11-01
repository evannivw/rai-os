# Bounded Meta batch proposals

Implemented November 1, 2026. This is the batch transport/execution gate of
[PERSISTENT_GOALS.md](PERSISTENT_GOALS.md), not the semantic workload or a new
performance comparison. The [original failed pilot](POLICY_PILOT_RESULTS.md)
remains unchanged. One plan can cover all currently missing work instead of
requiring a provider round trip per record. Fixed policies can batch too.

## Run from a checkout

Use the prerequisites in [GETTING_STARTED.md](GETTING_STARTED.md) and run from the repository
root. Rust, QEMU and Python 3 are required. No extra Python packages are needed.

```sh
. "$HOME/.cargo/env"
# No API key or network: explicit replay, real kernel/EL0 execution and checking.
sh scripts/meta.sh --guest-batch-replay fixtures/guest_batch/v1/goals.json fixtures/guest_batch/v1/replay.json
# Each participant configures their own ignored .env.local as in META_BRIDGE.md.
# This invokes Meta and can consume API credits.
sh scripts/meta.sh --guest-batch fixtures/guest_batch/v1/goals.json
# Add --graphics FIRST to render the existing native guest publication screen.
sh scripts/meta.sh --graphics --guest-batch fixtures/guest_batch/v1/goals.json
```

The fixture requests brief@1, empty@1 and notes@1. A successful run has one
batch_requested, one batch_response, three real worker completions, an independent
check and a batch_publication. It exits and closes QEMU after checking and
coord.stop. The native mode is not an interactive desktop or a paused video tour;
use [the publication walkthrough](PUBLICATION_DISPLAY.md) to inspect each state.

A scripted scenario has the same bounded format as [META_SET_BRIDGE.md](META_SET_BRIDGE.md):
1–8 ordered events, at_ms at most 30,000, 1–4 public revisions per requirement,
no conflicting revisions of one document. Events are delivered at host-loop
boundaries, not hardware interrupts or real mobile telemetry. Equal-time changes
are applied before planning. An identical goal currently creates a fresh generation;
a complete cached set requires verification/publication but no model request.

## Contract and admission

The model returns a strict object, at most 2,048 bytes:

```json
{
  "state_version": 1,
  "decision": "execute",
  "operations": [
    {"operation": "record.index", "record": "notes@1"},
    {"operation": "record.index", "record": "brief@1"},
    {"operation": "record.index", "record": "empty@1"}
  ],
  "reason": "Complete every missing revision."
}
```

An execute plan must contain exactly the original missing set, once each, with
1–4 operations. Only record.index is allowed. Decline must contain no operations;
a decline for the active goal is an error, never an undisclosed deterministic
fallback. Reasons must be nonempty, at most 240 bytes, without control characters.
Extra/duplicate JSON fields, unknown/denied/cached records, incomplete coverage,
duplicates, unsupported operations and mismatched versions are rejected.
All steps are validated before any job is dispatched: a valid prefix cannot
execute when a later step is invalid. A valid obsolete answer is discarded.
An invalid obsolete answer still fails closed instead of starting another call.

The original goal generation accompanies every action. The host checks observed
missing work immediately before dispatch; guest coord.select independently checks
the generation/membership and issues a per-record ticket; coord.propose checks
that ticket before the EL0 worker runs. The single transport does not ingest an
event between select/propose. Changes arriving during a job are reconciled before
the next job. Completed exact revisions remain reusable; the remaining obsolete
plan is discarded. Changes during a pending call coalesce into the next observed
snapshot; the in-flight request is not canceled, so its time and cost still count.

This is not an atomic all-or-nothing batch transaction. Successful earlier jobs
remain cached if a later job fails; only a complete checked artifact can publish.
No dynamic dependency graph, arbitrary program, new permissions or goal editing
is granted to the model. Catalog and permissions are the existing immutable
public fixture, not runtime-versioned catalogs or revocation services. Before
adding those, bind plans to their versions as well as the goal generation.

## Publication and ownership

After completion, the host requests coord.publish.prepare, feeds the exact guest
artifact to the existing independent Python checker, and checks newly due events
before committing the candidate ticket. A goal change during checking invalidates
that candidate. The guest retains the prior publication during updates; after a
successful commit the host reads back and compares the exact retained bytes.
Only this acknowledged state is CURRENT in the existing native display.

The checker validates exact source bytes, fixture hashes, membership, generation
and token counts. Its expected requirement comes from the observed guest goal,
not the model's proposed output. A failed check sends coord.publish.reject and
ends the run. The checker is trusted local code; the acknowledgment does not
protect against a malicious host. The CLI always stops the session on success or
error, which clears guest publication access. Retention is session memory, not
reboot durability. Captured publication JSON remains independently inspectable.

batch.rs owns pure plan admission. batch_bridge.rs sequences events/actions;
batch_publication.rs adapts the checker and publication handshake. Existing
set scenario ingestion, UART transport, typed async lifetime and provider envelope
diagnostics are reused. Generic payload parameters cover the two actual proposal
types; there is no agent framework. No kernel, syscall, unsafe or UI code changed.

## Provider settings and accounting

Model muse-spark-1.3, existing fixed HTTPS endpoint, reasoning_effort minimal,
stream false, strict rai_batch schema, max_tokens **1024**. The original single
operation modes retain their existing prompts/schema and 512-token cap. The new
cap allows a bounded array; it does not promise the provider will complete it.
There are no retries, redirects, endpoint overrides or hidden fallbacks. The
existing 30-second request timeout, 64 KiB envelope limit and redacted diagnostics
remain. At most eight calls and one pending call per session; a cached-complete
goal triggers none. No future events or source bodies are sent to Meta.

provider_diagnostic and batch_response summarize the SAME call; do not count
usage twice. Missing usage remains unknown. Total elapsed time includes boot,
provider delay, polling, execution, independent checking and publication up to
the finish event; CLI wall time also includes final cleanup. Provider latency is
adapter time, not pure inference time. The 10 ms host poll is not a guest scheduler
quantum. Existing early-teardown diagnostic limits still apply.

The current task already specifies the required files. The model chooses an order;
this does not demonstrate useful semantic interpretation or superiority over
ordinary deterministic batching. Fewer requests is a mechanism, not a measured
end-to-end improvement. The next gate requires independently authored semantic
cases and equally capable baselines with the same cache, batches and checks.

## Verification and planned live smoke probe

Passed: 45 host unit tests and all-target Clippy with warnings denied in both
debug and release; 20 offline QEMU batch cases per profile; two native-display
batch cases per profile; all 25 previous provider diagnostic cases per profile.
Release whole-set bridge (including 43 guest gate requests) and single-operation
Meta regressions also passed. Formatting and diff whitespace passed. New raw
batch traces are preserved in [evidence/2026-11-01-batch](evidence/2026-11-01-batch).
During test development, a delay-wrapper child outlived its killed wrapper; the
harness now records and terminates that real-QEMU child. This was test-process
cleanup, not a guest correctness failure.

```sh
python3 -B scripts/verify_batch_bridge.py
python3 -B scripts/verify_batch_bridge.py --release
python3 -B scripts/verify_batch_bridge.py --graphics
python3 -B scripts/verify_batch_bridge.py --release --graphics
python3 -B scripts/verify_provider_diagnostics.py --release
```

The batch suite independently reconstructs dispatch/publication authority from
wire traces and checks artifacts against fixture bytes. It rejects deliberately
corrupted summaries/artifacts/verdicts. Offline tests use replay or dummy-key curl
fixtures; they never represent live Meta results. Delayed real-QEMU output and a
delayed real checker exercise changes between jobs and during validation. No
fake successful guest result is injected. Fault tests also reject a truncated
EL0 worker stream and an explicitly failed checker. Artifacts under target/ are
local outputs, not automatically committed evidence.

Planned after source commit: exactly ONE live request with the checked-in
three-record fixture, these frozen settings, no retry/fallback. Preserve the
source commit, binary/kernel hashes, UTC observation timestamp, stdout, stderr,
latency, usage and independent audit whether it succeeds or fails. This is an
integration probe, not a before/after benchmark or a reason to discard old failures.
The observed result will be appended separately.
