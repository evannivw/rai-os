# Bounded event-driven Meta coordinator

Implemented October 29, 2026. This first host coordinator retains observations,
reacts to released record revisions, and requests another model decision only
when work is pending. It builds on [META_BRIDGE.md](META_BRIDGE.md). The kernel,
EL0 worker and UART ABI are unchanged. It is a bounded demonstration of an
observe/propose/validate/execute loop, not a general OS optimizer.

## Run with your own key

Follow the [credential setup](META_BRIDGE.md#each-clone-needs-its-own-key). No key
is distributed with the repository. Then, from the repository root:

```sh
sh scripts/meta.sh --coordinate fixtures/coordinator/v1/revisions.json
```

This performs real Meta calls. The checked-in scenario supplies synthetic release
events using actual monotonic elapsed time: notes@1 at 0 ms, its duplicate at
6000 ms, notes@2 at 8000 ms, and its duplicate at 12000 ms. These are not file
watcher notifications, phone sensors, measured battery changes or real downloads.
The fixture uses existing immutable record bytes; it does not modify source files.

The goal is fixed: maintain a token count for the latest released notes revision.
Expect English JSON events showing release, decision request/response, dispatch,
observed result and termination. Both revisions may be processed if the first
proposal arrives before the update. A slow first response may be discarded and
only the current revision processed. The final outcome requires an observed
count for the latest revision; it does not require spending work on obsolete data.

## State and responsibilities

- `host/meta/src/coordinator.rs`: pure released-revision state, result retention,
  dynamic state versions, duplicate suppression and admission policy.
- `host/meta/src/loop_runner.rs`: timed fixture delivery, one pending provider
  thread, one reusable QEMU connection, bounded dispatch and event output.
- `host/meta/src/provider.rs`: existing HTTPS boundary, now accepting a snapshot
  of released metadata, the required record and actual completed results.
- `domain.rs`: shared proposal validation and observed guest-result checks.

State starts at version 0. A newer release increments it; an observed result
increments it again. Duplicate or older release events do not change it. The
first observed count is retained when notes@2 arrives; it cannot satisfy notes@2.
At most two immutable results are stored, and completed exact revisions are never
admitted again. Snapshots expose only already-released metadata and observations.
The provider does not receive the scenario's future events or source bodies.

The current deterministic policy fixes which revision is required. Meta proposes
one permitted operation against that context. Correct deduplication, stale-result
handling and permissions come from code. This small case is also solvable with
ordinary rules; it establishes integration/lifecycle behavior, not an AI advantage.

## Events while inference is pending

The HTTPS request runs on one host thread. The coordinator keeps applying due
fixture events while waiting, so model latency does not freeze the event clock.
Each response is bound to its captured state version. If state changed, the
response is discarded before UART dispatch and a new snapshot is considered.
An outdated call still consumes the proposal budget and provider usage.

After a response, due events are applied again before validation and after any
QEMU boot. An operation already legally dispatched is allowed to finish. Before
recording its result, due events are applied once more. Its result is retained by
immutable revision, but does not mark a newer revision complete. There is no
mid-operation cancellation or event delivery inside the guest batch.

QEMU boots lazily on the first admitted action and remains connected for this
session. Each guest operation still creates and retires a bounded client/worker
batch. A persistent host connection is not a persistent guest service. The guest
UART still polls; no battery saving or hardware idle behavior is claimed.

## Bounds and failure behavior

- Scenario: at most 8192 bytes, 1-8 events, sorted nonnegative offsets no later
  than 30000 ms, only notes@1/notes@2 releases. Unknown fields/events are rejected.
- At most four proposal requests, including discarded stale requests; only one
  provider request may be outstanding. This is not a token/dollar budget.
- Existing provider limits remain: 512 output tokens including reasoning,
  10-second connect timeout, 30-second request timeout and 64 KiB response cap.
- No model calls occur merely because time passes or a duplicate arrives.
- Wrong-version, unauthorized/future/obsolete/already-completed record proposals
  fail before execution. HTTP, protocol or guest failures stop the session;
  there is no hidden retry or canned recovery result.
- Explicit decline stops with status=declined and a pending snapshot. It never
  claims completion. Future scheduled events may then remain unapplied.
- Success requires all events to be consumed, no pending proposal, and an
  observed result for the current revision. Process exit loses host state.

Normal cleanup joins the provider thread and reaps QEMU. An in-flight provider
request is not cancelled on shutdown; its adapter timeout bounds that wait.
There is no single global session deadline. This does not add signal handling,
persistent state, multiple agents, resource planning, arbitrary workloads or
coordination inside the OS. The older priority coordinator and panel remain separate.

## Offline replay and verification

```sh
python3 -B scripts/verify_coordinator.py
python3 -B scripts/verify_coordinator.py --release
python3 -B scripts/verify_meta.py
```

The first two commands build and run actual QEMU with explicitly labeled replay
proposals. They check two-revision completion, duplicates, reuse of one QEMU
connection, an event received during delayed inference, stale response discard,
future/wrong-version rejection and decline without boot. Completed guest counts
are checked independently against exact fixture bytes. A separate history audit
reconstructs release state, versions, snapshots, admission and completion; altered
history/final state is rejected. Pure tests also cover a result superseded during
execution and release ordering/size constraints.

To replay your own bounded proposals:

```sh
sh scripts/meta.sh --coordinate-replay /path/to/events.json /path/to/proposals.json
```

Events use the same schema as the checked-in fixture. Proposals are a JSON array
of `{ "delay_ms": 10, "proposal": { ... } }`, at most four entries and 16384 bytes.
Each proposal has the existing four fields and the expected dynamic state version.
Delay is bounded to 2000 ms. Unused or missing replay proposals are errors. Replay
never loads a credential or contacts Meta; source=replay/model=null labels it.
Counts labeled model_calls in a replay log are proposal attempts, not provider use.

## Actual evidence

The [live run](evidence/2026-10-29-coordinator-live.jsonl) used Meta Muse Spark 1.3
and debug host/guest builds. It observed notes@1 = 30 bytes/7 tokens and notes@2 =
44 bytes/9 tokens, with one QEMU boot, two model calls and no duplicate execution.
The second request included the first result. Measured API latencies were 2311 ms
and 1648 ms; provider-reported total tokens were 554 and 538 (1092 total).
The session ended at 12003 ms, including the intentionally scheduled waits.
This is not throughput, a benchmark comparison or measured energy efficiency.

The stale-response path was exercised using delayed offline proposals, separately
from the live run: [debug](evidence/2026-10-29-coordinator-stale-debug.jsonl),
[release](evidence/2026-10-29-coordinator-stale-release.jsonl). Those runs applied
the notes@2 event while the first proposal was pending, discarded the old proposal,
and executed only notes@2. They must not be presented as additional live Meta calls.

```sh
python3 -B scripts/verify_coordinator.py --check-live docs/evidence/2026-10-29-coordinator-live.jsonl
```

This audits internal evidence consistency and exact guest counts. Saved logs
alone cannot cryptographically prove a network call. This checker also does not
replace the full review-pack artifact verifier or establish semantic intent safety.

Next: expand from this small revision loop to a useful multi-step workload with
explicit constraints, independent result verification and a comparable deterministic
policy. Add new kernel mechanisms only when that workload needs them.
