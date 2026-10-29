# Host console

A Rust terminal application running on the host. It starts QEMU and exchanges
structured UART requests with the kernel. It is not a graphical kernel desktop
or an AI agent. AI and resource telemetry are explicitly marked unimplemented.
The uptime command reads the actual guest counter; it is elapsed guest time,
not CPU utilization or energy telemetry. See [CLOCK.md](CLOCK.md).

## Run

From the repository root:

```sh
. "$HOME/.cargo/env"
sh scripts/console.sh
```

The script builds the AArch64 kernel, detects the compiler host target, and builds
and runs control_console for that target. Host code uses std; the kernel remains
no_std/no_main. No dependencies were added. Only macOS ARM64 has been verified;
Linux target detection is supported by the script but untested.

Connection triggers a real info request. Commands: help, info, uptime, echo <ASCII text>,
exit. Task commands are `task.start <text>`, `task.status`, and `task.cancel <id>`;
see [TASKS.md](TASKS.md). Exit or input EOF terminates and reaps the QEMU child. The direct guest
console remains available through cargo run --locked.

## Boundaries

- host/shared/model.rs: pure operations and protocol validation;
  no process, terminal, or hardware access. Rejects injected separators,
  oversized requests, mismatched reply IDs, and terminal controls in responses.
- host/shared/qemu.rs: process/UART adapter. Requires both boot
  banner and Console ready, with a 10-second timeout and one pending request.
  A transport failure closes the session rather than retrying side effects.
- examples/control_console/view.rs: presentation of observed results.
- examples/control_console.rs: composition and use-case coordination.
- The kernel depends on none of these files.

Clean Architecture concerns responsibility and dependency direction. Add a trait
when a real second transport or use case requires it, not to fill an empty diagram.

## Verify

```sh
cargo test --locked --example control_console --target aarch64-apple-darwin
cargo clippy --locked --example control_console --target aarch64-apple-darwin -- -D warnings
python3 scripts/verify_console.py
```

On other hosts replace the target with the host value from rustc -vV. Integration
checks cover real boot, info/uptime/echo, TAB input rejection and recovery, missing ELF,
and EOF cleanup. Output: target/verification/host-console.log.

## Limits and next step

This is plain terminal output, without graphical controls or redraw. Connection
is checked at startup and on exchanges, not continuously; there is no automatic
reconnection. The adapter trusts the local QEMU executable and project kernel;
it is not a sandbox for hostile host programs.

The [graphical host panel](PANEL.md) now shares these model/transport modules.
It renders actual boot, clock, and command results before constrained jobs or
a Meta coordinator are introduced. Agent permissions,
resource budgets, and AI are pending. QEMU configuration is not measured CPU,
GPU, battery, or RAM usage. Future simulated budgets must be labeled explicitly.

## Memory foundation

The host terminal accepts `memory.info`, `memory.check`, and `mmu.info` with no arguments.
These query/test the guest's reserved page pool and read active translation
registers; see [MEMORY.md](MEMORY.md) and [MMU.md](MMU.md).
No arbitrary addresses can be read, written, or freed through this interface.

## Context return diagnostic

Enter `context.check` to run two real, bounded EL1 context round trips. The guest
checks integer registers, flags and stack restoration before replying. See
[CONTEXT.md](CONTEXT.md). This does not create or switch processes; the graphical
panel has no additional controls for it.

## Built-in EL0 program

Enter `user.run` to start the linked lower-privilege Rust program. Its write/exit
calls are numeric SVCs inside QEMU, distinct from this host command. The terminal
displays the returned state and output. A faulted state is not program success,
even if the request itself was handled. See [USER_PROGRAM.md](USER_PROGRAM.md).

Enter `timer.check` to receive six actual timer IRQs in two controlled EL1
rounds. A successful reply reports preserved integer state and disabled timer
afterward. See [TIMER.md](TIMER.md); this does not schedule the EL0 program.

Enter `user.spin` to run a deliberately infinite EL0 loop. The kernel terminates
it after the fixed deadline and returns `state=timed_out`; this is not success
for the user program. Then enter `user.run` to verify normal execution continues.
The [EL0 timeout guide](USER_TIMEOUT.md) explains the measured result and limits.

Enter `schedule.check` to alternate two linked EL0 contexts on one CPU. The reply
shows A/B order and twelve observed counters; each context continues on its later
turns. This is a bounded diagnostic with shared user mappings, not isolated
processes. See [SCHEDULING.md](SCHEDULING.md).

## Task completion and fault handling

Enter `schedule.run`: two finite Rust tasks return numeric results and finish.
Enter `schedule.fault`: A deliberately reads kernel memory, is stopped by the
MMU, and B still finishes. The host displays the real guest report; `a_state`
and `b_state` distinguish exited, faulted and limit_reached. No new graphical
panel controls are included. See [TASK_LIFECYCLE.md](TASK_LIFECYCLE.md).

## Private task stacks

Enter `schedule.isolate` to run the hardware access diagnostic: four denied reads,
four denied writes and twelve successful own-page accesses, across two rounds.
The guest also checks the complete stack pages and restores the base MMU root.
The command accepts no arguments and is available through the host terminal or
direct UART. See [TASK_MEMORY.md](TASK_MEMORY.md). No graphical controls changed.

## Messages between tasks

Enter `ipc.check`. A sends the number 41 to task ID 2, B responds with 42 to ID 1,
and both Rust tasks exit with code=0. The console displays actual task values,
message counters and syscall/scheduling trace. The kernel mediates the exchange;
the host does not calculate or transport these guest-to-guest messages. The
[bounded IPC guide](IPC.md) explains empty/full/closed inboxes and the current
two-task limit. The graphical panel is unchanged.
