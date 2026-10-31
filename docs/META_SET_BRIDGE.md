# Meta planning for complete guest index sets

`--guest-set` sends Meta the entire current requirement and the guest's missing
revisions. The model chooses one missing record to index next. RAI-OS produces
the index, retains completed revisions and publishes an exact set only when all
current members are present. Model text cannot edit requirements or expand rights.

This is a bounded integration slice. The model now has a choice of work order,
but these tiny independent jobs do not establish an AI scheduling advantage.
The cache and completeness rules are deterministic guest mechanisms.

## Run on a fresh checkout

Install the pinned Rust toolchain/target and QEMU as described in the README.
Each participant must supply their own `MODEL_API_KEY` in the ignored root
`.env.local` or environment; follow [credential setup](META_BRIDGE.md#each-clone-needs-its-own-key).
Never edit the tracked template to contain a real key. HTTPS and credentials
remain on the host; neither the guest nor the model context receives the secret.

```sh
. "$HOME/.cargo/env"
sh scripts/meta.sh --graphics --guest-set fixtures/guest_set/v1/goals.json > target/meta-set-live.jsonl
python3 scripts/verify_set_bridge.py --check-live target/meta-set-live.jsonl
```

Omit `--graphics` for UART-only execution. Native graphics use the existing
RAI-OS framebuffer inside the QEMU Cocoa window, not the old host panel. The
bridge closes its owned QEMU instance after completion or handled failure.
Redirecting output saves JSONL observations; choose a new filename when retaining
multiple runs. The example's `target/` directory already exists after a build.

For offline development, with no key or API request:

```sh
sh scripts/meta.sh --graphics --guest-set-replay fixtures/guest_set/v1/goals.json fixtures/guest_set/v1/replay.json
python3 scripts/verify_set_bridge.py
python3 scripts/verify_set_bridge.py --release
python3 scripts/verify_set_bridge.py --graphics
python3 scripts/verify_set_bridge.py --release --graphics
```

The fixture requests three records at startup, adds checklist@1 at 12 seconds,
and replaces notes@1 with notes@2 at 22 seconds. These are scripted requirement
events, not file downloads, sensors or measured resource changes. Their times
start after coordinator startup; actual delivery is logged. Events are processed
between serialized UART requests. Slow provider responses can become obsolete;
the bridge does not fabricate completion or silently substitute a fixed policy.
A sufficiently delayed run may supersede an earlier set before it completes.

## Two different versions, two checks

The original `--guest-index` mode uses a per-record ticket as `state_version`.
The explicit `--guest-set` mode instead uses the current **set goal_version**.
It reuses the four-field proposal schema but never sends that generation straight
to `coord.propose`. The sequence is:

1. Scripted requirements enter through `coord.goal`. EL0 coordinator A retains
   the membership and attests it; the kernel commits the generation and derives
   missing members from its validated result slots.
2. The bridge decodes `coord.bundle.status`. Meta receives `index_set.required`,
   `index_set.missing`, the completed-job count and that generation. Only missing
   keys and byte lengths are authorized choices. No future events, source text,
   index contents or local paths are sent. Completed membership is the difference
   between required and missing; historical jobs outside the set may be cached.
3. The host validates the response against the **original** request, including
   operation, generation and membership in its original missing list. A current
   decline ends with a visible error; an obsolete decline is logged and ignored.
4. `coord.select <goal_version>:<key>` checks the generation inside RAI-OS before
   issuing any new per-record ticket. It rejects stale goals, nonmembers and
   already completed members without changing coordinator state or running B.
   This small gate runs at EL1 against attested requirement metadata. It does not
   move the provider or model into the kernel or claim a new isolated service.
5. Successful selection uses the existing EL0 observation path and returns a
   pending per-record ticket. The bridge forwards `coord.propose <ticket>:<key>`;
   EL0 still admits the job and the existing kernel validates its stream/result.
   No event ingress occurs between these two requests in this bridge. Even if
   requirements change through another authorized caller in that interval, the
   original ticket cannot execute against the replacement goal.
6. The bridge retrieves the exact `coord.result`, checks observed progress, then
   rereads missing members. Complete goals yield the exact guest `coord.bundle`
   JSON in a `guest_bundle` event. The host never builds a replacement artifact.

One model request can be in flight. Set mode allows eight requests per session,
eight goal events over at most 30 seconds, one to four records per set, and the
same six public immutable revisions. Eight calls allow five distinct jobs in the
reference walkthrough plus bounded obsolete responses; this is not unlimited
retry. Existing count/per-record modes retain their four-call budget. Each
provider request retains the existing 30-second timeout and owned-thread cleanup.
On budget exhaustion, invalid output or guest failure the bridge stops explicitly.
A completed historical publication is evidence for its own generation only.

`coord.select` is an additional constrained entry point, not a privilege system
for an untrusted UART peer. Trusted manual console callers can still set goals
and use legacy record commands. Invalid or cached selection does not increment
the shared issue counter. Valid selection does; repeated goal commands also
issue new generations. Read returned values rather than calculating tickets.
The standard walkthrough's generations are 1, 5 and 7 only in a fresh session
with five successful selections and no extra observations.

## Verification and interpretation

The bridge checks structure, identity and accounting. `verify_set_bridge.py`
independently reconstructs each requested snapshot, original proposal authority,
selection generation, per-record ticket, worker execution/IPC, cache membership
and publication. It calls the independent source/index checker for every output.
It rejects altered snapshots, fabricated artifact events and wrong job counts.
Saved logs are useful reproducibility evidence, not cryptographic proof of an
API request. Replay is explicitly labeled and never reported as live inference.

Tests cover a multi-record choice order, added/revised records, a stale proposal
for a key shared by old and new goals, cached goal rollback without inference,
obsolete/current declines, simultaneous requirements, invalid/denied/cached
choices, exhausted call budget, a truncated guest stream and cleanup. Direct
UART checks also change a goal between selection and proposal, reject malformed
frames, preserve state on rejection and invalidate generations across restart.

The native screen shows current members and real completed-worker counts. It
still does not display model reasons or provide touch navigation. There is no
persistent filesystem, general agent runtime, dynamic power management, full
review-pack acceptance, comparative efficiency result or finished phone OS.
Next define a meaningful constrained choice and compare it with a competent
fixed policy using the same events, worker and independent acceptance criteria.


## Verified October 30 run

The [live release log](evidence/2026-10-30-meta-set-live.jsonl) records five real
Muse Spark 1.3 requests with the native guest screen. The chosen order was
brief@1, empty@1, notes@1, checklist@1, notes@2. The guest published generations
1, 5 and 7 after 3, 4 and 5 total worker jobs. Each newly added/revised requirement
needed one new job; unchanged results retained their original bytes/indexes.
The independent full-log, source/index and IPC audit passed, followed by explicit
coordinator stop. There was no stale response in this live run.

Exact guest publications: [goal 1](evidence/2026-10-30-meta-set-goal-1.json),
[goal 5](evidence/2026-10-30-meta-set-goal-5.json),
[goal 7](evidence/2026-10-30-meta-set-goal-7.json).
Provider latencies were 1965, 2092, 1466, 1995 and 1870 ms, with 2442 total reported
tokens. The completion event was at 24319 ms including the intentional
22-second fixture schedule, boot and transport; stop occurs afterward. These
are observations of one run, not comparative throughput, energy or cost savings.

Separately, delayed offline replies for a record shared by old/new goals were
rejected with no worker execution: [debug replay](evidence/2026-10-30-meta-set-stale-debug.jsonl)
and [release replay](evidence/2026-10-30-meta-set-stale-release.jsonl). Both then
completed only the current set. The delayed fixture is not presented as live
provider behavior.
