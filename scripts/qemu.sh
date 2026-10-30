#!/bin/sh
set -eu
kernel=${1:?Usage: qemu.sh kernel [--graphics|--capture]}
shift
case "${1:-}" in
  --graphics) shift; set -- -display cocoa -device ramfb "$@" ;;
  --capture) shift; set -- -display none -device ramfb "$@" ;;
  *) set -- -display none "$@" ;;
esac
exec qemu-system-aarch64 \
  -machine virt-9.2,gic-version=2,secure=off,virtualization=off -cpu cortex-a53 \
  -accel tcg -smp 1 -m 128M -monitor none \
  -serial stdio -nic none -kernel "$kernel" "$@"
