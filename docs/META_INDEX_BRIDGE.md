# Meta proposals for retained guest indexes

The `--guest-index` mode connects Meta proposals to the existing retained EL0
index coordinator. RAI-OS produces and caches exact artifacts; the model proposes
one allowed action for the current goal. This extends the useful-output path
without changing the kernel, worker, native renderer or default count modes.

## Run

Each checkout needs its own `MODEL_API_KEY` in the ignored root `.env.local` or
environment. Follow [the credential setup](META_BRIDGE.md#each-clone-needs-its-own-key).
Do not put a real key in the tracked template. The key stays on the host.

```sh
. "$HOME/.cargo/env"
sh scripts/meta.sh --guest-index fixtures/guest_index/v1/observations.json
# Same guest services with their native QEMU screen (macOS Cocoa):
sh scripts/meta.sh --graphics --guest-index fixtures/guest_index/v1/observations.json
```

The fixture observes notes@1 at 0 ms, notes@2 at 8000 ms, returns to notes@1 at
16000 ms and repeats it at 20000 ms. The last interval leaves the reused result
visible before session cleanup. Scheduling starts after service startup, and
actual delivery times are logged. Events are scripted observations, not real
file downloads or hardware sensors. A slow provider can cause stale proposals;
the run does not manufacture the expected results or retry silently.

Without credentials or network access:

```sh
sh scripts/meta.sh --graphics --guest-index-replay fixtures/guest_index/v1/observations.json fixtures/guest_index/v1/replay.json
```

Replay proposals are explicitly labeled `source=replay`, with no provider usage.
They exercise real QEMU execution but are not evidence of Meta inference. Omit
`--graphics` for UART-only automation. The graphical flag must come first and is
supported only for these two index modes. The wrapper builds the display-enabled
kernel separately; the bridge confirms its framebuffer-ready boot message and
fails visibly if given a non-display kernel. It closes its own QEMU instance
when the scripted session finishes. No new touch/input system is introduced.

## The complete path

1. The bridge starts `coord.start index` and sends an observation to the guest.
2. EL0 coordinator A returns either a pending ticket/record or a completed cache
   hit. Only a pending goal starts a provider request.
3. Meta receives that exact public record's metadata and original ticket. Its
   schema permits `record.index` or `decline`, never `record.count`, shell text,
   arbitrary files or new capabilities in this mode. No source bodies, local
   paths, credentials, future schedule or cached index text are sent to Meta.
4. The host checks response shape, operation permission and original-request
   correlation, then forwards `coord.propose <ticket>:<record>`. Current-goal
   admission remains in EL0. A late proposal reaches the guest and is rejected
   there; the bridge verifies that rejection did not execute the worker.
5. A current proposal runs the existing EL0 worker and bounded scalar transfer.
   The bridge validates the completed report, requests `coord.result`, checks
   artifact identity/shape/accounting, then emits a `guest_index` evidence event.
6. Returning to a previously completed revision reuses the guest's own stored
   artifact. No new model request or B execution is needed for the cache hit.
   The host retrieves it from RAI-OS again and checks that it exactly matches its
   earlier observed artifact. The host's history is an audit record, not the
   cache that supplies results or decides which jobs run.

If a different inference request was already in flight when a cache hit arrived,
that request is still joined and its late proposal is rejected by the guest.
Reuse prevents a **new** inference request; it does not cancel an existing one.
A current decline ends with `status=declined`, not a claimed completion. An
obsolete decline is ignored while the current goal continues.

Default `--guest`, `--guest-replay`, `--coordinate` and single-proposal modes
remain count-only. Adding index support does not widen their action allowlist.
The existing model/endpoint, HTTPS validation, key-loading rules, response limits
and no-retry policy are retained.

## Evidence and verification

The JSONL stream includes the selected operation/display mode, each exact UART
exchange, decision context, source, latency/usage, forwarded proposal and cleanup.
`guest_index` includes the original guest artifact, ticket, record and origin
(`executed` or `reused`). It is emitted only after validation. Duplicate observations
do not emit another artifact or request inference. The final report is the last
guest state before explicit stop; stop clears the guest cache and contexts.

```sh
mkdir -p target/verification
sh scripts/meta.sh --guest-index fixtures/guest_index/v1/observations.json > target/verification/meta-index.jsonl
python3 scripts/verify_guest_index_bridge.py --check-live target/verification/meta-index.jsonl
python3 scripts/verify_guest_index_bridge.py
python3 scripts/verify_guest_index_bridge.py --release
python3 scripts/verify_guest_index_bridge.py --graphics
python3 scripts/verify_guest_index_bridge.py --graphics --release
```

The independent auditor reads the fixture sources, verifies their hashes and
exact token frequencies, replays kernel IPC traces, reconstructs goal tickets,
checks model-call accounting and requires exact artifact equality on cache hits.
Altered contexts, counters, revisions, output text or missing artifact events
must fail. Host shape checks alone do not establish semantic source correctness;
saved logs alone are not cryptographic proof of remote provider access.

Offline real-QEMU cases cover revision reuse/duplicates, delayed stale replies,
reuse during a different pending inference, obsolete/current declines, empty
and larger outputs, wrong/denied/injected records, mode-confused operations,
four-call budget exhaustion and a truncated guest stream. Every handled failure
after boot attempts explicit stop; partial output is never a published artifact.
Native-mode checks confirm the display-enabled boot and exercise the same bridge
against the real Cocoa QEMU window.

Bounds remain one provider request in flight, four per session, eight observations
within 30 seconds, six immutable public revisions and 384 bytes per guest index.
Provider calls time out after 30 seconds; pending threads are joined. Events are
processed between serialized UART requests. These are bounded demonstrations,
not a general asynchronous OS event service or a hard real-time guarantee.

## What this demonstrates

A real model proposal can lead to an independently checkable result produced
inside the OS, while the guest enforces freshness and avoids repeat work.
The model currently chooses between one available action and declining; this is
also solvable with deterministic code. Cache reuse is deterministic guest policy,
not evidence that AI outperforms another OS. No battery, throughput, latency or
security superiority is claimed. Combined multi-record results, a fair baseline,
more meaningful model choices, the final video and submission remain pending.


## Verified October 30 evidence

The [live release capture](evidence/2026-10-30-meta-index-live.jsonl) used the native
QEMU screen and two real Muse Spark 1.3 requests. notes@1 and notes@2 produced
exact guest indexes; returning to notes@1 produced `origin=reused`, with two
completed jobs, one cache hit and zero B calls on that observation. Its later
duplicate triggered neither another model request nor another artifact event.
The session stopped explicitly. The independent content/IPC/log audit passed.

Provider latencies were 3350 ms and 1606 ms; reported usage totaled 438 and 399
tokens respectively. Total session duration was 20274 ms, including the fixture's
intentional 20-second schedule, boot, transport and cleanup. These are observations
from one run, not comparative benchmarks or measured energy/monetary savings.
There was no stale reply in that live run. Delayed-response rejection was checked
separately with replay in [debug](evidence/2026-10-30-meta-index-stale-debug.jsonl)
and [release](evidence/2026-10-30-meta-index-stale-release.jsonl). A separate
[release replay](evidence/2026-10-30-meta-index-reuse-inflight-release.jsonl) shows
reuse while another inference is pending, followed by rejection of its obsolete
proposal. Replay evidence is not described as a live model decision.
