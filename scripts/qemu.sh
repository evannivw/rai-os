#!/bin/sh
set -eu
exec qemu-system-aarch64 \
  -machine virt-9.2,secure=off,virtualization=off -cpu cortex-a53 \
  -accel tcg -smp 1 -m 128M -display none -monitor none \
  -serial stdio -nic none -kernel "$1"
