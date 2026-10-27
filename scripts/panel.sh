#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --locked --bin rai-os
host_target=$(rustc -vV | sed -n 's/^host: //p')
cargo build --locked --manifest-path host/panel/Cargo.toml \
  --target "$host_target" --target-dir target/host-panel
if [ "$(uname -s)" = Darwin ]; then
  python3 scripts/package_panel_macos.py
  exec "target/RAI-OS Device Lab.app/Contents/MacOS/rai-panel"
fi
exec "target/host-panel/$host_target/debug/rai-panel" \
  target/aarch64-unknown-none-softfloat/debug/rai-os
