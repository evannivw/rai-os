# RAI-OS

An educational AArch64 operating-system prototype in Rust, exploring how an
AI coordinator could adapt work to goals, events, and resource limits.

**Working today:** an optional [native guest display](docs/GUEST_DISPLAY.md), an [exact EL0 record index with independent checking](docs/RECORD_INDEX.md), bare-metal boot in QEMU, fatal EL1 exception diagnostics, a checked EL1 context return, controlled EL1 timer interrupts, one EL0 Rust program with bounded syscalls and a timer-enforced deadline, a two-context scheduling diagnostic, bounded task completion and fault handling, task-private stack mappings, bounded scalar messages between two tasks, blocking receive with wakeup, a bounded EL0 record-count service, a persistent EL0 worker with protected session memory, a [guest EL0 coordinator](docs/GUEST_COORDINATOR.md) with versioned proposal admission and a [Meta transport bridge](docs/GUEST_META_BRIDGE.md), PL011 UART input/output, help/info/uptime/echo,
a bounded structured protocol, a physical-page pool with diagnostics, active EL1 memory permissions, a cooperative word-count task, a Rust host terminal
console, a graphical host control panel, a bounded priority host coordinator, a
[bounded event-driven Meta coordinator](docs/META_COORDINATOR.md), and a
[Meta host bridge](docs/META_BRIDGE.md) that validates one real model proposal and executes a guest record operation.
**Not implemented:** a general autonomous OS coordinator, guest-resident agents, a window system or interactive guest desktop, general multi-task
scheduling, dynamically created process address spaces, persistent storage, or measured efficiency gains.

A [retained index mode](docs/RETAINED_INDEX.md) now transfers exact EL0 results
and reuses completed immutable revisions, with native created/reused views.
The [Meta index bridge](docs/META_INDEX_BRIDGE.md) now drives this path with
model proposals and independently auditable guest artifacts.
[Complete index sets](docs/INDEX_SETS.md) now combine required revisions and
source bytes, gate incomplete publication and reuse unchanged work.
The [whole-set Meta bridge](docs/META_SET_BRIDGE.md) now chooses among missing
revisions with guest-validated goal generations.
A [policy comparison pilot](docs/POLICY_PILOT.md) now runs two fixed policies and
Meta through the same guest, with measured deadlines and explicitly simulated quotas.
[First measured results](docs/POLICY_PILOT_RESULTS.md): both fixed rules passed 6/6;
the current per-action Meta configuration passed 1/6. No AI efficiency gain is claimed.

## Run

Install Rustup and QEMU (9.2 or newer with virt-9.2), then:

```sh
. "$HOME/.cargo/env"
cargo run --locked             # Direct kernel UART console
sh scripts/console.sh          # Rust host terminal controlling QEMU
sh scripts/panel.sh            # Native graphical host panel
sh scripts/desktop.sh          # RAI-OS draws its own screen inside QEMU
sh scripts/bundle.sh --graphics # Three exact multi-record sets, independent checks
sh scripts/index.sh notes@1 target/notes-v1.index  # New output path; real guest index + checker
# After configuring a local key as described in docs/META_BRIDGE.md:
sh scripts/meta.sh 'Count the words in the latest available revision of the notes.'
sh scripts/meta.sh --coordinate fixtures/coordinator/v1/revisions.json
sh scripts/meta.sh --guest fixtures/guest_bridge/v1/observations.json
sh scripts/meta.sh --graphics --guest-index fixtures/guest_index/v1/observations.json
sh scripts/meta.sh --graphics --guest-set fixtures/guest_set/v1/goals.json
```

Each person supplies their own API key in the checkout-root `.env.local`; see
[exact credential setup](docs/META_BRIDGE.md#each-clone-needs-its-own-key). The
[public template](config/meta.env.example) contains no key. Offline tests need none.

The repository pins Rust 1.90.0 and aarch64-unknown-none-softfloat.
Only macOS ARM64 has been tested. The host console is not a kernel GUI.

## Verify

```sh
cargo fmt --check
cargo clippy --bin rai-os --locked -- -D warnings
python3 scripts/verify.py
python3 scripts/verify.py --release
python3 scripts/verify_exceptions.py
python3 scripts/verify_exceptions.py --release
python3 scripts/verify_context.py
python3 scripts/verify_context.py --release
python3 scripts/verify_timer.py
python3 scripts/verify_timer.py --release
python3 scripts/verify_schedule.py
python3 scripts/verify_schedule.py --release
python3 scripts/verify_lifecycle.py
python3 scripts/verify_lifecycle.py --release
python3 scripts/verify_isolation.py
python3 scripts/verify_isolation.py --release
python3 scripts/verify_ipc.py
python3 scripts/verify_ipc.py --release
python3 scripts/verify_wait.py
python3 scripts/verify_wait.py --release
python3 scripts/verify_display.py
python3 scripts/verify_display.py --release
python3 scripts/verify_pilot.py
python3 scripts/verify_pilot.py --release
python3 scripts/verify_set_bridge.py
python3 scripts/verify_set_bridge.py --release
python3 scripts/verify_bundle.py
python3 scripts/verify_bundle.py --release
python3 scripts/verify_coord_index.py
python3 scripts/verify_coord_index.py --release
python3 scripts/verify_record_index.py
python3 scripts/verify_record_index.py --release
python3 scripts/verify_index_export.py
python3 scripts/verify_service.py
python3 scripts/verify_service.py --release
python3 scripts/verify_worker.py
python3 scripts/verify_worker.py --release
python3 scripts/verify_guest_coordinator.py
python3 scripts/verify_guest_coordinator.py --release
python3 scripts/verify_guest_index_bridge.py
python3 scripts/verify_guest_index_bridge.py --release
python3 scripts/verify_guest_bridge.py   # Offline proposals, real guest coordinator
python3 scripts/verify_guest_bridge.py --release
python3 scripts/verify_meta.py          # Offline; no provider calls
python3 scripts/verify_meta.py --release
python3 scripts/verify_coordinator.py   # Offline events/proposals, real QEMU
python3 scripts/verify_coordinator.py --release
python3 scripts/verify_user_timeout.py
python3 scripts/verify_user_timeout.py --release
python3 scripts/verify_user.py
python3 scripts/verify_user.py --release
python3 scripts/verify_memory.py
python3 scripts/verify_memory.py --release
python3 scripts/verify_mmu.py
python3 scripts/verify_mmu.py --release
python3 scripts/verify_console.py
python3 scripts/verify_panel.py
python3 scripts/verify_tasks.py
python3 scripts/verify_tasks.py --release
```

Python 3 is required for integration checks. Each kernel run verifies real boot
and 37 console/protocol cases, including the guest counter. See the guides for host unit tests.

## Documentation

- [Exact record index](docs/RECORD_INDEX.md)
- [Native guest display](docs/GUEST_DISPLAY.md)
- [Original native graphics scope](docs/GUEST_GRAPHICS_PLAN.md)
- [Getting started](docs/GETTING_STARTED.md)
- [Architecture and unsafe contracts](docs/ARCHITECTURE.md)
- [UART protocol](docs/PROTOCOL.md)
- [Kernel-to-service roadmap](docs/ROADMAP.md)
- [MMU tables and permissions](docs/MMU.md)
- [Physical pages and memory diagnostics](docs/MEMORY.md)
- [Event-driven Meta coordinator](docs/META_COORDINATOR.md)
- [Meta bridge, local credentials and live evidence](docs/META_BRIDGE.md)
- [Current delivery plan through November 4](docs/DEMO_PLAN.md)
- [Meta transport for the guest coordinator](docs/GUEST_META_BRIDGE.md)
- [Guest coordinator and proposal boundary](docs/GUEST_COORDINATOR.md)
- [Persistent guest worker and memory ownership](docs/PERSISTENT_WORKER.md)
- [First bounded guest record service](docs/RECORD_SERVICE.md)
- [Waiting, wakeup and closed-batch deadlock handling](docs/IPC_WAIT.md)
- [Bounded task messages and syscall ABI](docs/IPC.md)
- [Private task stacks and hardware access checks](docs/TASK_MEMORY.md)
- [Task records, completion and fault lifecycle](docs/TASK_LIFECYCLE.md)
- [Two resumable EL0 contexts](docs/SCHEDULING.md)
- [EL0 timeout and non-cooperative program diagnostic](docs/USER_TIMEOUT.md)
- [First EL0 Rust program and syscall ABI](docs/USER_PROGRAM.md)
- [Controlled timer interrupts](docs/TIMER.md)
- [Resumable EL1 context](docs/CONTEXT.md)
- [Kernel exception diagnostics](docs/EXCEPTIONS.md)
- [Kernel clock and uptime](docs/CLOCK.md)
- [Cooperative task and cancellation](docs/TASKS.md)
- [Small host coordinator](docs/COORDINATOR.md)
- [Host console](docs/CONSOLE.md)
- [Graphical Device Lab panel](docs/PANEL.md)
- [Hackathon plan](docs/HACKATHON_PLAN.md)
- [Proposed comparison experiment](docs/DEMO_EXPERIMENT.md)
- [Required demo result contract](docs/DEMO_ACCEPTANCE.md)
- [Correctness fixture and verifier implementation sequence](docs/DEMO_FIXTURE.md)
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
