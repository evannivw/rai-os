# Meta bridge for the guest coordinator

The --guest mode connects the persistent EL0 coordinator to the existing Meta
HTTPS adapter. Goal ownership and proposal admission remain inside RAI-OS. The
host supplies timed observations, forwards the guest's pending ticket/record to
Meta, validates the provider response format, and submits a typed proposal.
This mode does not modify the kernel or the earlier --coordinate host mode.

## Run from a checkout

Follow [the per-clone key setup](META_BRIDGE.md#each-clone-needs-its-own-key).
Each participant supplies their own MODEL_API_KEY in the ignored root .env.local
or environment. Never edit the tracked empty template to insert a real key.
HTTPS and credentials remain on the host; no credential is sent to QEMU.

```sh
. "$HOME/.cargo/env"
sh scripts/meta.sh --guest fixtures/guest_bridge/v1/observations.json
```

This starts one QEMU connection and one coord.* session. The fixture observes
notes@1 at 0 ms, repeats it at 6000 ms, observes notes@2 at 8000 ms and repeats it
at 12000 ms. Timing starts after guest service startup; total session duration
also includes boot and cleanup. Scheduled times are delivery targets: events are
polled and delivered between serialized UART exchanges, with actual delivery
times logged; this is not a hard real-time event source. Events are scripted, not hardware/file watchers.
The run permits at most four model requests. Provider failures stop visibly;
there are no hidden retries or fabricated fallback completions.

Offline version, with clearly labeled replay proposals:

```sh
sh scripts/meta.sh --guest-replay fixtures/guest_bridge/v1/observations.json fixtures/guest_bridge/v1/replay.json
python3 scripts/verify_guest_bridge.py
python3 scripts/verify_guest_bridge.py --release
```

Replay mode loads no key and makes no provider call. The replay file contains
bounded delays and proposals; it is not evidence of Meta inference. A live run
can be slower or choose to decline, and may encounter real stale responses.

## Who owns each decision

The kernel returns an actual coordinator report after every observation. The
bridge decodes its ticket, current record, state and observed completion; it
never computes a replacement goal version. Only awaiting_proposal starts an
inference request. Duplicate/completed observations therefore cause no new call.

The model sees only the current guest-required record's public metadata and
original ticket. It receives no future event schedule, forbidden bytes, secret,
local path or shell operation. The only operation is record.count or decline.
No host result cache changes the guest's decisions. Current guest state retains
one goal/result, not a multi-goal history; revisiting a replaced record creates
new work and is sent to Meta again.

While inference runs on a joined host thread, due observations still travel to
the guest. A valid response must echo the original request ticket. The host
validates that correlation and the existing operation/key allowlist, then sends
coord.propose with the original ticket even if a newer goal has arrived. The
guest rejects stale proposals; its subsequent status must confirm unchanged
goal/results and zero worker execution. Only this ordinary stale case proceeds
to a new bounded request. Wrong keys, malformed/unsupported proposals or guest
faults stop the run. An obsolete decline is ignored; a current decline finishes
with status=declined and the still-pending goal recorded, not success.

A valid completion must match the original ticket, current record, byte count,
EL0 service reports and a one-step completion advance. The independent Python
checker additionally computes exact tokens from fixture bytes and replays the
real IPC trace. Host shape checks alone do not prove semantic correctness.

## Bounds and cleanup

- One provider request in flight and at most four per session.
- One QEMU connection; serialized UART requests and up to eight observations.
- Nondecreasing event times within 30 seconds; keys restricted to six public
  immutable fixtures. Observations replace the current goal, including an older
  revision explicitly supplied later. This differs from the older host mode's
  latest-revision filtering; no hidden host filtering occurs here.
- Replay delays at most 2000 ms; bounded 8 KiB scenario / 16 KiB replay files.
- Existing 512-byte goal, 2048-byte proposal and 64 KiB HTTP limits apply.
  curl uses system TLS, 10-second connect / 30-second total HTTP timeouts, a fixed
  endpoint/model, no redirects/retries, and stdin for the authorization header.
- Provider threads are joined, not abandoned; shutdown does not cancel inference.
  The guest session is stopped on success, decline and handled failure, followed
  by QEMU child teardown. Cleanup failure is separately reported. No new signal
  handler or crash-recovery guarantee is introduced.
- The kernel's existing per-command budgets, private stacks and exclusive user
  RAM ownership remain unchanged. No energy/performance superiority is implied.

## Evidence format and audit

JSONL stdout distinguishes source=meta/replay, guest boot, scheduled observations,
each exact UART exchange, model context/latency/usage, forwarded proposals,
declines, cleanup and final status. It omits credentials and raw HTTP bodies.
An outer process success may still carry status=declined; inspect the final event.
The final snapshot is the last guest report before explicit coord.stop.

```sh
python3 scripts/verify_guest_bridge.py --check-live path/to/capture.jsonl
```

The checker reconstructs tickets and goals from the supplied fixture and actual
UART responses. It validates model contexts, current/stale admission, counts,
IPC traces, completion counters, single boot and stop. Altered contexts, final
state, model-call counts and token counts must fail. Saved logs are auditable
records, not cryptographic proof of a remote provider call.

Offline checks cover normal revisions/duplicates, a delayed stale response
rejected by EL0, obsolete/current decline, empty input, returning to an earlier
record, malformed/denied/injected proposals, wrong-key rejection inside the guest,
invalid scenario/replay bounds and exhaustion of the four-call budget. All use
real QEMU and finish with explicit session cleanup when boot succeeded.

## Implementation and remaining demo work

- guest_bridge.rs: timed transport, session lifecycle and JSONL evidence.
- guest_report.rs: strict report decoding and observed-transition checks.
- decision_source.rs: shared bounded provider/replay thread ownership.
- provider.rs / config.rs: existing HTTPS adapter and secret loading, unchanged.
- guest coordinator/worker: existing EL0 policy and execution, unchanged.

This closes the integration path, not the useful-workload comparison. The model
currently proposes the single available count operation for a specified record;
this small case is also solvable by deterministic code. The next delivery target
is a useful artifact and independent verification, followed by fair measurements
and the video. See [DEMO_PLAN.md](DEMO_PLAN.md).

## Verified October 30 evidence

The [live debug capture](evidence/2026-10-30-guest-meta-live.jsonl) completed
notes@1 (30 bytes/7 tokens) and notes@2 (44 bytes/9 tokens), with one guest boot,
two actual Muse Spark 1.3 requests and explicit guest stop. Duplicate observations
caused no further inference. API latencies were 2161 ms and 1584 ms; reported
total tokens were 462 and 491. Total session duration was 12121 ms, including
the fixture's intentional waits, boot, transport and cleanup. No monetary cost,
energy saving or comparative performance benefit was measured.

The live run had no stale response. Guest-side stale rejection was separately
verified using delayed replay in [debug](evidence/2026-10-30-guest-meta-stale-debug.jsonl)
and [release](evidence/2026-10-30-guest-meta-stale-release.jsonl): the first proposal
was sent with its original ticket and rejected by EL0, then the current revision
completed. Both kinds of evidence passed the independent log/count/IPC auditor.
