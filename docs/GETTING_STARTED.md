# Build and run RAI-OS

## Requirements

Rustup, QEMU >= 9.2 with qemu-system-aarch64 and virt-9.2, Python 3, and a POSIX
shell. On macOS install QEMU with `brew install qemu`. Install Rustup using
https://rust-lang.org/tools/install/ and load its environment:

```sh
. "$HOME/.cargo/env"
cd ~/Development/Rust/rai-os
rustup show
qemu-system-aarch64 --version
```

rust-toolchain.toml pins Rust 1.90.0, rustfmt, Clippy, and the bare-metal target.
Rustup downloads them on first use. Another computer can clone the public repo
and follow these steps. The kernel/offline checks need no API key, Apple linker
or cross-GCC. Live Meta modes require your own key; follow the
[checkout-root credential setup](META_BRIDGE.md#each-clone-needs-its-own-key).

## Kernel commands

```sh
cargo build --locked
cargo run --locked
python3 scripts/verify.py
python3 scripts/verify.py --release
cargo fmt --check
cargo clippy --bin rai-os --locked -- -D warnings
cargo build --release --locked
```

The debug ELF is target/aarch64-unknown-none-softfloat/debug/rai-os. Direct boot:

```sh
sh scripts/qemu.sh target/aarch64-unknown-none-softfloat/debug/rai-os
```

Expected real boot banner:

```text
RAI-OS: boot OK | AArch64 EL1 | UART PL011
```

Type help, info, uptime, or echo hello Rust, then Enter. The direct shell echoes input
and supports Backspace/Delete. Input is ASCII, without accented text or emoji.
Ctrl-C terminates QEMU. UART uses stdio without a multiplexed monitor; we do not
use -nographic. TCG emulates the same CPU on ARM64 or x86_64 hosts, but only macOS
ARM64 has been tested. Do not switch to HVF or change secure/virtualization/CPU
options without revisiting the boot contract. No disks or network are attached.

## Host interface

```sh
sh scripts/console.sh
```

A normal Rust host program starts QEMU and displays actual kernel replies.
Commands: help, info, uptime, echo <ASCII text>, exit. See [CONSOLE.md](CONSOLE.md).
This is a terminal tool, not the planned graphical host interface.
The programmatic wire format is documented in [PROTOCOL.md](PROTOCOL.md).

## Persistent guest worker

Inside either console, run `worker.start`, `worker.count notes@1`,
`worker.count notes@2`, `worker.status`, then `worker.stop`. The same EL0 worker
keeps its state between requests. Other EL0 diagnostics return `user_memory_busy`
until stop or worker failure releases the shared user RAM. See
[PERSISTENT_WORKER.md](PERSISTENT_WORKER.md) for limits and verification.
No API key is required; the Meta bridge remains a separate host program.

## Original host example

```sh
cargo run --example host_hello --target aarch64-apple-darwin
```

This program runs on macOS, unlike the kernel. Generic cargo test is not a
bare-metal integration test; use the real-QEMU checks above.

## Troubleshooting

- Missing Cargo: load `$HOME/.cargo/env`.
- Homebrew Rustup 1.29.2 crashed during setup; official Rustup 1.28.2 installed
  the required toolchain successfully. This was not a kernel failure.
- Missing virt-9.2: update QEMU.
- A timeout is a failed check, not evidence of boot. Inspect
  target/verification/uart.log (or uart-release.log). Empty output can mean
  execution never reached Rust.
- Boot requires EL1. Rust panic prints a message and halts; restart QEMU.

## Graphical host panel

Run `sh scripts/panel.sh` from the repository root. Select Start kernel to boot
the real guest, Send message for an echo round trip, and Stop kernel to finish.
The mobile-shaped view is rendered by the host, not the guest kernel.
See [PANEL.md](PANEL.md) for dependencies, architecture, and verification.
