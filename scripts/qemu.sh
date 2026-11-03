#!/bin/sh
set -eu
kernel=${1:?Usage: qemu.sh kernel [--graphics|--capture]}
shift
case "${1:-}" in
  --graphics)
    shift
    backend=${RAI_DISPLAY:-}
    if [ -z "$backend" ]; then
      case "$(uname -s)" in Darwin) backend=cocoa ;; *) backend=gtk ;; esac
    fi
    case "$backend" in
      cocoa|gtk|sdl) set -- -display "$backend" -device ramfb "$@" ;;
      vnc) set -- -display none -vnc 0.0.0.0:0 -device ramfb "$@" ;;
      *) echo 'RAI_DISPLAY must be cocoa, gtk, sdl or vnc' >&2; exit 1 ;;
    esac ;;
  --capture) shift; set -- -display none -device ramfb "$@" ;;
  *) set -- -display none "$@" ;;
esac
exec qemu-system-aarch64 \
  -machine virt-9.2,gic-version=2,secure=off,virtualization=off -cpu cortex-a53 \
  -accel tcg -smp 1 -m 128M -monitor none \
  -serial stdio -nic none -kernel "$kernel" "$@"
