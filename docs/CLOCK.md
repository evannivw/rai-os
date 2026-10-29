# Kernel clock and uptime

Implemented October 26, 2026. The kernel reads the emulated Arm generic physical
counter and reports elapsed milliseconds since clock initialization. This is
actual guest execution in QEMU, not a UI animation or hard-coded uptime.

## Try it

From the repository root:

```sh
. "$HOME/.cargo/env"
sh scripts/console.sh
```

Enter `uptime`, wait briefly, and enter `uptime` again. Both requests reach the
running AArch64 kernel. Example from the captured debug verification run:

```text
uptime_ms=11; counter_hz=62500000
uptime_ms=267; counter_hz=62500000
```

These are recorded values, not promises about the next boot. `uptime_ms` counts
elapsed milliseconds, rounded down; `counter_hz` is ticks per second. Counter
frequency is not CPU frequency. The direct UART shell and structured protocol
also accept uptime. The host console's exit command stops its QEMU child.

## Hardware boundary

- `clock.rs` reads CNTFRQ_EL0 for frequency and CNTPCT_EL0 for count. An ISB
  precedes count reads to order them after preceding instructions. The approach
  follows Arm's [Generic Timer guide](https://documentation-service.arm.com/static/66c4754f32f35b31ceb317ff).
- The existing supported boot is QEMU virt-9.2, Cortex-A53, TCG, one CPU, EL1,
  secure=off and virtualization=off. It permits these reads and supplies the
  counter frequency. Other boot environments may trap or need setup; they have
  not been tested. Unsafe blocks document these assumptions.
- Clock initialization occurs after Assembly stack/BSS setup and before UART
  initialization. It does not include all time since QEMU process creation or
  reset. Invalid/zero frequency halts through the existing panic path before
  a successful boot banner. The invalid-frequency path has not been fault-injected.
- Conversion uses integer arithmetic with a widened intermediate, avoiding
  overflow in ticks times 1000, and saturates the public u64 result. Supported
  sessions exclude counter resets, wraps, and live migration. They are not tested.
- Command execution receives an uptime value; it does not access registers.
  UART streams the formatted response without heap allocation. The host receives
  the value through the same bounded protocol as help/info/echo.

## What this does not add

The clock module itself only reads the counter. The later [timer diagnostic](TIMER.md)
programs CNTP and handles controlled EL1 IRQs; exception vectors and a host panel
also exist now. There are still no alarms, sleeping tasks or scheduler. UART still polls. A readable clock alone does
not make the kernel multitask or reduce its CPU use. This is elapsed time, not a
calendar clock, CPU usage, RAM usage, battery measurement, or performance claim.

The guest counter belongs to the emulator's time domain. Pausing QEMU or changing
its timing mode can affect its relationship with host elapsed time. Later demo
benchmarks must use host monotonic end-to-end timing including model overhead;
never replace that metric with guest uptime.

## Verification

`python3 scripts/verify.py` and `python3 scripts/verify.py --release` each boot
real QEMU and perform 37 exchanges. New checks cover structured and shell uptime,
invalid arguments, positive/stable frequency, progress across a 250 ms host wait,
and repeated nondecreasing readings. The deliberately broad 100-10,000 ms
progress window detects gross unit errors; it is not a calibration benchmark.
Existing malformed-input and recovery checks remain included.

`python3 scripts/verify_console.py` also verifies an uptime request through the
Rust host console. Formatting, kernel/host Clippy, and the existing two host
validation unit tests passed. Raw latest logs are in target/verification;
this increment's [captured uptime evidence](evidence/2026-10-26-uptime.txt) is
retained in Git. No physical board or non-macOS host was tested.
