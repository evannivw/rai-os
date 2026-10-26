#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --locked --bin rai-os
host_target=$(rustc -vV | sed -n 's/^host: //p')
exec cargo run --locked --example control_console --target "$host_target" -- \
  target/aarch64-unknown-none-softfloat/debug/rai-os
