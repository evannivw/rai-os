#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
headless=false
for arg in "$@"; do
  case "$arg" in
    --headless) headless=true ;;
    --auto) ;;
    *) echo 'Usage: sh scripts/publication.sh [--auto] [--headless]' >&2; exit 1 ;;
  esac
done
if [ "$headless" = true ]; then
  cargo build --locked --bin rai-os --release
  kernel_path=target/aarch64-unknown-none-softfloat/release/rai-os
else
  cargo build --locked --bin rai-os --release --features guest-display --target-dir target/display
  kernel_path=target/display/aarch64-unknown-none-softfloat/release/rai-os
fi
host_target=$(rustc -vV | sed -n 's/^host: //p')
output_dir="target/publication-demo-$(date +%Y%m%d-%H%M%S)-$$"
exec cargo run --locked --example publication_demo --target "$host_target" -- "$kernel_path" "$output_dir" "$@"
