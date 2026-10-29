# Two resumable EL0 contexts on one CPU

Implemented October 28, 2026. `schedule.check` runs a bounded round-robin diagnostic:
two linked EL0 loops alternate for six 20 ms slices each. An IRQ pauses the current
context, the kernel saves its integer state in RAM, and a later turn restores it.
This demonstrates preemptive context switching, not a general process runtime.

## Try it

```sh
. "$HOME/.cargo/env"
sh scripts/console.sh
```

Enter `schedule.check`, then `user.run`, `timer.check` and `echo still running`.
A successful schedule reply has the following field order; numbers below are
placeholders, not measured results:

```text
tasks=2; slice_ms=20; irqs=12; switches=11; trace=ABABABABABAB; a_turns=6; b_turns=6; a_ms=<decimal>; b_ms=<decimal>; progress=<12 comma-separated counters>; context=true; stacks=true
```

`progress` follows the trace: A's first counter, B's first counter, A's second,
and so on. Both subsequences must strictly increase. B increments by three, A by
one; their counters are synthetic work, not a useful-work or performance benchmark.
Each task accumulates at least 120 ms of observed slice intervals. This shows why
an application's lifetime is different from the duration of a CPU turn.

The older `user.run`/`user.spin` commands still use their separate 100 ms invocation
deadline. That is a diagnostic policy, not an architectural maximum for future
applications. Here a 20 ms expiry pauses a context; the twelve-turn test limit
ends the diagnostic. Both paused contexts are then discarded. No application
exits voluntarily, and nothing remains scheduled after the command returns.

## Context versus program memory

| Data | What happens at a switch |
| --- | --- |
| x0-x30, saved PC, NZCV/DAIF and SP_EL0 | Saved in a kernel-owned Frame in RAM; restored on the context's next turn. |
| User code and constants | Stay in their existing mappings. |
| User stack contents | Stay in their assigned RAM half; the other context uses a different half. |
| Kernel caller and handler stacks | Retained privately at EL1, inaccessible to EL0. |

The CPU has one currently executing register set per core. The two stored Frames
let this single core alternate state. There is no copying of the entire program
at every switch and no change to translation tables in this increment.

## Implementation and safety boundaries

- `src/schedule/mod.rs` owns two Frames and chooses A/B in a fixed order. Initial
  x1-x30 seeds and NZCV patterns differ by context. x0 starts at zero and records
  work. All initial PCs, stacks and status values come from kernel code, never UART.
- The existing 8 KiB user-stack mapping is split into two 4 KiB halves. Each loop
  uses the last sixteen bytes of its half: a counter store and a distinct marker.
  Outer guard pages remain; there is no guard between the halves. Both halves
  are accessible from either EL0 context in the shared address space. These are
  separate storage assignments, NOT enforced isolation between processes.
- `src/schedule/programs.S` uses small assembly probes so all integer registers
  and flags have predictable values. Each normal loop is exactly ADD/STR/B, with
  no syscall, yield, timer read or voluntary exit. General application code remains
  future work; the kernel and orchestration are Rust.
- The timer owner programs a fresh absolute 20 ms deadline for each selected turn.
  Kernel execution remains masked. `resume.S` saves the C caller's callee-saved
  registers on the private EL1 stack, loads PC/status/SP_EL0 and restores all 31
  user GPRs from a read-only input Frame. x30 is a temporary pointer restored last.
  ERET enables IRQs only at EL0. SIMD/FPU state is not supported by this soft-float
  runtime, and no task address-space register changes are needed here.
- Slot 9 acknowledges/disables CNTP before GIC EOI and copies the interrupted Frame
  into a separate kernel-owned Observation. Slot 8 also captures a synchronous
  event, but that aborts the diagnostic. The existing fixed `user_return` trampoline
  returns to masked kernel Rust, rather than restoring an unchecked user frame.
- After each return the active pointer is cleared and timer/GIC shutdown/masks are
  checked. Only an expired slice delivered through vector 9 may advance the test.
  Before retaining a frame, the kernel validates PC/alignment, exact status and SP,
  x1-x30, x0 progress/arithmetic, stack marker and its stored counter. The counter
  may lag x0 by one increment if interrupted between ADD and STR. Bad state is
  never passed back into the restore function.
- The selected context retains the actual interrupted PC and registers. The next
  turn for that context restores those values. The other context's Frame remains
  in kernel RAM. Successful output is emitted only after all twelve observations
  pass; the trace/counters come from those observations.

One CPU, no DMA/remapping, valid kernel stacks and masked non-nested handlers are
required. The fully initialized read-only input Frame is separate from the IRQ
output Observation, whose pointer is published only while its caller is suspended.
The caller does not access that output during EL0 execution. The shared entry guard
excludes recursive dispatch; the pointer is cleared before moving any result or
leaving scope. Neither this AtomicPtr nor the stored Frames make the code SMP-safe.
The legacy user invocation and schedule diagnostic cannot own user RAM concurrently.

An IRQ pauses user work; kernel Rust then stops/checks hardware and selects the
next context. This is not a direct frame replacement inside the IRQ handler.
Observed `a_ms`/`b_ms` include entry/handler time within each slice, rounded down
per slice, and exclude masked work between slices. They are not precise CPU
accounting, end-to-end host timing, battery telemetry or guaranteed latency.

## Verification and limits

```sh
python3 scripts/verify_schedule.py
python3 scripts/verify_schedule.py --release
```

Rust 1.90.0, QEMU 11.1.2, macOS ARM64. Each profile passed:

- Four normal checks (three structured and one direct shell): 48 actual EL0 timer
  IRQs and 44 context switches. The verifier checks both ELF loops' exact ADD/STR/B
  instructions and sizes, the reported alternating trace, six increasing counters
  per context, B's arithmetic and at least 120 ms of observed slices per context.
- `schedule-test-corrupt-register` zeros A's x7. Its saved frame is rejected with
  `schedule_context_corrupt`. `schedule-test-user-fault` injects a BRK in B after
  A's first slice; the run returns `schedule_user_fault`. Each negative image is
  invoked twice and must leave the kernel and timer reusable.
- Normal hello runs, EL1 context/timer checks, the previous spin timeout, unchanged
  MMU/page ownership, malformed arguments and echo all remain usable after these
  checks. The pure timer-budget test covers distinct 20/100 ms boundaries.

Captured actual output: [debug](evidence/2026-10-28-schedule-debug.txt) and
[release](evidence/2026-10-28-schedule-release.txt). Historical captures stay intact;
regressions are recorded in [WORKLOG.md](WORKLOG.md). Test features are mutually
exclusive and use separate build directories. Normal images contain no injected
MOV/BRK prefix in these probes.

This is a fixed two-context scheduling diagnostic. It has no dynamic task creation,
exit/wait lifecycle, runnable queue, priority policy, IPC or isolation between user
contexts. The host coordinator and graphical panel have not moved into the guest.
Console/cooperative-job progress waits until this bounded command returns. A user
fault aborts the whole diagnostic rather than continuing the other context.

If timer delivery fails, a loop can still hang; the verifier's host timeout reaps
QEMU but does not recover the guest. Corrupted kernel stacks, spurious/unexpected
IRQs, full stack exhaustion, cross-context malicious memory writes, missing timer
cleanup, counter resets/wrap and SMP were not injected. The stack check observes
two words per half, not every byte of stack capacity. The separate
[schedule.run/schedule.fault lifecycle](TASK_LIFECYCLE.md) now supports explicit
completion/fault states and continuing a ready peer. The original schedule.check
remains a strict all-or-error register diagnostic.
