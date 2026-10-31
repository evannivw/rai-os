# Retained indexes inside RAI-OS

Implemented October 30, 2026. The guest EL0 coordinator can now request an exact
index from its retained EL0 worker and reuse previously completed immutable
revisions. The host transports commands. It does not calculate indexes or choose
cache hits. This is a bounded extension of the existing two-service runtime,
not a filesystem, dynamic process loader or general cache framework.

## Try it

From a fresh boot, run `sh scripts/desktop.sh` for the native guest screen, or
`cargo run --locked` for UART only. Enter:

```text
coord.start index
coord.observe notes@1
coord.propose 1:notes@1
coord.result
coord.observe notes@2
coord.propose 2:notes@2
coord.observe notes@1
coord.result
coord.stop
```

The first two proposals build different exact indexes. The last observation
returns `event=reused`, `completed=2`, `a_calls=2`, `b_calls=0`,
`cached_records=2`, `cache_hits=1`, `output_bytes=33`. No third worker job runs.
The result is again `10:1,7:1,a:1,at:1,pump:2,review:1`, for **notes@1**.
It does not substitute notes@2. The screen shows waiting, created and reused
states from the guest's validated results. Input still comes through the console;
there is no touch interaction or background file watcher.

Use the ticket returned by `coord.observe` when continuing an existing boot:
tickets survive stop/start, and even duplicate observations consume a new issue
number while preserving the active goal's ticket. The literal numbers above
assume a fresh boot and exactly the displayed sequence.

`coord.start` without an argument retains the previous counting behavior and
14-field report. `coord.start index` selects indexing for that session. Stop
before changing modes. The existing Meta `--guest` bridge still selects **count**; the new
[`--guest-index` mode](META_INDEX_BRIDGE.md) connects proposals to this index path.
This walkthrough uses manual proposals and makes no model-performance claim.

## Responsibilities and memory

- Coordinator A at EL0 retains the current goal, ticket, completion counter and
  a six-bit completed-revision set. It decides whether to await a proposal or
  reuse a completed revision. Duplicate observations preserve the current goal.
- Worker B at EL0 builds the canonical token/count sequence using the same
  constant-space cursor as standalone `record.index`. Its counter survives waits.
- The kernel routes scalar messages, validates bounded transfer and publication,
  and holds six optional result slots of at most 384 ASCII bytes each. It supplies
  committed metadata to A on input. The cache payload storage is at EL1; it is
  **not** a writable buffer shared between the programs or stored on A's stack.
- The view renders the published result. It does not select a job or cache hit.

Each record ID identifies a fixed immutable fixture revision. The six slots
require at most 2,304 payload bytes, plus metadata and runtime state. There is no
allocation, eviction, mutable-file identity, disk persistence or cross-boot reuse.
A future mutable source will need explicit identity/invalidation rules. Existing
private stack pages and exclusive user-memory lease are unchanged. Public
fixture data remains shared read-only, not confidential per-service storage.

Only a complete, canonical stream with matching record metadata, worker sequence,
proposal ticket and coordinator publication becomes a committed result. Both
services must be waiting again. `coord.result` is unavailable for a pending or
failed goal, even when another revision has a cached slot. Stop or a fatal session
error clears every slot, discards both continuations and releases their lease.
A failure preserves the previous confirmed completion count for diagnosis;
partial work cannot increase it. Restart begins with an empty cache.

## Scalar transfer

This mode reuses SVC 10/send, SVC 12/receive-wait and two-entry mailboxes. The
kernel binds sender identity. No new pointer or general shared-memory ABI exists.

| Direction | Kind | Scalar value |
| --- | --- | --- |
| A to B | 40 | Validated record ID |
| B to A | 41 | ID in high 32 bits, source bytes in next 16, tokens in low 16 |
| B to A | 42 | Zero-based chunk ordinal in high 8 bits, seven ASCII bytes in low 56 bits |
| B to A | 43 | Worker completion sequence in high 32 bits, output length in low 32 bits |

Chunk bytes are packed least-significant byte first. Only the final chunk can
contain trailing zero padding. The kernel rejects missing/repeated/out-of-order
chunks, zero-only chunks, nonzero bytes after padding, unsupported characters,
more than 384 bytes, wrong length, repeated headers or data after the end marker.
A partial stream with both tasks blocked fails as `coord_deadlocked`. The format
requires strictly sorted unique lowercase alphanumeric tokens, positive canonical
counts and their sum matching the header. `-` represents an empty index.
This validates shape and accounting; it does **not** independently recalculate
semantic correctness from source bytes. The Python checker does that separately.

In index mode SVC 13's x3 carries the record ID in the high 32 bits plus committed
bytes/tokens when available; otherwise the low bits are zero. A still decides
reuse using its own retained completed set. SVC 14 adds event 8 (`reused`). The
kernel accepts that event only for an observation of an existing committed slot
with matching metadata. A forged well-shaped empty result without a slot fails.

Each command allows at most 128 syscall attempts per task (attempt 128 is not
served), six timer expirations per task, and 192 trace pairs. Count mode retains
its old 16-attempt/96-pair limits. A slice is 20 ms and restarts after a syscall;
this is not an aggregate 120 ms command deadline. A normal index with N chunks
makes N+5 calls in A and N+3 in B; reuse makes two in A and zero in B.
Commands still execute synchronously and UART still polls.

Index reports append `operation=record.index`, `cached_records`, `cache_hits`
and `output_bytes` to the previous 14 fields. `coord.result` returns the same
artifact format as standalone `record.index`, suitable for the independent
[exact-content checker](RECORD_INDEX.md). Machine syntax is
`@rai1\t1\tcoord.start\tindex\n` and `@rai1\t2\tcoord.result\n`.
Extra arguments are rejected. Lifecycle/admission errors remain unchanged;
new errors include `coord_not_indexing`, `coord_result_unavailable`,
`coord_index_request`, `coord_index_order`, `coord_index_encoding`,
`coord_index_limit`, `coord_index_length`, `coord_index_format`,
`coord_index_changed` and `coord_cache_limit`.

## Verification

```sh
python3 scripts/verify_coord_index.py
python3 scripts/verify_coord_index.py --release
```

The harness checks all six indexes against independently read fixture bytes,
replays actual IPC traces, verifies revision reuse with no B dispatch, checks
stale/wrong/duplicate proposals, clears cache across restart and refuses old
tickets. It rejects mutations of reports and mismatched artifact revisions.
Separate QEMU images inject wrong chunk order/length, truncation, overflow,
post-end data, noncanonical text, forged reuse and five worker failure modes.
Stream failures are injected after a successful notes@1 job so cleanup must
remove a real previous cache entry as well as partial notes@2 output.
Native graphical captures cover pending/created/reused/error/stop, and duplicate
or status reads retain the reuse label. Two host unit tests check encoding,
padding, length/capacity boundaries and canonical format.

This is evidence that completed work can be reused correctly. It does not prove
AI superiority, lower battery consumption or performance against Android. The
[Meta index bridge](META_INDEX_BRIDGE.md) now connects proposals to this mode;
combined multi-record artifacts and fair baseline comparisons remain pending.

Actual [debug UART](evidence/2026-10-30-coord-index-debug.txt),
[release UART](evidence/2026-10-30-coord-index-release.txt),
[pending screen](evidence/2026-10-30-coord-index-pending.png) and
[reused screen](evidence/2026-10-30-coord-index-reused.png) are retained in Git.
These are real QEMU outputs, not mockups. Pending/reused PNGs matched byte for
byte between debug and release. Complete captures remain under target/verification.
