#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ "$#" -gt 1 ] || { [ "$#" -eq 1 ] && [ "$1" != '--graphics' ]; }; then
  echo 'Usage: sh scripts/bundle.sh [--graphics]' >&2
  exit 1
fi
kernel_path=target/aarch64-unknown-none-softfloat/debug/rai-os
if [ "${1:-}" = '--graphics' ]; then
  cargo build --locked --bin rai-os --features guest-display --target-dir target/display
  kernel_path=target/display/aarch64-unknown-none-softfloat/debug/rai-os
else
  cargo build --locked --bin rai-os
fi
host_target=$(rustc -vV | sed -n 's/^host: //p')
output_dir="target/bundle-demo-$(date +%Y%m%d-%H%M%S)"
exec cargo run --locked --example bundle_demo --target "$host_target" -- "$kernel_path" "$output_dir" "$@"
