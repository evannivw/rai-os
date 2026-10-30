# First useful artifact: an exact record index

Implemented October 30, 2026. This is a small correctness increment toward the
review-pack contract, not the complete pack or an AI performance comparison.

## Try it

```sh
. "$HOME/.cargo/env"
sh scripts/index.sh notes@1 target/notes-v1.index
python3 scripts/check_record_index.py target/notes-v1.index --record notes@1 --find pump
```

The output path must not already exist. The first command builds the kernel and
Rust exporter, boots real QEMU, requests `record.index`, saves the guest's exact
response and independently checks it. It does not require an API key. The second
command verifies again before looking up a complete token (case insensitive).
Example actual guest output:

```text
record=notes@1; bytes=30; worker=EL0; state=completed; index=10:1,7:1,a:1,at:1,pump:2,review:1
```

The search prints `pump: 2 occurrence(s)`. This lookup runs on the host against the
verified artifact; the index itself was constructed inside RAI-OS at EL0. To see
raw guest output interactively, run `cargo run --locked` or `sh scripts/console.sh`
and enter `record.index notes@1`, `record.index notes@2` or `record.index empty@1`.
A finished empty index is `index=-`, not a missing result.

## Boundaries and implementation

- `records.rs`: public immutable fixture IDs, token normalization and selection.
  Tokens are maximal ASCII alphanumeric runs, lowercased and sorted by ASCII
  lexicographic order. Each distinct token has its exact occurrence count.
- `user/index_program.rs`: linked Rust EL0 application that walks those tokens,
  formats `token:count` entries and submits text through existing write/exit SVCs.
- `record_index.rs`: use case that invokes the application and publishes a result
  only after normal exit with zero code and no rejected syscalls. A timeout or
  failed invocation publishes an error without its partial output.
- `user/`: existing single-program runtime, exclusive user-RAM lease, checked
  copies, 100 ms absolute budget, and register/context recovery. The transition
  now passes one validated scalar ID in x0; legacy entries still receive zero.
- `examples/index_record.rs`: Rust host UART adapter/exporter. It preserves the
  returned text; it does not build the index. Existing files are never overwritten.
- `scripts/check_record_index.py`: independent acceptance checker. It takes the
  expected revision separately, verifies immutable fixture length/SHA-256, then
  uses Python regex and a Counter to reconstruct the exact canonical result.
  It rejects omissions, extras, wrong counts/revisions/order/case and malformed
  output, rather than trusting `state=completed` or an exit code.

The source is one of six synthetic public records embedded in the ELF, each at
most 128 bytes. This is not a filesystem, download handler, arbitrary-file reader
or storage persistence. The denied record is not linked. All allowed records are
shared user read-only data; the allowlist is not per-record confidentiality.

Each SVC write remains limited to 64 bytes. Cumulative output is 384 bytes for
this program only; hello/spin retain their 64-byte limit. At most 64 maximal tokens
fit in a 128-byte source. Even without duplicate elimination, token bytes plus
commas/colons and two-digit counts fit under 384 bytes. No heap or mapping change
is required. Only the selected built-in index program gets the larger bound.

The indexer currently runs as one synchronous EL0 invocation. It cannot run while
the persistent worker/coordinator owns user RAM: `user_memory_busy` is intentional.
The Meta bridge still proposes counts, not indexes. The index is not yet a retained
coordinator job, multi-record index, review-pack manifest or final READY state.

## Measurement and optimization sequence

The exporter reports observed host boot milliseconds and request microseconds.
Request time includes UART, guest work and host scheduling; it is not CPU-only
work, battery use or inference latency. One October 30 debug run observed 91 ms
boot and 7880 microseconds for notes@1. This single tiny run is a smoke measurement,
not a benchmark or comparison. Build time is outside those two measurements.

This first reference implementation rescans the bounded record to select the
next lexicographic token and uses one-byte writes for simple checked transfer.
These are obvious candidates for improvement (bounded sorting and buffered writes),
not a deliberately weak baseline for an AI win. Before comparing policies, improve
and freeze a reasonable deterministic executor shared by both sides. Increasing
input complexity must preserve exact output checks and include all overhead.

Next useful-work steps: retain validated results by exact revision, combine several
records into a current-goal index, invalidate only affected results, then add
interruptions and budgets. Both policies must have the same cache/operations.
The full [review-pack contract](DEMO_ACCEPTANCE.md) remains the target.

## Verification

```sh
python3 scripts/verify_record_index.py
python3 scripts/verify_record_index.py --release
python3 scripts/verify_index_export.py
```

Each profile boots three images: normal, partial-output/nonzero-exit, and spin.
Normal checks cover all six allowed records twice, direct shell use, changed
revision and empty input. Tests mutate outputs to challenge the independent
checker, reject denied/malformed requests, exercise memory-owner contention and
verify recovery with user.run, timer checks and allocator/map checks. Failure
images return errors with no partial index. The exporter tests exact search,
non-overwrite, denied/malformed keys, missing ELF and corrupt/oversized/non-ASCII
artifacts. Logs and generated artifacts go to ignored `target/verification/`.

The checker establishes exact content for the selected immutable record. A saved
text file alone does not authenticate which CPU produced it; actual execution is
established separately by real-QEMU tests and linked EL0 entry checks. An exported
file remains untrusted until the checker succeeds. An interrupted disk write may
leave an incomplete file that must be rejected, not treated as a committed pack.

## Retained execution

`coord.start index` now runs the same index serializer through the retained EL0
worker. Completed immutable revisions can be reused without another worker job;
`coord.result` exposes the identical independently checkable artifact format.
Standalone `record.index` remains available outside retained sessions. See
[RETAINED_INDEX.md](RETAINED_INDEX.md) for commands, limits and storage ownership.
