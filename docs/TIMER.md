# Controlled EL1 timer interrupts

Implemented October 28, 2026. `timer.check` receives real emulated Arm physical
timer IRQs through GICv2, saves/restores integer state, then disables the timer.
This command is a kernel diagnostic, not scheduling. The same timer owner now
also enforces an [EL0 invocation deadline](USER_TIMEOUT.md). Ordinary kernel Rust
still runs masked; EL0 enables IRQs only while its deadline is armed.

## Try and verify

```sh
. "$HOME/.cargo/env"
sh scripts/console.sh
```

Enter `timer.check`, then `user.run` and `echo still running`. The timer command
performs two rounds of three IRQs. Each round seeds distinct values in all 31
integer registers and a different NZCV flags pattern. A successful reply reports
six ticks, preserved registers/flags/stack, IRQ masking and hardware shutdown.
`elapsed_ms` is measured guest counter time and varies between runs. Each interval
is approximately 10 ms, rounded up in counter ticks; rearming occurs inside the
handler, so this is not a drift-free periodic clock or a latency benchmark.

```sh
python3 scripts/verify_timer.py
python3 scripts/verify_timer.py --release
python3 scripts/verify_user.py
python3 scripts/verify_user.py --release
```

## Execution path

1. `main.rs` constructs one timer owner after MMU activation. The board explicitly
   selects `virt-9.2,gic-version=2,secure=off,virtualization=off`, one Cortex-A53
   CPU under TCG. The GIC stays disabled outside the diagnostic.
2. `timer/hardware.rs` owns GIC distributor/CPU-interface registers at 0x08000000
   and 0x08010000. Only the first 4 KiB page of each is mapped, as Device-nGnRnE,
   EL1 read/write, execute-never, inaccessible to EL0. One additional L3 table
   raises table ownership to six pages (24 KiB), leaving 250 pool pages free.
3. `timer/mod.rs` publishes one stack-owned round, enables physical-timer PPI 30
   and programs CNTP_TVAL_EL0/CNTP_CTL_EL0. With security extensions disabled,
   Group 0 is routed as IRQ, not FIQ; split EOI mode remains disabled.
4. `timer/probe.S` seeds registers/flags, unmasks only IRQ, then branches to itself.
   The loop does not poll a counter, call Rust or issue a syscall. Only the actual
   IRQ path can redirect it to the continuation.
5. Vector slot 5 saves the full 288-byte frame before using scratch registers and
   switches to the dedicated exception stack. The existing entry guard prevents
   nested dispatch. Rust acknowledges the GIC, checks ID 30, the active round,
   exact interrupted loop PC and EL1h saved masks, then records the counter.
6. The first two ticks rearm CNTP. The third disables it, redirects the saved PC
   to the approved continuation and sets the saved IRQ mask. Deasserting/rearming
   the timer precedes GIC EOI; barriers order device accesses. ERET restores state.
7. Assembly captures registers, flags, SP and DAIF before using scratch registers.
   Back in masked Rust, the timer owner disables GIC delivery, clears the active
   pointer and checks the observations, tick intervals and actual hardware state.
   Only successful checks produce the public success fields.

## Safety and limits

- One privileged owner, one CPU, soft-float code, fixed mappings and no other IRQ
  drivers or DMA. Initialization resets GIC enables/pending/active state; it must
  not be reused alongside another interrupt owner. No SIMD/FPU context is saved.
- During this EL1 diagnostic, IRQs are unmasked only in the assembly loop. Rust retains no live borrow of the
  round across handler access: its raw pointer is published while IRQs are masked
  and cleared before the stack value dies. The suspended caller and non-nested
  handler cannot access the round concurrently. The AtomicPtr alone would not
  make this safe for SMP or arbitrary interrupted Rust code.
- Successful handlers use neither UART, allocation nor service/coordinator state.
  The caller holds the normal timer owner; the handler accesses only the published
  round, saved frame and dedicated hardware. No locks are needed in this window.
- Return restores all integer registers, NZCV and SP; the third IRQ intentionally
  changes PC and I to finish the diagnostic. This is not a general process context.
- Unexpected non-spurious IDs, wrong PC/state or absent active round are fatal.
  The reporter takes UART permanently and never resumes. IDs 1020-1023 are treated
  as spurious: no tick and no EOI. Spurious delivery was not fault-injected.
- If IRQ delivery fails, the assembly loop cannot time out by itself. The verifier
  has a host-side deadline and reaps QEMU; manual use requires restarting QEMU.
  A host timeout does not recover the guest. Nested/corrupt kernel stacks, alternate
  boards, SMP, counter wrap/reset and general interrupt-driven services are untested here; see the separate EL0 tests.
- This busy loop and polling UART do not reduce CPU use. No energy, speed, phone
  support, agent isolation or AI claim follows. `user.run` occupies
  execution until it exits, faults or reaches its fixed deadline. [EL0 timeout](USER_TIMEOUT.md) is now
  implemented; a [bounded two-context scheduling diagnostic](SCHEDULING.md) also uses 20 ms
  slices. General task lifecycle remains future work; see [ROADMAP.md](ROADMAP.md).

## Observed verification

Rust 1.90.0 and QEMU 11.1.2 on macOS ARM64, debug and release:

- Four checks per normal image: 24 actual timer IRQs, both NZCV patterns, all 31
  GPRs, before/after SP, DAIF and hardware enable/active readback. The verifier also
  checks the ELF loop instruction is a branch to itself and its continuation is
  adjacent, and checks unchanged MMU state, repeated start/stop, bad arguments,
  context/user execution and memory/echo continuity.
- A separate `timer-test-unexpected-irq` image injects real SGI 1. It reaches slot 5
  and halts with the expected ID, exact loop PC, saved status and valid stack
  bounds. An info request receives no response during a 250 ms observation.
  The normal ELF excludes the injection marker; test features are mutually exclusive.
- At this increment the EL0 suite had eight image cases. Its additional `user-test-gic` case
  performs a forbidden GIC distributor read, verifies a level-3 permission fault
  at the exact instruction/address, then confirms the kernel remains usable.

Captured UART and assertion summaries: [debug](evidence/2026-10-28-timer-debug.txt),
[release](evidence/2026-10-28-timer-release.txt). New EL0 GIC captures:
[debug](evidence/2026-10-28-timer-user-gic-debug.txt),
[release](evidence/2026-10-28-timer-user-gic-release.txt).
Earlier evidence is retained verbatim. Broader regression results are recorded in
[WORKLOG.md](WORKLOG.md). The GUI did not change and received no new visual review.

## References

- [Arm Generic Timer](https://documentation-service.arm.com/static/633d612a4c59b30b51771183)
- [Arm timer interrupt handling](https://documentation-service.arm.com/static/6294c39db334256d9ea8ae53)
- [QEMU virt board](https://www.qemu.org/docs/master/system/arm/virt.html)
- [QEMU board register map](https://github.com/qemu/qemu/blob/master/hw/arm/virt.c)

The later [private-task-map increment](TASK_MEMORY.md) raises current table
ownership to twelve pages, leaving 244 pool pages free. Earlier six-table counts
and captures above describe the original timer/deadline increments.
