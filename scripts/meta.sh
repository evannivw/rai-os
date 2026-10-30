#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --locked --bin rai-os
host_target=$(rustc -vV | sed -n 's/^host: //p')
exec cargo run --locked --manifest-path host/meta/Cargo.toml --target "$host_target" \
  --target-dir target/host-meta -- \
  target/aarch64-unknown-none-softfloat/debug/rai-os "$@"
