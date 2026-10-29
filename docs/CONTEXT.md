# One resumable EL1 context

Implemented October 28, 2026. The kernel can now handle one explicitly allowed
synchronous SVC and resume the interrupted EL1h code. It preserves the integer
execution state used by this runtime. Other exceptions remain fatal. This is a
prerequisite for service calls, not by itself an EL0 syscall ABI or a scheduler. The subsequent
[EL0 program increment](USER_PROGRAM.md) reuses and extends this frame.

## Try it

```sh
sh scripts/console.sh
```

Enter `context.check` and then `echo still running`. The first command performs
two real SVC/ERET round trips in the guest and returns:

```text
round_trips=2; registers=31; flags=true; stack=true; resumed=true
```

For each trip, assembly seeds all x0-x30 with distinct 64-bit values, selects one
of two complementary NZCV flag patterns, and records the stack pointer. After
return it captures the actual registers, flags, DAIF masks and SP before using
scratch registers. Rust compares observations to expected values. A mismatch
returns `context_corrupt`; a fatal exception or missing return is a failed check,
not a success report. The counters are per invocation, not lifetime telemetry.
This diagnostic is bounded, synchronous and does not use the physical-page pool.

## Entry and return

1. SVC #0x5242 enters vector slot 4 (current EL, SP_EL1, synchronous).
2. Assembly saves all 31 general registers on the normal stack before reusing
   any register. It captures ELR, SPSR, ESR and FAR in a 288-byte aligned frame.
3. The nested-entry guard is set; Rust dispatch runs on the dedicated exception
   stack. The frame remains on the suspended normal stack.
4. Pure policy requires the exact SVC syndrome, the exact linked return address
   of our probe, EL1h and all DAIF masks set. Only NZCV may vary in saved status.
5. For that event, dispatch returns without touching UART, allocator state or
   the frame. Assembly restores ELR/SPSR, clears the guard, restores x0-x30/SP,
   and executes ERET. For SVC, ELR already points after the instruction: adding
   four would incorrectly skip an instruction.
6. Other EL1 events use the fatal report and halt. The later EL0 increment adds
   a separate slot-8 path for user syscalls and contained user faults. Other slots
   retain fatal-only entry.

The register layout is explicit in `src/context/mod.rs` with compile-time size
and offset assertions matching `src/exceptions.S`. The acceptance policy is
in `src/context/policy.rs`, independent of registers, UART or QEMU. The probe
in `src/context/probe.S` preserves its Rust caller's AAPCS64 callee-saved registers
separately from the values used to test exception restoration. Neither normal
commands nor host requests supply arbitrary PCs, saved status or raw contexts.

## Safety and limits

- One CPU, EL1h, masked interrupts, installed vectors and active identity mappings.
  The normal stack must be valid/aligned with at least 288 bytes available at
  exception entry. Unlike the original fatal-only slot 4, entry now writes a
  frame before switching stacks. A damaged/full stack can prevent diagnostics.
- The exception stack must remain mapped, private to this entry path and large
  enough for the handler. The immutable Rust frame borrow ends before restoration.
  No locks, mutable service state or interrupted UART writer are entered on the
  successful path. Rust cannot unwind across the assembly boundary.
- The guard blocks recursive dispatch after a successful initial frame save. A
  nested entry may write another frame before checking it; stack-save faults are
  not covered. The guard resets only for the approved return. Nested faults and
  stack corruption have not been injected in this increment.
- The later EL0 increment stores SP_EL0 in the former reserved frame slot.
  No SIMD/FPU, process address-space state or thread-local registers are saved. The target is `aarch64-unknown-none-softfloat`; the runtime and probe
  do not use SIMD/FPU. This frame is not yet a complete process context.
- IRQ/FIQ/SError servicing, preemption, recovery from memory faults and arbitrary
  exception returns remain unsupported. No isolation or performance gain is claimed.
- The allowlist constrains this diagnostic, not malicious privileged kernel code.
  All current modules still share EL1 authority.

## Verification

```sh
python3 scripts/verify_context.py
python3 scripts/verify_context.py --release
```

On Rust 1.90.0 / QEMU 11.1.2 / macOS ARM64, each profile passed:

- One pure policy test covering valid NZCV combinations, wrong class/immediate,
  instruction length, reserved syndrome/status bits, return address and mode/masks.
- 21 structured exchanges and two shell commands: 18 successful diagnostics,
  totaling 36 SVC returns, plus argument rejection and memory/echo continuity.
- Two isolated negative boot images: an unknown SVC number at another site, and
  the allowed number at an unapproved site. Both first complete valid round trips,
  then halt. The verifier checks exact syndrome, ELF return PC, stacks and saved
  state; an info request must produce no response while QEMU remains alive.

Negative features `context-test-unknown-svc` and `context-test-wrong-site` are
mutually exclusive with every other fault-injection feature. They build into
`target/context-self-test/<case>` and never replace the normal image. Repeated
success checks exercise guard reset; the negative tests also check fatal entry
still works after successful returns. The shared QEMU capture helper always
terminates and reaps its child. Captures: [debug](evidence/2026-10-28-context-debug.txt)
and [release](evidence/2026-10-28-context-release.txt).

The original BRK and all five MMU fault tests are regression checks for this
entry change. Their earlier captures remain historical; current test results
are recorded in [WORKLOG.md](WORKLOG.md). That lower-privilege increment is now implemented in [USER_PROGRAM.md](USER_PROGRAM.md);
see [ROADMAP.md](ROADMAP.md) for subsequent scheduling/service work.

Reference: [Arm AArch64 exception model](https://documentation-service.arm.com/static/63a065c41d698c4dc521cb1c)
for saved return state, SVC return address and ERET semantics.

## Later timer use

The [timer diagnostic](TIMER.md) reuses the 288-byte frame and snapshot validation
for slot 5. Its assembly loop permits asynchronous IRQs only at a known PC.
The original SVC allowlist and its rejection tests are unchanged. This is not a
general scheduler or permission to unmask IRQs in arbitrary Rust code.

The later [scheduling diagnostic](SCHEDULING.md) copies the shared Frame into
kernel RAM and restores all user integer registers on a later turn. Its bounded
EL0 restore path is separate from the original controlled EL1 SVC diagnostic.
