# Native guest display: original scope and completed gates

Scope decision, October 30, 2026. Requested by the user after the first useful
workload: make RAI-OS draw its own visible screen in QEMU. This supersedes the
blanket graphics-driver exclusion in the earlier delivery plan. No graphics
implementation or screenshot was claimed by the original plan below.

**Later on October 30:** both gates are now implemented and tested; see
[GUEST_DISPLAY.md](GUEST_DISPLAY.md) for actual captures, commands, safety and
limits. This plan is retained to distinguish intended scope from delivered work.

## Why it can start now

The kernel can boot, own physical memory, enforce mappings and run bounded guest
programs. The first index supplies a concrete output to display. We do not need a
finished kernel, filesystem, touchscreen, phone port or AI model running locally
before drawing a small screen. The existing Mac Device Lab panel is a separate
host application and does not satisfy this goal.

QEMU's Arm virt board has no default display device. Its documentation recommends
virtio-gpu-pci for a general graphics path. For our fixed TCG demonstration, the
initial candidate is ramfb: QEMU displays a pixel buffer in guest RAM configured
through fw_cfg. Confirm the installed device and validate actual boot before
committing to that adapter. This is an initial software-rendered display, not GPU
acceleration or physical-phone support.

## Two small acceptance gates

1. **Actual guest pixels.** Add an opt-in graphical QEMU runner, explicitly reserve
   framebuffer RAM outside the existing allocator, map the necessary device/data
   regions with EL0 access denied, and configure the display with bounded failure
   handling. Draw a simple background and original bitmap text from Rust. Capture
   the actual QEMU framebuffer and verify dimensions and expected pixels. Existing
   headless boot, memory protections and automated checks must continue to work.
2. **Actual work on screen.** Render a portrait view with RAI-OS identity, requested
   record/revision and its observed index or error. Drive it through existing UART
   commands first. Change notes@1 to notes@2 and verify that the rendered result
   changes; show empty and rejected outcomes honestly. Do not fabricate progress,
   battery levels, activity or verification status. A host verifier's decision is
   external evidence, not something the guest can infer from its own output.

Keep presentation separate from the index/coordinator policy. The display adapter
owns device access and memory; a bounded renderer consumes a read-only snapshot.
An initial EL1 renderer is still code inside the guest, but not an isolated EL0
GUI service. Document that placement honestly. Moving presentation into a separate
EL0 service requires an explicit drawing/syscall boundary and is a later step.

The first screen is an information view. Mouse/touch input, windows, a compositor,
fonts loaded from files and application lifecycle are not prerequisites for this
increment. It should make the existing execution understandable, not consume the
remaining delivery time. Stop at these gates and return to useful-work integration,
comparison and the three-minute submission video.

## Evidence sources

- [QEMU Arm virt board](https://www.qemu.org/docs/master/system/arm/virt.html)
- [QEMU ramfb implementation, v11.1.0](https://github.com/qemu/qemu/blob/v11.1.0/hw/display/ramfb.c)
- [QEMU fw_cfg specification](https://www.qemu.org/docs/master/specs/fw_cfg.html)

These describe the virtual-device interfaces. No external implementation has been
copied into RAI-OS. The subsequent implementation and execution evidence are in [GUEST_DISPLAY.md](GUEST_DISPLAY.md).
