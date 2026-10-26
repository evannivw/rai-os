# Milestone 1 architecture

An educational AArch64 kernel for QEMU, with a long-term microkernel direction.
Current modules share privilege and memory. There is no scheduler, process IPC,
MMU, heap, userspace, or isolated agent execution.

## Boot

QEMU loads the ELF through -kernel. The linker places _start at 0x40080000,
inside virt RAM starting at 0x40000000, leaving room at the start for the DTB.
Supported configuration: virt-9.2, TCG, Cortex-A53, one CPU, 128 MiB, EL1,
secure=off, virtualization=off, no firmware, MMU/caches off.

boot.S masks interrupts, checks EL1, selects SP_EL1, creates a 16-byte-aligned
64 KiB stack, clears BSS, then calls kernel_main. The stack is outside the
cleared BSS range. An unexpected exception level halts before Rust; the boot
check will time out rather than claim success.

Rust uses no_std, no_main, and core. aarch64-unknown-none-softfloat avoids
requiring SIMD/FPU setup for compiler-generated code. There are no external
crate dependencies. Rust's bundled rust-lld links the ELF; Apple ld is not an
ELF linker. build.rs triggers relinking when linker.ld changes.

## UART and unsafe contracts

PL011 is fixed at 0x09000000 for this QEMU machine. Aligned 32-bit volatile
accesses preserve MMIO operations, but do not provide synchronization.
Configuration: 115200, 8N1, no FIFO or interrupts; the driver polls readiness.
Polling consumes CPU while waiting and is not evidence of energy efficiency.

Unsafe blocks document the mapping, alignment, and exclusive ownership:
one CPU, interrupts disabled, MMU off, and no other UART owner. These assumptions
must be revisited for another board, paging, or concurrency. Linker symbols and
Assembly entry form the boot contract. CPU exception vectors are not installed;
a CPU fault can hang the guest. Rust panic prints and halts without recovery.

## Responsibilities

- uart.rs: bytes and MMIO.
- console.rs: framing, input validation/editing, and response rendering.
- protocol.rs: structured request parsing without execution.
- command.rs: human-command parsing and execution of allowed operations.
- main.rs: composition and panic handling.

Both parsers converge on an operation enum; neither executes arbitrary text.
See [PROTOCOL.md](PROTOCOL.md). The [host console](CONSOLE.md) already transports
requests over UART. Meta API calls and the coordinator are not implemented.
Credentials and HTTPS will remain on the host. The guest must independently
validate operations. Real service isolation will require virtual memory, EL0,
syscalls, scheduling, and IPC; module names do not supply those guarantees.

## References

- https://doc.rust-lang.org/stable/rustc/platform-support/aarch64-unknown-none.html
- https://www.qemu.org/docs/master/system/arm/virt.html
- https://github.com/qemu/qemu/blob/master/hw/arm/virt.c
- https://github.com/qemu/qemu/blob/master/hw/char/pl011.c
