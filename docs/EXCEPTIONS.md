# EL1 fatal exception diagnostics

Implemented October 27, 2026. The kernel now installs an AArch64 exception vector
table before entering Rust. A fatal exception reports its architectural registers
through UART and halts. This is a diagnostic foundation for later service runtime
work, not recovery, process isolation, preemption, or a complete interrupt system.
Updated October 28: slot 4 also supports one [controlled EL1 return](CONTEXT.md);
the fatal path and historical captures below remain applicable to rejected EL1 events.
The later [EL0 increment](USER_PROGRAM.md) adds bounded syscalls and contained
user faults through slot 8. Slot 5 handles [controlled EL1 timer IRQs](TIMER.md), and slot 9 handles
[EL0 timeout IRQs](USER_TIMEOUT.md). Unexpected kernel exceptions still halt the guest.

## What happens

1. `boot.S` masks interrupts, selects SP_EL1, sets the boot stack, and clears BSS.
2. It writes the 2 KiB-aligned table address to VBAR_EL1 and executes ISB.
3. The table has sixteen 128-byte slots. Slot 4 (current EL, SP_EL1,
   synchronous, offset 0x200) now saves an integer context before dispatch.
   Slots 5, 8 and 9 also save frames for timer IRQ and EL0 dispatch. Other slots record their
   number and enter the fatal-only path.
4. Assembly masks interrupts, uses a single-CPU nested-entry guard and captures
   ESR_EL1, ELR_EL1, SPSR_EL1, FAR_EL1 and entry SP. Rust runs on a dedicated
   16 KiB, 16-byte-aligned stack. Slot 4 first needs space on the normal stack.
5. At EL1, the approved SVC site and controlled timer probe may restore registers and execute
   ERET. Rejected EL1 events initialize/take over PL011, report and wait forever.
   Slot 8 separately resumes valid user syscalls or terminates faulted/expired
   invocations. Slot 9 terminates a user invocation whose timer deadline expired. There is no automatic retry, unwinding or return from a fatal report.

`EXCEPTION` and `HALTED` lines are fatal console diagnostics, not successful
`@rai1` responses or task results. The normal boot banners and protocol are
unchanged. A host request encountering a fault cannot count the text as success;
normal transport validation/timeouts still apply. No graphical fault view was
added in this increment.

## Reading the output

- `vector`: which entry the processor selected, not a task ID.
- `esr`: raw syndrome; `ec` extracts the architectural exception class.
- `elr`: exception return address. In the BRK test this is the faulting instruction.
- `spsr`: saved processor status, including prior mode and interrupt masks.
- `far_raw`: raw address register. Its meaning/validity depends on the exception;
  BRK does not supply a valid fault address. Zero does not mean address zero failed.
- `entry_sp`: SP seen at the assembly entry, not a saved lower-EL user stack.
- `handler_sp`: SP sampled in Rust on the dedicated diagnostic stack.

ESR/ELR interpretation also depends on exception type; do not treat all sixteen
slots as synchronous faults with a valid memory address. IRQ/FIQ/SError remain
masked during ordinary kernel execution. The controlled EL1 assembly timer probe
and a deadline-protected EL0 invocation enable IRQs and use GICv2/CNTP handlers. IRQ ESR/FAR values are not decoded
as synchronous fault information.

## Verify a deliberate exception

From the repository root with Rust and QEMU available:

```sh
python3 scripts/verify_exceptions.py
python3 scripts/verify_exceptions.py --release
```

The optional `exception-self-test` Cargo feature executes BRK #0x5241 immediately
after the boot banner. It is off by default and is not a console/panel command.
The script builds into `target/exception-self-test/`, leaving the normal image
used by the panel intact. Do not enable this feature (or all features) for normal
use: that image intentionally halts rather than entering the console.

Both debug and release were executed in QEMU. The verifier checks the linked ELF
vector alignment/size and stack placement, actual vector 4, ESR 0xf2005241, class
0x3c, saved EL1h mode/masks, and ELR equal to the named BRK instruction's ELF symbol.
It checks both stack ranges and 16-byte alignment. After HALTED, an info request
produces no further bytes during a 250 ms observation and QEMU remains alive.
The script always terminates/reaps its child, including on failure.

[Debug capture](evidence/2026-10-27-exceptions-debug.txt) and
[release capture](evidence/2026-10-27-exceptions-release.txt) preserve actual UART
output with CR/LF made explicit. Addresses differ between build profiles.

Regression checks passed in each profile: 37 normal boot/console/protocol cases,
28 task exchanges, and three task-domain tests. Fourteen panel tests (including
four real-QEMU tests) and terminal-console integration also passed. Formatting
and kernel Clippy passed for both the default and self-test configurations.

## Safety conditions and limitations

- One CPU, EL1, caches off, expected RAM/UART mappings, one user privilege boundary, no separate service address spaces.
  The later MMU increment enables identity translation before the boot banner.
- The guard is zeroed before vector installation; no atomics are needed under
  this single-CPU, masked-interrupt contract. The allowed return clears the guard.
  A nested slot-4 entry saves a frame before checking it; nested dispatch/reporting
  is blocked, but faults during that stack save are not recoverable.
- The Rust entry follows AAPCS64 on a reserved aligned stack. It never returns,
  so clobbered general registers cannot corrupt resumed execution. The separate
  approved-return path saves/restores its integer context; see [CONTEXT.md](CONTEXT.md).
- UART ownership is permanently transferred on a fatal exception. No Rust MMIO
  references or locks are held across accesses. This needs redesign for SMP,
  general resumable interrupts, or concurrent drivers. The controlled timer handler
  never uses UART and cannot interrupt ordinary UART access.
- Diagnostic output still depends on working RAM and PL011; a fault in the
  reporting path can halt silently. UART polling can block on broken hardware.
  Faults before VBAR installation are not covered. The nested guard is implemented
  but not fault-injected in these tests.
- This original increment injected only synchronous EL1 BRK. The later
  [MMU tests](MMU.md) also inject data/instruction aborts. The later EL0 tests also exercise slot 8. Corrupted stacks and
  FIQ/SError remain untested; controlled IRQ tests are documented in [TIMER.md](TIMER.md). No universal recovery claim.

## Relation to the coordinator and future services

The kernel is evolving infrastructure, not permanently fixed code. The host
coordinator continues choosing jobs by priority; this increment changes the
kernel's failure path only. To host isolated services inside RAI-OS, subsequent
work still needs separate protected service mappings, scheduling, IPC, and broader recovery rules. Built-in user-mode entry and
bounded system calls are already implemented. Exception entry is one prerequisite; the
controlled EL1 return plus the later EL0 program are prerequisites, not a complete
service runtime.

Architectural references:

- [Arm AArch64 exception model](https://documentation-service.arm.com/static/67ac57fb091bfc3e0a9479cc): vector layout, exception state, VBAR and return context.
- [Arm architecture reference manual](https://documentation-service.arm.com/static/685eb35c3cb5b729562ba389): BRK syndrome class and immediate value.
