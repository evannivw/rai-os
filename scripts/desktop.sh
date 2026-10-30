#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
profile=release
case "${1:-}" in
  '') ;;
  --debug) profile=debug ;;
  *) echo 'Usage: sh scripts/desktop.sh [--debug]' >&2; exit 1 ;;
esac
if [ "$profile" = release ]; then
  cargo build --locked --bin rai-os --features guest-display --target-dir target/display --release
else
  cargo build --locked --bin rai-os --features guest-display --target-dir target/display
fi
echo 'RAI-OS native preview. Enter record.index notes@1 in this terminal.'
echo 'The window currently displays results; input uses the terminal. Ctrl-C exits.'
exec sh scripts/qemu.sh "target/display/aarch64-unknown-none-softfloat/$profile/rai-os" --graphics
