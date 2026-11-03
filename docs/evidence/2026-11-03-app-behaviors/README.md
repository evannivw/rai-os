# Composable behavior evidence, November 3

Offline implementation checks precede the source freeze. Reports record the old
HEAD plus dirty=true and the exact kernel hash; source-files.json records the
final executable-source bytes. The verifier schemas are disclosed test inputs,
not live Meta generations. The final debug/release reports use three input rows
and independent expected results, including two forward-branch outcomes.

The pre-const debug run passed in QEMU, but emitted assembly revealed overlapping
startup stack frames beyond the 64KiB reservation. A passing run did not prove
stack correctness. Explicit const initialization eliminated the large constructor
temporary; stack-frames.json records measured before/after frames and limitations.
Final debug/release and existing-service regressions were run after that fix.

The portable logs exercise real QEMU in Linux ARM64 Docker on the Mac with network
disabled. They do not establish physical Windows/phone testing. The previous
saved workshop/quote demo also passes unchanged. No new device drivers, native
code generation, background events or hardware app isolation are claimed.
