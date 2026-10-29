# Milestone 1 architecture

An educational AArch64 kernel for QEMU, with a long-term microkernel direction.
Kernel modules share privilege and memory. One built-in Rust program now runs
at EL0 with bounded write/exit syscalls and a timer-enforced invocation deadline; see [USER_PROGRAM.md](USER_PROGRAM.md).
There is no multi-task scheduler, process IPC, heap, separate process address
spaces or isolated agent execution. EL1 stage-1 translation is active
with one identity-mapped address space; see [MMU.md](MMU.md).

## Boot

QEMU loads the ELF through -kernel. The linker places _start at 0x40080000,
inside virt RAM starting at 0x40000000, leaving room at the start for the DTB.
Supported configuration: virt-9.2, TCG, Cortex-A53, one CPU, 128 MiB, EL1,
gic-version=2, secure=off, virtualization=off, no firmware. Entry requires MMU/caches off. Rust constructs page tables and
enables the MMU before the boot banner; caches remain off.

boot.S masks interrupts, checks EL1, selects SP_EL1, creates a 16-byte-aligned
64 KiB stack, clears BSS, installs VBAR_EL1 and synchronizes instruction execution,
then calls kernel_main. The stack is outside the cleared BSS range. An unexpected exception level halts before Rust; the boot
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
one CPU, interrupts masked during UART access, exclusive UART access, and either pre-MMU access
or the active Device-nGnRnE identity mapping. A fatal handler
may take over UART permanently without resuming the interrupted writer. These assumptions
must be revisited for another board, remapping, or concurrency. Linker symbols and
Assembly entry form the boot contract. EL1 exception vectors support one
allowlisted EL1 return, controlled timer IRQs and bounded EL0 syscall/fault handling; other events enter a fatal UART diagnostic on
a dedicated stack; see [EXCEPTIONS.md](EXCEPTIONS.md) for
assumptions and limits. Rust panic still prints and halts without recovery.

## Responsibilities

- memory/frames.rs: pure page-address bookkeeping; memory/mod.rs: exclusive
  linker-reserved physical pool, ownership tokens, zeroing, and diagnostics.
  See [MEMORY.md](MEMORY.md). This allocator is not a heap.
- mmu/tables.rs: pure fixed-board mapping construction; mmu/mod.rs: owned
  table pages and EL1 register activation/readback. Code is read-only/executable,
  data is non-executable, and tables/constants are read-only/non-executable.
  Explicit user code/constants/stack pages permit EL0; kernel/device pages do not.
- exceptions.S / exceptions.rs: EL1 entry, full integer frames for slots 4, 5, 8 and 9,
  dedicated handler stack, allowlisted EL1 return, EL0 dispatch, or fatal report.
- context/: pure return policy, fixed frame layout and bounded SVC/ERET diagnostic.
  See [CONTEXT.md](CONTEXT.md); no task switching.
- user/: one linked Rust program, privilege transition, bounded copy/exit policy
  and synchronous invocation lifecycle. User faults and elapsed deadlines terminate
  only that invocation; [USER_TIMEOUT.md](USER_TIMEOUT.md) describes the boundary.
- timer/: exclusive GICv2/CNTP adapter, controlled EL1 IRQ diagnostic and absolute
  EL0 budget. IRQs are enabled in the diagnostic loop or EL0; successful handlers
  never use UART or allocate. User timeout dispatch accesses only its paused session. See [TIMER.md](TIMER.md).
- clock.rs: read-only generic counter adapter and boot-relative time.
- uart.rs: bytes and MMIO, including allocation-free formatted output.
- console.rs: framing, response rendering, and interleaving a bounded task step
  with nonblocking UART receive. Transmit still polls.
- task.rs: one owned word-count job, pure state transitions, progress/cancellation.
  See [TASKS.md](TASKS.md); this kernel job remains cooperative and unisolated.
- protocol.rs: structured request parsing without execution.
- command.rs: human-command parsing and execution of allowed operations. It
  receives typed uptime/MMU snapshots and delegates memory use to its owned adapter.
- main.rs: composition and panic handling.

Both parsers converge on an operation enum; neither executes arbitrary text.
See [PROTOCOL.md](PROTOCOL.md). The [host console](CONSOLE.md) already transports
requests over UART. The [small host coordinator](COORDINATOR.md) sequences a bounded priority queue (FIFO ties) using
observed completion. Its host placement is provisional; the target is a guest
service above the kernel once service execution/isolation exists. See the
[placement and migration rationale](COORDINATOR.md#why-the-coordinator-is-currently-on-the-host). Meta API calls are not implemented.
Credentials and HTTPS will remain on the host. The guest must independently
validate operations. Multi-service isolation still requires separate service mappings, scheduling
and IPC; module names do not supply those guarantees.

## Clock

The kernel samples the Arm generic physical counter before UART initialization
and supplies measured uptime to commands. The adapter is owned by kernel_main;
there is no mutable global clock. See [CLOCK.md](CLOCK.md) for access assumptions,
frequency conversion, real-QEMU checks, and the distinction from timer interrupts.
The host console and [graphical panel](PANEL.md) transport uptime like other
commands through shared host adapters. The panel is a separate native application;
its GUI dependencies do not enter the kernel. A guest desktop remains pending.

## References

- https://doc.rust-lang.org/stable/rustc/platform-support/aarch64-unknown-none.html
- https://www.qemu.org/docs/master/system/arm/virt.html
- https://github.com/qemu/qemu/blob/master/hw/arm/virt.c
- https://github.com/qemu/qemu/blob/master/hw/char/pl011.c

See [ROADMAP.md](ROADMAP.md) for completed foundations and the next service prerequisites.
