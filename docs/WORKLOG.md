# Work log and provenance

Dates use America/Guatemala (UTC-06:00). Preserve real dates and distinguish
pre-existing work from eligible event contributions. Historical captured bytes
and already-published commit messages remain unchanged.

## Before October 26

- 46e681e: Oct 13, initial commit.
- 3e6d9c9: Oct 13, normal macOS Rust example, README, and Apache-2.0 license.
- The initial local inspection found a clean tree at
  /Users/jv/Development/Rust/rai-os on an ARM64 Mac, Darwin 25.3.0, with origin
  git@github.com:evannivw/rai-os.git. No AGENTS.md existed in the repo or ancestors.
- Original README/source are preserved in Git history. The host example remains
  in examples/host_hello.rs; current text/output were later translated to English.
  LICENSE was not changed.

## October 26: minimal boot, then milestone 1

Implemented Assembly entry, linker layout, no_std/no_main Rust, and UART output.
Verified the real boot banner before expanding to input, help/info/echo, bounded
structured requests, validation, and recovery. No external crate dependencies,
API keys, Meta integration, heap, userspace, or agent isolation.

Verification actually performed on this Mac:

- Rust/Cargo 1.90.0; target aarch64-unknown-none-softfloat; bundled rust-lld.
- file identified a statically linked AArch64 ELF.
- QEMU 11.1.2, virt-9.2, Cortex-A53, TCG, one CPU, 128 MiB.
- Real initial boot, followed by 28 console/protocol checks in debug AND release,
  plus boot in each profile. Includes editing, valid requests, bad versions/IDs,
  argument errors, unknown operations, NUL/escape/non-ASCII, exact 256-byte limit,
  overflow, recovery, CRLF, consecutive requests, and fragmented delivery.
- Formatting and Clippy with warnings denied passed.
- Interactive cargo run: echo and Ctrl-C exit verified.
- Original macOS example compiled and ran with its original Spanish output.

Historical evidence: [first boot](evidence/2026-10-26-first-boot.txt),
[debug UART](evidence/2026-10-26-debug-uart.txt),
[release UART](evidence/2026-10-26-release-uart.txt).
These are earlier actual captures, not regenerated translations. CR, TAB, and
Backspace are rendered as literal \r, \t, and \b; LF separates lines. Raw bytes
from subsequent runs are stored under target/verification. Echo fixture text
in historical evidence may be Spanish and must not be silently rewritten.

Historical checked ELF SHA-256 values (not promises about future rebuilds):

- debug: a483404dfd1f6b4f341efc369c8fcbce34e7ffc82f68c8b785b55b85ccea4a7c
- release: ab4cc476869087391f2a87e52cc39b4d4b228f2fd2f3d7cc22268e9a825b76fd

Not tested: physical hardware, Linux/Windows hosts, electrical UART errors,
CPU exception recovery, process isolation, or performance superiority.

## Tool installation

Homebrew installed QEMU/Rustup and dependencies. Homebrew Rustup 1.29.2 crashed
with SIGSEGV during component installation; official Rustup 1.28.2 successfully
installed the pinned compiler/components. A later self-update attempt failed
without preventing the verified builds. Shell startup files were not modified;
load `. "$HOME/.cargo/env"` in a new terminal.

## Event rules and publication

The user first supplied the event requirements, then pasted the Participant Guide
on Oct 26. It requires a public repo, identifiable new work, real Meta integration,
and a three-minute video. The guide lists Oct 27-Nov 4 as the build period, while
public event pages announce Oct 26 kickoff and workshops starting Oct 27.
Eligibility of Oct 26 work and the earlier base for Indie remains unresolved.
Keep the discrepancy visible; do not change dates. API access was requested by
the user; event acceptance and model access are separate unverified dependencies.

Deadline: Nov 4, 2026, 20:00 Guatemala (Nov 5, 02:00 UTC). See
[HACKATHON_PLAN.md](HACKATHON_PLAN.md) for sources, open questions, and checklist.
No Discord messages were sent and no terms were accepted for the user.
Public visibility was verified with unauthenticated GitHub REST on Oct 26:
GET /repos/evannivw/rai-os returned private: false.

Initially changes were local only. The user subsequently authorized regular
commits and pushes. Published increments:

- 75b69a5: standing collaboration rules in AGENTS.md.
- 3b846d7: honest snapshot of accumulated Oct 26 kernel work, not a fabricated
  sequence of earlier commits. Formatting, Clippy, and both QEMU profiles passed
  before publication. Common credential-pattern checks found no matches; this
  is not a security audit.
- 0dd0788: Rust host terminal console, validation/process/presentation boundaries,
  and instructions. Two validation unit tests, Clippy, and real-QEMU integration
  passed: info/echo, invalid input/recovery, missing ELF, and EOF cleanup.
  The console also passed from a clean Git export before publication.

Local and remote HEAD matched after these pushes. No graphical UI, telemetry,
Meta calls, agent processes, or battery measurements existed at that point.

## English-language policy — October 26

The user requested English for project artifacts and commit titles AND bodies.
Earlier titles were English but bodies and documentation were Spanish. Preserve
those published commits; do not rewrite or backdate history. Translate current
documentation, UI, and examples, retaining historical evidence verbatim.
Chat teaching remains Spanish. The current graphical UI and controlled comparison
are still proposals, not implemented benchmarks or performance results.

English migration checks: host protocol unit tests (2), host Clippy, real-QEMU
console integration, and both kernel verification profiles (28 cases each) passed.
Historical UART captures were retained unchanged. No scheduling or AI logic was
introduced by this language change.

## Comparison design — October 26

Added a proposed review-pack experiment with competitive baselines, equal inputs,
independent correctness checks, model overhead, held-out scenarios, and explicitly
simulated budgets. No experiment has run and no efficiency gain is claimed.
The next gate is a small policy pilot, not a complete new kernel or dashboard.

## Required demo result contract — October 26

Defined mandatory records, byte integrity, exact indexing, consistent manifests,
version-aware publication, and execution limits in DEMO_ACCEPTANCE.md. Optional
previews cannot compensate for missing required output. Events and constraints
apply equally to all policies. This increment defines acceptance only: no new
execution engine, model integration, resource measurements, or benchmark results.
Validation: specification consistency, local Markdown links, and git diff --check.

## Correctness fixture and verifier sequence — October 26

Added seven immutable synthetic source revisions, three goal references, and
ordinary/changing/blocked traces with twelve total event checkpoints. Expected
indexes, preview bytes, permissions, and resource-transition rules are explicit.
The data is deliberately small and public: it is neither a benchmark nor a
held-out set. Added the step-by-step implementation sequence for the independent
verifier, which will audit candidate output and engine evidence separately.

Validation performed: fixture consistency checker passed; six host Python tests
passed, including five deliberately corrupted disposable fixture copies. Checked
local Markdown links and git diff --check. No kernel code changed or QEMU tests
were rerun. No candidate pack, independent result verifier, job engine, Meta
integration, resource enforcement, or performance result was produced.

## Kernel clock foundation — October 26

At the user's request, prioritized kernel progress and visible controls before
continuing the demo verifier. Added the Arm generic counter adapter and uptime
command through the guest shell, structured protocol, and Rust host console.
Clock readings are supplied to command execution; commands do not read registers.
Formatted UART replies require no heap. No external dependencies were added.

Actual verification: debug and release each passed real QEMU boot and 37
console/protocol checks. Uptime advanced from 11 ms to 267 ms across the test wait
in both captured runs, with a 62,500,000 Hz counter. Host console integration,
two existing host validation unit tests, formatting, and kernel/host Clippy all
passed. Documentation links and git diff --check passed.

See [CLOCK.md](CLOCK.md) and [capture](evidence/2026-10-26-uptime.txt). These results
do not establish timing precision, energy efficiency, or performance superiority.
No scheduler, interrupts, graphical UI, process isolation, or Meta integration was
added. The next visible increment is a graphical host panel connected to QEMU;
it will not be described as a native guest desktop.

## Graphical Device Lab panel — October 26

Added a native Rust host panel with a portrait mobile preview, Start/Stop, live
guest uptime, System info, echo, and a hideable activity log. The panel is a
separate Cargo package using pinned eframe/egui; no GUI dependency enters the
no_std kernel. Shared command/transport adapters moved into host/shared and
remain in use by the terminal console. A background worker owns QEMU and
serializes requests. All displayed guest values come from observed responses.

The user clarified a long-term mobile OS goal. The panel is explicitly labeled
host-rendered; QEMU virt is not a Samsung emulator. The project contains our
small custom kernel, not an Android/Linux kernel. Phone drivers, a native guest
display, AI services, and efficiency comparisons remain future work.

Verification: debug and release host builds, formatting, panel Clippy, and five
panel tests passed. Tests include real QEMU lifecycle/clock/echo/rejection/restart
and a missing-ELF failure. The existing terminal integration also passed after
the shared-file move. Native window review verified Start, advancing guest time,
System info, hiding/restoring Activity, editable echo, invalid Unicode rejection
and recovery, and closing a running session. A subsequent process check found
no remaining panel or QEMU process. The launcher script and local macOS app bundle
were also opened successfully. No kernel source/build settings were changed.

The first visual review exposed clipped lower controls; the card was compacted
and System info moved to the top bar. macOS package replacement uses a new file
and a local ad-hoc signature to avoid stale executable-signature caching; this
is not a notarized standalone distribution. See [PANEL.md](PANEL.md).

Retained [native panel capture](evidence/2026-10-26-panel.png) and
[panel test output](evidence/2026-10-26-panel-tests.txt). The final native review
also confirmed Stop returns to OFFLINE and clears live readings. The panel was
left open with QEMU stopped. Local documentation links and whitespace checks
passed before publication.

## First cooperative task — October 26

Added a single owned, 192-byte ASCII word-count job with bounded 8-byte steps,
progress snapshots, exact-ID cancellation, and retained completion/cancel state.
The UART receive path no longer waits for input, so tasks advance even during an
unfinished line. Deliberate 100 ms educational pacing is labeled in the panel;
this is not a throughput or energy benchmark. No new unsafe task code, allocator,
interrupts, process isolation, multi-task scheduler, or AI was introduced.

Extended shell/protocol and both host interfaces. The graphical panel has a
Kernel task tab, Start/Cancel controls, validated observed progress, and final
versus partial counts. Guest rejections no longer disconnect a healthy panel
session. The small coordinator remains the next increment.

Verification: three pure task unit tests; 28 real-QEMU task exchanges (25 structured and three direct-shell) and the
existing 37 console/protocol exchanges, each in debug and release; six panel
tests including real task progress/cancel/result; terminal integration; kernel,
console, and panel Clippy; formatting; optimized host build. The partial-frame
test waited three seconds without a complete command and observed task completion.
Native UI verified cancellation at 32/149 bytes and a subsequent completed result
of 27 words. Stop cleared task readings. Captures and contract: [TASKS.md](TASKS.md).

The GUI remained host-rendered. No physical phone or comparative performance
claim was tested. QEMU was stopped after the UI check and the panel left open.

## Small FIFO host coordinator — October 26

Added a pure bounded FIFO policy and wired it into the existing QEMU worker and
native panel. Eight total entries, printable ASCII up to 192 bytes each, one
active guest job. Submission is reserved before I/O; only matching observed
completion permits the next submission. Pause lets the current job finish.
Manual mutation cannot interfere with queue ownership. Stop/failure interrupts
uncertain work and requires explicit continuation of pending jobs after restart;
interrupted submissions are not replayed. Queue state is memory-only.

The UI exposes Add, Run, Pause, and Clear, with observed guest IDs, byte progress,
and counts. Compact queue headers keep two completed results visible at the
normal window size. Existing manual Task and Connection views remain available.
No kernel/protocol/shared-adapter changes, dependencies, unsafe code, AI, isolated
processes, or performance claims were added. See [COORDINATOR.md](COORDINATOR.md).

Verification: eleven panel tests passed (eight pure host tests, three real-QEMU
checks), including ordered results 2/3/0, pause, ownership rejection, interruption,
reboot without replay, and the existing lifecycle/clock/manual cancellation and
missing-kernel checks. The missing-kernel assertion was updated to allow the new
empty paused-queue notification before failure; it still rejects a connection
success event. Debug and optimized host builds, formatting, all-target Clippy,
local documentation links, and git diff --check passed.

Native UI review confirmed adding two jobs, Run, Pause (first completes while
second stays pending), Resume, and automatic completion of two 149-byte jobs at
27 words each. Window close cleaned up a running session; final Stop cleared live
readings and retained historical results. QEMU was stopped and the panel left
open. Retained the [window capture](evidence/2026-10-26-coordinator.png) and
[test output](evidence/2026-10-26-coordinator-tests.txt). The unchanged kernel
verification suites were not rerun for this host-only increment.

## Host dispatch priorities — October 26

Added typed Low/Normal/High priority (Normal default), captured per enqueue.
The coordinator chooses the highest pending priority with stable FIFO ties.
Existing active work is never preempted; later urgent arrivals and pause/resume
retain the same ownership and observed-completion rules. Storage and displayed
row numbers remain in insertion order; actual guest IDs show dispatch order.
The panel exposes the priority selector and each job's priority.

Documented why host placement is provisional and how agents, coordination,
executors, and kernel mechanisms differ. Guest-native service execution and
isolation require additional kernel facilities; moving a module alone would not
supply them. No kernel/protocol changes, model calls, CPU scheduler priority,
new dependencies, or efficiency claims were introduced.

Verification: fourteen tests passed (ten pure host tests and four real-QEMU
checks). New pure tests cover descending priority, FIFO ties, stable row identity,
and an urgent arrival during active work across pause/resume. A live test enqueues
Low/High/Normal/High and observes guest IDs 4/1/3/2 in those rows, with expected
counts 1/2/3/4. Earlier queue, interruption, cancellation, clock, and transport
checks still pass. Debug/release builds, formatting, all-target Clippy, local
Markdown links, and git diff --check passed. Unchanged kernel suites were not rerun.

Native review added Low before High, then observed High execute as guest #1 and
Low as guest #2, both completing 149 bytes with 27 words. Saved the
[window](evidence/2026-10-26-priority.png) and
[test output](evidence/2026-10-26-priority-tests.txt). Stop cleared live readings;
the panel remains open with historical results and QEMU stopped.

## Fatal EL1 exception foundation — October 27

Returned to kernel infrastructure after the host priority increment. Installed a
2 KiB-aligned sixteen-slot EL1 vector table after stack/BSS setup and before Rust.
Fatal entry captures architectural registers, guards nested reporting, switches
to a dedicated aligned 16 KiB stack, takes over UART, prints diagnostics, and
halts without returning. Unsafe/assembly preconditions are documented. No new
external dependencies, enabled IRQs, process isolation, scheduling, or recovery.

Added an opt-in BRK self-test feature and verifier that builds into a separate
target directory, so fault injection does not replace the panel's normal kernel.
Normal banners/protocol and graphical controls are unchanged. The feature is
disabled by default. Clarified that the kernel evolves while the coordinator's
policy remains separate; guest service execution is still a future milestone.

Actual verification: debug and release each trapped BRK #0x5241 into vector 4,
reported ESR 0xf2005241 / class 0x3c, and matched ELR to the linked test-instruction
symbol. Both stack ranges/alignment and saved EL1h state were checked. An info
request after the halt received no output during a 250 ms observation; QEMU
remained alive and was then reaped. The nested-fault guard and other exception
classes were not injected. See [EXCEPTIONS.md](EXCEPTIONS.md) and its captures.

Regression verification passed: 37 normal console/protocol checks and 28 task
exchanges in both debug/release, the three task-domain tests, fourteen panel
tests (four real-QEMU checks), and terminal integration. Formatting and Clippy
passed for default/self-test kernel configurations. ELF inspection confirmed
normal images contain the vector table but no test-trigger symbol. Local
Markdown links and git diff --check passed. No GUI code changed, so native-window
review was not repeated. No physical phone, fault recovery, or efficiency claim
was tested. Captures retain the actual October 27 date separately from prior work.

## First physical-page pool — October 27

Reserved a linker-aligned, NOLOAD 1 MiB pool after kernel code/data and both
stacks. Added pure bitmap bookkeeping for 256 pages of 4096 bytes and an exclusive
physical adapter that clears pages before handing out non-copy ownership tokens.
Release consumes a token; raw-address frees are not exposed. Unmanaged RAM is
not counted as free. No heap, MMU, memory protection, page-backed word-count
storage, new dependencies, or service isolation was introduced.

Added argument-free memory.info and memory.check to shell, protocol, and host
terminal. The diagnostic allocates the entire pool, observes zeroed bytes using
volatile reads, writes/verifies distinct end markers, checks exhaustion, releases
all pages, and verifies zeroed reuse. It is synchronous and can delay cooperative
task steps; this is not a performance test. The current diagnostic leaves the
pool fully free. A memory_busy guard exists for future owners but is not exercised
by current UART operations. See [MEMORY.md](MEMORY.md) for complete contracts.

Verification: three pure frame tests cover full/partial bitmap capacities,
unique allocation, exhaustion/reuse, invalid ranges/frees and double free without
mutation. Debug/release each passed 18 real-QEMU memory exchanges (15 structured,
three shell), including repeated physical checks, invalid arguments and task/echo
recovery; ELF symbols independently confirm pool boundaries and stack exclusion.
The mutable linker-region declaration was clarified during review and both
memory runs were repeated successfully. Captures retain real results and dates.

Regression checks passed: 37 console/protocol and 28 task exchanges per profile,
three task-domain tests, exception injection in both profiles, host terminal
integration including memory commands, and fifteen panel tests (four real QEMU).
Formatting, default/self-test kernel Clippy, terminal/panel Clippy, and optimized
host-panel compilation passed. Local documentation links and whitespace checks
passed. No GUI code changed, so no native-window review was repeated. QEMU test
children were cleaned up; the existing native panel was not modified or restarted.
