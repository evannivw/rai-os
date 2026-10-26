# RAI-OS

An educational AArch64 operating-system prototype in Rust, exploring how an
AI coordinator could adapt work to goals, events, and resource limits.

**Working today:** bare-metal boot in QEMU, PL011 UART input/output, help/info/echo,
a bounded structured protocol, and a Rust host terminal console.
**Not implemented:** Meta integration, autonomous agents, graphical UI, task
scheduling, memory isolation, persistent storage, or measured efficiency gains.

## Run

Install Rustup and QEMU (9.2 or newer with virt-9.2), then:

```sh
. "$HOME/.cargo/env"
cargo run --locked             # Direct kernel UART console
sh scripts/console.sh          # Rust host terminal controlling QEMU
```

The repository pins Rust 1.90.0 and aarch64-unknown-none-softfloat.
Only macOS ARM64 has been tested. The host console is not a kernel GUI.

## Verify

```sh
cargo fmt --check
cargo clippy --bin rai-os --locked -- -D warnings
python3 scripts/verify.py
python3 scripts/verify.py --release
python3 scripts/verify_console.py
```

Python 3 is required for integration checks. Each kernel run verifies real boot
and 28 console/protocol cases. See the guides for host unit tests.

## Documentation

- [Getting started](docs/GETTING_STARTED.md)
- [Architecture and unsafe contracts](docs/ARCHITECTURE.md)
- [UART protocol](docs/PROTOCOL.md)
- [Host console](docs/CONSOLE.md)
- [Hackathon plan](docs/HACKATHON_PLAN.md)
- [Work log and evidence](docs/WORKLOG.md)
- [Contribution rules](AGENTS.md)

## Historical first step

The project began on October 13, 2026 as a normal macOS Rust program, without a
kernel. The original README and source remain available in commit 3e6d9c9.
At that time Xcode and GitHub SSH worked, but Rust downloads failed; build/check,
formatting, and test commands could not run until Rust was installed on Oct 26.
The original host example remains at examples/host_hello.rs; its output is now
English. Run it on an ARM64 Mac with:

```sh
cargo run --example host_hello --target aarch64-apple-darwin
```

Cargo.toml describes the package; edition 2024 selects language rules, not a
compiler version. Cargo.lock records dependencies (none external currently),
.gitignore excludes generated target/ and credential files, and LICENSE is
Apache-2.0. rustc compiles Rust; Cargo manages building and dependencies.

## Event status

Prepared for the Meta Global AI Developer Hackathon 2026. Keep Oct 13 and Oct 26
work identifiable; eligibility needs clarification because the supplied guide
separates the Oct 26 kickoff from the Oct 27-Nov 4 build period. No dates have
been altered. New project text and commits are in English; historical evidence
and already-published commit messages retain their original form.
