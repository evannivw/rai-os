# Prompt workspace verification — November 2–3, 2026, Guatemala

This folder records an implementation increment, not a performance comparison.
The source snapshot is identified by base_commit in manifest.json and exact file
hashes in each Linux build-info.json. Live Meta evidence, if present, is separate.

- workspace-debug / workspace-final: real RFB 3.8 frame, keyboard and pointer
  interaction, real guest plan preview/commit and independently checked bytes.
  These use explicit replay, not inference. Screenshots are guest QMP captures.
- native-viewer: macOS Screen Sharing manual session using QEMU VNC authentication.
  The first lowercase variant correctly produced a replay limitation notice; the
  exact documented example then previewed and applied both copies. Actual UART and
  proposal captures are retained. The viewer's window was observed with computer
  use, not reconstructed. No test password is included here.
- linux-arm64 / linux-amd64: same offline interaction checks in Linux containers.
  AMD64 is emulated on an ARM Mac; this is not a physical Windows host test.
- JSON-wrapped logs retain exact stdout/stderr for unit, Clippy, kernel regression
  and initial build failure. Development now uses opt-level=1 with debug symbols.

The guest stores eight files of 512 bytes in RAM. Sources/destinations and capacity
are validated before writes; stale/cancelled plans cannot apply. Tests cover an
invalid final action changing nothing, a catalog change after preview blocking
commit, and correct unchanged originals. These are not crash-durability, isolation,
Unicode, driver completeness or unrestricted natural-language correctness claims.

No earlier negative timing result is replaced. The native interface and one-plan
model path are product/integration progress; AI superiority is still unproven.


The live/ directory contains one fresh Meta response on frozen source a330bd3.
It passed the independent expected-source/scope checks before confirmation and
exact guest byte checks afterward. HTTP 200, 3093 ms provider latency, 817 tokens,
no retry. Live input used the UI's UART adapter; viewer interaction was separately
verified offline. Screenshots show the actual Meta reason and guest results.
