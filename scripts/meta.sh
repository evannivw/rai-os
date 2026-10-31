#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
kernel_path=target/aarch64-unknown-none-softfloat/debug/rai-os
if [ "${1:-}" = "--graphics" ]; then
  cargo build --locked --bin rai-os --features guest-display --target-dir target/display
  kernel_path=target/display/aarch64-unknown-none-softfloat/debug/rai-os
else
  cargo build --locked --bin rai-os
fi
host_target=$(rustc -vV | sed -n 's/^host: //p')
exec cargo run --locked --manifest-path host/meta/Cargo.toml --target "$host_target" \
  --target-dir target/host-meta -- \
  "$kernel_path" "$@"
