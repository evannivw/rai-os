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
exit. Exit or input EOF terminates and reaps the QEMU child. The direct guest
console remains available through cargo run --locked.

## Boundaries

- examples/control_console/model.rs: pure operations and protocol validation;
  no process, terminal, or hardware access. Rejects injected separators,
  oversized requests, mismatched reply IDs, and terminal controls in responses.
- examples/control_console/qemu.rs: process/UART adapter. Requires both boot
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

The user wants a graphical host interface. Design it around real events and
state before adding constrained jobs and a Meta coordinator. Agent permissions,
resource budgets, and AI are pending. QEMU configuration is not measured CPU,
GPU, battery, or RAM usage. Future simulated budgets must be labeled explicitly.
