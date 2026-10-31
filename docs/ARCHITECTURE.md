# Milestone 1 architecture

An educational AArch64 kernel for QEMU, with a long-term microkernel direction.
Kernel modules share privilege and memory. One built-in Rust program now runs
at EL0 with bounded write/exit syscalls and a timer-enforced invocation deadline; see [USER_PROGRAM.md](USER_PROGRAM.md).
A [bounded two-context scheduling diagnostic](SCHEDULING.md) now saves/resumes
EL0 integer state on one CPU. A [bounded task lifecycle](TASK_LIFECYCLE.md) now
lets one task finish or fault while its peer continues. There is no general task
creation, heap or isolated agent execution. Two fixed tasks can now exchange
[bounded scalar messages](IPC.md) through kernel-owned mailboxes and use
[blocking receive](IPC_WAIT.md) with message/closure wakeup. A first
[record-count service](RECORD_SERVICE.md) now reuses those mechanisms to process
immutable fixture records at EL0 and return results over UART. A separate
[persistent worker](PERSISTENT_WORKER.md) now retains its EL0 continuation and
private stack between console requests. An initial [guest coordinator](GUEST_COORDINATOR.md)
now runs at EL0 alongside that worker, admitting versioned manual proposals and
retaining results. The [guest Meta bridge](GUEST_META_BRIDGE.md) now transports its
pending goal/ticket to Meta and forwards typed proposals for guest admission. EL1 stage-1 translation
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
- records.rs: allowed immutable fixture IDs and bounded ASCII token counting;
  source bytes are shared user read-only data, and counting/token selection execute in user text.
- record_index.rs / user/index_program.rs: synchronous exact per-record index
  at EL0, using checked output copies and normal-exit publication. The host exports
  its text unchanged; an independent Python checker validates the exact revision.
  See [RECORD_INDEX.md](RECORD_INDEX.md). This is not yet a coordinator operation.
- service.rs: validated input -> bounded guest execution -> completion/result checks.
  schedule/service_programs.rs: fixed EL0 request/reply client and worker. This is
  one closed batch per host request, without persistent lifetime or external wakes.
- user_memory.rs: exclusive lease over fixed user RAM, held across suspended
  worker lifetime and acquired by every legacy EL0 entry before clearing stacks.
- schedule/worker.rs: one retained frame, mailbox pair, per-request budgets,
  ingress/reply validation and explicit stop/failure cleanup. Endpoint 1 is kernel
  host ingress; worker_program.rs executes at EL0 as endpoint 2. The console owns
  the session. This is not a general daemon scheduler or guest AI coordinator.
- schedule/coordinator_program.rs: persistent EL0 goal/proposal policy; scalar
  host input, duplicate/stale rejection, IPC dispatch and result retention.
  schedule/coordinator.rs: kernel-owned two-context session, coordinator-only
  host gateway, bounded turns, report checks and pair cleanup. It shares the
  worker entry/IPC and exclusive RAM lease, not host Meta state or credentials.
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
- display/: optional guest framebuffer adapter, original renderer and glyph data.
  The console owns it and supplies observed index results. It currently runs at EL1,
  not as an isolated GUI service. See [GUEST_DISPLAY.md](GUEST_DISPLAY.md).
- main.rs: composition and panic handling.

Both parsers converge on an operation enum; neither executes arbitrary text.
See [PROTOCOL.md](PROTOCOL.md). The [host console](CONSOLE.md) already transports
requests over UART. The [small host coordinator](COORDINATOR.md) sequences a bounded priority queue (FIFO ties) using
observed completion. Its host placement is provisional; the target is a guest
service above the kernel once service execution/isolation exists. See the
[placement and migration rationale](COORDINATOR.md#why-the-coordinator-is-currently-on-the-host). A separate
[Meta bridge](META_BRIDGE.md) now makes one real structured proposal and validates
it before guest execution. host/meta/src separates proposal policy (domain), local
configuration, HTTPS provider transport and composition. It reuses the shared UART
adapter; JSON dependencies stay in its own host manifest. Its new
[bounded coordinator](META_COORDINATOR.md) maintains released revisions and actual
observations, applies timed fixture events while a provider request is pending,
and rejects stale or duplicate work before UART dispatch. State policy remains
separate from thread/clock/QEMU adapters. The existing priority
coordinator and graphical panel have not been connected to this bridge.
A new --guest mode reads the actual EL0 coordinator reports instead of maintaining
host goal/admission policy. guest_bridge.rs owns transport/lifecycle,
guest_report.rs validates decoded observations, and decision_source.rs shares
bounded provider/replay thread ownership with the older host mode. Its late
proposals are rejected inside the guest. See [GUEST_META_BRIDGE.md](GUEST_META_BRIDGE.md).
Credentials and HTTPS remain on the host. The guest must independently
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
its GUI dependencies do not enter the kernel. The separate [native guest display](GUEST_DISPLAY.md)
now draws a bounded view inside QEMU without those dependencies. An interactive
desktop/window system remains pending.

## References

- https://doc.rust-lang.org/stable/rustc/platform-support/aarch64-unknown-none.html
- https://www.qemu.org/docs/master/system/arm/virt.html
- https://github.com/qemu/qemu/blob/master/hw/arm/virt.c
- https://github.com/qemu/qemu/blob/master/hw/char/pl011.c

See [ROADMAP.md](ROADMAP.md) for completed foundations and the next service prerequisites.

## Retained exact indexes

The optional coordinator index mode retains EL0 policy and worker contexts.
A owns goal/admission/reuse decisions, B serializes exact indexes through bounded
scalar IPC, and the kernel holds six committed 384-byte result slots after
stream/publication validation. No task receives another task's writable pointer.
The native view renders pending/created/reused outcomes. See
[RETAINED_INDEX.md](RETAINED_INDEX.md); count-mode Meta transport is unchanged.
The [Meta index bridge](META_INDEX_BRIDGE.md) selects this mode explicitly. Its
artifact history validates observed reuse but never supplies guest results or
chooses cache hits. Only the provider boundary selects the allowed operation
schema; count-only modes retain their narrower permission.
