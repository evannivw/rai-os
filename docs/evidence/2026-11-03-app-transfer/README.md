# Atomic-transfer offline evidence

Recorded November 3 in Guatemala (UTC November 4). Tests ran on changes based on
e4a93caa; source-hashes.json identifies the actual uncommitted source contents.
No Meta requests were made in these runs. Scripted proposals are disclosed fixtures.

- 111 Rust tests passed, including conservation, destination ambiguity/self/missing,
  invalid quantities, overflow, terminal rows, late derived failure and Git versions.
- Real QEMU debug/release: rule/page review, pointer preview/cancel/apply, one-revision
  commit, insufficient balance rejection, record recovery. Release also checks the
  new saved-checkpoint demo preparation. Debug was run before that host-only addition.
- Linux ARM64 Docker: the same transfer verifier passed with network disabled.
- Pages, batches, Git and demo-entry regressions passed. Console/protocol passed 37
  checks in both profiles; host console exercised memory, timer, EL0 and coordinator.
- Host/guest Clippy, formatting and Python syntax checks passed.

The debug linker map ends the page pool at 0x401ff000: only 4096 bytes remain before
0x40200000. No MMU/stack enlargement was made. Selected assembly frames are retained:
console run 42432, controller command 8176, execute 400, begin_spec 7440, key 1296,
run_move 1552, renderer 1552, materialize 1984. These selected frame sizes are not a
complete call-graph or recursive stack proof. Exercised QEMU paths passed.

Screenshots are actual guest framebuffer captures. Values are fictional test data.
The app runtime remains shared EL1 and checkpoints remain a host responsibility.
A frozen-source live generation run is a separate next step, not covered here.
