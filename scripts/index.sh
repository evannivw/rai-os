#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ "$#" -ne 2 ]; then
  echo 'Usage: sh scripts/index.sh <record key> <new output file>' >&2
  exit 1
fi
cargo build --locked --bin rai-os
host_target=$(rustc -vV | sed -n 's/^host: //p')
cargo run --locked --example index_record --target "$host_target" -- \
  target/aarch64-unknown-none-softfloat/debug/rai-os "$1" "$2"
python3 scripts/check_record_index.py "$2" --record "$1"
