# New semantic decisions in one retained RAI-OS session

November 2, 2026. This increment returns to actual guest execution after the
host-only offline package. It connects newly interpreted events to an existing
RAI-OS coordinator without rebooting QEMU. It does not enlarge the guest corpus or
claim a decision-quality advantage merely because inference is connected.

## What runs where

- RAI-OS owns the retained EL0 coordinator/worker contexts, admitted record jobs,
  exact index cache and checked publication lifecycle. The guest draws its native
  optional screen from observed execution/publication state.
- The Mac transports requests and Meta responses. The existing semantic interpreter
  receives one current natural-language request and the six authorized guest
  document contents. HTTPS, credentials and semantic request versions stay here.
- The independent host checker evaluates the fixed development-case meaning before
  dispatch; the artifact checker validates actual guest output before publication.
  Neither is general proof of intent, nor an untrusted-host security boundary.
- Controlled host input delivers the next event only after `stream_ready`. Model
  responses are generated then, not loaded as a prerecorded sequence. The model
  never receives future events or expected answers. The host test driver knows the
  fixture schedule; this is not spontaneous file watching or sensor telemetry.

The new Rust `--selection-stream` entry point accepts bounded newline-delimited
trusted-host approvals. It boots once, starts the coordinator once and shares the
existing sequence execution path. Each selected requirement is executed through
existing guest goal/ticket checks, EL0 jobs and publication verification. Both
policies use this same path, canonical record order and retained cache.

Clarification, decline or independent intent rejection schedules no guest job and
leaves the last accepted publication alone. The native display retains the LAST
ACCEPTED guest target during interpretation/clarification; it does not yet display
the new request text or clarification question. Those are in host logs. Do not
present the old visible output as satisfying an unresolved new request.

No new kernel code, syscall, driver, unsafe code or Cargo dependency. The prior
recorded replay is preserved as a distinct mode. Source data remains six immutable
linked fixture records (279 bytes), not a filesystem or general document store.
Retained state lasts for this session, not a reboot; explicit stop releases it.

## Frozen three-event development probe

The hypotheses must be separated:

1. Integration: freshly interpreted events can change actual guest execution in
   one session without discarding the previous output or repeating unchanged jobs.
2. Decision quality: Meta makes more correct decisions than a competent fixed
   resolver given the SAME information. Integration alone does not establish this.

The protocol in fixtures/semantic_session/v1 uses three already known goals from
the earlier comparison, with increasing request versions. They are deliberately
continuity checks, NOT new unseen reasoning tests or a harder benchmark:

1. Historical pump handoff: select brief@1, checklist@1, notes@1. Three guest jobs
   and one verified publication.
2. Ambiguous handoff: ask for clarification with no record selection. Zero jobs,
   no replacement, previous publication retained.
3. Explicit current handoff: select brief@1, checklist@1, notes@2. Reuse two exact
   indexes; run only notes@2. Verify and publish the replacement in the same guest.

The expected decisions and membership are frozen before live calls. Run rules then
Meta, one session each and one pass per policy. At most THREE real Meta requests,
unchanged muse-spark-1.3, minimal reasoning, rai_selection schema, 1024 output-token
cap. No retries, fallback or post-result tuning. Both get fresh QEMU sessions and
identical event ordering. Failed decisions are rejected before dispatch, recorded
as failures and do not prevent delivery of remaining protocol events. A transport
or guest-process failure aborts the session and preserves available raw logs.

Primary metric: correct decision and exact membership. Also report verified
publications, actual worker jobs, unchanged previous bytes, exact reuse, model
usage and elapsed time. Do not credit the model for deterministic cache savings;
both policies can obtain them. Single-pass fixed ordering is not a statistically
controlled timing study. Preserve all earlier negative comparison outcomes.

## Run and inspect

```sh
. "$HOME/.cargo/env"
# Real guest execution, explicitly oracle-supplied selections, no network/key:
python3 -B scripts/semantic_session.py --source replay --output target/session-replay-new
# Same lifecycle with the native QEMU display; explicit 1500 ms presentation pauses:
python3 -B scripts/semantic_session.py --source replay --graphics --output target/session-visual-new
# Frozen live probe: clean committed source and your own ignored .env.local required.
python3 -B scripts/semantic_session.py --source compare --output target/session-compare-new
# Re-audit saved decisions, requests and guest artifacts without inference:
python3 -B scripts/semantic_session.py --audit-only --output target/session-compare-new
```

Only new output directories are accepted. `--debug` selects the debug build.
`--source rules` or `--source meta` runs one policy explicitly. A successful compare
stores manifest.json and summary.json plus each policy's raw stdout/stderr, exact
input stream, per-event interpretation traces/provider metadata, report and audit.
Complete timing includes guest startup, interpretation, independent checks,
execution and cleanup; compilation is excluded. Graphic presentation delays are
reported and must not be used for latency comparison. Usage diagnostics and
semantic_response describe the same call; count it once.

The audit verifies source requests/selections, ordered ready/admission events,
nonexecution for ambiguity, exactly one boot and complete stream termination.
It reuses the earlier two-publication UART auditor through an explicit structural
projection, retaining ALL original guest operations, artifacts and historical
checks. Synthetic projection markers do not become saved execution evidence or
claims of recorded/live inference. Inference provenance comes from the separate
per-event raw provider traces. If a decision fails, it remains failed even when
later guest publication succeeds.

## Bounds and failure handling

Maximum four events per stream, 8192 bytes per input line checked DURING reading,
strict fields/typed selections, nonzero strictly increasing request versions,
matching goal/catalog/permission identities, exact allowed record IDs. Explicit
stop is required; unexpected EOF is an error. Invalid input cleans up the guest
session; it cannot execute an invalid suffix after a valid publication.

This is sequential event delivery at publication boundaries. Events arriving
DURING inference/jobs are not processed by this mode; concurrency, cancellation,
permission revocation and catalog mutation are not implemented here. Existing
batch/stale-ticket tests cover their own modes and must not be relabeled as new
semantic-stream capabilities. The stream trusts its host approval sender; a
caller bypassing the independent checker can submit an authorized but semantically
wrong selection. The Rust authority gate intentionally does not guess intent.

```sh
python3 -B scripts/verify_semantic_session.py --debug
python3 -B scripts/verify_semantic_session.py
python3 -B scripts/verify_semantic_session.py --graphics
```

Tests cover replay and rules in real QEMU, malformed/stale/unauthorized stream
suffixes, wrong-meaning provider responses, prior-result preservation and corrupted
measurement evidence. Fake provider tests use dummy credentials and are labeled.
Native screen startup/execution is tested; no new screenshot or visual redesign is
claimed. Keep final video, clean-checkout rehearsal and submission time protected.
Deadline November 4, 20:00 Guatemala; internal target 18:00. October 26 kickoff /
October 27 build-period eligibility is still unresolved.
