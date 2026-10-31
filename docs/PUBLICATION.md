# Retained publication lifecycle

Implemented 2026-10-31 in the guest, using the existing exact index-set executor.
This is the first [persistent-goal](PERSISTENT_GOALS.md) slice, not semantic
planning or a new AI benchmark. No live model request was needed for this work.

## What changed

The active requirement and last committed publication are independent. A goal
change immediately makes the previous publication historical, while its exact
source/index output remains readable. A candidate replaces it only after a
trusted host acknowledges independent validation with the current candidate
ticket. Completeness alone does not commit it.

A failed worker clears the live cache and releases EL0 RAM, as before. The owned
publication survives that cleanup and stays readable even when another user
program subsequently uses that RAM. Explicit coord.stop clears publication
access as well as the session; restart/reboot does not preserve it. This is one
in-memory snapshot, not a filesystem, archive or durable transaction.

## Commands and validation handshake

Shell syntax is shown; @rai1 requests use a tab between operation and argument.

| Command | Effect |
| --- | --- |
| coord.publish.status | Report target_version, published_version, candidate_ticket, state, availability and reason. Zero versions/tickets mean absent. |
| coord.publish.prepare | Require an active, complete index set; return its exact rai-index-set-v1 artifact and issue a unique candidate ticket. Read that ticket from status. |
| coord.publish.commit TICKET | Trusted host acknowledgment: check ticket, current goal and completeness again, then retain the artifact as the publication. |
| coord.publish.reject TICKET | Report failed independent validation, consume this ticket and mark the target blocked; keep the earlier publication. |
| coord.publish.read | Return the exact last committed artifact with its original goal_version, including while historical or blocked. Fail if none exists. |

The host uses one serialized connection: prepare, capture the candidate ticket
and artifact, independently check against the expected requirement/generation,
then commit that ticket. On checker rejection, send reject instead. A state change
between checking and commit invalidates the old ticket. Re-preparation within the
same goal also invalidates the previous ticket. Duplicate acknowledgments fail.
Tickets share the existing monotonic boot-scoped counter, never wrap, and are not
reused after stop/start. They are freshness identifiers, not secrets or signatures.

An untrusted host could skip the independent checker and acknowledge directly.
The guest checks identity/completeness, not cryptographic proof of host validation.
The model must not receive or control this acknowledgment path. Current Meta
adapters are unchanged and do not invoke these commands yet.

Existing coord.bundle still exports a structurally complete set without committing
this new publication. Existing SET COMPLETE graphics describe that completeness,
not a publication acknowledgment. The new lifecycle is exposed through UART;
a native current/historical/blocked publication view is the next increment.

## Observable states

| State | Meaning |
| --- | --- |
| empty | No active publication target or retained result. |
| building | A target is active; no candidate awaits validation. This can include complete cached work not yet prepared. |
| validating | A complete candidate awaits a trusted verdict. |
| current | The committed artifact matches the active target and no failure is recorded. |
| blocked | Validation or the guest session failed; reason identifies which. |

Availability is independently none, current or historical. Historical does not
satisfy the active objective. Invalid input or an old ticket leaves valid state
unchanged. A successful new goal clears the earlier failure and pending ticket;
a new prepare can retry a rejected validation with fresh authority. Fatal worker
failure ends the execution session: output remains readable until stop, but a
restart currently clears it. Automatic restart with retained history is pending.

## Ownership and bounds

[publication.rs](../src/publication.rs) contains pure lifecycle rules. The kernel's
existing Coordinator owns this Store in EL1 memory. It holds one complete owned
Artifact: four bounded index Text buffers (384 bytes each, plus lengths), required
revision/goal metadata and readiness metadata. Source bytes remain linked immutable
public fixtures; serialization reads them without depending on the retired cache.
The host checker bounds the serialized artifact to 4096 bytes.

The candidate holds a ticket, not a second retained artifact. Its bytes derive
from the current immutable revision slots; all invalidating transitions cancel
its ticket. Commit copies the complete artifact before replacing the previous
snapshot. Single-core masked console execution prevents event interleaving within
commit; this is not a multicore atomicity claim.

The EL0 coordinator still attests required sets and the EL0 worker computes indexes.
Publication storage/admission here runs at EL1, not in a newly isolated service.
No new syscall, writable user mapping, unsafe block, heap or page allocation was
introduced. Shared large reply paths limit debug stack temporaries. Real native-
display builds were exercised in both profiles; this is not a formal stack bound.
Read permission remains the fixed public record allowlist. Dynamic revocation,
secure erasure, persistence and arbitrary files are not implemented.

## Reproduce and inspect

From the repository root, with the pinned Rust toolchain and QEMU installed:

```sh
. "$HOME/.cargo/env"
python3 scripts/verify_publication.py
python3 scripts/verify_publication.py --release
```

Each command runs normal, deliberately truncated-worker and native-display QEMU
builds. The script uses independent check_bundle.py validation before each
successful commit. It also mutates a checker input to demonstrate rejection;
that mutation is on the host, not a claim of spontaneous guest corruption.
No credentials or Meta API access are needed. The native display is exercised
for integration safety, not a new graphical publication demonstration.

Verified cases include the first publication; one-job notes@1 -> notes@2 update
with two unchanged indexes reused; old proposal rejection; corrupted checker
input; validation retry; repeated prepare; changed goal after checking; duplicate,
wrong and malformed tickets; four-record capacity; cached rollback; stop/restart
freshness; and output survival after fatal worker cleanup and user RAM reuse.

All new cases passed in debug and release. Existing bundle tests (including 45
valid subsets and native views) and boot/protocol tests passed in both profiles.
Formatting and Clippy passed for default/debug, native-display/debug and default/
release configurations. No AI performance, battery or security superiority was
measured. See the [captured normal run](evidence/2026-10-31-publication/normal-release.log)
and [fatal worker run](evidence/2026-10-31-publication/failure-release.log).
Raw captures include CRLF; preserve those bytes. Exact initial/updated artifacts
and remaining transcripts are in that same evidence directory. Reproductions
write fresh test outputs under target/verification; their timing may differ.
