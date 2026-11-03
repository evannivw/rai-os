# Composed pages — offline verification

Real AArch64 QEMU runs on macOS debug/release and Linux ARM64 Docker, with explicit
verifier-only app declarations. No Meta calls. Each retained report has verified=true.
Per-attempt proposal output, native captures, UART, snapshots and independent results
are preserved. Four block kinds, signed bar pixels, eight rows, data edits, filters,
reviewed evolution, cancellation, invalid binding and a second app were checked.

The host suite passed 106 tests, including real Git page roundtrip. Clippy passed
for host and guest; Python syntax/formatting checks passed. Earlier apps, repair,
Git, batch behavior and demo-entry regressions passed. Console debug/release boot
checks and the full host-console system journey passed. Linux ran without network
and its saved-demo check also passed. Windows was not executed natively.

source-hashes.json identifies changed executable inputs before the source freeze.
The Linux image does not contain .git, so its report does not infer a commit hash.
The stack report measures selected emitted-assembly CFA frames; it is not a complete
whole-program stack bound. Known conservative action frames sum to 56944 bytes,
excluding recursive/standard/indirect callees, within the unchanged 65536-byte
reservation. The final page-pool end is 0x401fc000, within the existing 0x40200000
window. Initial builds exposed window pressure; borrowing schema output, one debug
compilation unit and separate constructor temporaries resolved it before final checks.

Live generation, if run, belongs to a separate evidence directory. The existence
of these offline reports does not claim the model generated the fixture apps.
