# Whole-table behavior: offline evidence

Recorded November 3, 2026 from the changed working tree based on 6b1f2c6.
source.json records code hashes; reports retain their real dirty/unknown provenance.
The live-only evolution branch was added after the local offline runs; Linux ran
the complete current verifier. It does not alter the offline execution path.

Real QEMU debug and release passed eight-row branch arithmetic, preview/cancel,
one-revision commit, hidden-row inclusion, failure in row eight with no partial
writes, reviewed program evolution preserving data, stale admission, an unrelated
text/check app and exact restart recovery. All provider proposals in these runs
are explicitly supplied test data validated by real Rust, not Meta responses.

94 host/shared tests passed. They also cover last-row division/overflow failure,
terminal workflow locks, empty batches, wrong scope, old definitions, invalid
scope and denied expansion during repair. Host/kernel Clippy and formatting pass.
Existing behavior, correction (13 adverse cases) and demo-entry regressions pass.
Console debug/release each passed boot and 37 checks. Linux ARM64 Docker passed
the new verifier and saved demo with networking disabled. No Meta calls occurred.

The initial verifier used the wrong filter key (ordinary f rather than F4) and
therefore did not prove hidden-row inclusion. That check was corrected before the
retained final runs. A new unit test initially used the wrong Rust action-method
name; it was corrected before the 94-test pass. Neither fix relaxed validation.

stack-frames.json records measured debug frames and explicit limits of the
analysis. Native screenshots were inspected for scope labels, row paging and
readable six-field comparisons. Raw UART and provider bytes remain unmodified.
Live generation/evolution evidence is a separate subsequent bounded run.
