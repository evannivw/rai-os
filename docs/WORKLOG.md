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

## Active EL1 page tables and permissions — October 27

Used five pages from the existing physical pool to enable stage-1 translation
before the normal boot banner. Added a pure fixed-board table builder, explicit
linker permission boundaries, a one-time EL1 activation adapter and read-only
mmu.info status. Code is read-only/executable; constants/tables are read-only and
non-executable; writable data/stacks/pool and UART are non-executable. All leaves
deny EL0 access. Tables remain owned for the lifetime of the kernel. The sparse
identity map covers only live kernel/pool RAM and the PL011 page. Caches stay off.

The pool still totals 256 pages, with five used and 251 free. memory.check now
exercises only free pages and preserves table ownership/mappings. A zero-free
pool returns memory_busy; current UART operations cannot create that state.
Command execution receives the MMU snapshot from the adapter, like uptime,
rather than reading architectural registers in command handling. No GUI change,
new dependency, service isolation, fault recovery, AI or efficiency claim.

Verification passed on the final code: two pure mapping tests; debug/release each
ran a normal MMU boot (seven structured exchanges and one shell query) and five
fault images. Hardware emulation rejected writes to text, constants and tables,
execution from data, and an unmapped read. Reports matched architectural class,
level-3 status, exact ELF fault PC/address, saved EL1h state, stack ranges and fatal
halt. Normal ELF images contain no fault triggers. Separate build directories
preserve normal images; combining fault features was rejected at compile time.
[MMU.md](MMU.md) links the new captures. Earlier memory/exception evidence remains
unchanged and describes its actual pre-MMU increment.

Regression checks passed in debug/release: 37 console/protocol checks, 18 memory
exchanges with three pure frame tests, 28 task exchanges with three pure task
tests, and the original BRK diagnostic. Terminal integration and all fifteen
panel tests (eleven pure, four real QEMU) passed. Formatting, default plus each
individual fault-feature Clippy, host console/panel Clippy and the optimized host
panel build passed. Local documentation links and git diff --check passed.
QEMU test processes were reaped; the existing native panel was not restarted or
visually re-reviewed because its UI did not change.

Added [ROADMAP.md](ROADMAP.md) to explain the direction: resumable context,
lower-privilege execution/syscalls, scheduling, bounded IPC, then guest services.
This is a sequence of future increments, not a promise to complete all kernel
infrastructure before the hackathon. The demo can integrate Meta through the host
bridge earlier. Eligibility questions and actual contribution dates are preserved.

## Controlled EL1 context save and return — October 28

Added one explicit resumable SVC site at EL1h. Slot 4 saves x0-x30 and architectural
exception registers in an aligned frame on the current stack, switches to the
existing handler stack, and delegates to a pure allowlist policy. Only the exact
syndrome/return site and masked EL1h state may return. Assembly restores integer
registers, SP and ELR/SPSR before ERET. Unknown events still use fatal diagnostics;
no EL0 programs, preemption, task switching or general fault recovery were added.

Added argument-free context.check to the shell, structured protocol and host
terminal. Assembly seeds distinct 64-bit values in all 31 registers, tests two
complementary NZCV patterns and captures post-return state before using scratch
registers. Rust verifies those observations, DAIF and before/after SP; success is
reported only after two actual returns. No allocator or UART work occurs inside
the successful handler. Physical-page ownership and normal boot banners remain
unchanged. No GUI code or external dependency changed.

First checked normal boot, then four real SVC returns with console continuity.
The repeatable verifier subsequently passed in debug/release: one pure policy
test, 21 structured exchanges plus two shell commands (36 successful returns per
normal run), invalid argument handling, and memory/echo continuity. Separate
unknown-SVC and wrong-site images first perform valid returns, then are rejected
with the expected syndrome, exact ELF return PC, saved state, stack ranges and
fatal halt. Combining context and other fault-injection features was rejected at
compile time. Captures and limits are in [CONTEXT.md](CONTEXT.md).

Regression checks passed in both profiles: original BRK diagnostic; all five MMU
faults plus normal MMU status/ownership checks; 37 console/protocol checks; 18
memory exchanges and three pure frame tests; 28 task exchanges and three pure
task tests. Terminal integration and fifteen panel tests (four real QEMU) passed.
Formatting, Clippy for the default kernel and all eight individual negative-test
features, console/panel Clippy, and optimized panel compilation passed. Local
Markdown links and git diff --check passed. Test QEMU children were reaped; no
native-window review was repeated because the GUI did not change.

The new slot-4 path needs a valid stack with room for its 288-byte frame before
switching to the handler stack. Corrupt/overflowing stacks and nested faults
remain untested and may prevent diagnostics. SIMD/FPU, SP_EL0 and process address
spaces are not saved; this is the current soft-float EL1 runtime's integer context,
not a complete process context. The roadmap now points to one lower-privilege
program and a bounded kernel call. No claim of isolation or AI efficiency is made.

## First built-in EL0 Rust program — October 28

Added one linked no_std Rust program that requests a checked write of "Hello from
EL0", validates the reply and exits. user.run starts it synchronously through
ERET with cleared integer registers and a freshly zeroed 8 KiB user stack. The
kernel retains its own stack/session and the program runs with EL0 permissions.
Dedicated user text/constants/stack mappings and two unmapped guard pages are
linker-reserved outside the existing page pool; five table pages and 251 free
pool pages remain unchanged. All kernel/device mappings still deny EL0 access.

Extended vector 8 and the shared frame to handle lower-EL synchronous entry,
including SP_EL0. Write calls validate complete ranges, overflow, length and ASCII
before committing to a private bounded output buffer. Exit uses a fixed kernel
trampoline. A synchronous user fault returns a faulted report and terminates only
that invocation; kernel faults retain the existing fatal policy. Normal UART
requests cannot provide arbitrary binaries, PCs or context data. Output is
published after completion so the structured transport stays intact.

First normal debug boot and two user runs passed with context/echo continuity.
The initial fault-test build rejected named inline assembly labels; moved those
probes into explicit test-only global assembly in the user text region, then
verified the corrected images. Debug and release each passed seven image cases:
normal, eight invalid syscall requests followed by valid write/exit, and forbidden
kernel read, user-code write, UART read, stack execution and privileged register
read. Every case runs twice and checks continued kernel context/echo operation,
unchanged MMU status, page diagnostics and malformed UART arguments. The normal
image also passed direct shell execution. Fault syndromes, exact ELF PCs/FAR where
valid, saved EL0 state and stack bounds were checked. Pure range validation and
three table-builder tests passed, including user permissions and guard/layout
rejection. New captures are in [USER_PROGRAM.md](USER_PROGRAM.md).

Regressions passed in both profiles: 37 boot/protocol checks, 36 original EL1
context returns plus both rejection images, BRK diagnostics, five MMU fault images
and normal mapping checks, 18 memory exchanges with three pure allocator tests,
and 28 task exchanges with three pure task tests. Terminal integration and all
fifteen panel tests (four real QEMU) passed. Formatting, default kernel and all
fourteen individual test-feature Clippy runs, console/panel Clippy and optimized
panel compilation passed. A mixed user/kernel test-feature build was rejected.
Local Markdown links and whitespace checks passed. QEMU test children were reaped.
The GUI did not change, so native-window review was not repeated.

This is a tested kernel/user privilege boundary for one built-in image, not
independent process address spaces, arbitrary application loading, a scheduler,
resource isolation or an agent migration. No timer/preemption exists: a looping
program could block progress. Cumulative output exhaustion and corrupt/nested
kernel stacks were not fault-injected. No phone, AI or efficiency claim was tested.
The next area is timer/interrupt infrastructure, followed by scheduling, in small
verified increments. Existing provenance and earlier evidence remain unchanged.

## Controlled EL1 timer IRQs — October 28

Added a sole GICv2/CNTP owner and argument-free timer.check through the shell,
structured UART protocol and host console. QEMU explicitly selects GICv2. Two
EL1-only Device pages map the distributor and CPU interface; a sixth owned table
page leaves 250 pages free. Historical captures and prior log entries are intact.

The assembly diagnostic seeds all 31 integer registers and two NZCV patterns,
unmasks IRQ only in a branch-to-self loop, and receives three physical-timer IRQs
per round. Slot 5 saves the existing integer frame and uses the handler stack.
The handler validates ID 30, saved PC/state and the active stack-owned round;
rearm/disable precedes EOI. On the third tick it redirects to a fixed continuation
and restores IRQ masking. Rust checks the captured context, timing and actual
hardware shutdown before emitting success. Successful handlers never allocate,
use UART or touch service state. Ordinary Rust and EL0 remain IRQ-masked.

First normal debug boot and a six-IRQ run passed with context/user/echo continuity.
The dedicated verifier then passed in debug and release: four checks (24 IRQs)
per normal image, exact loop instruction, register/flags/SP preservation, repeated
start/stop, live hardware masks/enables, invalid arguments, unchanged mappings and
memory checks. A separate image injected real SGI 1 and verified the exact fatal
IRQ/vector/PC/state/stack report and absence of subsequent responses. EL0's new
GIC read probe was denied by hardware; all eight user images per profile passed,
including repeated execution and kernel continuity. Captures are in [TIMER.md](TIMER.md).

Regressions passed in both profiles: 37 boot/protocol checks, 36 SVC returns and
both rejected SVC images, BRK diagnostics, five MMU fault images with normal
mapping/ownership checks and three table tests, 18 memory exchanges with three
allocator tests, and 28 task exchanges with three task tests. Terminal integration
and all fifteen panel tests (four real QEMU) passed. Formatting, default kernel
and sixteen individual test-feature Clippy runs, console/panel Clippy and optimized
panel compilation passed. A mixed GIC-user/IRQ-test configuration was rejected.
The ad-hoc check runner initially requested Python's unavailable tomllib module;
rerunning it with standard-library text parsing completed the checks without
installing dependencies. Local Markdown links and whitespace checks passed.
No native-window review was repeated because the GUI did not change.

This is controlled EL1 IRQ delivery, not EL0 preemption, scheduling or agent
migration. If delivery fails the probe spins; host test timeouts terminate QEMU,
but there is no guest recovery deadline yet. Spurious IDs have a non-EOI/non-tick
path but were not injected. Corrupt/nested stacks, SIMD, SMP and alternate boards
remain untested. Busy-loop execution is not an efficiency result. The next small
step is timer-based control of a running EL0 invocation, before multiple services.

## Timer-enforced EL0 invocation deadline — October 28

Kept the single-CPU scope and added one fixed 100 ms guest-counter deadline per
EL0 invocation. user.run retains the Rust hello program; new argument-free
user.spin selects a linked one-instruction infinite loop. The timer owner programs
an absolute CNTP compare value once. Syscalls cannot reset the deadline. Vector 9
captures the user frame and terminates an expired invocation through the existing
kernel-owned trampoline; vector 8 also checks expiry before dispatch. Expiry is
reported as timed_out with observed elapsed time, vector, PC/status/SP and the
number of calls executed before expiry. It is not a successful user result.

Kernel Rust remains IRQ-masked; ERET enables IRQs only in EL0. The active Session
is fully initialized before publication and is borrowed only by non-nested
handlers while the caller is paused. Normal exit, fault and timeout all clear
session ownership, stop the timer/GIC and check hardware/mask readback. Explicit
SCTLR.UMA and CNTKCTL settings prevent user code from masking IRQs or changing the
timer. No mappings or pool ownership changed. The graphical panel is unchanged.

The first live debug run terminated the non-cooperative loop at 102 ms and then
completed hello, the EL1 timer diagnostic and echo. Dedicated verification passed
in debug/release: four normal-image spin timeouts with exact ELF loop/PC/state/SP
checks, three interleaved hello runs, malformed arguments and repeated timer,
context, memory and console continuity. Separate syscall-loop images expired
after many calls without renewing the deadline. A test-only bounded handler delay
forced vector-8 expiry deterministically; each test image ran twice plus a shell
spin. A pure budget test checked exact boundaries, fractional tick rounding,
invalid frequency and overflow. Captures are in [USER_TIMEOUT.md](USER_TIMEOUT.md).

The user suite now passed ten images per profile, two invocations each plus the
normal shell run: hello, invalid calls and eight hardware fault probes. New DAIFSet
and CNTP_CTL write probes trapped with class 0x18 at their exact ELF instructions;
subsequent timer/context/echo and memory checks passed. All earlier fault tests
now verify saved EL0 IRQ-enabled status. Existing historical captures are unchanged.

Regressions passed in both profiles: 37 boot/protocol checks; 24 controlled EL1
timer IRQs plus unexpected-SGI fatal handling; 36 EL1 SVC returns plus both rejected
SVC images; BRK diagnostics; five MMU fault images, normal ownership checks and
three table tests; 18 memory exchanges and three allocator tests; 28 task exchanges
and three task tests. Terminal integration including user.spin and fifteen panel
tests (four real QEMU) passed. Formatting, default and all twenty individual
feature Clippy runs, host console/panel Clippy and optimized panel compilation
passed. Mixed user test features were rejected. Local Markdown links and whitespace
checks passed. Verification QEMU children were reaped; no GUI visual review was
repeated because presentation did not change.

This terminates one invocation; it does not suspend/resume it, schedule two jobs,
move the coordinator, add Meta integration or prove CPU/battery efficiency. The
100 ms is elapsed guest-counter time including bounded kernel work, not a hard
real-time bound or precise CPU accounting. IRQ/handler latency can add delay; a
broken timer or stuck kernel handler can still hang. Spurious/unexpected lower-EL
IRQs, cleanup readback failure, counter wrap/reset, nested/corrupt kernel stacks
and SMP were not injected. The next area is saving/resuming two built-in contexts
on one CPU, with explicit ownership and lifecycle tests before broader services.

## Two resumable EL0 contexts on one CPU — October 28

Added argument-free schedule.check as a bounded round-robin diagnostic. Two linked
ADD/STR/B loops receive six 20 ms turns each. The kernel stores their integer
Frames in RAM and restores the interrupted PC, x0-x30, status and SP_EL0 on later
turns. The existing 8 KiB user RAM region supplies two distinct 4 KiB stack halves;
there is no new mapping, separate address space or enforced isolation between
those halves. Ordinary user.run/user.spin retain their separate 100 ms deadline.

Each IRQ captures an output Observation separate from the read-only restore
input. The fixed kernel return trampoline resumes masked Rust, which stops/checks
timer hardware, validates the frame/stack, retains it and selects the other
context. Validation checks thirty distinct seeded registers, progressing x0,
complementary NZCV patterns, exact SP, a permitted PC and two observed stack words.
The reply includes actual per-turn counters and A/B trace only after all checks
pass. Both paused contexts are discarded when the twelve-turn test ends.

The first Clippy run caught a modulo expression that should use is_multiple_of;
that was corrected before boot verification. Normal debug boot then passed, and
the first live run alternated ABABABABABAB with 137/136 ms of observed slice time
and increasing counters. Hello, the EL1 timer diagnostic and echo still worked.
The dedicated verifier passed debug/release: four normal runs per profile (48
EL0 timer IRQs and 44 context switches), exact ELF loop instructions, alternating
trace, strictly increasing per-context counters, arithmetic and at least 120 ms
per context. The pure budget test now distinguishes 20 ms slices from 100 ms
invocation deadlines. Timing is guest-counter observation, not CPU accounting.

Two separate negative images passed twice each: A corrupts x7 and is rejected as
schedule_context_corrupt; B executes BRK after A's first slice and the diagnostic
returns schedule_user_fault. Both paths stop before resuming rejected state and
leave hello, user.spin timeout, context/timer checks, memory/MMU state and echo
usable. Argument rejection and direct shell operation were checked. Captures and
unsafe/lifecycle limits are in [SCHEDULING.md](SCHEDULING.md).

Regressions passed in debug and release: 37 boot/protocol checks, all controlled
EL1 timer checks and unexpected-SGI rejection, EL1 SVC returns/rejection images,
BRK diagnostics, MMU normal/five-fault checks and three table tests, memory/task
integration and their three pure tests each, ten user image cases, and all three
EL0 timeout image cases. Terminal integration now includes schedule.check;
fifteen panel tests (four real QEMU), formatting, default plus twenty-two individual
feature Clippy runs, console/panel Clippy and optimized panel compilation passed.
Mixed schedule/user fault features were rejected. Local Markdown links and staged
whitespace checks passed. Test QEMU children were reaped. GUI presentation was
unchanged, so no native-window visual review was repeated.

This verifies preemptive pause/resume for two fixed contexts, not a general task
runtime, runnable queue, service isolation, process loader, agent migration or
SMP. A synchronous task fault aborts the whole diagnostic. Stack observations
cover only the counter and marker words, not the full stack capacity. Failed IRQ
delivery can still hang until the host verifier times out; no guest recovery for
broken timer delivery is claimed. The next step is explicit task completion and
fault lifecycle so one context can finish/fail while another continues, before
adding broader scheduling policy and IPC. Historical evidence remains intact.

## October 28, 2026 — bounded task records and terminal states

Added schedule.run and schedule.fault to the shell, structured protocol and host
terminal model. Two kernel-stack task records now retain saved integer context,
stack bounds, lifecycle and scalar outcomes. Pure lifecycle rules are separate
from the runtime and timer/exception adapter. Ready tasks alternate; terminal
records are skipped. A valid exit, a user fault and a six-preemption cap produce
explicit exited, faulted and limit_reached results. The original schedule.check
retains its strict register-integrity diagnostic and all-or-error behavior.

Two finite linked Rust tasks return independently verifiable numeric results.
The public fault diagnostic makes A read kernel memory: QEMU reports the expected
EL0 permission fault, while B completes and the console remains usable. The small
exit-only ABI validates syscall number and exit code, carries an opaque scalar
in x1 and accepts no pointers. Saved context is checked before re-entry. Full task
records stay inside the runtime; compact outcome records keep the command reply
enum small (the initial full-frame reply triggered Clippy's large-enum check and
was replaced before verification).

Actual verification passed in debug and release: three pure lifecycle tests and
five QEMU images per profile (normal, A spinning, both spinning, exit code 256,
syscall 99), with 22 lifecycle invocations per profile. The host independently
replays selection/terminal traces, verifies numeric results and checks exact
fault syndrome/site/address and saved context. Repeated commands plus subsequent
user/context/timer/schedule/MMU/memory/echo operations verify continued operation.
Normal finite tasks exited before a first tick in these captures; dedicated spin
fixtures establish preemption, continued peer completion and the exact cap.

Regression suites passed in both profiles: boot/protocol (37 cases), schedule,
timer, context, exceptions, MMU, memory, cooperative tasks, user and user-timeout.
Host console integration and the panel's 15 tests passed; the release panel built.
Formatting passed for kernel/console and panel. Clippy passed for the normal
kernel, all 26 individual diagnostic features, console and panel. A mixed
lifecycle/schedule fault-feature build was correctly rejected. Local Markdown
links and staged whitespace were checked before commit.

See [TASK_LIFECYCLE.md](TASK_LIFECYCLE.md) and its captured debug/release evidence.
No physical page mappings changed: six table pages and 250 free pool pages.
Task records are temporary kernel RAM, not persistent processes. User tasks still
share writable mappings; this does not provide inter-task isolation, IPC, dynamic
loading, a guest coordinator, AI or efficiency gains. The next foundation is
separate task memory before claiming isolated guest services. Historical captures
and contribution dates remain unchanged.

## October 28, 2026 — private writable stack maps

Added immutable base/A/B translation roots. Each scheduled EL0 task can access
only its own writable 4 KiB stack page; shared user code/constants stay read-only.
The peer page remains readable/writable by EL1, with identical physical addresses
and memory types across roots. Six additional page-table pages raise permanent
ownership from six to twelve (48 KiB), leaving 244 free pages in the reserved pool.
No old capture or contribution date was altered.

The MMU adapter owns typed fixed-root selection. It changes TTBR0_EL1 with full
local EL1 translation invalidation, ordering barriers, masked kernel interrupts
and base-root restoration on both successful and error-return continuations.
All table pages remain read-only/non-executable in every root; no live descriptor
mutation, per-task ASID allocation or dynamic map API was added. schedule.check
and the bounded lifecycle both reuse this entry path. The legacy single-program
commands still own the entire user region exclusively under the base map.

Added schedule.isolate to the shell, structured protocol and host console. It
checks permitted own-page reads/writes, then forbidden peer reads/writes in both
directions, repeated across two rounds with warmed translations. Exact saved
syndrome, instruction address and target address must match. Both entire pages
are checked before the victim's next marker write, preventing the positive control
from repairing and hiding a changed byte. The victim's own access then succeeds.
The compact report includes actual final-round fault registers; success flags
are emitted only after both rounds pass.

Actual verification passed in debug and release: four pure table tests, four
normal diagnostic invocations per profile (32 denied accesses and 48 permitted
own-page probes), and three expected rejections per profile of an intentionally
open-peer image. The negative control returns isolation_expected_fault and
continues serving later commands. The host independently verifies fault registers
against ELF symbols, argument rejection, base-MMU snapshot restoration and later
lifecycle/scheduling/user/memory/echo operations. The final isolation tests were
rerun after moving full-page integrity checks before the positive-control write.

Regression suites passed in both profiles: 37 boot/protocol checks, scheduling,
lifecycle, timer, context, exceptions, MMU, memory, cooperative tasks, user and
user-timeout. Host console integration and all 15 panel tests passed. Formatting,
normal kernel and all 27 individual diagnostic-feature Clippy checks,
console/panel Clippy and optimized panel compilation passed. Mixed isolation and
lifecycle fault features were correctly rejected. No GUI controls changed.
Local Markdown links, staged whitespace and final included files were reviewed.

See [TASK_MEMORY.md](TASK_MEMORY.md) for contracts and actual debug/release
captures. This protects the two fixed stack pages on the supported single-core
QEMU board. It is not a complete process model, a proof against all malicious
programs, a measured speedup, or agent/Meta integration. No SMP, cache/ASID
optimization, private executable images, demand paging or dynamic memory lifetime
was introduced. The next foundation is bounded message exchange with validated
ownership, before moving a coordinator or executors into guest services.

## October 28, 2026 — bounded scalar task messages

Added ipc.check using the existing private-stack task runtime. Two linked Rust
EL0 programs exchange request kind=1/value=41 and response kind=2/value=42.
The kernel copies scalar messages into two-slot FIFO inboxes and returns them
in saved registers; no user pointer is dereferenced or shared writable user
buffer added. Numeric TaskId validation and kernel-configured route permissions
are separate from UART, MMU and scheduling. Caller identity is derived from the
selected task record; receive has no selectable foreign inbox.

The runtime now handles send/receive syscall returns by yielding to the next
ready task while preserving its timer count. Empty/full/invalid/denied/closed
results return explicit scalar statuses with unused output registers zeroed.
The sixteenth IPC attempt terminates before servicing the request; the existing
six-preemption bound remains. Endpoint closure on every terminal state discards
pending incoming messages; messages already sent to a live peer remain queued.
Each invocation starts with fresh inboxes and batch-scoped IDs. Legacy lifecycle
commands and the earlier single-program syscall ABI remain unchanged.

The first debug bring-up caught an out-of-user-text instruction access after a
Rust tuple comparison. MMU protection stopped the programs and the console
remained usable. Replacing tuple comparisons with explicit scalar checks kept
that logic in the linked user code. The corrected request/reply then ran in
actual QEMU before negative cases were added; final debug/release checks passed.

Verification passed: three pure mailbox tests and four lifecycle tests, including
FIFO/wrap, full rejection without overwriting, invalid IDs/kinds, explicit routes,
close/discard, delivery after sender exit, fresh queues and syscall-yield limits.
Each build profile ran five QEMU images: normal, invalid/full requests, receive
flood, peer exit and peer fault. Sixteen IPC batches per profile passed, including
a normal shell run. The host independently replayed scheduling and call limits,
checked result values/counters and the peer fault's exact architectural report.
Real Rust tasks checked full/closed/denied/error returns, zeroed unused result
registers and inability to receive from B by passing B's ID while A was running.
Subsequent scheduling, lifecycle, legacy user, isolation, memory and echo checks
passed with unchanged MMU snapshots. Repeated runs verified queue cleanup.

Regressions passed in debug and release: 37 boot/protocol checks, scheduling,
lifecycle, private-stack isolation, timer, context, exceptions, MMU, memory,
cooperative tasks, user and user-timeout suites. Host console integration and all
15 panel tests passed. Formatting, default plus all 31 individual diagnostic
feature Clippy checks, console/panel Clippy and release panel compilation passed.
Mixed IPC/lifecycle fault features were rejected. Local Markdown links, included
files and staged whitespace were checked before committing. GUI controls did not
change. No historical captures, commits or contribution dates were rewritten.

See [IPC.md](IPC.md) and its actual debug/release captures. Physical-page ownership
remains twelve table pages and 244 free pool pages. IPC storage is bounded kernel
stack metadata. This is still two fixed tasks with nonblocking receive, not a
general process/service runtime or agent framework. Expanding to C/D/E requires
coordinated task, memory-map, scheduler and route changes; changing one constant
is insufficient. Waiting/wakeup is the next small foundation before general
service lifecycle. No AI, phone support or efficiency gain is claimed.

## October 28, 2026 — blocking receive and message/closure wakeup

Added ipc.wait and receive_wait syscall 12 in its own runtime mode. A pending
receiver becomes Waiting and is excluded from dispatch. A successful send
completes the pending receive through the protected saved result registers and
marks the task Ready; its original saved PC resumes after the same syscall.
Already-queued messages return immediately. All result and identity handling
remains scalar, with private user stacks and no new unsafe copy boundary.

Queue-first pending/Closed policy remains in the pure mailbox module; checked
Waiting/wake/Deadlocked transitions remain in the lifecycle module. The runtime
owns completion and closed-batch deadlock handling. Waiting does not close an
endpoint. Exit, fault and timer/call limits do close it; if the receiver has no
authorized live sender and no queued message, its pending call completes Closed.
If the closed batch has no ready task, remaining waiters become Deadlocked and
the console returns. No external event source exists in this diagnostic, so this
is not a persistent-service idle policy or a hardware power-saving implementation.

Normal QEMU bring-up observed AD,BS,AW,AS,BR,AE,BE: A blocked once, B sent 41,
A woke once and replied 42, then both exited correctly. The older ipc.check
request/reply still worked. The trace now stores 92 one-byte task/event entries,
preserving the prior 92-byte trace-array footprint while adding wake/cancellation
events. Wakeup preserves timer and syscall budgets and never counts a second call.

Actual verification passed in debug and release: four pure mailbox tests and
five lifecycle tests. Eight QEMU images per profile covered normal wakeup, mutual
waiting, peer exit, peer fault, peer spin, peer call limit, queued-before-receive
and invalid-send-before-valid-send. Twenty-five ipc.wait batches per profile
passed, including a normal shell run. An independent host replay checked that
waiters were never dispatched, W followed the peer's send, C followed a terminal
peer, and X occurred only with no ready task. It also checked results/counters,
call/timer bounds and the exact architectural fault. In the spin case A retained
one pending receive and zero timer turns while B exhausted its six preemptions,
then A woke with Closed. Invalid sends did not wake it. Subsequent IPC, user,
scheduling, isolation, memory, MMU snapshot and echo checks passed. Repeated runs
verified fresh records/queues; extra arguments were rejected.

Full regressions passed in both profiles: 37 boot/protocol checks, scheduling,
lifecycle, nonblocking IPC, isolation, timer, context, exceptions, MMU, memory,
cooperative tasks, user and user-timeout. Host console integration and all 15 panel
tests passed. Formatting, default plus all 38 individual diagnostic-feature Clippy
checks, console/panel Clippy and release panel compilation passed. Mixed wait/IPC
features were rejected. Final local links, staged whitespace and included files
were reviewed. The GUI did not change; historical evidence and dates were preserved.

Expanded [ROADMAP.md](ROADMAP.md) with the dependency sequence and the reason for
each foundation, as requested. The next step is one small guest request/result
service; external event delivery and persistent lifetime must be separate explicit
increments. [IPC_WAIT.md](IPC_WAIT.md) documents the ABI, transitions, bounded
failure policy and actual captures. Page ownership remains twelve tables and
244 free pool pages. No receive timeout, WFI idle loop, SMP, dynamic task creation,
guest coordinator, Meta integration or measured efficiency gain is claimed.

## October 29, 2026 — first bounded guest record service

Added record.count <key>: host/UART parsing validates a fixed immutable record
identity, then an EL0 client sends a scalar request to an EL0 worker. The worker
validates the sender, operation kind and numeric ID, scans read-only public
fixture bytes, and returns packed record identity/byte length/ASCII token count.
The client waits using existing IPC, validates the reply and exits. The service
adapter requires both successful exits, matching values and valid result bounds.
The normal trace observed AS,BR,AD,BS,AW,AE,BE. notes@1 returned 30 bytes/7 tokens,
notes@2 44 bytes/9 tokens, and empty@1 zero bytes/tokens.

Kept responsibilities separate: records.rs owns immutable data/count semantics;
schedule/service_programs.rs owns linked user request/reply programs; service.rs
owns validated execution/completion; existing runtime/MMU/mailboxes enforce the
bounds. Only the client's scalar x0 input is initialized differently. No new
user-pointer copy or unsafe memory access was added. The source table is shared
user read-only data with a compile-time 128-byte maximum; denied source bytes
are absent from the ELF. This does not establish per-record confidentiality.
All six allowed revisions are immediately available; scenario events are pending.

First debug bring-up exposed a core ASCII helper compiled into kernel text:
nonempty records faulted while the empty record succeeded. Replaced that helper
with an equivalent in-function byte pattern; retained all permissions. The
fault remained contained and the console continued. This failed bring-up is not
presented as a passing run. Final evidence is linked from RECORD_SERVICE.md.

Final verification passed in debug and release: five real QEMU images per
profile, covering normal work, invalid scalar ID, wrong service kind, wrong
record identity in a reply, and worker forbidden read. Each image ran all six
allowed requests twice, plus a normal direct-shell request: 61 service
invocations per profile. The independent host regex checker reads trusted
fixture bytes, validates catalog lengths/digests and checks exact counts, packed
values, both task completions and scheduling/message transitions. Corrupted
counts, identities, lengths and completion states were rejected. Its replay also
accepted two synthetic valid timer interleavings, explicitly not QEMU evidence.
Denied/unknown inputs, malformed/extra arguments, repeated fresh invocations and
post-failure user/IPC/isolation/MMU/memory/echo continuity passed.

Full existing regression suites passed in both profiles: boot/protocol,
scheduling, lifecycle, IPC, waiting, private-stack isolation, timer, context,
exceptions, MMU, memory, cooperative tasks, user and user-timeout. Host console
integration now exercises record.count; all 16 panel tests passed, including the
shared-model request validation test. Formatting, default plus all 42 individual
fault-feature Clippy checks, host console/panel Clippy, release panel compilation
and mixed-feature rejection passed. Fixture consistency and its six corruption
checks also passed. GUI controls did not change. No benchmark or AI claim is made.

Updated ROADMAP.md with the complete 19-step project journey and remaining demo
path, as requested. HACKATHON_PLAN.md retains the original schedule and explicitly
revises targets at the October 29 checkpoint: Meta integration has not yet been
completed. The next increment is one real model proposal through the host bridge
to a validated guest operation and observed result, then useful workload/event
handling, full artifact verification and a fair baseline. This small service
checker does not replace the full independent demo verifier. Persistent guest
services, external wake sources, phone support and measured gains remain pending.
All new artifacts/commit text are English; historical evidence/dates are preserved.

## October 29, 2026 — first real Meta proposal and guest execution

Added the standalone Rust host/meta package and scripts/meta.sh. The bridge loads
MODEL_API_KEY from the environment or ignored .env.local without shell evaluation,
requests one structured proposal from Muse Spark 1.3, validates the fixed record
allowlist/state version/operation, then sends a typed request over the existing
UART adapter. The kernel and its protected record worker are unchanged. The
model does not receive record bodies or calculate the reported guest result.

Kept domain validation and dispatch gating separate from local configuration,
HTTPS transport and main composition. Invalid/duplicate/extra JSON fields,
unknown operations/records, stale versions, malformed or incomplete provider
responses and refusal stop before execution. Decline is an explicit outcome with
no QEMU boot. Result reporting requires matching record identity, expected length,
bounded counts and successful matching exits. It reports observed, not verified
AI intent or full demo correctness. Existing priorities/GUI are not connected yet.

The local key was accepted by the authenticated Meta model catalog. Python's
initial TLS check failed certificate validation; system curl 8.7.1 succeeded
with verification enabled. The Rust adapter therefore uses curl with fixed HTTPS
endpoint, private stdin configuration, no secret arguments, no curlrc, no redirect
or automatic retry, 10-second connection/30-second total timeout and a 64 KiB
body cap. All process error messages are sanitized. Both curl and QEMU children
exclude MODEL_API_KEY from their environment; the secret file remains ignored.

Two actual inference requests were made, with no automatic retries. For
"Count the words in the latest available revision of the notes.", Meta selected
record.count notes@2. Real QEMU execution returned 44 bytes/9 tokens. API wall
latency was 2012 ms with provider-reported usage 333 prompt/142 completion/475
total tokens; boot plus guest operation was 105 ms. For "Delete the notes and
read restricted@1.", Meta declined, with no QEMU start. That API request measured
3745 ms and reported 330/139/469 tokens. These are individual observations, not
performance comparisons. Dollar cost and inference energy were not measured.
Captured credential-free JSON events are linked from META_BRIDGE.md.

Verification passed: twelve host Rust tests in debug/release; six real-QEMU
record requests in each offline profile checked against exact fixture bytes by
the independent Python tokenizer; malformed/denied/stale/unknown/oversized
proposals and explicit decline; empty key failure before network access. A dummy
curl sentinel verified offline replay never invokes the provider; a process
wrapper verified the QEMU child lacks the dummy credential. Both live captures
passed log consistency checks; the successful count was independently checked.
A saved log is not cryptographic proof of a network call, and offline replay is
explicitly labeled rather than passed off as Meta output.

Existing host console integration and all sixteen panel tests passed after the
shared adapter's environment-filter change. Kernel/root, bridge and panel
formatting checks, bridge all-target Clippy, console and panel Clippy passed.
No unrelated kernel changes required repeating all kernel fault-injection suites.
Updated architecture, roadmap, setup/use instructions and delivery targets to
reflect the first real Meta path while retaining historical pending-work entries.
Continuous event handling, useful multi-step adaptation, guest-resident
coordination, the complete artifact verifier and fair baseline remain pending.

## October 29, 2026 — bounded event-driven Meta coordination

Added a pure revision/observation state module and a separate host loop adapter.
The new --coordinate mode consumes bounded, explicitly scripted release events,
keeps one provider request pending while continuing to receive due events, and
passes only released metadata and actual prior observations to Meta. New releases
and completed observations advance state_version; duplicate/older events do not.
The host admits only the latest pending notes revision and never repeats a
completed exact revision. A response tied to an older snapshot is discarded
before UART dispatch. Due events are also applied after boot and before result
recording; a superseded immutable result cannot fulfill a newer goal.

One QEMU connection is reused, while every record operation still runs its own
bounded EL0 client/worker batch. The kernel/ABI and shared UART adapter are
unchanged. The session permits at most four proposal attempts and eight release
events scheduled within 30 seconds. Invalid proposals/provider/guest errors stop
without hidden retries. A decline explicitly reports unfinished work. Provider
threads are joined on normal cleanup, with the existing HTTP timeout; this does
not add signal handling or cancellation of an in-flight model request.

The live debug session made two real Muse Spark 1.3 calls. It processed notes@1
as 30 bytes/7 tokens, then notes@2 as 44 bytes/9 tokens. The second request included
the first observation. Two duplicate events produced no model call or execution.
There was one guest boot. API latencies measured 2311 and 1648 ms; reported total
tokens were 554 and 538, respectively. Session duration was 12003 ms including
intentional fixture waits. No cost/energy/comparative benefit was measured.
This sequence demonstrates coordination mechanics; deterministic rules can also
solve this small case. Future-workload planning remains the substantive demo goal.

Eighteen Rust tests passed in debug and release, including prior proposal/secret
checks, bounded event state, stale/future/completed admission rejection, duplicate
suppression, superseded completion and provider snapshots excluding future events.
Real-QEMU offline coordinator tests passed in both profiles: revision/duplicate
sequence, delayed first proposal crossing a new release, wrong-version/future
rejection and decline without boot. In the delayed case the old response was
discarded and only notes@2 executed. Independent Python auditing reconstructed
state/versions/history and verified exact guest counts; altered histories were
rejected. The live capture passed the same audit. Existing single-proposal bridge
offline tests also passed in both profiles with all six allowed guest records.
Root/bridge formatting and bridge all-target Clippy passed. No kernel, panel or
shared transport changes required broader unrelated regression repeats.

Added config/meta.env.example with an empty setting, plus explicit per-checkout
credential instructions: reviewers supply their own key in root .env.local,
never the tracked template. Copy instructions preserve an existing local file;
Git exclusion and environment precedence are explained. No real credential was
changed or included. Updated README, getting-started, architecture, roadmap and
hackathon plan. META_COORDINATOR.md separates real Meta evidence from delayed
replay evidence and explains the fixed goal, event source, limits and host placement.
The older priority coordinator/panel remain separate; this is not guest-resident
coordination, arbitrary file watching or the full review-pack verifier/benchmark.


## 2026-10-30 — Persistent EL0 worker and exclusive user RAM

At the user's request, returned to kernel consolidation instead of extending
model/workload experiments. Added worker.start/count/status/stop in the direct
shell, structured UART protocol and Rust host console. The kernel retains one
linked EL0 worker's integer frame and private B stack across requests. Its own
local completed counter is sent as an IPC sequence and checked against each
admitted request. Empty receive waits for future host ingress; it is not treated
as the existing closed-batch deadlock. Endpoint 1 is kernel host ingress, endpoint
2 is the worker; no extra EL0 client is implied.

An exclusive user-memory lease now covers all legacy EL0 entry paths as well as
the new session. Other EL0 commands return user_memory_busy while the suspended
worker owns the fixed region. Stop/failure retires its continuation, clears RAM
and releases the lease before legacy execution resumes. EL1 diagnostics remain
available. The existing twelve table pages and 244 free pool pages are unchanged.
Each start/count has six timer expirations and sixteen syscall-attempt limits;
the sixteenth attempt is not served. Limits span all resumes within that operation,
not the entire session. A timer slice restarts after a syscall, so the six-slice
cap is not a 120 ms aggregate deadline. Waiting gets no worker CPU turns, but
the console still polls. No power/performance advantage is claimed.

Built and executed on macOS ARM64, Rust 1.90.0 and QEMU 11.1.2, fixed virt-9.2,
Cortex-A53/TCG/one CPU. New verification passed in debug and release: six images
per profile, 670 structured requests plus two direct-shell count requests per
profile. Normal sessions each processed 25 records and then stopped/restarted;
all six record counts were independently derived with Python regex from fixture
bytes. EL1 timer/context diagnostics also ran after twelve completed records,
then the same continuation completed subsequent requests. Invalid framing, denied
records, repeated starts/stops and overlapping user entries were checked. Five
fault modes covered a forbidden kernel read, infinite computation, invalid-call
flood, wrong result ID and wrong sequence. Each was contained, retained a failed
status and permitted legacy EL0 execution after releasing ownership. Selected
actual replies are retained in docs/evidence/2026-10-30-worker-{debug,release}.txt;
full generated transcripts remain under target/verification.

Regression suites passed in both profiles: boot/protocol, schedule, lifecycle,
IPC/wait, isolation, timer/context/exceptions, MMU/physical memory, cooperative
jobs, user/timeout, record service, offline Meta bridge and offline coordinator.
The host-console integration passed with new worker commands; four console Rust
tests and sixteen panel tests passed. Formatting, default kernel Clippy, all 47
individual fault-feature Clippy builds, host console/panel Clippy, panel release
build and mixed-feature rejection passed. Repository Markdown file links and
Git whitespace checks passed. No live provider requests or credentials were
needed or changed in this increment.

Added PERSISTENT_WORKER.md and updated architecture, protocol, setup, README,
roadmap and the scope note in the hackathon plan. The current Meta coordinator
still lives on the host and uses record.count batches. This adds one guest
service lifetime; it does not provide multiple persistent workers, dynamic
process loading, a guest AI coordinator, persistent storage or a finished OS.
The useful demo workload, full independent artifact verifier, fair comparison
and three-minute video remain pending against the unchanged November 4 deadline.


## 2026-10-30 — Guest EL0 coordinator and versioned proposal admission

Implemented the next requested guest-service boundary: retained coordinator A
and counting worker B at EL0. Each uses its existing private stack map. A owns
the current goal, compares proposal tickets/record identity, rejects completed
or superseded work, sends allowed work to B, and retains the observed result.
The kernel runtime owns frames, mailboxes, per-command budgets, the exclusive
RAM lease, host gateway and publication validation. These are linked programs,
not dynamically loaded processes or a complete microkernel.

Added coord.start/observe/propose/status/stop to UART, the direct shell and host
console. Proposal arguments are bounded ticket:key pairs. A coordinator-only
host-wait syscall transports validated scalars; a separate publication syscall
returns typed state. B cannot use that host gateway. IPC routes remain 1->2 and
2->1 with two-slot inboxes. Both programs must be waiting again before success
is exposed to the host. New completion reports must match result and sequence
messages actually sent by B. A non-executing event can only repeat an already
confirmed completion, preventing fabricated results disguised as duplicates.

Tickets are issued monotonically across stop/start within one boot, including
admitted duplicate observations. Duplicates preserve A's existing goal ticket;
new goals receive the newly issued ticket. Counters do not wrap. A reboot still
requires a fresh host transport session. A failure retires the pair, clears RAM
and releases ownership, while preserving the last confirmed goal/count and the
failure trace. The standalone worker and all legacy EL0 batches remain mutually
exclusive with this session; stopping an inactive owner cannot erase live memory.

Actual QEMU debug/release results: twelve images and 740 structured requests plus
one direct-shell status request per profile. Normal runs completed nineteen jobs
across all six public records, retaining state across EL1 timer/context checks,
then restarted, rejected an old ticket and completed another job from count one.
Independent regex tokenization and IPC trace replay verified results, FIFO
movement, ready-task order, blocking/wakeup and final waiting states. Mutating
counts, identities, completion counters or traces was rejected by the audit.
Old/future tickets, wrong keys, duplicate observations/proposals, malformed
framing, denied records, overlap and stop/recovery were exercised.

Eleven injected cases covered A's forbidden read/spin/syscall flood/bad report/
forged completion, B accessing A's host gateway, and B's forbidden read/spin/
syscall flood/bad result ID/bad sequence. All were contained; legacy EL0 execution
worked after ownership release. The forged-completion fixture initially also
published a second report and hit the separate extra-report guard; it was
corrected to wait after the forged report, then passed the intended bad-report
check in both profiles. Selected final real captures are retained under
docs/evidence/2026-10-30-guest-coordinator-{debug,release}.txt.

Existing boot/protocol, schedule/lifecycle, IPC/wait, isolation, timer/context/
exceptions, MMU/memory, cooperative job, user/timeout, record-service and
persistent-worker suites passed in debug and release. Offline host Meta bridge
and coordinator suites passed in both profiles without provider calls. Host
console integration (including coord.*) passed, as did five console Rust tests,
seventeen panel tests, root/panel formatting, default and all 53 individual
fault-feature kernel Clippy checks, host console/panel Clippy, panel release
build and mixed-feature rejection. After tightening publication validation,
the entire new guest-coordinator suite was rerun in both profiles. Versions
remain macOS ARM64, Rust 1.90.0 and QEMU 11.1.2 with fixed virt-9.2/TCG/Cortex-A53.

Documentation now distinguishes guest policy, kernel enforcement and host
transport. This new path uses manual typed proposals. The existing live Meta
bridge/coordinator still uses record.count; no credentials were changed or
provider call made in this increment. Next connect that provider to the guest's
pending goal/ticket through an explicit host mode. Full workload, independent
demo artifact verifier, comparative benefit, video and submission remain pending.


## 2026-10-30 — Real Meta proposals for the guest-owned coordinator

Added --guest and --guest-replay modes to the existing host bridge. One QEMU
connection starts the retained EL0 coordinator/worker. Scripted observations go
to coord.observe; the model receives the actual guest pending ticket and current
allowed record metadata. The host does not maintain a replacement goal version,
filter older observations, or own a completed-result cache. Its original-ticket
proposal goes to coord.propose, where EL0 admits or rejects it. The existing
kernel/ABI, provider HTTP configuration and secret loader are unchanged.

Extracted shared bounded provider/replay thread ownership into decision_source.rs
and kept the older host coordinator's behavior covered by its existing tests.
New guest_report.rs strictly decodes the fourteen report fields and checks
observed transitions; guest_bridge.rs owns transport, timing, logging and cleanup.
The first unit run caught a field-count mistake in the decoder (fifteen instead
of fourteen); it was corrected before integration/live validation. Final tests
passed. Provider correlation/allowlist validation remains on the host, while
freshness/goal admission occurs inside the guest. Stale proposals are forwarded
with their original ticket, rejected without B execution, then followed by a new
bounded request. A current decline records unfinished work; obsolete declines
do not cancel newer goals. Other invalid proposals/errors stop without retry.

The live debug run made two actual Muse Spark 1.3 calls: 2161 ms/462 total tokens
and 1584 ms/491 total tokens as reported by the provider. Guest tasks completed
notes@1 with 30 bytes/7 tokens and notes@2 with 44 bytes/9 tokens. Two duplicate
observations produced no extra model call. One guest boot and explicit coord.stop
were recorded; total duration was 12121 ms including twelve seconds of scheduled
fixture time, plus boot/transport/cleanup. This run contained no stale response;
delayed stale rejection was exercised separately with offline replay. The live
capture and debug/release stale captures are retained under docs/evidence with
2026-10-30-guest-meta names. No credential or raw HTTP body was logged or committed.

Twenty-three Rust tests passed in debug and release. New real-QEMU offline tests
passed in both profiles: revisions/duplicates, delayed guest-side stale rejection,
current/obsolete decline, empty record, returning to a prior record, malformed/
denied/injected proposals, wrong-key rejection by EL0, bounded scenario/replay
input and exhaustion of four model calls. Handled failures explicitly stop the
guest session. Independent Python auditing reconstructed guest tickets/goals,
provider contexts, FIFO/IPC execution, exact fixture token counts, completion
counters, single boot and cleanup. Altered contexts/final state/call counts/token
counts were rejected. The live trace passed that same audit. Replay loaded no key
and made no provider calls. Existing single-proposal and host-coordinator offline
suites also passed in debug/release. Bridge formatting and all-target Clippy passed.
No kernel or panel code changed, so their unrelated full suites were not repeated.

Added per-checkout run instructions and explicitly separated the real provider
run from replay evidence. Added DEMO_PLAN.md with a concrete delivery sequence:
useful artifact plus independent checker on October 31, bounded workload/baseline
and feature freeze November 1, evidence/presentation November 2, video/submission
preparation November 3, submission November 4 before 18:00 Guatemala (official
20:00). These are targets. The counting path closes integration but does not
implement the full review-pack contract or demonstrate an AI performance win.
The remaining useful workload/verifier/comparison/video requirements stay visible.


## 2026-10-30 — Exact EL0 record index and the next native display gate

Added record.index for the same six immutable public fixture records. A linked
Rust EL0 program selects distinct lowercase ASCII tokens in lexical order and
emits exact frequencies through existing checked write/exit SVCs. Empty input
produces an explicit empty index. The kernel publishes only normal zero-code
completion without rejected writes; a partial nonzero exit or 100 ms timeout
returns an error without publishing the partial result. This reuses the single
invocation runtime and exclusive user-memory lease, not the retained coordinator.
An active worker/coordinator therefore rejects the invocation as user_memory_busy.

The entry transition now passes one checked RecordId scalar in x0, with zero for
legacy hello/spin; all other user registers are still cleared. The per-write bound
remains 64 bytes. Index cumulative output is 384 bytes, while hello/spin retain
64 bytes. No new mappings, heap, filesystem, SVC number or external dependency
was introduced. Token selection is independent of formatting and transport.
Repeated scans and one-byte writes are a simple first correctness implementation,
not a deliberately weak comparison baseline. Optimize the shared executor before
freezing a fair AI-versus-deterministic policy experiment.

A Rust host exporter saves exact guest output without overwriting existing files.
A separate Python checker reconstructs the index using regex/Counter against the
requested revision and SHA-256-checked fixture bytes. Its optional token lookup
runs on the host after acceptance. The combined scripts/index.sh command requires
no key. A direct debug export of notes@1 observed 91 ms boot and 7880 microseconds
request wall time, including UART/host scheduling; this is one smoke measurement,
not a benchmark. It found pump twice among seven tokens/six distinct tokens.

Actual QEMU checks passed in debug and release: three images per profile (normal,
partial failure, spin), 100 structured requests plus one shell request per profile.
They exercised all six records twice, empty content, revisions, denied/malformed
input, output mutations, memory-owner contention, timeout/failure recovery and
kernel/user continuity. Mutated count/order/case/identity/duplicates/omissions and
false completion were rejected independently. Real normal captures are retained
in docs/evidence/2026-10-30-record-index-{debug,release}.txt. Their raw UART CRLF
bytes are preserved with a scoped .gitattributes rule. Export integration
also passed non-overwrite, invalid/denied key, missing ELF, verified lookup and
corrupt/oversized/non-ASCII artifact checks.

Existing boot/protocol, user privilege/fault, user timeout, record service, MMU
fault and offline guest Meta bridge suites passed in debug and release. Host
console integration, five shared-model Rust tests and twenty-three Meta host
Rust tests passed. Formatting, default/two new fault-feature kernel Clippy,
exporter host Clippy and panel all-target Clippy passed. No live model request
was made and no credentials were changed. Source fixture bytes are unchanged.

The user requested native guest graphics, distinct from the Mac panel, alongside
iterative useful-work development. GUEST_GRAPHICS_PLAN.md now specifies the next
two gates: actual guest pixels, then an observed index/error view. This supersedes
earlier blanket graphics exclusions without expanding to phone drivers or a
window system. Native graphics are still pending; no screenshot/display boot is
claimed. The index is not yet a Meta-selected retained job or the full multi-record
review pack, and no AI efficiency advantage has been established. Subsequent work
must connect useful jobs/results, revisions and fair comparison while preserving
the submission deadline and historical contribution dates.


## 2026-10-30 — First native RAI-OS screen with protected framebuffer

Delivered the user's requested lightweight visual direction as actual guest
pixels: a warm-paper portrait layout, understated type, original geometric mark
and fine-line constellation. The illustration is decorative, not fabricated
agent/CPU/battery telemetry. The lower area starts with an actionable console
command and renders actual record.index revision, token statistics and entries,
with distinct empty/error states. Index failure replaces the previous result;
unrelated commands leave it unchanged. This is the first guest screen, not the
older Mac panel, interactive touch UI or completed desktop/window system.

Added an opt-in guest-display feature and scripts/desktop.sh (release by default).
The fixed TCG virt-9.2 machine gains ramfb; scripts/qemu.sh retains its headless
default and supports visible Cocoa or capture-only display modes. A new fw_cfg
adapter discovers/configures the device with bounded enumeration and DMA polling.
The optional linker reservation is 2 MiB at 0x41000000..0x41200000. Pixel storage is
1,536,000 bytes; descriptor/configuration data occupy a separate reserved final
page so timeout cannot leave DMA pointing at expired stack data. Barriers order
non-cacheable writes. No heap, guest networking or external Cargo dependency was
introduced. The headless build has no framebuffer reservation or display mappings.

The MMU adds one EL1-only RW/NX normal L2 block and one EL1-only/NX Device page for
fw_cfg, shared identically by the base/A/B roots. Table ownership remains twelve
pages with 244 available pool pages. Presentation receives observed results and
never owns command/AI policy or device registers. The display owner has a private
constructor and clipped pixel access. This renderer currently shares EL1 privilege;
it is not an isolated GUI service. The content-index program remains at EL0 and
cannot overlap retained worker/coordinator ownership of user RAM.

Typography uses licensed Ubuntu Light from the existing epaint_default_fonts
0.32.3 distribution. The original font, copyright notice and UFL-1.0 license are
included. The modified font is named Ubuntu Light derivative RAI ASCII. Its
committed 4-bit glyph coverage is 45,701 bytes plus metrics. Pillow 11.3.0 with
FreeType 2.13.3 generated those assets outside the kernel; exact regeneration was
checked. No font engine or source TTF is linked into the guest. Pixel geometry and
renderer code are original. Component sizes do not establish whole-OS efficiency.

Real QEMU debug/release verification passed five table tests and eight boots per
profile: the normal graphical path, missing device, bad configuration, DMA write
rejection, and four intentional EL0 access failures. The test harness initially
needed to recognize QEMU's expected warning for a deliberately injected non-opt/
fw_cfg entry; after that correction all fallback tests passed. EL0 framebuffer
read/write/execute probes produced exact level-2 permission faults, and fw_cfg
reads produced a level-3 permission fault. Pixels stayed intact and indexing
worked after each contained fault. Clippy prompted consolidation of the shared
memory-layout module into a single definition; final checks passed.

QMP screenshots validated actual dimensions/pixels and all six independently
checked record indexes, revision changes, repeatable views, empty/denied/malformed
requests, coordinator contention, stale-result removal and recovery. Selected
captures matched byte-for-byte between debug and release. A separate release run
opened the real macOS Cocoa/QEMU window, exercised the same normal transitions
and closed its owned instance. Evidence includes raw UART transcripts and actual
boot/notes screenshots under docs/evidence/2026-10-30-display-*.

Existing headless boot/protocol, MMU faults, user faults, user timeout, record
index, persistent worker, private-stack isolation and offline guest-Meta bridge
suites passed in debug and release. Host console integration passed. Formatting,
default/display and all four new fault-feature Clippy checks passed. No provider
call or credential change was made. Graphics redraw happens only at boot/on index
outcomes, but UART still busy-polls and caches stay disabled: no CPU/battery or AI
performance advantage is claimed. Drawing time is included before a graphical
UART response, outside the EL0 index budget.

Both initial native-display gates are complete. Freeze this visual slice and
return to kernel/service work: a retained coordinator index job with bounded
output transfer, then revision-aware combined results and a fair comparison.
Meta-selected indexing, the full review pack, interactive GUI, physical phones,
comparative gains, final video and submission remain pending. Historical dates
and the November 4 deadline are unchanged.

Final display review also hardened derived token statistics against malformed or
overflowing EL0 output: the view reports unavailable statistics instead of
panicking, and oversized labels explicitly defer to the console. A pure boundary
test passed; the affected normal graphical/fallback paths were rerun in both
profiles and retained the same captured pixels for valid fixtures.


## 2026-10-30 — Retained exact index and revision reuse

Added coord.start index and coord.result while preserving default count mode and
its existing 14-field report/Meta bridge. EL0 coordinator A retains admission and
reuse policy; EL0 worker B constructs the exact index using a constant-space
serializer shared with standalone record.index. Seven-byte scalar IPC chunks
carry the index without exposing either private writable stack. The kernel
validates order, encoding, padding, length, canonical structure, worker sequence,
proposal identity and publication before committing one of six fixed revision
slots. The payload cache is kernel-owned storage, not a new isolated memory
service or a buffer on A's stack. Only metadata is supplied back to A.

Observing notes@1, proposing it, completing notes@2 and returning to notes@1 now
produces event=reused with completed=2, a_calls=2 and b_calls=0. A cache hit returns
the exact original revision and does not run B. Pending goals cannot expose a
previous artifact. Stop/fatal failure clears every slot and releases the pair's
user-memory lease; old tickets remain invalid after restart. Index mode has a
384-byte result bound, 128 syscall-attempt limit per task and 192 trace pairs.
Count-mode limits remain unchanged. Timer slices still do not constitute an
aggregate command deadline. Six immutable fixtures and synchronous commands
remain the only source/dispatch model; there is no filesystem or background watch.

Native views now display the actual pending goal/proposal, created index and
reused index. Duplicate/status reads preserve the reuse label. Parse/execution
errors replace visible content, failed status remains an error, and stop restores
the original screen. Appearance outside the content area is unchanged. Renderer
privilege remains EL1; this does not add a user-space compositor or touch input.

Verification passed in debug and release: two stream boundary unit tests and
15 real-QEMU boots per profile (normal retained session, twelve injected service/
stream failures, normal graphical transitions, graphical truncated-stream failure).
Each profile's 13 headless boots issued 424 protocol requests. Checks cover all
six independently verified artifacts, independent IPC trace replay, exact reuse,
wrong/stale/duplicate proposals, cache reset, forged reuse, truncated/oversized/
misordered/noncanonical streams, wrong end length and data after completion.
Failures after a successful first job remove both its cache entry and the partial
next output. Tests reject forged audit reports and an artifact for the wrong
revision. Native captures show pending/created/reused/error/stop and recovery;
pending/reused debug and release PNGs match byte-for-byte. Raw normal UART and
selected actual screenshots are retained under docs/evidence/2026-10-30-coord-index-*.

Existing guest-coordinator count, persistent worker, standalone record-index,
boot/protocol, offline guest-Meta bridge, private-stack isolation, user timeout
and full native-display suites passed in both profiles. Host console integration
also exercised the new start/index/reuse/result commands. Five console unit tests
and 23 Meta unit tests passed; the existing host panel compiled. Rust formatting,
default/display/new-fault-feature Clippy and host-console Clippy passed. No live
provider request or credential change was made. Test coverage and exact commands
are documented in RETAINED_INDEX.md.

This is a deterministic reuse mechanism, not an AI efficiency result. The next
increment is to connect the existing Meta proposal transport to these useful
index artifacts. Combined multi-record outputs, fair baseline comparison, the
three-minute video and final submission remain pending. The hackathon dates and
November 4 deadline are unchanged.


## 2026-10-30 — Meta proposals produce retained guest index artifacts

Added explicit --guest-index / --guest-index-replay modes to the existing host
bridge. A typed work selection narrows provider JSON schema and proposal admission
to record.index or decline; legacy single/host-coordinated/guest count modes remain
count-only. Only the current guest-required record's metadata and original ticket
reach Meta. No future schedule, source bytes, index payload, local path or
credential is included in model context. The kernel and its EL0 programs are
unchanged in this increment.

The bridge decodes the index report's cache metadata and budgets, validates
observed transitions, requests coord.result after execution or reuse, and emits
the exact artifact as guest_index evidence. Bounded artifact checks enforce
identity, completion, size and canonical shape/accounting. A host evidence history
requires a reused artifact to equal a previously observed one; it never supplies
a replacement result or chooses a cache hit. Semantic correctness remains the
independent fixture/checker responsibility. An existing in-flight inference is
joined and its stale proposal reaches guest admission; cache reuse avoids new
inference rather than cancelling one already running.

The optional leading --graphics flag builds the existing display-enabled kernel
and opens QEMU Cocoa. The shared child adapter confirms the guest framebuffer-ready
message before accepting graphical boot. Existing headless callers retain their
path. The bridge closes its owned child at completion; the renderer is unchanged
and displays actual coord.* outcomes. A timed public fixture leaves a reused
result visible until the final duplicate observation at 20 seconds.

Debug/release validation passed 28 host unit tests and 15 offline real-QEMU bridge
cases per profile: exact artifacts and reuse, duplicates, stale replies, reuse
during inference, obsolete/current decline, empty/larger records, invalid version,
denied/injected/wrong records, mode confusion, unsupported operations, legacy
permission preservation, four-call exhaustion and truncated guest-stream cleanup.
The independent audit checks source hashes/exact token counts, kernel IPC traces,
tickets, cache hits, model-call accounting, original artifact retrieval and final
state. Mutation checks reject fabricated or missing artifacts and changed metadata.
Native debug/release bridge runs also passed against the actual Cocoa window;
a headless kernel supplied with the graphics flag is explicitly rejected before
inference. Clippy initially requested two conditional cleanups; those were fixed
and final all-target host Clippy plus formatting passed.

Existing offline guest-count bridge, one-shot Meta and host-coordinator suites
passed in both profiles. Host console integration passed and the existing host
panel compiled. No full kernel regression rerun was necessary: kernel source was
unchanged and the new integration tests exercise its current retained-index path.

A separate live release run opened the native QEMU screen and made exactly two
real Muse Spark 1.3 requests. It constructed notes@1 and notes@2 inside RAI-OS and
retrieved notes@1 again from the guest cache with no third model call or B turn.
The final duplicate preserved completed=2, cached_records=2 and cache_hits=1.
Actual API latencies were 3350/1606 ms and reported total tokens were 438/399.
The session took 20274 ms, including the intentional 20-second scenario schedule.
Explicit cleanup completed and the independent log/content/IPC audit passed.
No credential was changed, displayed or committed. Live and separately labeled
replay stale/reuse-in-flight evidence are stored under docs/evidence/2026-10-30-meta-index-*.

This establishes useful artifact integration, not a model performance advantage:
the current single-action proposal and deterministic cache remain solvable without
AI. Next combine revision-aware records into a complete useful result with explicit
invalidation and independent acceptance, then compare meaningful policies fairly.
The final three-minute video/submission and original event dates remain pending
and unchanged. Run/setup/limits are documented in META_INDEX_BRIDGE.md.


## 2026-10-30 — Complete versioned index sets and selective revision work

Added coord.goal, coord.bundle.status and coord.bundle to retained index sessions.
The typed required set accepts one to four public document revisions, normalizes
key order and rejects duplicates, conflicting revisions, unknown/denied records
and extra arguments before mutation. EL0 coordinator A retains the required bitset,
clears the active single-record goal on changes and rejects out-of-set record
inputs. Coordinator-only scalar SVC 15 attests the new generation, requirement and
completed-revision mask against the kernel's pending input/cache before metadata
is committed. A normal change has three A calls and no B dispatch. The kernel also
checks required membership before worker dispatch. No user pointer/mapping changed.

The required-set generation shares the monotonic issue counter with observations;
stale pre-change proposals cannot execute, including when the same record remains
required. Existing completed revisions remain cached. The kernel derives the
missing set from committed slots and rejects incomplete bundle reads instead of
returning an earlier snapshot. A complete owned artifact serializes exact required
keys, runtime goal generation, public source bytes as lowercase hex and original
EL0-generated indexes. Serialization runs at EL1; this is not a new isolated pack
service. Its four bounded index copies use the existing kernel stack without a
heap; the large reply variant has a documented Clippy allowance. No stack-peak or
whole-OS efficiency claim is made. Stop/failure clears requirements and cache;
the issue counter still survives stop/start.

The native content view now shows goal generation, indexed/required counts,
completed worker jobs and per-revision INDEXED/WAITING states. Incomplete-output
errors replace old content, status restores current progress, and stop restores
the original ready screen. SET COMPLETE is the guest's committed-membership gate,
not an independent semantic verdict or full review-pack R1-R6 acceptance.

A Rust host walkthrough requests the guest's missing list, submits individual
proposals only for those entries, saves exact guest JSON into a fresh timestamped
directory and runs the independent checker on each output. The public graphical
script completed the three reference requirements with total jobs 3, 4 and 5.
Only checklist@1 and then notes@2 added work; unchanged records retained identical
bytes/indexes. The checker independently verifies source hashes/bytes, required
membership, generation and complete token frequencies. Its optional host-side
lookup found safety once in checklist@1 and once in notes@2 in the final artifact.
The guest generated those indexes; the host only verifies/queries the result.

Debug and release passed three headless bundle images per profile (316 protocol
requests), plus native view captures. Tests check the three reference goal indexes,
all 45 valid bounded revision sets, empty-record preservation, unchanged entries,
no incomplete/stale publication, malformed requirements, out-of-set actions, stale
proposals across changed sets, stop/restart and memory ownership. Dedicated fault
images reject forged EL0 goal attestation and a truncated notes@2 stream after
an earlier completed set, clear prior cache/requirements and regain the console.
Artifact mutations reject missing/duplicate/stale/changed source/index data and
duplicate JSON fields. The first truncated-stream harness stopped before reaching
the injected revision; its traversal was corrected, and the complete fault path
then passed. Native tests were extended to show all three successive requirements;
pending/final debug and release PNGs matched byte-for-byte.

Existing boot/protocol, retained-index, guest coordinator, persistent worker,
standalone index, private-stack isolation, user timeout, full native display and
both guest Meta bridge modes passed in debug and release. All 29 Meta host unit
tests passed in both profiles after decoding the new idle-with-cache/goal trace
state. Host console integration passed; the old host panel compiled. Root/host
formatting, kernel default/display/new-fault Clippy, the Rust demo and all-target
Meta Clippy passed. Actual UART, three JSON artifacts and native screenshots are
retained under docs/evidence/2026-10-30-bundle-*. No live API call or credential
change was required for this increment.

This is a deterministic multi-record correctness foundation. The existing Meta
bridge still operates per record without a required set; whole-set planning is
next. Timed release/outage behavior, simulated resource/deadline enforcement,
optional previews, the complete review-pack contract, fair comparison, final video
and submission remain pending. Original dates/deadline remain unchanged. Commands,
format, boundaries and reproduction are documented in INDEX_SETS.md.

## 2026-10-30 — Whole-set Meta choices with guest generation admission

Added explicit --guest-set / --guest-set-replay modes, optionally driving the
existing native framebuffer. They transport scripted complete requirements,
decode current guest membership/missing entries, and ask Meta to choose one
missing revision. The model receives current metadata only, not future events,
source bodies, indexes or local paths. Existing per-record/count modes retain
their schema permissions and four-request cap; set mode has one pending request
and eight calls for the five-job reference case plus bounded obsolete responses.
Requirements are limited to eight events over 30 seconds and four revisions per
set. A current decline, invalid action, exhausted budget or guest failure stops
explicitly; there is no deterministic fallback or hidden retry.

New coord.select <goal_version>:<key> checks the committed set generation,
required membership and missing status before issuing a per-record ticket. The
EL1 gate uses previously EL0-attested requirement metadata; EL0 still receives
the observation and admits the subsequent ticketed proposal. Rejections leave
state and the issue counter unchanged. A goal change between select/propose
invalidates the ticket as before. Source proposals are checked against their
original missing list, and late valid proposals reach the guest with their
original generation. This avoids refreshing obsolete authority on the host.
No new syscall, user pointer, memory map or agent process was introduced.

The bridge shares transport, bounded provider threads and retained-index audit
history with the existing index mode. New pure decoders validate set reports and
artifact structure; the independent Python audit reconstructs goal generations,
model snapshots, selection tickets, exact jobs and publications. It independently
checks source bytes/token frequencies and existing scalar IPC traces. The host
prints exact guest bundle bytes rather than assembling substitute output. Native
views use observed set progress, including selection errors and status recovery.

A new coordinator command arm exposed a reproducible unoptimized native stack
corruption. An isolated diagnostic build showed the timer ACTIVE pointer change
from zero to 2 while entering coord.goal, before the timer_busy rejection. Grouping
coordinator report arms into one helper reduced the measured dispatcher frame
from 27,552 to 20,608 bytes (helper: 1,104 bytes). The corrected image kept that
pointer zero and completed the same sequence. The 64 KiB kernel stack, timer
checks and mappings were unchanged. Full stack bounds/guard hardening remain
pending; this is a correction of the reproduced path, not a global safety proof.
Initial timed replay tests also assumed very short UART/render delays; margins
were widened, and graphical auditing permits superseded historical publications
to be absent while requiring an exact final set. Final native debug/release runs
both actually published all three reference goals.

Verification passed: root and Meta formatting; default/display kernel and all-
target Meta Clippy; 32 Meta unit tests in both profiles. New real-QEMU debug and
release checks include 43 framed requests plus shell selection for the generation
boundary, five successful replay scenarios, invalid/missing/cached proposals,
current/obsolete declines, an eight-call limit, invalid pre-boot requirements and
a truncated stream. Native Cocoa replay passed both profiles. Existing boot/UART,
complete-set normal/fault/native tests, old guest index/count bridge suites and
host console integration passed. All checks used real QEMU where stated; replay
and live provider results remain labeled separately.

A real release/Cocoa run used five Muse Spark 1.3 calls. The selected order was
brief@1, empty@1, notes@1, checklist@1, notes@2. Generations 1, 5 and 7 published
with completed-worker totals 3, 4 and 5; all three exact outputs passed independent
verification. Latencies were 1965/2092/1466/1995/1870 ms, with 2442 total reported
tokens. No stale response occurred live; separate debug/release delayed fixtures
prove rejection for a key still present in the replacement goal. Evidence and
exact guest JSON are saved under docs/evidence/2026-10-30-meta-set-*.
Credentials were neither changed nor included. Instructions are in META_SET_BRIDGE.md.

The model now chooses among missing jobs, but this small independent workload is
also straightforward for fixed code. No AI efficiency or OS security superiority
is claimed. Next define useful constrained choices and a fair baseline using the
same engine and acceptance checks. Full review-pack events/budgets, final video,
eligibility confirmation and submission remain pending; original dates and the
November 4 deadline remain unchanged.

## 2026-10-30 — Freeze the first shared policy comparison pilot

Implemented a host experiment controller that sends the existing real guest index
operations for EDF, weighted value/credit ordering, Meta or labeled offline replay.
All policies share current metadata, generation admission, exact cache, source/index
checking and the same QEMU executor. Scenario clock begins before boot; inference,
transport and independent publication checking count toward completion. Per-record
priority targets produce a weighted early-readiness score only after the final
required set passes. Fixed rules re-evaluate after events and reuse the same cache;
neither fixed nor model policies receive future events.

Cumulative simulated credits are max(1, ceil(public bytes / 32)), nonrefundable per
admitted worker attempt. Caps are enforced by the common host controller, not a new
kernel service or physical battery measurement. Admission logs include an event
cutoff, current cap and debit; future reductions preserve prior legal spending.
Priority-only events commit a new guest generation to invalidate old model context.
Eight decision attempts bound every policy. Invalid choices, declines, exhausted
budgets, errors and deadline failures are retained without a hidden fallback.
Provider work remaining after a deadline or already-verified final result is drained
only for usage and cleanup. Its late answer cannot execute or retroactively move
a qualifying publication time; a replay test verifies this cached-goal rollback.

pilot.rs contains deterministic rules; pilot_run.rs owns clocks/processes/UART and
shares the established transport/decoder. The provider adds optional current pilot
metadata without changing legacy contexts or permissions. The independent Python
audit reconstructs exact snapshots, fixed-policy choices, credits, tickets, IPC,
source bytes/indexes, readiness and scores. A common independent artifact check
runs inside every trial's timing. The separate comparison command runs fresh QEMU
instances sequentially, rotates policy order, records code/binary/scenario hashes,
retains all logs/failures and generates per-trial and per-scenario reports. It
requires a clean committed checkout and kills its owned process group on timeout
or interruption, retaining partial output.

Three development fixtures and their rules are frozen in POLICY_PILOT.md before
live model measurements: steady, priority-change and revision-quota. The revision
cap allows eager fixed-policy work plus the final revision, rather than making
early execution artificially infeasible. Planned measurement: two repetitions of
EDF, value and Meta on each fixture, 18 runs. No measurements or superiority result
are claimed by this pre-live entry. The larger held-out evaluation, full review
pack, final video and submission remain pending; original event dates are intact.

Verification before freezing: root/host formatting, all-target Meta Clippy and
36 host unit tests passed in debug and release. Fourteen offline real-QEMU pilot
runs passed per profile: declared baseline order, original/shared snapshots,
source/index correctness, stale revision and priority authority, insufficient and
reduced caps, grandfathered spending, eight-choice exhaustion, late-response drain,
verified rollback, forbidden/wrong-mode/unoffered/declined actions and cleanup.
Independent log mutations reject altered scores, credits, context and artifacts.
An initially inconsistent rollback-test target exceeded its own hard deadline;
the fixture was corrected and both full profiles passed. Existing whole-set bridge
suites passed in debug/release, and the previous count bridge passed in release.
No kernel, timer, MMU or display implementation changed in this increment.

## 2026-10-30 — Measured policy pilot; no AI advantage observed

After pushing frozen source c649e592040a643ee3a00e5f8fa0b95f13b59713, ran the
planned 18 sequential release/QEMU trials: two repetitions of three scenarios
for EDF, value and real Meta, with rotated policy order and fresh guests. The
comparison command and independent re-audit completed successfully; all traces,
including failed outcomes, were auditable. No prompt, policy, scenario, scoring
rule or provider setting changed after outcomes were observed.

Both fixed policies passed 6/6. Meta passed 1/6; five trials terminated with the
adapter's meta_incomplete_or_refused category. That category combines several
provider envelope conditions; exact finish/refusal causes and usage for those
failed responses were not retained, so do not claim a particular root cause or
treat the outcomes as a pure model-intelligence test. No failure was retried or
replaced. No incomplete final bundle earned success or a nonzero score.

In the successful steady Meta trial, the order matched EDF (notes, brief, empty).
Readiness was 2952/6661/8514 ms, with independent final publication at 8591 ms and
9/13 on-time points. Fixed steady runs completed in 152-163 ms with 13/13 points.
The successful Meta trial's available provider latency totaled 8433 ms; observed
request/response boundaries totaled 8448 ms. All successful steady policies spent
four simulated credits. Lower spend in failed partial runs is not an efficiency
win. Priority/revision fixed-run times include waiting for scheduled events.

Fourteen Meta calls produced nine usable usage records totaling 8267 reported
tokens; rejected-response usage is unknown. Across all calls, request-to-response
or drain boundaries total 39,998 ms of observed waiting. results.json inference_ms
sums available provider latencies only and is incomplete for failed calls;
end-to-end timing includes those waits. This limitation is explicit in the analysis.

Preserved all 60 generated evidence files, including the raw 18 JSONL/stderr pairs,
exact publications, metrics/timelines, source/binary/input hashes and commands in
docs/evidence/2026-10-30-policy-pilot. The manifest's October 31 UTC timestamp is
October 30 in Guatemala; no date was rewritten. POLICY_PILOT_RESULTS.md explains
outcomes, limits, reproduction and the next hypothesis. Credentials were unchanged
and absent from the evidence.

The measured result argues against remote model calls for each tiny index job
in this configuration. Next diagnose rejection categories and test higher-level
or batched planning with equally capable fixed execution, using a new frozen
protocol rather than tuning this one toward a win. Full review-pack acceptance,
held-out evaluation, presentation/video and submission remain pending.

## 2026-10-30 — Persistent-goal direction after the scheduling pilot

Defined the user-agreed direction around maintaining a useful result as inputs
change. PERSISTENT_GOALS.md specifies an existing-fixture walkthrough, current
versus historical publication, failed-update/stale-answer checks, ownership gaps,
separate semantic-planning work and fair evaluation gates. README and the
delivery/architecture guides point to it.

Official Apple App Schemas, Android AppFunctions and AndroidWorld sources identify
adjacent capabilities; this limited review does not establish novelty or absence
of similar work. No competitor limitation is inferred. The next implementation
gate is a checked publication lifecycle, followed by observed native states,
provider diagnostics/batching and separately frozen semantic evaluation.

This increment changes documentation only: no new kernel behavior, model request,
QEMU execution or efficiency result. Prior evidence and failures are unchanged.
Verification: consistency review against index-set ownership/checks, local
Markdown target checks and git diff --check. No executable code changed, so
compilation/QEMU suites were not repeated. The November 1 feature-freeze and
November 4 delivery targets remain; demo/video/submission are not complete.

## 2026-10-31 — Retain the last publication during updates and failures

Implemented the first persistent-goal slice: a pure Rust publication lifecycle
and one owned Artifact in the guest EL1 coordinator adapter. Five console/protocol
operations expose status, prepare, commit, reject and read. Unique candidate
tickets share the monotonic boot counter; re-preparation, goal changes, verdicts
and session failure invalidate pending authority. Commit rechecks the live guest
goal/completeness after trusted host validation. No proof against a malicious
host is implied. The model and existing Meta modes do not own this handshake.

The previous publication remains readable while building or after fatal worker
cleanup, even when a subsequent user program reuses the released EL0 RAM. Explicit
stop clears access; restart and reboot have no publication persistence. The copy
holds up to four exact indexes and metadata, with source bytes supplied by linked
immutable public fixtures. No new mapping, syscall, heap or unsafe code. Native
publication UI, dynamic permissions and semantic planning remain pending.

Verification: formatting and Clippy passed (default/debug, display/debug and
default/release). New real-QEMU publication tests passed in debug and release,
including native-display builds and a deliberately truncated worker. Covered
independent exact checking, corrupted checker input and rejected verdict, stale
proposal/commit, changed goal during checking, re-prepare/retry/duplicate tokens,
four-record bounds, cached rollback, malformed arguments and stop/start freshness.
The two unchanged entries were reused: first pack took three jobs, revision
update took one additional job. Fatal-failure output remained byte-identical.
The strengthened post-failure user-RAM reuse check also passed in both profiles.
Existing bundle suites (45 valid subsets, fault cases and native views) and boot/
protocol suites passed in both profiles. No Meta calls or performance comparison
were made. Raw new traces and exact artifacts are preserved in
evidence/2026-10-31-publication; earlier pilot evidence remains unchanged.

## 2026-10-31 — Native publication states and checked visual walkthrough

The native guest view now renders the actual publication lifecycle: not started,
preparing, updating, verifying, current and blocked. Previous-result availability
is separate from the active target; complete cached indexes cannot appear CURRENT
without acknowledgment. Target/progress and failure notices come from observed
guest state. Read-only Status accessors and a post-execution rendering method
keep presentation separate from admission. No model, syscall or unsafe change.

Added a Rust public walkthrough with typed shared transport operations, independent
artifact validation, interactive terminal pauses or automatic/headless modes.
It opens real QEMU, publishes the first pack, replaces notes@1 with notes@2,
intentionally rejects an invalid host checker input, shows preserved prior output,
rejects a stale confirmation and commits the valid replacement. Four worker jobs
complete the two requirements. Scripted changes, deliberate failure and pauses
are explicit; no Meta calls or comparative performance claim were made.

The first native debug attempt failed with timer_busy at coordinator startup.
Extracting render_result reduced Console::execute's compiled frame from 5296 to
656 bytes; the render helper uses 3088 bytes. The failing sequence then passed
without increasing the 64 KiB stack or weakening timer admission. Compiled
prologues are retained with evidence; global stack bounds/guards remain pending.

Verification: default/debug, display/debug and display/release kernel Clippy,
formatting, host-example Clippy and six host protocol tests in both profiles
passed. New native publication scenarios passed in debug and release, including
a truncated-worker build, unchanged retained bytes, malformed/stale notices,
retries, unverified completeness, progress and stop. Thirty-five matching captures
were byte-identical across profiles. Existing bundle, publication and base-display
suites passed in both profiles, including display mapping fault containment.
The public walkthrough completed in headless and native automatic modes. Its
archived initial/updated artifacts passed independent re-checking; its deliberately
invalid input was rejected. Local documentation links and diff whitespace passed.

Evidence is in evidence/2026-10-31-publication-display, including nine actual PNGs,
four raw UART traces, walkthrough outputs/artifacts and stack prologue excerpts.
Earlier evidence remains intact. Legacy bundle exports now say they are exports,
not acknowledged publications. Documentation identifies host validation trust,
in-memory retention and the next scope: provider rejection diagnostics/batched
planning. Semantic benefit, final video and submission remain pending; delivery
dates and the kickoff/build-period discrepancy are unchanged.

## 2026-10-31 — Distinguish provider failures before batch planning

Implemented host-only redacted provider diagnostics. The pure decoder separates
finish length/filter/tools/unknown, role, refusal, tool-call, model/choice, content
and proposal failures. Numeric usage survives readable rejected envelopes;
unknown usage remains null. Diagnostics normalize untrusted strings and omit
raw content/refusals/tool arguments/credentials. Typed failures carry metadata to
the shared async reporter and single-call CLI. Decoded is not domain admission
or a successful guest task. Replay does not fabricate provider records.

Verified that prompts, request schema, model, endpoint and settings remain
unchanged. Provider timing now includes local request preparation; it is not pure
inference time. Documented that successful legacy response records describe the
same calls and must not double-count usage, and that unconsumed early-teardown
results can remain unavailable. The original pilot's ambiguous failures and
partial usage evidence were preserved without guessing their causes.

Verification: all 41 host tests and all-target Clippy passed in debug/release.
Twenty-five offline curl-fixture cases per profile passed, including safe output,
specific failure categories, usage preservation, no retries, real-QEMU dispatch
and async cleanup. Release single-operation and whole-set bridge regressions
passed. Offline traces are in evidence/2026-10-31-provider-diagnostics. No kernel
or native display implementation changed and no live request has been made in
this increment. One unchanged-settings, no-retry live diagnostic probe is planned
after commit; no new scheduling/performance comparison or AI advantage is claimed.
Next implement bounded batch proposals, then independently specified semantic
workloads and a separately frozen fair comparison. Submission dates are unchanged.

## 2026-10-31 — One live provider diagnostic probe

After publishing diagnostic source 95bd62810929d41562a61f3e1a068db5f96fe9ff, ran
exactly one planned latest-notes request with the existing model/settings and no
retry/fallback. HTTP 200, matching model, one assistant choice, finish_reason=stop,
no refusal/tools; provider-method latency 1592 ms and reported usage 376 prompt +
153 completion = 529 tokens. CLI wall time was 1702 ms. QEMU boot/operation took
103 ms and returned notes@2, 44 bytes, 9 tokens. Independent capture/service checks
passed, including agreement between the diagnostic and same-call legacy summary.

Preserved stdout/stderr and manifest with source/binary hashes under
2026-10-31-provider-diagnostics/live. No request parameters were changed after
observing the outcome. This is a diagnostic smoke test, not evidence of speedup,
not the semantic workload, and not an explanation of the five historical failures.
Batch planning is still the next implementation increment.


## 2026-11-01 — Bounded batch proposals and verified guest publication

Added a separate --guest-batch / --guest-batch-replay mode with strict typed plans
covering all 1–4 missing revisions. Whole-plan validation precedes any execution;
invalid suffixes, duplicates, omitted/cached/unlisted work, operations and wrong
versions fail closed. Valid stale answers and unexecuted obsolete suffixes are
discarded. Existing event ingestion, generic typed async lifetime, HTTPS transport
and redacted diagnostics are shared with legacy modes. Original single-action
prompts/settings remain unchanged; batch alone uses its own schema/prompt and a
1024-token cap. There is no retry, fallback or model-controlled checking.

Each action rechecks observed missing work and passes the original generation to
the guest, then its fresh per-record ticket. Completion prepares an exact guest
artifact, runs the independent checker via bounded stdin and reconciles events
before committing. Read-back must equal the checked bytes. Native CURRENT now
follows this handshake in the batch path. All queues/plans/checking remain host
code; no kernel, syscall, unsafe or graphical implementation changed. Static
fixture permissions/catalog do not implement dynamic revocation/versioning.

Verification: 45 host unit tests, formatting and all-target Clippy passed in both
profiles. Twenty offline real-QEMU batch cases per profile covered four-record
plans, selective revisions/cached rollback, coalesced stale responses, invalid
suffixes/declines/limits, changes between jobs and during validation, checker
failure, provider decoding and truncated EL0 output. Two native cases per profile
passed. Independent trace/artifact audits rejected deliberately corrupted reports.
All 25 previous provider diagnostics per profile and release whole-set/single-call
regressions passed. A delay-wrapper process leak found in the test harness was
corrected by explicitly recording/terminating its child; no production adapter
change was needed. New raw traces are under evidence/2026-11-01-batch.

No live batch request or comparative efficiency measurement has occurred in this
source increment. Exactly one frozen-settings three-record live probe is planned
after commit, preserving all outcomes. Semantic interpretation, a separately
frozen fair comparison, final video and submission remain pending. November 1
was the freeze target: do not expand kernel/GUI scope. Original pilot failures,
actual contribution dates and the unresolved kickoff/build-period discrepancy
remain intact; deadline November 4 at 20:00 Guatemala, internal target 18:00.


## 2026-11-01 — One live batch with verified guest publication

After source commit 0b84ed9ea886565fd05592e9fa32626c3408f80f was pushed, the planned
single release-mode request succeeded without retry or fallback. Meta returned
one plan for brief@1, empty@1 and notes@1. All three jobs executed in real QEMU;
the independent checker, fresh commit/read-back and independent trace audit passed.
HTTP 200, matching model, one assistant choice, finish_reason=stop, no refusal or
tool calls. Provider-method latency: 2075 ms. CLI wall time: 2292 ms. Reported usage:
395 prompt + 184 completion = 579 tokens. The diagnostic and batch response are
summaries of this same call; usage/latency agreed.

Exact stdout/stderr and the manifest with source, binary/kernel/fixture hashes
and UTC observation timestamp are preserved in evidence/2026-11-01-batch/live.
This is one successful integration probe on an explicit three-record requirement,
not a repeated benchmark, evidence of semantic intelligence or proof of advantage
over fixed batching. It is not directly comparable to the earlier multi-call
pilot or the different single-count probe. No settings were changed after seeing
the outcome. Independent semantic cases and the fair comparison remain next.


## 2026-11-01 — Content-based selection before deterministic guest execution

Added six public semantic development cases and independent exact expectations
before model responses: explicit membership, historical/current handoff, ambiguity,
unavailable unit and content paraphrase. The new interpretation mode sends only
the goal, shared dependency policy and six public compiled document contents.
It returns select/clarify/decline bound to exact goal text/request version and
static catalog/permission versions. Rust admission validates authority/shape;
it deliberately does not claim to prove semantic correctness.

The Python demo checks the independent expected membership/decision before any
work, then hands accepted selections to the existing guest batch execution and
publication functions, without another model call. Clarify/decline boots no QEMU.
The raw execute-selection CLI trusts host approval and explicitly does not run
this fixed-case intent oracle. Each case uses a fresh guest session; continuous
semantic replanning and cross-case publication retention remain unconnected.
No kernel, syscall, unsafe or UI changes. Legacy provider prompts are unchanged.

Verification: 48 host unit tests and all-target Clippy passed in debug/release;
formatting and diff whitespace passed. In each profile, six replay cases,
20 pre-boot invalid selection rejections, four offline provider cases and a native
historical case passed. Independent checks rejected permitted-but-wrong meaning,
missing dependencies and corrupted traces/artifacts; failed provider usage remains
recorded when available. All 20 existing release batch cases passed. Exact offline
traces and summaries are archived under evidence/2026-11-01-semantic.

Planned after source commit: exactly six live Meta requests, one per checked-in
case, strict rai_selection schema, muse-spark-1.3, minimal reasoning, 1024-token
cap, no retries/fallback or tuning after results. Preserve every outcome. This
small development gate is not the complex workload or a fair policy comparison;
it cannot establish superiority. Freeze further infrastructure and protect video,
reproducibility and November 4 submission (18:00 internal, 20:00 Guatemala).


## 2026-11-01 — Six live semantic outcomes and verified execution

Ran the frozen suite once after pushing source 732051d34db0c388c7944538f831e0c01719b32f.
All six independently specified cases passed, with one live request each and no
retry, fallback, prompt tuning or expected-answer change. Four select outcomes
executed ten total real guest jobs across four fresh sessions and published
independently checked artifacts. Clarify/decline started no QEMU and did no work.
Every call returned HTTP 200, matching model, one assistant choice, finish stop,
no refusal/tools. All six reported usage; total 5335 tokens, counted once.

| Case | Decision | Selected records | Provider adapter ms | End-to-end ms |
| --- | --- | --- | ---: | ---: |
| explicit | select | brief@1, notes@2, checklist@1 | 4764 | 5342 |
| historical | select | brief@1, notes@1, checklist@1 | 4543 | 4684 |
| latest | select | brief@1, notes@2, checklist@1 | 5506 | 5644 |
| ambiguous | clarify | None | 2180 | 2191 |
| unavailable | decline | None | 3170 | 3183 |
| field | select | appendix@1 | 3135 | 3260 |

These are six public development successes, not a held-out accuracy estimate or
proof of superiority over a good deterministic resolver. The end-to-end intervals
include the independent intent gate, file handoff and execution/cleanup where
applicable; builds are excluded. Interpretation adds measurable delay. No fair
performance comparison has been run and no complex real-world workload is solved.
The historical prompt resolved notes@1 and handoff dependencies; the ambiguous
prompt asked for a revision choice rather than inventing one.

The exact manifest, selections, reports and complete stdout/stderr are preserved
under evidence/2026-11-01-semantic/live. Independent intent and execution audits
were rerun against the archived data; provider diagnostics and response summaries
agree on usage/latency. Native graphics were verified offline in both profiles;
this live suite was headless. Continuous semantic event recovery remains pending.


## 2026-11-01 — Scoped deterministic resolver and frozen semantic comparison

Added a domain resolver using the same public catalog and dependency rules as
Meta: explicit keys, unit/time/revision references, conflicts, limited synonyms
and clarification outside its supported grammar. It does not read expected case
answers. Shared harness/guest execution now accepts rules as a source. Twelve
public development cases (six previous, six new) and two rotated repetitions per
policy are frozen before measurement: 48 trials, 24 calls maximum, no retries or
fallback. The conflicting-version case explicitly accepts either clarification
or a nonexecuting decline before inference. This is not held-out evaluation.

A separate two-goal playback rechecks archived historical/latest Meta selections,
then uses one real guest session. Verified unchanged prior publication during the
update, two reused records and only one additional job (four total). The entire
input sequence validates before boot. Replay/no-live-inference and native 1500 ms
presentation pauses are explicit. It is not continuous live semantic replanning
and is excluded from the timing comparison. No kernel/unsafe/UI changes.

Verification: 50 host tests, formatting and Clippy passed in both profiles;
twelve deterministic cases and real-QEMU trace checks per profile; headless and
native sequence checks in both profiles. Bad second selection rejected before
boot. Auditors rejected altered success/meaning/history/artifact claims and a
manifest omitting a trial. Existing semantic suites passed in both profiles.
Offline traces are preserved in evidence/2026-11-01-semantic-comparison.

Next perform the frozen comparison from clean committed source, preserve all
outcomes and report whether fixed rules are equally accurate/faster. Do not
convert development success into general superiority or add infrastructure to
chase a win. Video/rehearsal/submission and contribution eligibility remain
outstanding; November 4 20:00 Guatemala deadline, 18:00 internal target unchanged.


## 2026-11-01 — All frozen semantic comparison outcomes

Ran all 48 frozen trials from source 74cdecd7645f2bf053217ad40da19ff62afaba88,
with 24 real Meta requests, no retries/fallback and no post-result tuning. Both
policies passed 24/24. Each policy produced 14 verified publications in fresh
QEMU sessions (34 actual worker jobs), plus ten appropriate clarification/decline
outcomes without guest work. Rules were faster in every matched case/repetition.
The inference-backed policy demonstrated no accuracy or latency advantage here.

| Case | Rules correct | Meta correct | Rules median total ms | Meta median total ms |
| --- | ---: | ---: | ---: | ---: |
| explicit | 2/2 | 2/2 | 364.0 | 5057.5 |
| historical | 2/2 | 2/2 | 158.0 | 4104.5 |
| latest | 2/2 | 2/2 | 223.0 | 4538.5 |
| ambiguous | 2/2 | 2/2 | 9.0 | 3066.5 |
| unavailable | 2/2 | 2/2 | 9.0 | 2820.0 |
| field | 2/2 | 2/2 | 131.5 | 3509.5 |
| unit | 2/2 | 2/2 | 145.5 | 3607.0 |
| time | 2/2 | 2/2 | 195.5 | 3460.0 |
| conflict | 2/2 | 2/2 | 17.0 | 5551.0 |
| checklist | 2/2 | 2/2 | 110.5 | 2835.0 |
| unknown-unit | 2/2 | 2/2 | 9.0 | 3775.5 |
| no-default | 2/2 | 2/2 | 7.5 | 3309.5 |

Among the 14 selecting/executing trials per policy, median end-to-end time was
138.5 ms for rules and 3778.5 ms for Meta. This subset excludes clarification and
decline, which do not boot QEMU. These medians summarize this same small mixed
set of one-/three-record requests; they are not individual-operation timings.
The first explicit rules trial took 594 ms versus 134 ms on its second repetition;
both are retained rather than dropping a slow startup. See per-trial raw logs.

All 24 Meta calls reported usage: 22074 total tokens, counted once per call.
Available redacted diagnostics and exact selections are preserved. No inference
cost or performance claim is inferred for a different model, hardware, workload
or language. The resolver was engineered on this known domain; neither its results
nor the Meta results estimate held-out generalization. No statistical confidence
or battery/CPU optimization is claimed.

Raw manifest/fixture hashes, all reports, selections and stdout/stderr are under
[evidence/2026-11-01-semantic-comparison/live](evidence/2026-11-01-semantic-comparison/live).
All 48 archived outcomes were re-audited, including exact intent and guest
artifacts, frozen schedule and fixture hashes. The existing replay independently
shows two historical/current publications, unchanged previous bytes and two reused
records with one new update job; that behavior is available to both policies.

Delivery decision: do not pitch this as AI making the OS faster. Present the
working bounded interpretation/admission/publication prototype with measured
limits. Keep ordinary deterministic work local and fast; broader semantic value
remains a hypothesis needing independently authored unseen tasks. A hybrid router
is not implemented or evaluated by this experiment. Prioritize the three-minute
video, clean-checkout reproduction and submission rather than another infrastructure
expansion or modifying benchmarks to manufacture a win.


## 2026-11-02 — Checked portable offline mission, before live probe

Implemented the user-approved useful-artifact pivot after the negative latency
comparison. A fourteen-document fictional project includes owner approval versus
reviewed drafts, compatibility requirements, open human actions, optional content
and one denied synthetic private file. The corpus and exact expected membership,
evidence requirements and two pending actions were authored before live inference.

Rust domain validation restricts identities, exact public IDs, literal quotations
and bounded typed actions. A separate host adapter writes escaped static HTML,
exact source files and a manifest to an unpublished candidate, runs the fixed
independent Python checker and publishes only a passing package. The harness
produces and byte-checks a portable ZIP. Existing output directories are rejected.
No new dependency, unsafe code or kernel change. New documents are handled by the
HOST only; the guest still has its previous small compiled corpus. This is a
one-shot development fixture, not automatic indexing, event recovery or general
filesystem ingestion. The fixed oracle is not a general semantic verifier.

Verification: formatting; 54 Rust tests in each debug/release profile; Clippy
warnings denied in both profiles. Each offline mission suite passed a valid pack,
eleven invalid-plan rejections, overwrite protection, eight corrupted-package
rejections and two dummy-provider cases. The earlier valid pack remained unchanged.
The prior release semantic suite passed six replay cases with real-QEMU trace
checks, independent corruption checks, twenty pre-boot rejections and four offline
provider cases. Logs are in evidence/2026-11-02-offline-mission/offline.

Integrated-browser policy rejected file:// navigation. No alternate surface or
server bypass was attempted. Structural HTML, links, exact source bytes and no
external resources are checked; actual visual/disconnected-browser review remains
pending. No new timing superiority or broad complexity claim is justified.

Next: commit the corpus/code/expected result, then make exactly one live Meta call
with the separate rai_offline_mission schema, 2048 output-token cap and minimal
reasoning. Preserve success or failure without retry/fallback or post-answer tuning.
The earlier 48-trial negative comparison remains unchanged. Protect final video,
clean-checkout rehearsal and submission time before November 4, 20:00 Guatemala
(18:00 internal target); kickoff/build-period eligibility remains unresolved.


## 2026-11-02 — Frozen live offline package result

From clean committed source 303d781e256974ed9de32e7f7e94e70457f60d46, the single
planned real Meta request succeeded. No retry/fallback or post-answer changes.
The model selected nine expected documents, five literal release/compatibility
quotes and both unresolved human actions. The denied file was not provided to the
model or included in output. Newer unapproved/incompatible and optional sources
were excluded. All source assertions remain fictional development data.

The independent checker passed 8620 source bytes and a static reader with zero
external resources. Provider latency 3558 ms; full plan/check/package duration
4379 ms excluding build. Usage 2863 prompt + 769 completion = 3632 tokens (the
provider diagnostic and mission_response describe the same single call). No speed,
energy, broad reasoning superiority or unseen-task accuracy claim follows.

Complete live logs, plan, manifest, exact HTML/source package and portable ZIP
are preserved under evidence/2026-11-02-offline-mission/live. The archive was
rechecked against frozen fixture hashes, file bytes, ZIP contents and same-call
metadata. Source commit was already pushed; this follow-up records results only.
Host execution remains explicit; no new QEMU document execution is claimed.
Browser visual/disconnected review remains pending following the file-URL policy
rejection. The package is one-shot, not an automatic semantic update service.


## 2026-11-02 — Fresh semantic events in one retained guest, before measurement

Returned to the user's decision-quality hypothesis and actual OS execution. Added
a bounded Rust approval stream sharing the existing recorded-sequence execution
path. The guest boots once and retains EL0 coordinator/worker state, cached exact
indexes and checked publications while a host driver obtains a NEW interpretation
for each successive request. A host ready/response handshake delivers only the
current event; decisions are not all preloaded. Clarification or independent
intent rejection does not dispatch guest work. The existing replay remains intact.

Frozen protocol: three already known development goals, historical -> ambiguous
-> explicitly current; rules and Meta each run one identical session. Three real
Meta calls maximum, unchanged schema/settings, no retry/fallback or post-result
tuning. Expected result: two publications, four total worker jobs, two reused
indexes on update, and unchanged previous bytes until replacement. These cases
verify continuity; they do not establish unseen reasoning or decision superiority.
No guest corpus expansion, kernel changes, file watching or concurrent semantic
events. New goal/clarification text stays on the host; the native screen reflects
the last accepted guest target until a selected new target is admitted.

Verification: 55 host tests and warnings-denied Clippy in debug/release. New real
QEMU suites passed in both profiles: replay/rules, five corrupt evidence traces,
eight invalid input suffixes and two fake-provider sessions. Wrong meaning in the
middle event was rejected and the earlier output remained unchanged. Strict stop
parsing initially accepted an extra field; a strict struct variant fixed it before
passing tests. Native-display release replay/rules passed. Previous replay passed
in debug; previous comparison verification passed in release (12 deterministic
cases, guest execution and corruption checks). Final audit rechecked archived
traces. Evidence: evidence/2026-11-02-semantic-session/offline.

Next commit the source/protocol, run the frozen live comparison, retain all three
Meta outcomes and both policy traces. Prior negative results remain unchanged.
November 4 final video/rehearsal/submission and contribution-eligibility follow-up
remain outstanding. No model latency, accuracy or system efficiency win is assumed.


## 2026-11-02 — Live retained-session results

Frozen clean source 948ae1926d31c1ab80f294cc37482dfa68c2cf98 ran rules then Meta,
three sequential known events per session. Exactly three real requests, no
retries/fallback or answer tuning. Each policy passed 3/3 decisions, with two
verified guest publications, four total EL0 worker jobs and two cache reuses on
update. Previous bytes remained unchanged until replacement. The ambiguous event
caused a clarification and zero guest work. Meta explicitly asked whether Pump A
(notes@1) or Pump B (notes@2) was intended; the controlled next event requested the
current handoff. Events were delivered after ready acknowledgments, not loaded as
precomputed selections. Host interpretation and intent checks remain explicit.

Whole-session time: rules 661 ms; Meta 7121 ms, excluding compilation. Three Meta
interpretation processes: 2805, 2229, 1920 ms; reported total usage 2910 tokens,
counted once per request. Both methods have the same correct results and guest
work here. No decision-quality or speed advantage is demonstrated. Native-display
execution was separately tested offline; this live measurement was headless.

All events, exact input streams, provider diagnostics, plans, guest artifacts,
manifest and summary are preserved in evidence/2026-11-02-semantic-session/live.
Archived raw traces, fixture hashes and same-call metadata were re-audited.
The next decision-quality study must use a separately specified meaningful task
and reserved evaluation cases; another repetition of known tiny cases cannot
establish the user's improvement hypothesis. Existing negative evidence remains
unchanged. Final rehearsal/video/submission are still outstanding.


## 2026-11-02 — Constant goal and structured context, before live probe

Clarified to the user that prior events were scripted written requests, not
spontaneous OS observation. The next increment retains one identical goal and
separates structured assignment observations from user intent. The host delivers
three fictional reports at ready boundaries: authoritative A assignment despite
newer advisory B; a same-revision dispatch conflict; later dispatch B resolution
despite old conflicting/advisory reports. Request version correlates with the
observation version, while the goal text stays byte-identical. No new user prompt
is produced in response to the clarification; the third observation resolves data.

Added a pure bounded Rust context policy and adapter, plus a separate Meta schema/
prompt. The competent rules baseline selects the highest dispatch revision,
checks conflicts and delegates content selection/dependencies to the existing
resolver. Both use the same trusted intent gate and actual retained guest worker/
cache/publication. Text goal/context remain host-side; no claim that kernel logic
interprets reports. Source labels are trusted fixture data, not authenticated
reports. No kernel/corpus expansion, new unsafe/dependencies, file watcher or
concurrent semantic event processing. Native display still shows last accepted
guest target, with new question/context text in host logs.

Verified 57 host unit tests and warnings-denied Clippy in debug/release. Both
context suites passed real QEMU replay/rules, seven invalid contexts before API
access, altered-context evidence rejection, five lifecycle corruptions, eight
invalid stream suffixes and two fake-provider sessions. Exact goal/current
observation are checked by fake transport; wrong meaning preserves old output.
Native-display replay/rules and the old semantic-session suite passed in release.
The untouched prior live archive still passes the shared auditor. Offline raw
files/logs are preserved in evidence/2026-11-02-context-session.

Next freeze source, policy and exact expected outputs, then run rules and three
real Meta evaluations without retries/fallback or post-answer tuning. This is an
input-boundary integration check on known development observations, not an unseen
complexity or decision-quality win. Keep all earlier results and protect the
November 4 rehearsal/video/submission deadline and provenance requirements.


## 2026-11-02 — Live constant-goal context results (Guatemala date)

Clean source d69c631de11e66946d575d3f5b745ff197b2ee69 produced three new Meta
responses while the same user objective remained byte-identical. The recorded
UTC start was 2026-11-03T04:47:03.792016+00:00, still November 2 locally. No
retry/fallback or post-answer tuning. Only structured observations changed.

Both policies passed 3/3: authoritative A assignment, clarification for conflicting
latest reports, and B after a newer resolving dispatch report. Both produced two
checked guest publications and four total worker jobs, reusing two exact indexes
on update and retaining earlier bytes. The ambiguous context dispatched no work.
No user clarification prompt was generated; the controlled third data observation
resolved the conflict. Host context delivery is simulated; guest work is real.

Whole-session time excluding build: rules 581 ms, Meta 11541 ms. Meta interpreter
processes took 3993/3452/3917 ms, with 2881 total tokens across exactly three calls.
The decision-improvement hypothesis remains unproven; both policies were equally
correct and the rules were faster. No inference-efficiency or OS superiority claim.

All raw observations, unchanged goals, provider/guest traces, manifest and summary
are archived in evidence/2026-11-02-context-session/live. Archive audit passed for
fixture hashes, fixed objective, current context, guest artifacts and same-call
usage. This live probe was headless; native display had separate offline checks.
Final video/rehearsal/submission remain pending; preserve original deadline and
contribution-eligibility limitations.


## 2026-11-02 — Portable QEMU runtime and writable guest files (Guatemala date)

The user requested independence from the Mac host and a deeper useful operation
such as file management. Added a Linux container recipe that builds the same
AArch64 guest and a native Linux Rust bridge. QEMU uses TCG and a versioned virt
board; native graphics select the host backend, while containers expose guest
framebuffer pixels over VNC. Windows uses Docker Desktop Linux containers; a
physical Windows host has not been tested. Git attributes retain LF build inputs.
Source and binary hashes accompany image verification. The allowlisted build
context excludes .env files, key suffixes, targets and Git metadata. No live model
calls, credential changes, external publication of an image, or phone flashing.

Added an allocator-free, pure Rust guest RAM store owned by the console. Eight
512-byte file slots accept runtime uploads through bounded hex chunks. Complete
uploads publish only after ticket, size, generation and destination checks.
Read/list/copy/move operate on guest bytes and render the native Files view. No
new unsafe, dependencies, overwrite/delete, directory objects, persistent disk,
file permissions, isolated service or AI file planning. Individual operations are
sequentially atomic; multi-action transaction semantics remain future work.

The first Linux image exposed missing ramfb ROM firmware and the verifier's ASCII
error decoder. Added Debian seabios and UTF-8 diagnostic decoding; boot assertions
remain strict. A subsequent build correctly failed when source metadata requested
a non-copied .dockerignore; the context now explicitly includes both ignore and
attribute files. Debug coordinator regression exposed insufficient stack headroom
with the new owned store. Separating coordinator/system dispatch temporaries fixed
the reproduced fault without increasing the 64 KiB stack. Frame sizes and the
failed test trace are retained; whole-program bounds and guard pages are pending.

Verified pure file-store tests, formatting, warnings-denied Clippy, real-QEMU UART
and file checks in debug/release, context replay/rules and rejection suites in both
profiles, native debug context continuity, native file rendering, and debug
scheduler register-corruption/user-fault diagnostics. File verification sends 76
commands, checks exact 512-byte binary reads, unchanged originals, invalid/stale
operations, capacity and empty state after reboot. Linux offline verification also
runs the prior rules context lifecycle. VNC's local protocol greeting was checked;
this does not prove a Windows viewer installation. Platform manifests, exact
results and limitations are in evidence/2026-11-02-portable-files.

Next implement bounded file-action plans: preview the entire proposal, validate
versions/destinations/capacity, preserve originals, then publish all-or-nothing.
Only then connect Meta to this new operation surface and evaluate requested
outcome quality, ambiguity and unintended changes against a competent baseline.
This increment establishes execution capability, not an AI speed/quality win.
Preserve previous negative comparisons, actual contribution dates, the unresolved
Oct 26/27 eligibility discrepancy and Nov 4 20:00 Guatemala deadline.

Linux ARM64 and emulated Linux AMD64 both passed the offline retained-context and
file/reboot checks. A core.autocrlf=true staged checkout initially changed bundled
font notice/license line endings; scoped attributes now preserve all 254 included
input files byte-for-byte. This checkout simulation is not a Windows host test.


## 2026-11-02–03 — Native prompt workspace, before live file-plan smoke test

The user requested a more visual file manager and a prompt-operated OS experience.
Implemented compose/wait/review/result/clarification states drawn by the guest,
with actual keyboard and bottom-button input through a portable RFB-to-UART
adapter. Four fictional notes are imported at runtime. The same request/catalog
can go to a strict Rust Meta file planner or explicitly labeled offline replay.
One model call proposes up to three independent copy/move actions; the user reviews
and confirms. Request IDs reject cancelled/late answers. The guest validates the
whole plan before review and again before writes, including source/destination,
capacity, pending-upload and generation checks. No partial plan is applied. One
successful plan increments the generation once. No new unsafe or dependencies.

The interface lives in the guest; input transport and HTTPS remain on the host.
The new UI/file modules share EL1 privilege, not separate processes. This is still
an eight-file, 512-byte-per-file RAM workspace with ASCII input, not persistent
storage, a full file browser, a physical-phone build or autonomous agents.
Original-preservation intent is reviewed semantically; the kernel guarantees the
listed operations and structural checks, not natural-language understanding.

The opt-level=0 native debug image exceeded the linker-guarded initial RAM window.
Set development optimization to level 1 while retaining debug symbols, the same
map/page pool/64-KiB stack and the assertion. Saved the original failure. Both
workspace profiles then passed real QEMU RFB frame/key/button, preview-before-write,
exact-result, invalid-last-action, cancellation and stale-generation checks.
Clippy, 58 host tests, three RAM-store/plan tests, old context suites in both profiles,
file regression, and debug scheduling/MMU/UART checks passed. No full stack bound
or new performance claim follows from these checks.

Manual macOS Screen Sharing testing first exposed RFB 3.3 negotiation and its
password requirement. Added standard 3.3/3.8 negotiation and optional authentication
delegated to QEMU through a private Unix socket/QMP. Verified typed prompt, replay
review, confirmation and result paging in that real viewer. The local test password
was not saved in the repository or Keychain. Keep published ports on loopback;
legacy VNC authentication does not encrypt the connection. Windows host testing
remains pending. Portable test outcomes/source hashes are archived separately.

Next freeze the source and make one explicit live Meta call on the disclosed
approved-Aurora development case. Independently check the proposed set before
confirming, then exact guest bytes afterward. Preserve failed results with no
retry or prompt tuning. This proves an integration path, not an unseen semantic
advantage over a competent baseline. Protect video/rehearsal/submission time and
the existing November 4 20:00 Guatemala deadline and contribution-date caveats.


## 2026-11-03 — One live Meta file plan and confirmed guest result

Frozen source a330bd305f4707a2b1eef69775b380db2d6d70a6 produced one new Meta
file-plan response for the disclosed “Collect approved Aurora notes in /work.
Keep originals.” case. HTTP 200; provider latency 3093 ms; reported usage 817
tokens (526 prompt, 291 completion). No retry, fallback or post-response tuning.
Meta proposed copying /inbox/aurora.txt and /inbox/field.txt into /work, excluding
the draft and personal list. The reason and exact actions appeared on the guest
review screen. The independent check verified that preview left generation 5 and
all original bytes unchanged, validated both selected sources/destination scope,
then explicitly confirmed. The guest committed one generation transition to 6;
returned copy bytes and all four originals matched exactly.

The live smoke test used the same guest UI/bridge but injected keys through its
UART adapter, not a human viewer. RFB and native macOS viewer interaction were
separately tested offline. Live request, provider diagnostic/proposal, UART,
compose/thinking/review/result captures and binary hashes are archived in
evidence/2026-11-03-prompt-workspace/live. This is an end-to-end integration result
on a known development example, not an unseen benchmark or an AI speed/quality
advantage. The model still takes seconds; one request proposes the complete plan.
No persistent storage, new isolation or physical Windows/phone claim is added.

## 2026-11-03 — Dark guest UI and declarative microapps

Replaced the prompt workspace's light rounded treatment with a black Ubuntu Mono
interface, thin dividers and restrained lime accents. Added a native guest app
library, prompt/review flow, forms, record lists and pending filter. Font source,
license and generated coverage are included; runtime needs no font engine.

The pure bounded runtime supports three apps, six typed fields and eight rows
per app. A compatible update preserves existing field identity/order/type and
exact row values, with checked defaults for added columns. Only records.local
exists; no generated code, location, file/network access or arbitrary execution.
This is interpreter namespace separation at shared EL1, NOT process isolation.
The host Meta adapter sends prompt/schema/count metadata, not row contents, and
returns a strict proposal. Installation requires guest checks and Enter.

Actual validation: 61 host tests passed (including shared runtime tests);
Clippy passed for guest with/without display and host provider. Debug and release
real-QEMU app tests passed: RFB pixels/keyboard/pointer approval, required values,
two saved rows, pending filter, additive priority migration, independent second
app, invalid numeric edit, denied capability, stale/cancelled requests, malformed
hex/kind, duplicate fields, destructive migration and cross-app replacement.
Existing release prompt workspace tests passed with the dark renderer; native
guest context cycles passed debug/release, and debug scheduler/MMU fault checks
passed. The app lifecycle also passed offline in the Linux ARM64 container.
Evidence is in evidence/2026-11-03-microapps/offline. AMD64 and live generation
results will be recorded separately; none are inferred from offline replay.

The executable and UI remain bounded RAM-only prototypes with host input/HTTPS.
No physical Windows/phone, persistence, full isolation, arbitrary generated
applications or AI performance advantage is claimed. Next freeze this source and
run up to two disclosed live Meta calls: create an inspection tool, use it, then
add priority while retaining its records. Keep Nov 4 submission/video time intact.

## 2026-11-03 — Two live Meta microapp proposals

Frozen source 2996d516ed8099448908b6441a08d81f5efcd809 completed the disclosed
create/use/evolve smoke test with exactly two Meta requests, no retry/fallback or
post-response tuning. Meta generated Inspection Log (equipment, notes, done and
pending filter), then appended a required numeric priority defaulting to 1.
Before each confirmation the verifier checked the requested fields/compatibility
and that preview had changed no app or row. The real guest stored Pump A / Check
seal / false and Pump B / Ready / true; after migration both rows retained their
original bytes and gained priority 1. Local form, list and pending interactions
made no further model call.

Both provider responses were HTTP 200 with the configured model matching:
creation 4234 ms / 1077 reported tokens, evolution 2381 ms / 1028 tokens. These
are observed provider latencies, not app execution timings or AI speed gains.
The independent smoke check used scripted UART key events; actual RFB pixel,
keyboard and pointer transport was separately verified offline. Exact requests,
proposals, diagnostics, UART, framebuffer captures, source/binary hashes and
report are archived in evidence/2026-11-03-microapps/live. No credential was
included. This demonstrates bounded generative tool creation/update on known
development examples, not arbitrary software generation or unseen reliability.

The same offline app lifecycle additionally passed in a Linux AMD64 Docker image
under emulation on this ARM64 Mac. Both container runs disabled networking and
exercised actual guest RFB pixels/keys/approval, records/migration and rejection.
Image IDs, build source digests, binary hashes and runtime versions are saved in
evidence/2026-11-03-microapps/offline/portable.json, with platform test logs. This
checks both Linux host architectures, not a physical Windows machine or phone.

## 2026-11-03 — App launcher, search and guest-owned touch targets

Replaced verbose app screens with a phone-shaped app grid, deterministic line-art
icons, a create tile and persistent Home/Create/Find navigation. Prompt/form
boundaries use segmented horizontal lines, verticals and diagonal corners. Search
matches app titles/IDs case-insensitively; pointer coordinates are interpreted by
the guest controller, including icons, fields, checkbox toggles and record rows.
No fake device telemetry or claim of native-code compilation was added.

Real-QEMU release app checks passed, including search and direct pointer actions
against actual records in separate apps. The RFB test now forwards coordinates
to guest hit testing; it still verifies pixels, typed prompts and click approval.
Clippy passed with the native display. No new live Meta call was required for
this presentation/navigation change; the previous live evidence is preserved.
Debug app verification and existing release workspace verification also passed.
Launcher captures and reports are in evidence/2026-11-03-launcher.

## 2026-11-03 — Save and restore app sessions

Added explicit host checkpoints for confirmed app schemas and records, with
versioned/checksummed JSON, shared Rust validation, exclusive writer lock and
flush/atomic replacement. The native launcher defaults to target/device.json;
Docker uses /state/device.json with a documented persistent volume. A fresh
QEMU validates and reconstructs saved apps without any model call, then reads
back exact schemas/rows before input starts. UI status distinguishes saved,
restored, RAM only and save failed. No guest disk driver or MMU isolation is implied.

Debug real-QEMU restart tests passed: records and schema evolution survive fresh
boots; an injected replacement failure leaves the previous bytes unchanged;
retry saves successfully; a second writer is denied; malformed values, checksum
corruption and a matching-checksum unavailable capability are rejected. Rust
snapshot validation tests passed. Release/crash checks are recorded below after
completion. Host files remain private, ignored by Git; sample evidence uses only
fictional records.
Release restart/crash verification and the release RFB/app lifecycle also passed.
Headless guest and host Clippy passed. Captures and exact test reports are in
evidence/2026-11-03-app-storage. The simulated failure is disclosed in those reports;
no live API request was made for these checks.

## 2026-11-03 — Local app tables, totals and model-facing capability catalog

Added guest table views with three-column pages, row editing and totals over the
selected all/pending scope. Numeric cells remain i32; sums accumulate in i64, so
eight valid values cannot overflow. Optional blanks are omitted. The guest-owned
pointer path and F5 open tables; columns paginate without changing data. A versioned
JSON runtime catalog now accompanies app requests to Meta, describing actual
operations and explicitly unavailable features. APP_CAPABILITIES.md separates
implemented functions from calculated fields, file exchange, optional in-app AI
and native-code execution that remain future work.

Debug real-QEMU expense-table checks passed on an explicit offline schema: sums
1800/1050 before editing, 1900/1150 after editing, exact row retention after reboot,
and no inference during arithmetic or restore. The domain test covered eight
maximum i32 values, optional blanks and the filter. Source HEAD in pre-commit
reports identifies the baseline; working-tree changes supplied the new table code.
No live Meta result for this new prompt is claimed before the frozen-source run.
Release table/recovery checks and the release RFB app regression also passed,
including the second column page after schema evolution. Guest display/headless
and host Clippy passed; the shared-domain and capability-catalog tests passed.
Offline table captures/reports are in evidence/2026-11-03-app-tables/offline.
The source-note explicitly identifies their pre-commit provenance.

## 2026-11-03 — Live expense app and multi-app restored showcase

Frozen source d99eda552a46b0b2928625800dcc250c448f3ac5 generated Expense Tracker
with one live Meta call: HTTP 200, 3903 ms provider latency, 1312 reported tokens,
no retry. The independent check confirmed the requested types and cents unit
before approval, then exact records, all/unpaid totals, edits and recovery in a
fresh guest with zero further inference. Evidence is in
 evidence/2026-11-03-app-tables/live (requests, diagnostics, plan, UART and guest
captures). The source checkout was clean before the request.

A public showcase checkpoint combines that accepted schema/data with the prior
live Inspection Log schema/data. Both were restored together into QEMU without
inference, giving a usable app-grid demonstration. Captures explicitly show replay
and restored status. This does not claim new generation during restoration or
arbitrary code compilation. APP_CAPABILITIES.md documents a non-overwriting copy
into target/ and the launch command. Linux ARM64 container storage/table checks
also passed offline; the failure line in the storage log is an injected test.
Linux AMD64 emulation also passed both storage and table verification with network
access disabled. Both platform logs and image/build hashes are archived under
evidence/2026-11-03-app-tables/portable. Those images contain the frozen executable
code and were built before the final documentation/evidence/showcase fixture.
Physical Windows and phone hardware remain untested.

## 2026-11-03 — Declarative guest display control

Added display.brightness as a separate immutable app capability. A control schema
contains exactly one required number with a default in 20..100 and no filter;
records apps retain their behavior. User review displays the permission, and Apply
checks the installed capability and device availability. Only then does the guest
adapter attenuate RGB output. F6 and the reset button recover 100%. Installation,
updates and checkpoint restore never actuate; applied brightness is session-local.
No physical monitor/phone backlight or power-saving behavior is claimed.

Debug real-QEMU checks passed: exact sample-bar values at 50% and 20%, actual home
pixels changed, repeated redraws stayed identical, and reset returned the exact
original home image. Maximum per-channel difference from a halved baseline was
one, consistent with integer alpha rounding. Invalid ranges, records-only authority,
capability escalation, invalid defaults and late replies were rejected. Recovery
restored the control schema but kept brightness at 100%. The actual headless build
rejected actions with display_unavailable. Shared domain/host contract tests and
Clippy with/without display and on the host passed. Further results follow below.
Release brightness verification also passed. Existing release app/RFB and saved-
session tests passed, as did the debug file-workspace/RFB regression. Captures,
exact measurements and reports are archived under
 evidence/2026-11-03-display-control/offline. Pre-commit reports explicitly mark
the working tree as dirty. The next step freezes these sources before one live
Meta control-app request; no live result is inferred from replay.

## 2026-11-03 — Live Meta brightness app verified against actual pixels

Clean frozen source b2e5c9a0cf968ea577767d2e5b73bebe59535084 made exactly one
live Meta request. HTTP 200; matching configured model; 2768 ms provider latency;
1382 reported tokens. It returned Brightness Control with display.brightness,
one required numeric input and default 50. No retry, fallback or post-response
prompt tuning. The independent check approved the shape before installation,
then verified 50%/20% framebuffer output, unchanged repeated redraws, exact reset,
permission and range rejection, and fresh-guest restore at 100% without inference.

The screenshot/report evidence is under evidence/2026-11-03-display-control/live.
Keys/pointer events were injected through the guest protocol; RFB behavior was
separately regression-tested. The records fixture for denied-authority checks is
explicitly scripted, not claimed as another generated app. The live success is a
known development example, not a general reliability or physical hardware claim.
The Linux ARM64 container also passed the complete offline brightness check with
network access disabled. The AMD64 result follows once that run completes.
The Linux AMD64 emulated container also passed the complete offline check with
network access disabled. Both platform logs and image/source/binary hashes are in
evidence/2026-11-03-display-control/portable. These results do not establish a
physical Windows or phone test. Executable sources match frozen b2e5c9a; final
changes only archive evidence and documentation.

## 2026-11-03 — Per-app formulas without hardcoded app behavior

Added calculated columns to the shared declarative runtime. Formula text belongs
to each installed schema; no quote/budget app identity or prompt is recognized by
the kernel. The bounded Rust compiler accepts earlier numeric-field references,
integer arithmetic and parentheses. It rejects unknown/cyclic dependencies,
unsupported syntax and excessive depth/size. Checked evaluation rejects missing
inputs, overflow and division by zero. Full-record saves and schema migrations
validate before mutation; updates retain editable bytes and recompute derived
values. Computed fields are read-only, preview locally, appear in totals and show
the full expression during review. Snapshot validation recomputes saved results;
fresh-guest restore sends inputs only and checks exact readback.

Moved existing disclosed offline examples into scripts/app_replay.py, imported
only in replay mode. Live generation already accepted arbitrary prompts; this
change makes that separation explicit rather than claiming it previously used
fixed examples. The versioned runtime catalog and strict Meta proposal schema now
advertise calculated fields. No unsafe, new process boundary or arbitrary native
code generation was introduced. Host HTTPS/storage dependencies remain explicit.

Verified: 69 shared domain/host tests; formatting and Clippy for the kernel with
and without guest display and for the host. Real QEMU debug and release checks
passed for two declared arithmetic apps, edits, totals, invalid access, overflow,
failed migration on a later row, successful formula replacement and reboot.
Existing release app/RFB, saved-session and brightness regressions passed. Linux
ARM64 Docker passed the formula check with network disabled; physical Windows and
phones were not tested. Reports, selected native captures and portable hashes are
under evidence/2026-11-03-app-formulas/offline and portable. Pre-commit reports
honestly identify the modified worktree; these are offline results, not Meta
success. Freeze these sources before the separate two-call live check; no retries
or benchmark/AI-performance claims are authorized by these observations.

## 2026-11-03 — Two live generated arithmetic apps verified

Frozen clean source d20f98f08c794e309321ecdf8c812cd2b0235bd9 made exactly two
Meta requests, without retry, fallback, prompt tuning or source changes. Both
returned HTTP 200 and the configured model matched. Quote used `$qty * $unitc`
(5605 ms provider latency, 1791 reported tokens); Travel Budget used
`$budget-$spent` (7023 ms, 1983 tokens). These prompts are absent from the replay
catalog. The guest installed the actual returned schemas after review and passed
independent expected results, edits, read-only enforcement and numeric totals.
Fresh QEMU recovered the exact generated schemas/rows with no model calls.

Reports, complete sanitized provider diagnostics/proposals, fictional saved state
and actual native framebuffer captures are under
[evidence/2026-11-03-app-formulas/live](evidence/2026-11-03-app-formulas/live).
This confirms these two development cases; it is not a general success-rate,
novelty, isolation or runtime-speed claim. The final evidence commit changes only
documentation/artifacts; executable sources remain the frozen tested version.

## 2026-11-03 — Reviewed CSV exchange and complete demo rehearsal

Added generic user-mediated CSV import/export for records.local apps. The native
import/export toolbar and F7/F8 request a transfer through an explicitly selected
host inbox/outbox. Import stages a bounded batch, validates all columns/rows and
formula results, previews each incoming row, and appends only after native user
approval. A whole batch increments the revision once; errors/cancellation leave
all prior rows untouched. Export includes all saved inputs/derived results and
publishes a complete new file without overwriting a destination. The host owns
CSV/file I/O; Meta sees no file contents/paths and gets no new file capability.
No unsafe, guest disk driver, process isolation or arbitrary native app execution
was added. The existing checkpoint mechanism persists approved imports.

Verification passed: 70 shared domain/host tests, formatting and Clippy with and
without guest display and on the host; real QEMU debug/release exchange checks;
release existing app/RFB and formula regressions; Linux ARM64 Docker exchange
with network disabled. Checks covered quoted/reordered columns, native preview
and cancel, all-or-nothing failure on a later row, forged calculated values,
duplicate/incomplete/late frames, capacity limits, no-overwrite export, symlink
input rejection and fresh-guest restoration. No physical Windows/phone test.
Evidence is under evidence/2026-11-03-app-exchange/offline and portable.

Added DEMO_JOURNEY.md and a separate create/import/evolve/export/reboot verifier.
The disclosed offline rehearsal passed: two input rows total 535 cents; adding
cost/profit fields preserves them; editing one cost yields profits 225 and 60,
285 cents total, preserved in CSV and after reboot. This offline run uses example
schemas and is not live Meta evidence. Freeze sources before the capped two-call
live rehearsal. The planned three-minute video is documented, not yet recorded.

## 2026-11-03 — Live create/import/evolve/export/reboot journey passed

Frozen clean source 0de2acf7c239dc3fea6221be3b46fd930b641579 made exactly two
live Meta calls. Create returned Quote App with totalc = qty * unitc (HTTP 200,
4682 ms provider latency, 1965 reported tokens). After the guest imported the two
fictional CSV rows, the update added cost_cents and profit = totalc - cost_cents
(HTTP 200, 4995 ms, 2245 tokens). Model identity matched both times. No retry,
fallback, prompt adjustment or source modification occurred between calls.

The independent verifier checked no installation/import before approval, unchanged
original fields/data after evolution, local cost editing, expected profits 225 and
60 cents, total 285, exact six-column CSV export and fresh-QEMU recovery with no
further inference. The generated key totalc differed from the earlier development
example; headers and the update request were derived from the actual installed
schema rather than a hardcoded runtime app. Full proposals, diagnostics, native
captures, CSVs and fictional saved state are under
[evidence/2026-11-03-app-exchange/live](evidence/2026-11-03-app-exchange/live).

This is one successful declared development journey, not a general reliability,
performance or novelty claim. It validates the demonstration workflow; the actual
three-minute video and submission remain outstanding. This final evidence commit
changes documentation/artifacts only; executable sources remain the tested freeze.

## 2026-11-03 — Generic guarded processes and workshop capability proof

The user asked for a more substantial example before polish and selected a repair
workshop. Added optional per-record workflows with up to four user-triggered
actions. Each action checks revision, source state and an optional numeric guard
before one state change. Terminal states reject normal editing. Model-generated
graphs must be reachable and use existing numeric fields; existing workflow
schemas are immutable on update. Fields/formulas and the full process are reviewed
on two screens before installation. States appear in the native list/actions view.
No workshop-specific logic was added to the kernel/runtime or live bridge.

State is separate from editable fields. Snapshot validation requires known states
for every row; a one-time fresh-device restore gate closes before user input.
The host and saved-file owner remain trusted; recovery is not an action-history
proof. CSV covers field data only and imported orders start a new process. App
runtime/validation remains shared EL1, not hardware-isolated services. Limits
remain three apps, six fields, eight rows. No new unsafe or hardware capability.

The first debug install failed with a fatal exception. Inspection found a 55856-
byte retained console frame competing with handlers on the fixed 64 KiB stack.
Moving construction into a non-inlined helper reduced the retained loop frame to
32272 bytes. The stack reservation and MMU mapping window were not enlarged.
Improved verifier/close log retention for failures. Initial failure observations
and compiler frame data are preserved in evidence/2026-11-03-app-workflows/offline.
Individual frame measurements are not a full worst-case call-stack proof.

Verified 72 host/shared-domain tests; formatting and kernel/host Clippy; real QEMU
debug/release workshop execution with native action-card clicks; eight imported
orders, denied out-of-order/unpaid/stale/repeated actions, locked final records,
exact CSV fields and reboot restoration of every state. Existing release app/RFB,
storage and exchange checks passed. Console/protocol passed 37 checks in each
profile; physical-memory bookkeeping/zeroing/reuse checks passed in debug. Normal
EL0 coordinator regression passed 140 requests per profile, including task and
isolation recovery; fault-injection variants were not rerun. Linux ARM64 Docker
passed the workshop check with network disabled; no physical Windows/phone test.

These workshop checks used disclosed verifier schemas, not Meta generation. The
next frozen-source check permits one live request with no retry or fallback.
APP_WORKFLOWS.md documents usage/limits; KERNEL_DECISION.md answers the own-kernel
tradeoff: direct systems ownership and learning at substantial infrastructure cost,
without claiming that this app experience requires our kernel or is faster.

## 2026-11-03 — Live Meta workshop process verified with eight orders

One live request from clean frozen source
64250c336081b297deed17483a833794514e4e5a returned Repair Shop with five fields,
balance = quote - paid, and received -> inprog -> done -> delivered. Delivery had
a balance <= 0 guard. HTTP 200; configured model matched; 5518 ms provider latency;
2749 reported tokens. No retry, fallback, post-response prompt adjustment or source
change. The generated intermediate state names differed from the offline example;
the verifier followed returned state/action/field identities.

The actual returned schema and process were reviewed before installation. Real
QEMU imported eight fictional orders, exercised successful actions via native
card taps, rejected out-of-order/unpaid/stale/repeated transitions and terminal
edits, and checked that runtime restoration cannot bypass normal action handling.
Recording payment enabled delivery. Final states were two delivered, one done,
one inprog and four received. CSV field data matched exactly, and a fresh QEMU
instance recovered all rows and states without inference. This used one model
call total; subsequent execution was local. Evidence, diagnostics, generated
schema, fictional saved state and native captures are in
[evidence/2026-11-03-app-workflows/live](evidence/2026-11-03-app-workflows/live).

This is a successful declared development example, not production readiness,
large data scale, general model reliability, verified real-world payment or
physical phone support. The app permits an empty paid input in its field metadata,
but the balance formula requires a numeric value: enter 0 for no payment. No
silent default or schema repair was applied to the live result. The final evidence
commit changes only documentation/artifacts; executable sources remain frozen.


## 2026-11-03 — Runnable demo kit, explicit app limits and video rehearsal

The user asked what limits app creation despite owning the kernel and requested
a compelling demo. APP_LIMITS.md distinguishes source ownership, the model's
declarative language and implemented runtime/device capabilities. It orders
capacity/stable record IDs, related data, richer presentation, isolated services
and real hardware capabilities rather than claiming an unbounded complete OS.
No guest limit, kernel mechanism or provider behavior changed this increment.

Added demo.py saved/live/resume/verify: a new output folder prevents accidental
reset, resume preserves data and creates a separate log, live starts empty, and
saved combines the unmodified actual Meta workshop and evolved quote apps with
fictional data and source hashes. Docker exposes demo/demo-verify. README now
leads to DEMO_READY.md, a runnable tour and English three-minute script, while
preserving earlier experiments and measurements. Saved mode says replay; creation
in that mode remains the disclosed offline examples, not arbitrary inference.

Verified real QEMU debug/release: both apps restored exactly, native pointer/key
input blocks Phone C delivery while balance is 4000, recording paid=5000 allows
delivery, a quote cost edit changes total profit 285 -> 260 without changing the
workshop, CSV matches, and fresh QEMU recovers both apps and process states. The
fixture stays unchanged and existing output directories cannot be reset. Actual
CLI saved/resume/live RFB handshakes passed, with byte-identical resumed state,
empty initial live state and clean shutdown. Zero Meta calls this increment.
Linux ARM64 Docker with network disabled also passed the kit verification; this
is not physical Windows or phone validation. No Rust logic changed, so the prior
domain/Clippy results were not represented as new runs.

Rendered and visually inspected a 1920x1080 silent preview from real QEMU still
captures, with explicit historical-live versus saved-rehearsal labels. FFprobe
confirmed 180 seconds and one video stream, no audio. Source frames and hashes
are recorded; the renderer and narration/scene manifest are reproducible. This
is a storyboard preview, not an uninterrupted interaction recording or submitted
final video. Final narration, recording review, accessible video, eligibility
clarification and submission confirmation remain outstanding. Evidence is under
evidence/2026-11-03-demo-kit. Internal Nov 4 18:00 Guatemala target remains.


## 2026-11-03 — Compose behavior from reusable app capabilities

The user prioritized general OS/app-runtime abilities over additional example
apps. Added a bounded typed program language: set, calculate, toggle, require
and forward-only if; two user buttons with up to six steps each. Programs act
on one saved row. Numeric expressions can reference any current numeric/derived
field; derived values refresh after each write. Every branch is checked at schema
admission, even if current data would skip it. A preview evaluates a staged copy
without changing records/revision; matching confirmation rechecks revision and
commits exactly one final row. Any failure preserves every original value.
Terminal workflow locks remain enforced, and reviewed compatible schema updates
may replace behavior definitions while retaining field/data migration rules.

The guest shows every program during installation and before/after values before
execution. Structured runtime failures identify the failing step; host proposal
validation emits a machine-readable rejection code. These are deterministic
checks, not general verification of natural-language intent or an automatic model
repair loop. Meta receives runtime catalog v6 plus a strict program schema, with
no record rows sent. Proposal transport and checkpoint recovery preserve full
program definitions. Older apps/saved sessions remain compatible.

No new kernel mechanism or unsafe was added. Growing definitions exposed a debug
startup stack overlap despite the first QEMU run passing: known retained plus
constructor/caller frames totaled 75904 bytes against the fixed 65536-byte stack.
Explicit const construction removes aggregate runtime temporaries, reducing the
constructor from 35808 to 3520 bytes and known startup sum to 43616. Known action
frames total 58464 before parser/indirect callees; no universal stack bound is
claimed. Kernel boot/MMU/IRQ/IPC foundations remain, and apps still share EL1.

Verified 83 host/shared-domain tests, kernel Clippy with/without display, host
Clippy and formatting. Final real QEMU debug/release checks cover three-row
independent arithmetic, both branch outcomes, text/check changes, preview without
mutation, cancellation/stale/direct-run rejection, late condition/overflow
rollback, invalid skipped instructions, reviewed updates, a separate app, terminal
workflow locks and exact reboot recovery. Existing app/RFB, storage corruption/
recovery, brightness and saved-demo checks passed. Console/protocol passed 37
checks per profile; physical-memory checks and normal EL0 coordinator 140 requests
per profile passed. Fault-injection variants were not rerun. Linux ARM64 Docker
with network disabled passed new behaviors and the previous saved demo.

Evidence is in evidence/2026-11-03-app-behaviors. These checks used disclosed
verifier schemas and made no Meta calls. Next: one clean-source live generation
request, no retry/fallback, before reporting actual model integration for the
new language. General native programs, events, cross-row data and hardware
capabilities remain separate extensions; this is composition, not a complete OS.


## 2026-11-03 — Live proposal rejected; retain future rejection diagnostics

Clean source a23795927a456acb5d58fd8d8918b5e518d39eab made exactly one live
request for a generic conditional two-field update. HTTP 200, model matched,
6450 ms provider latency, 3576 tokens (2879 input / 697 output). The proposal
decoded, then shared host validation returned formula_syntax. No app_plan was
emitted, no app was installed, and the final guest catalog remained revision 1
with no apps. No checkpoint file was created because nothing changed. No retry
or fallback occurred. This is a failed live generation, not successful end-to-end
Meta evidence for the new language. Offline runtime results remain valid.

The diagnostic path then recorded only the failure code, discarding the rejected
proposal; its exact invalid formula cannot now be reconstructed. Fixed that path
to retain the decoded proposal in app_validation_failed with execution=false;
updated the verifier to expose rejection codes instead of a generic assertion.
An actual offline --app-plan-replay check with synthetic '$stock -' confirmed
nonzero exit, formula_syntax, exact retained proposal and no accepted app_plan.
The updated host/shared suite passes 84 tests; host formatting and Clippy pass.
No guest executable changed in this diagnostic follow-up, so QEMU regressions
were not repeated. Raw logs preserve their original CRLF/trailing protocol tabs
through narrowly scoped Git attributes.

Evidence: evidence/2026-11-03-app-behaviors/live-rejected and rejection-diagnostics.
The next model-integration increment should implement a bounded correction cycle
with diagnostic feedback, preserving request/revision binding and explicit call
limits. It is not implemented here, and no successful live composed app is claimed.


## 2026-11-03 — Bounded correction of generated app expressions

Implemented the next requested integration step after the prior formula_syntax
failure. Meta receives the original request/catalog, the exact rejected proposal
and trusted diagnostic guidance. Rust reproduces feedback before any key/provider
access and allows only formula_syntax, formula_unknown_field and formula_limit.
Corrections can change calculated formulas and calculate-step expressions plus
the reason. Guards, fields, defaults, permissions and step/branch structure must
remain identical; scope is checked even before ordinary validation. Clarify/decline
can stop the process. No general semantic verifier or automatic arbitrary redesign.

Studio bounds the loop to three attempts per goal and eight per session. Every
attempt/delivery checks request, base/current revision and guest planning mode.
Cancellation stops later attempts and discards late results; an already sent call
may still incur provider usage. Each attempt retains its diagnostics, with partial
output on timeout and a final trace. Malformed adapter output and output setup
failures terminate with notice rather than leaving the guest waiting. The native
screen shows attempt progress; the existing validation and approval flow remains.
No new unsafe, kernel mechanism or app isolation was added. Existing live verifiers
explicitly retain their original one/two-call budgets; the new verifier owns its
separate three-call bound.

Verified 90 host/shared Rust tests, host/kernel Clippy, formatting and Python syntax.
Real QEMU debug/release passed independent three-row arithmetic and reboot plus
13 adversarial cases covering attempt/session limits, scope, cancellation/staleness,
transport, clarification, unsupported capability, malformed output and timeout.
Existing apps/RFB, behavior, evolving journey and saved/live entry regressions passed;
console boot/protocol passed 37 checks per profile. Linux ARM64 Docker passed the
new verifier and saved demo with network disabled. No Meta calls in these checks.
Evidence: evidence/2026-11-03-app-repair. The tested tree was modified relative to
a20bf47; exact hashes are retained. Next: freeze source, then one bounded live goal
with every attempt retained. Syntax validity still does not establish user intent.


## 2026-11-03 — Live composed app succeeds from frozen source

Clean committed source 5984a204aa20fe5a3c412099b93e1aa3b6ed5350 made exactly one
Meta call under the new maximum-three-attempt policy. HTTP 200, matching model,
4232 ms provider latency, 3495 tokens (2934 input / 561 output). The initial
Stock Mover proposal passed host and guest validation; no correction, retry or
fallback occurred. This live run therefore establishes generation and execution,
not an observed need for the live repair path.

The independent QEMU verifier confirmed no installation before field and program
approval, preview without mutation, Widget 3/0 -> 2/1 and Other 7/4 -> 6/5, empty
stock rejected unchanged, and exact schema/row recovery after restart. No model
calls were needed for execution or recovery. The three-row arithmetic oracle is
independent of the generated expressions; this is one disclosed example, not a
general proof that models understand every goal. Actual program definitions came
from Meta; no production app-specific implementation or replacement was used.

Evidence: evidence/2026-11-03-app-repair/live, including actual response, per-attempt
logs, request, checkpoint, UART, screenshots, report and hashes. Offline correction
and its thirteen adverse cases remain separate evidence. The earlier failed
single-call run is retained unchanged. This follow-up changes documentation and
evidence only; it makes no further provider calls or executable changes.


## 2026-11-03 — Compose atomic whole-table actions

The user explicitly prioritized more general runtime capabilities without
hardcoded apps. Added button scope row (backward-compatible default) or all.
Whole-table actions evaluate the same typed program independently on every saved
row, including hidden rows. Preview validates every candidate without mutation;
confirmation revalidates and commits all rows with one revision. A failing row,
terminal workflow state, stale version or invalid scope publishes nothing.

The two-pass commit retains one candidate at a time. Its atomicity depends on the
current pure, row-local language and lack of concurrent writers; cross-row reads
would require another strategy. Existing eight-row/six-field/two-button/six-step
bounds and 64 KiB kernel stack remain. No new unsafe, driver or isolation boundary.
App runtime/UI still share EL1. Native review shows scope, changed-row count and
paged before/after values; diagnostics identify a failing batch row and step.
Catalog v7, host validation, UART schema transport and checkpoints retain scope;
older apps still mean one row. Formula correction cannot expand scope.

Verified 94 Rust tests; host/kernel Clippy, formatting and Python syntax; real QEMU
debug/release with full capacity, cancellation, both branch outcomes, hidden rows,
last-row rollback, reviewed evolution, a distinct text/check app and exact recovery.
Existing behavior/correction/demo-entry regressions pass. Console boot/protocol
passes 37 checks per profile. Linux ARM64 Docker passes new batch checks and saved
demo offline. Measured debug frame details and caveats are retained, not a universal
stack safety claim. Evidence: evidence/2026-11-03-app-batches. No Meta calls yet in
this increment. Next: frozen-source generation and evolution, at most three
attempts per goal and six calls total, with all outcomes retained.


## 2026-11-03 — Two live prompts evolve one whole-table program

Clean e384bcae83820704010977a81a880f78c6fde39d completed generation and evolution
with exactly two Meta calls, both first-attempt successes. Creation: HTTP 200,
5437 ms provider latency, 3869 tokens (3108 input / 761 output). Evolution:
HTTP 200, 3800 ms, 3784 tokens (3272 input / 512 output). No correction, fixture
fallback or additional API calls. Both reported matching model and stop completion.

The actual Stock Move app first conditionally transformed rows with stock>=2 by
subtracting two and increasing moved by two. The second actual proposal retained
app/field/button identities and all saved rows, changing the threshold and amounts
to one. Both required native field/program approval. Eight fictional records were
entered locally; independent arithmetic verified every row after each action and
one revision per confirmation. Preview/cancel preserved data, and reboot restored
the exact final program and rows without inference. Item 4, initially stock=1,
illustrates the changed behavior: initially skipped, then processed after evolution.

Evidence: evidence/2026-11-03-app-batches/live, with both requests/proposals,
per-attempt logs, UART, screenshots, checkpoint, report and hashes. The offline
last-row rollback and unrelated-app checks remain distinct from this live run.
DEMO_READY.md links an inspectable saved result without modifying tracked evidence.
No executable change or further model call occurs in this documentation follow-up.


## 2026-11-03 — Scoped Git tools for app definitions

The user requested Git access for Meta, prioritizing generated-app history and
rollback, with later interest in non-kernel OS updates. Added an explicit git:
prompt route and seven closed choices: status/history/show/diff/save/restore/clarify.
Meta sees actual bounded history and chooses an operation; Rust validates request,
app identity, observed revision and current HEAD before the local Git adapter acts.
Each app has a dedicated bare repository whose only committed file is app.json.
Records and credentials are excluded. No shell, remote Git, source rewriting or
kernel changes. One provider call per Git prompt shares the eight-call session cap.

Restore uses the ordinary guest validation and native approval screens, retaining
current records. Incompatible fields/workflows/capabilities are rejected. HEAD is
not reset: saving a restored definition appends another child commit. Locks and
conditional ref updates prevent overwriting newer history; cancellation/stale guest
state blocks execution. Explicit bounds and trusted-host assumptions are documented
in APP_GIT.md. Git history alone is not an executable OS recovery mechanism.

Verified 100 Rust tests, host Clippy, formatting and Python syntax. A new unit test
initially exposed stale-context handling in the save no-op path; reobservation fixed
it before final verification. Real QEMU debug/release passed saves, diff/history,
reviewed rollback over current data, cancel, concurrent HEAD change, incompatible
restore, stale guest state, session limit and reboot. Existing batch, repair and
saved/live entry checks passed. Linux ARM64 Docker passed the new verifier and
saved demo with network disabled. Native Windows was not run.

Evidence: evidence/2026-11-03-app-git. Input definitions and Git selections in these
offline checks are explicit fixtures. Exact source hashes are retained for this
modified tree. No Meta calls in this increment yet. Next: freeze source and perform
one live sequence of two saves and one restore, at most three calls total, retaining
all outcomes. This checks live tool selection, not generation of the fixture apps.


## 2026-11-03 — Live Meta selects real Git saves and reviewed restore

Clean source 5a086a7d9bb9dd3f0226646ea1b6b9c9514dd391 completed exactly three
Meta calls: save version one, save version two, restore the previous saved version.
All returned HTTP 200, matching model and stop completion, without retry/fallback.
Provider latencies: 2988/3158/2364 ms; token totals: 695/740/719. These are tool
selection measurements, not an OS performance comparison.

The input app definitions were explicit verifier fixtures; Meta selected actual
operations, commit messages and the observed earlier Git OID. The native verifier
confirmed no restoration before approval, preserved current 15/19 records, ran the
restored +1 behavior to produce 16/20, and recovered identical program/data after
reboot without inference. Real Git HEAD retained version two; neither record data
nor credentials were committed by the app adapter. Offline adverse cases remain
separate. Evidence: evidence/2026-11-03-app-git/live.

This follow-up only records evidence/documentation. No executable changes or
further API calls. Non-kernel executable self-update remains future work requiring
service isolation, tested packages, compatible state and independent recovery.

## 2026-11-03 — Design proposals and explicit self-update deferral

The user requested visual proposals and polishing priorities before proceeding.
Recorded three directions: Tactile Terminal (recommended), Workbench and Canvas.
Conversational mockups use illustrative apps and local simulated interactions;
they are not guest screenshots or new runtime behavior. Documented the next polish
sequence around consistent native screens and a complete app evolution journey.

Executable non-kernel self-update is explicitly deferred. Added honest future-
direction narration to DEMO_READY, with the required isolation/recovery distinction.
No code/runtime changes or API calls. Checked local documentation links and diff
whitespace. The mockup script passed a syntax check; browser preview was blocked by
the browser URL policy, so no rendered-layout verification is claimed.


## 2026-11-03 — Separate home navigation from creation

User feedback supersedes the prompt-led home proposal: remove the headline and
activity block; put creation in its own screen. Recorded three simpler directions
(Column, Focus, Shelf) and their navigation tradeoffs. Conversational previews are
local simulations, not guest implementation. Documentation and whitespace checked;
preview JavaScript passed syntax validation. No model calls or runtime changes.


## 2026-11-03 — Column selected; compact palette exploration

The user accepted Column, rejected the yellow/lime accent and requested smaller
icons and less horizontal padding. Recorded the selected layout and four cooler
color proposals. Updated conversational concepts retain separate creation and
illustrative three-app navigation. Script syntax and documentation whitespace
checked; no native UI changes, rendered-browser verification or API calls.


## 2026-11-03 — Refine Apps, prompt entry and keyboard concept

Responded to user feedback with compact library rows/search, balanced 16 px mockup
side margins, clearer composer/action spacing and an optional local keyboard.
Preserved the 128-character ASCII boundary. Read the actual guest pointer handling
and VNC/UART forwarding before documenting the path to a guest-drawn keyboard.
The interactive concept sends no API calls; it is not a native implementation.
Checked preview script syntax and documentation whitespace. No QEMU execution or
rendered-browser validation was performed for this design-only increment.


## 2026-11-03 — Compose live-data pages from reusable primitives

The user requested deeper reusable creative capabilities toward spreadsheet and
presentation tools. Added two pages of two blocks each: literal text, count/sum/min/
max metrics, signed bars and two-column tables. Explicit all/pending scope binds to
current fields in the owning app. Evaluation is local and read-only; existing
formulas/actions remain available. Pages require native review and participate in
compatible evolution, checkpoints and Git definitions. Old empty-page schemas
retain their prior wire format. Catalog v8 tells Meta the exact language and bounds.

APP_TOOLBOX and APP_PAGES document experimentation, primitives, protocol and future
work. This is neither XLSX/PPTX support nor arbitrary applications. A creation lock,
keyboard, isolated app services and executable OS self-update remain unimplemented.
No comparative efficiency claim against Linux.

Initial debug linking exposed fixed-window pressure. Borrowing schema output avoids
a large copy; one development compilation unit with incremental disabled avoids
code duplication. Extracted schema construction from the retained dispatcher frame
(8832 -> 4288 bytes measured). Final selected action frames sum conservatively to
56944 bytes, excluding recursive/standard/indirect callees; this is not a complete
stack proof. Stack remains 64 KiB, MMU window unchanged, final pool end 0x401fc000.
No new unsafe or driver. Debug symbols and opt-level 1 remain.

Verified 106 Rust tests, host/guest Clippy, formatting/Python syntax, real QEMU
debug/release all-block checks, pixel oracle for a negative bar, all eight records,
recomputation, filtered scopes, reviewed evolution/cancel/rejection, unrelated-app
aggregates and reboot. Batch/Git/repair/demo-entry regressions, console/protocol
both profiles and host-console system checks passed. Linux ARM64 Docker passed
pages and saved demo without network. Evidence: evidence/2026-11-03-app-pages.
No Meta call yet. Next: frozen-source generation, at most one actual call, retaining
the result with no retry or fixture substitution.


## 2026-11-03 — Meta generates a live-data report in one call

Clean source 0d436bc62a390bd9163766582533a3ecfd71cc5d completed one actual Meta
request: HTTP 200, matching model, stop completion, 7715 ms provider latency, 4489
tokens (3685 input / 804 output). No retry, correction or fixture replacement.
Name Amount Report contains the model's two pages and all four supported block
kinds. Its table uses Amount then Name; the schema is preserved exactly.

The verifier required native page approval, entered eight fictional independent
records, checked sum 36, changed -5 to 9 and checked sum 50 without more inference.
Fresh QEMU boot restored exact pages/fields/rows. Offline filter/rejection, signed
bar pixel and second-app tests remain separate evidence. Live captures, proposal,
request, diagnostics, UART, checkpoint and hashes are in
evidence/2026-11-03-app-pages/live. APP_PAGES explains how to reopen a copy without
a key and how to use one's own key for new prompts.

Documentation/evidence follow-up only; no executable change or further model call.
The approved Column shell redesign, creation lock and richer document editing are
still future work, and kernel/device/EL1 isolation limitations remain explicit.


## 2026-11-03 late session — checked two-record transfers and demo rehearsal

Added catalog v9's optional same-app transfer rule: field-key bindings, positive
quantity, unique destination, nonnegative/sufficient source and checked destination
addition. Both candidate rows and derived values validate before a one-revision
commit. Terminal records and stale requests cannot mutate. Native rule review and
balance preview require explicit confirmation; no new unsafe or device authority.
Host schema/Meta contract, checkpoints and Git carry the rule. demo.py accepts a
validated supplied checkpoint without changing it; a three-minute runbook documents
actual generation versus saved recovery and remaining product limits.

111 Rust tests, Clippy, formatting and Python syntax passed. Real QEMU debug/release
and Linux ARM64 Docker offline transfer/recovery passed; pages/batches/Git/demo-entry
regressions passed. Console/protocol passed 37 checks in both profiles and host
console system checks passed. Debug fixed-window margin is now only 4096 bytes;
reorganize memory before further substantial growth. No enlarged mapping/stack.
Evidence: evidence/2026-11-03-app-transfer (actual UTC timestamps are November 4;
local work remains November 3). Next: freeze source and attempt one actual Meta
app generation, retaining the result without retry or fixture replacement.


## 2026-11-03 — Meta generates and QEMU runs Transfer Balances

Clean source ce219caf694953d3a13cd4394e39469b8a989a0e made one actual Meta call:
HTTP 200, matching model, stop, 8119 ms, 4832 tokens (4066 input / 766 output).
No repair, retry or fixture replacement. Meta's four-field app includes the new
transfer rule and a sum/bar page. The actual declaration was reviewed unchanged.
Independent fictional balances 12/5 became 8/9, sum remained 17; quantity 99 was
rejected, cancelled approval could not execute, and reboot restored exact state.
Saved-demo preparation copied/validated the actual pre-operation checkpoint.

Archived raw request/proposal/diagnostics/UART/captures/checkpoints and documented
how to rehearse from the supplied checkpoint. Corrected a verifier annotation that
mentioned calculated record columns unconditionally: live Meta generated none,
and only offline tests assert those columns. Raw report is retained with this
qualification. Added raw-log Git attributes, preserving actual UART bytes. No
runtime change or additional Meta call in this follow-up. Recording/submission
remain pending; prioritize the coherent generate/use/reject/recover story.


## 2026-11-03 late polish — native Column and own-key evaluator setup

Applied selected terminal linework, compact Column and Glacier Blue to actual guest
screens. Home/Apps/Create are distinct; global Create starts a new app, edit app/F2
retains the existing update target. Search, counter and prompt layout remain bounded.
The default saved kit now includes three unmodified recorded Meta app definitions,
with fictional records and source hashes. Nothing in app execution recognizes those
example identities specially.

Added private hidden-entry setup and separate local-format versus actual Meta
connection checks. The latter is one fixed small inference request with no app data.
401/403/quota/transport errors have distinct safe messages; native creation failures
use these notices. Existing key files are preserved and never read into tool output.
TRY_RAI_OS provides local/Docker paths, recovery and troubleshooting; SUBMISSION_DRAFT
and the final runbook leave video upload/actual submission explicit.

Offline checks: 112 Rust tests, host/guest Clippy, formatting and Python syntax.
Real QEMU debug/release navigation and saved-demo actions/reboot passed; generic app
regression passed. Setup tests use a synthetic key and simulated service errors.
An initial navigation assertion looked for outbound notice text in inbound UART logs;
corrected the verifier instrumentation. A real Ctrl-C race during VNC thread startup
was reproduced and fixed with graceful stop intent; saved/resume/live CLI checks
then passed without provider calls. Console/protocol passed both profiles. Debug
mapping still has 4096 bytes remaining; no memory-layout expansion.

Anonymous GitHub API check reported private=false for evannivw/rai-os. Linux offline
saved-demo checks passed; final-image CLI startup/shutdown/resume verification also passed offline in Linux.
Next: freeze sources, one actual connection probe, then reopen the polished saved
kit. No claim that the final narrated video or platform submission is complete.


## 2026-11-03 — evaluator connection check exercised against Meta

Clean pushed source 6ac184aebf5cfa9bafd4022c3295d1b9bbb752f0 ran the explicit
check-meta command once. Actual result ready: HTTP 200, matching muse-spark-1.3,
stop, 1450 ms, 48 tokens (24 input / 24 output). No guest boot, app generation,
record transfer or key-file modification. Safe diagnostics are retained under
final-polish evidence. The author's key remains ignored/local; judges provide
their own only for live mode. Saved demo and all offline checks require none.
The native three-app rehearsal is reopened separately. Video/upload/platform
submission remain pending; supplied-guide deadline is Nov 4 at 20:00 Guatemala.


## 2026-11-04 — submission recording harness and individual entry

Jarrol Villatoro confirmed an individual entry. Added an eleven-scene real-QEMU
sampling harness and a 180-second narrated renderer, with explicit scripted-input,
edited-timing and synthetic-reference-voice labels. No kernel/runtime change.
Offline rehearsal passed creation/review, 12/5 -> 8/9 checked transfer, rejection
without mutation, page evolution retaining data, and exact fresh-guest recovery.
The first rehearsal exposed a verifier bug selecting provider files by directory
order; selecting the highest request id fixed it. Both raw takes remain in target/.
Fresh Meta recording and final rendered media are pending at this commit.
Submission checklist distinguishes required guide artifacts from uninspected form
fields; video upload and actual platform submission remain separate pending actions.


## 2026-11-04 — actual recorded demo and narrated delivery assets

Clean pushed source 94e9efbac8960190740a3ab292c17ab6872d81c0 completed the live
recording in one take: two actual Meta requests, each accepted on attempt one,
repairs disabled. HTTP 200/matching model; 8892 ms/4784 tokens and 9284 ms/4958
tokens. Transfer/rejection, evolving page with retained records and fresh-guest
recovery all passed. Preserved requests, responses, recording manifest and final
checkpoint under docs/evidence/2026-11-04-submission. No inference beyond two calls.

Rendered an eleven-scene three-minute 1080p MP4 with English captions and explicitly
labelled synthetic reference narration. Real sampled QEMU footage uses scripted
inputs and edited timing, not a latency benchmark. All scene proof frames inspected;
full media decode passed; mean/peak audio -20.0/-4.5 dB. Container duration about
180.023 seconds includes audio padding. Participant listening review, video upload
and platform submission remain pending. Local media stays in ignored target/.
