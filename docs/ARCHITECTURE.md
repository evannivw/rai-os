# Milestone 1 architecture

An educational AArch64 kernel for QEMU, with a long-term microkernel direction.
Kernel modules share privilege and memory. One built-in Rust program now runs
at EL0 with bounded write/exit syscalls and a timer-enforced invocation deadline; see [USER_PROGRAM.md](USER_PROGRAM.md).
A [bounded two-context scheduling diagnostic](SCHEDULING.md) now saves/resumes
EL0 integer state on one CPU. A [bounded task lifecycle](TASK_LIFECYCLE.md) now
lets one task finish or fault while its peer continues. There is no general task
creation, heap or isolated agent execution. Two fixed tasks can now exchange
[bounded scalar messages](IPC.md) through kernel-owned mailboxes and use
[blocking receive](IPC_WAIT.md) with message/closure wakeup. EL1 stage-1 translation
now has a base map and two fixed task maps with private writable stack pages;
user code/constants remain shared read-only. See [MMU.md](MMU.md) and
[TASK_MEMORY.md](TASK_MEMORY.md).

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
  See [CONTEXT.md](CONTEXT.md); scheduling reuses the Frame layout.
- user/: one linked Rust program, privilege transition, bounded copy/exit policy
  and synchronous invocation lifecycle. User faults and elapsed deadlines terminate
  only that invocation; [USER_TIMEOUT.md](USER_TIMEOUT.md) describes the boundary.
- schedule/mod.rs: shared validated EL0 entry/capture and the original twelve-IRQ
  context diagnostic. schedule/lifecycle.rs: pure states and ready-task selection.
  schedule/runtime.rs: two kernel-owned records, frame/stack validation, bounded
  execution and compact results. schedule/programs.rs: finite linked Rust user
  tasks plus a deliberate forbidden-read fixture. Terminal tasks are skipped;
  each turn uses a task map that denies EL0 access to the peer stack.
  schedule/isolation.rs checks real own/peer accesses and recovery. See
  [TASK_LIFECYCLE.md](TASK_LIFECYCLE.md) and [TASK_MEMORY.md](TASK_MEMORY.md).
- ipc.rs: pure bounded FIFO mailboxes, checked task IDs, route permissions and
  endpoint closure. schedule/runtime.rs binds sender identity to the current
  record and translates scalar syscall inputs/results; schedule/ipc_programs.rs
  contains linked Rust request/reply clients. schedule/wait_programs.rs exercises
  blocked receivers. Lifecycle Waiting states are skipped until the runtime
  completes their pending result; a closed batch with no runnable task reports
  remaining waiters as Deadlocked. No user pointers are dereferenced.
- timer/: exclusive GICv2/CNTP adapter, controlled EL1 IRQ diagnostic and absolute
  EL0 budget or scheduling slice. IRQs are enabled in the diagnostic loop or EL0; successful handlers
  never use UART or allocate. User/schedule dispatch accesses only its paused session or observation. See [TIMER.md](TIMER.md).
- clock.rs: read-only generic counter adapter and boot-relative time.
- uart.rs: bytes and MMIO, including allocation-free formatted output.
- console.rs: framing, response rendering, and interleaving a bounded task step
  with nonblocking UART receive. Transmit still polls.
- task.rs: one owned word-count job, pure state transitions, progress/cancellation.
  See [TASKS.md](TASKS.md); this kernel job remains cooperative and unisolated.
- protocol.rs: structured request parsing without execution.
- command.rs: human-command parsing and execution of allowed operations. It
  receives a typed uptime snapshot and delegates memory/MMU operations to their
  owned adapters; the MMU supplies register snapshots and fixed task-map switches.
- main.rs: composition and panic handling.

Both parsers converge on an operation enum; neither executes arbitrary text.
See [PROTOCOL.md](PROTOCOL.md). The [host console](CONSOLE.md) already transports
requests over UART. The [small host coordinator](COORDINATOR.md) sequences a bounded priority queue (FIFO ties) using
observed completion. Its host placement is provisional; the target is a guest
service above the kernel once service execution/isolation exists. See the
[placement and migration rationale](COORDINATOR.md#why-the-coordinator-is-currently-on-the-host). Meta API calls are not implemented.
Credentials and HTTPS will remain on the host. The guest must independently
validate operations. General isolated services still require service-specific memory
ownership/lifetimes, external event delivery and richer permission policies; the fixed
stack maps and scalar IPC are foundations,
not a complete service security model. Module names do not supply those guarantees.

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
