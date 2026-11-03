# Milestone 1 architecture

An educational AArch64 kernel for QEMU, with a long-term microkernel direction.
Kernel modules share privilege and memory. One built-in Rust program now runs
at EL0 with bounded write/exit syscalls and a timer-enforced invocation deadline; see [USER_PROGRAM.md](USER_PROGRAM.md).
A [bounded two-context scheduling diagnostic](SCHEDULING.md) now saves/resumes
EL0 integer state on one CPU. A [bounded task lifecycle](TASK_LIFECYCLE.md) now
lets one task finish or fault while its peer continues. There is no general task
creation, heap or isolated agent execution. Two fixed tasks can now exchange
[bounded scalar messages](IPC.md) through kernel-owned mailboxes and use
[blocking receive](IPC_WAIT.md) with message/closure wakeup. A first
[record-count service](RECORD_SERVICE.md) now reuses those mechanisms to process
immutable fixture records at EL0 and return results over UART. A separate
[persistent worker](PERSISTENT_WORKER.md) now retains its EL0 continuation and
private stack between console requests. An initial [guest coordinator](GUEST_COORDINATOR.md)
now runs at EL0 alongside that worker, admitting versioned manual proposals and
retaining results. The [guest Meta bridge](GUEST_META_BRIDGE.md) now transports its
pending goal/ticket to Meta and forwards typed proposals for guest admission. EL1 stage-1 translation
now has a base map and two fixed task maps with private writable stack pages;
user code/constants remain shared read-only. See [MMU.md](MMU.md) and
[TASK_MEMORY.md](TASK_MEMORY.md).

## Boot

QEMU loads the ELF through -kernel. The linker places _start at 0x40080000,
inside virt RAM starting at 0x40000000, leaving room at the start for the DTB.
Supported configuration: virt-9.2, TCG, Cortex-A53, one CPU, 128 MiB, EL1,
gic-version=2, secure=off, virtualization=off, no firmware. Entry requires MMU/caches off. Rust constructs page tables and
enables the MMU before the boot banner; caches remain off.

boot.S masks interrupts, checks EL1, selects SP_EL1, creates a 16-byte-aligned
64 KiB stack, clears BSS, installs VBAR_EL1 and synchronizes instruction execution,
then calls kernel_main. The stack is outside the cleared BSS range. An unexpected exception level halts before Rust; the boot
check will time out rather than claim success.

Rust uses no_std, no_main, and core. aarch64-unknown-none-softfloat avoids
requiring SIMD/FPU setup for compiler-generated code. There are no external
crate dependencies. Rust's bundled rust-lld links the ELF; Apple ld is not an
ELF linker. build.rs triggers relinking when linker.ld changes.

## UART and unsafe contracts

PL011 is fixed at 0x09000000 for this QEMU machine. Aligned 32-bit volatile
accesses preserve MMIO operations, but do not provide synchronization.
Configuration: 115200, 8N1, no FIFO or interrupts; the driver polls readiness.
Polling consumes CPU while waiting and is not evidence of energy efficiency.

Unsafe blocks document the mapping, alignment, and exclusive ownership:
one CPU, interrupts masked during UART access, exclusive UART access, and either pre-MMU access
or the active Device-nGnRnE identity mapping. A fatal handler
may take over UART permanently without resuming the interrupted writer. These assumptions
must be revisited for another board, remapping, or concurrency. Linker symbols and
Assembly entry form the boot contract. EL1 exception vectors support one
allowlisted EL1 return, controlled timer IRQs and bounded EL0 syscall/fault handling; other events enter a fatal UART diagnostic on
a dedicated stack; see [EXCEPTIONS.md](EXCEPTIONS.md) for
assumptions and limits. Rust panic still prints and halts without recovery.

## Responsibilities

- memory/frames.rs: pure page-address bookkeeping; memory/mod.rs: exclusive
  linker-reserved physical pool, ownership tokens, zeroing, and diagnostics.
  See [MEMORY.md](MEMORY.md). This allocator is not a heap.
- mmu/tables.rs: pure fixed-board mapping construction; mmu/mod.rs: owned
  table pages and EL1 register activation/readback. Code is read-only/executable,
  data is non-executable, and tables/constants are read-only/non-executable.
  Explicit user code/constants/stack pages permit EL0; kernel/device pages do not.
- exceptions.S / exceptions.rs: EL1 entry, full integer frames for slots 4, 5, 8 and 9,
  dedicated handler stack, allowlisted EL1 return, EL0 dispatch, or fatal report.
- context/: pure return policy, fixed frame layout and bounded SVC/ERET diagnostic.
  See [CONTEXT.md](CONTEXT.md); scheduling reuses the Frame layout.
- user/: one linked Rust program, privilege transition, bounded copy/exit policy
  and synchronous invocation lifecycle. User faults and elapsed deadlines terminate
  only that invocation; [USER_TIMEOUT.md](USER_TIMEOUT.md) describes the boundary.
- schedule/mod.rs: shared validated EL0 entry/capture and the original twelve-IRQ
  context diagnostic. schedule/lifecycle.rs: pure states and ready-task selection.
  schedule/runtime.rs: two kernel-owned records, frame/stack validation, bounded
  execution and compact results. schedule/programs.rs: finite linked Rust user
  tasks plus a deliberate forbidden-read fixture. Terminal tasks are skipped;
  each turn uses a task map that denies EL0 access to the peer stack.
  schedule/isolation.rs checks real own/peer accesses and recovery. See
  [TASK_LIFECYCLE.md](TASK_LIFECYCLE.md) and [TASK_MEMORY.md](TASK_MEMORY.md).
- ipc.rs: pure bounded FIFO mailboxes, checked task IDs, route permissions and
  endpoint closure. schedule/runtime.rs binds sender identity to the current
  record and translates scalar syscall inputs/results; schedule/ipc_programs.rs
  contains linked Rust request/reply clients. schedule/wait_programs.rs exercises
  blocked receivers. Lifecycle Waiting states are skipped until the runtime
  completes their pending result; a closed batch with no runnable task reports
  remaining waiters as Deadlocked. No user pointers are dereferenced.
- records.rs: allowed immutable fixture IDs and bounded ASCII token counting;
  source bytes are shared user read-only data, and counting/token selection execute in user text.
- record_index.rs / user/index_program.rs: synchronous exact per-record index
  at EL0, using checked output copies and normal-exit publication. The host exports
  its text unchanged; an independent Python checker validates the exact revision.
  See [RECORD_INDEX.md](RECORD_INDEX.md). This is not yet a coordinator operation.
- service.rs: validated input -> bounded guest execution -> completion/result checks.
  schedule/service_programs.rs: fixed EL0 request/reply client and worker. This is
  one closed batch per host request, without persistent lifetime or external wakes.
- user_memory.rs: exclusive lease over fixed user RAM, held across suspended
  worker lifetime and acquired by every legacy EL0 entry before clearing stacks.
- schedule/worker.rs: one retained frame, mailbox pair, per-request budgets,
  ingress/reply validation and explicit stop/failure cleanup. Endpoint 1 is kernel
  host ingress; worker_program.rs executes at EL0 as endpoint 2. The console owns
  the session. This is not a general daemon scheduler or guest AI coordinator.
- schedule/coordinator_program.rs: persistent EL0 goal/proposal policy; scalar
  host input, duplicate/stale rejection, IPC dispatch and result retention.
  schedule/coordinator.rs: kernel-owned two-context session, coordinator-only
  host gateway, bounded turns, report checks and pair cleanup. It shares the
  worker entry/IPC and exclusive RAM lease, not host Meta state or credentials.
- timer/: exclusive GICv2/CNTP adapter, controlled EL1 IRQ diagnostic and absolute
  EL0 budget or scheduling slice. IRQs are enabled in the diagnostic loop or EL0; successful handlers
  never use UART or allocate. User/schedule dispatch accesses only its paused session or observation. See [TIMER.md](TIMER.md).
- clock.rs: read-only generic counter adapter and boot-relative time.
- uart.rs: bytes and MMIO, including allocation-free formatted output.
- console.rs: framing, response rendering, and interleaving a bounded task step
  with nonblocking UART receive. Transmit still polls.
- task.rs: one owned word-count job, pure state transitions, progress/cancellation.
  See [TASKS.md](TASKS.md); this kernel job remains cooperative and unisolated.
- protocol.rs: structured request parsing without execution.
- command.rs: human-command parsing and execution of allowed operations. It
  receives a typed uptime snapshot and delegates memory/MMU operations to their
  owned adapters; the MMU supplies register snapshots and fixed task-map switches.
- display/: optional guest framebuffer adapter, original renderer and glyph data.
  The console owns it and supplies observed index results. It currently runs at EL1,
  not as an isolated GUI service. See [GUEST_DISPLAY.md](GUEST_DISPLAY.md).
- main.rs: composition and panic handling.

Both parsers converge on an operation enum; neither executes arbitrary text.
See [PROTOCOL.md](PROTOCOL.md). The [host console](CONSOLE.md) already transports
requests over UART. The [small host coordinator](COORDINATOR.md) sequences a bounded priority queue (FIFO ties) using
observed completion. Its host placement is provisional; the target is a guest
service above the kernel once service execution/isolation exists. See the
[placement and migration rationale](COORDINATOR.md#why-the-coordinator-is-currently-on-the-host). A separate
[Meta bridge](META_BRIDGE.md) now makes one real structured proposal and validates
it before guest execution. host/meta/src separates proposal policy (domain), local
configuration, HTTPS provider transport and composition. It reuses the shared UART
adapter; JSON dependencies stay in its own host manifest. Its new
[bounded coordinator](META_COORDINATOR.md) maintains released revisions and actual
observations, applies timed fixture events while a provider request is pending,
and rejects stale or duplicate work before UART dispatch. State policy remains
separate from thread/clock/QEMU adapters. The existing priority
coordinator and graphical panel have not been connected to this bridge.
A new --guest mode reads the actual EL0 coordinator reports instead of maintaining
host goal/admission policy. guest_bridge.rs owns transport/lifecycle,
guest_report.rs validates decoded observations, and decision_source.rs shares
bounded provider/replay thread ownership with the older host mode. Its late
proposals are rejected inside the guest. See [GUEST_META_BRIDGE.md](GUEST_META_BRIDGE.md).
Credentials and HTTPS remain on the host. The guest must independently
validate operations. General isolated services still require service-specific memory
ownership/lifetimes, external event delivery and richer permission policies; the fixed
stack maps and scalar IPC are foundations,
not a complete service security model. Module names do not supply those guarantees.

## Clock

The kernel samples the Arm generic physical counter before UART initialization
and supplies measured uptime to commands. The adapter is owned by kernel_main;
there is no mutable global clock. See [CLOCK.md](CLOCK.md) for access assumptions,
frequency conversion, real-QEMU checks, and the distinction from timer interrupts.
The host console and [graphical panel](PANEL.md) transport uptime like other
commands through shared host adapters. The panel is a separate native application;
its GUI dependencies do not enter the kernel. The separate [native guest display](GUEST_DISPLAY.md)
now draws a bounded view inside QEMU without those dependencies. An interactive
desktop/window system remains pending.

## References

- https://doc.rust-lang.org/stable/rustc/platform-support/aarch64-unknown-none.html
- https://www.qemu.org/docs/master/system/arm/virt.html
- https://github.com/qemu/qemu/blob/master/hw/arm/virt.c
- https://github.com/qemu/qemu/blob/master/hw/char/pl011.c

See [ROADMAP.md](ROADMAP.md) for completed foundations and the next service prerequisites.

## Retained exact indexes

The optional coordinator index mode retains EL0 policy and worker contexts.
A owns goal/admission/reuse decisions, B serializes exact indexes through bounded
scalar IPC, and the kernel holds six committed 384-byte result slots after
stream/publication validation. No task receives another task's writable pointer.
The native view renders pending/created/reused outcomes. See
[RETAINED_INDEX.md](RETAINED_INDEX.md); count-mode Meta transport is unchanged.
The [Meta index bridge](META_INDEX_BRIDGE.md) selects this mode explicitly. Its
artifact history validates observed reuse but never supplies guest results or
chooses cache hits. Only the provider boundary selects the allowed operation
schema; count-only modes retain their narrower permission.

## Required multi-record sets

The optional index-session goal bitset is now retained by coordinator A at EL0.
A set change is attested through scalar SVC 15 before the kernel commits requirement
metadata; it invalidates the active per-record ticket while retaining exact revision
slots. The kernel derives missing entries and serializes a bounded complete set
from those slots plus read-only public source bytes. Serialization remains at EL1.
See [INDEX_SETS.md](INDEX_SETS.md) for ownership, versioning and the independent
content checker. The host walkthrough uses the guest's missing list; whole-set
Meta planning and full review-pack budget/release semantics remain pending.


## October 30: whole-set model proposals

The [whole-set Meta bridge](META_SET_BRIDGE.md) now exposes the current complete
requirement and guest-reported missing list to the model. The explicit set mode
binds proposals to a goal generation; `coord.select` rejects obsolete authority
before issuing a per-record ticket. Existing EL0 admission/execution, exact cache
and complete-set publication remain the guest's responsibility. This permits
choices of missing-job order, not a demonstrated AI performance advantage.
Next establish a useful constrained choice and fair comparison; full review-pack
budget/release semantics, final video and submission remain pending. Original
event dates and the November 4 deadline are unchanged.

The native debug integration exposed excessive stack temporaries in the command
dispatcher. An isolated pre-fix image changed the idle timer ACTIVE pointer from
zero to 2 while entering coord.goal; the corrected image retained zero and
completed the same request. The measured debug dispatcher frame fell from 27,552
to 20,608 bytes by grouping coordinator report arms in one helper (1,104-byte
frame). This fixes the reproduced stack corruption without enlarging the 64 KiB
kernel stack or weakening timer checks. It is not a whole-program stack bound:
a kernel-stack guard/high-water diagnostic remains future hardening. Native
set tests cover the previously failing path in both build profiles.


## October 30: frozen development policy pilot

The [policy pilot](POLICY_PILOT.md) adds a shared host experiment controller for
EDF, weighted value/credit ordering and Meta, using the same existing EL0 index
worker, exact cache and guest generation gate. Priorities, real deadlines and
simulated credit admissions now have declared rules and an independent log/output
audit. The simulation is on the host; this does not add kernel power management
or complete the full review-pack contract. Three development scenarios and two
repetitions per policy are frozen before live measurement. Report all outcomes
and model overhead; no efficiency claim follows from implementing the experiment.
The larger held-out study, final presentation and submission remain pending.

## Planned persistent-goal lifecycle — October 30

The [persistent-goal contract](PERSISTENT_GOALS.md) separates the current target
from the last verified publication. Existing cache and generation gates remain
the execution foundation; publication retention/commit and semantic batch plans
are planned, not working features. The guest will own admission/publication
identity; the host carries events and provider calls, and the independent checker
supplies a trusted validation acknowledgment. Document ownership and storage
bounds at implementation; exported host files are not guest persistence.

## Retained publication lifecycle — 2026-10-31

[Publication](PUBLICATION.md) now separates the active target from one committed
Artifact in the kernel-owned Coordinator. Pure publication.rs rules manage target,
candidate authority, status and last output. EL0 attestation still establishes
requirements; the worker still builds indexes. The trusted host checks an exported
candidate and acknowledges a fresh ticket; EL1 rechecks identity/completeness
before retaining it. Fatal service cleanup preserves this owned snapshot, while
explicit stop clears publication access. Source bytes are immutable linked data.
No new syscall, mapping or unsafe code; no disk/reboot persistence. The native
publication-specific view and batch model planning remain future work.

## Native publication presentation — October 31

Read-only publication::Status accessors supply the [native view](PUBLICATION_DISPLAY.md).
Console executes first, then renders the resulting lifecycle, bundle progress and
request notice. Presentation does not own admission or verdicts. Moving rendering
temporaries into a separate method reduced the debug execution frame from 5296 to
656 bytes and corrected the reproduced timer_busy startup path without changing
stack capacity or timer admission. Kernel-wide stack bounds remain unproven.
The Rust public walkthrough uses existing host transport and the independent
checker; its scripted failure and pauses are explicit, with no model calls.

## Provider outcome diagnostics — October 31

The host-only provider_report module inspects allowlisted metadata and decodes the
existing envelope without changing domain admission. Typed provider failures
carry redacted diagnostics/available usage to the common decision-source reporter;
the single-request path uses it too. A decoded response can still fail domain or
guest validation. See [META_DIAGNOSTICS.md](META_DIAGNOSTICS.md) for accounting,
privacy and incomplete-capture limits. No guest API or provider request settings
changed; batch proposals are the next separate increment.


## November 1: bounded batch integration

[Batch proposals](META_BATCH.md) now cover all 1–4 currently missing operations
in one response. Pure all-before-dispatch admission, original-generation checks,
per-job guest tickets and the independent publication handshake constrain the
plan. Events during inference, between jobs and during validation supersede old
authority. Current catalog/permissions remain immutable fixtures. The host holds
the pending plan; the guest retains goals, execution admission and publication.
No new kernel mechanism or GUI scope is added. Single-operation modes and the
original pilot remain separate and unchanged in policy/settings.

This is a delivery gate, not completion of semantic planning or a new efficiency
comparison. November 1 was the feature-freeze target; the independently authored
semantic workload/fair baseline, final video and submission are still pending.
Prioritize one bounded demonstrable case, evidence and rehearsal over additional
infrastructure. If no semantic advantage is established, present the measured
recovery prototype with its actual Meta role. November 4 remains the deadline
(18:00 internal target, 20:00 official Guatemala); preserve contribution dates and
the unresolved October 26 kickoff / October 27 build-period eligibility question.


## November 1: first content-based selection gate

[Semantic selection](SEMANTIC_SELECTION.md) now separates interpretation from
execution: natural-language goal plus six public document contents -> bounded
Meta selection/clarification/decline -> independent fixed-case intent check ->
existing deterministic guest batch and verified publication. Expected answers
are authored before inference and excluded from provider context. Host request
text/version and static catalog/permission versions bind selection; guest goal
and per-job tickets bind execution. No new kernel or UI mechanisms.

Each case uses a fresh guest session. Continuous semantic replanning, a genuinely
complex workload and fair deterministic comparison remain pending. These six
public development cases do not establish general correctness or superiority.
Freeze additional infrastructure; prioritize measured outcomes, reproducibility
and the three-minute video. November 4 dates and provenance questions are unchanged.


## November 1: scoped resolver, frozen comparison and same-session replay

[The comparison](SEMANTIC_COMPARISON.md) adds a deterministic content/reference
resolver with explicit language coverage, sharing permissions, dependency rules,
intent oracle and guest execution with Meta. Twelve development cases and two
rotated repetitions per policy are frozen before live measurement (24 calls max).
No held-out or general efficiency claim is made. A separate recorded-decision
replay uses two archived Meta selections to show historical/current publication,
exact reuse and one new job in a single real guest session. It performs no live
inference; graphic pauses are presentation-only. Continuous live semantic event
recovery remains pending. No kernel or graphical implementation changes.


## November 2: useful offline artifact pivot

The user approved prioritizing a useful result after no semantic speed advantage
was measured. [OFFLINE_MISSION.md](OFFLINE_MISSION.md) defines a larger fictional
project corpus with approved versus draft releases, compatible dependencies,
permission exclusion and unresolved human actions. The Rust host bridge produces
a portable reader plus exact source files/manifest, with independent checking
before publication. This new corpus does NOT execute in QEMU. The kernel remains
unchanged; do not relabel host packaging as a guest filesystem or new native UI.

This first slice is a one-shot checked host publication. Automatic semantic event
recovery, incremental updates and general file ingestion remain pending. Prior
negative performance evidence is retained. Protect final verification, browser
review, video and submission time; the original November 4 targets remain intact.


## November 2: fresh semantic decisions in a retained guest

[SEMANTIC_SESSION.md](SEMANTIC_SESSION.md) connects fresh host-side interpretation
to ONE retained guest session, with externally checked streamed approvals. Guest
EL0 jobs/cache/publication and native presentation remain the execution boundary.
Controlled sequential request events replace the previous preloaded-decision-only
walkthrough; the recorded replay remains available separately. Ambiguity schedules
no work. During interpretation the screen still represents the last accepted guest
target; new request/clarification text is host-side only.

This is an integration increment on the existing six tiny guest records, not a
new decision-quality win. Freeze three known events, both rules and Meta, and three
Meta calls maximum before measurement. No kernel expansion, concurrent event
handling, file discovery or resource-efficiency claims. The host-only offline
mission remains separate. Prioritize final evidence/rehearsal/video/submission;
preserve original deadlines and unresolved contribution eligibility.


## November 2: constant objective with context-triggered decisions

[CONTEXT_SESSION.md](CONTEXT_SESSION.md) keeps one fixed natural-language goal and
supplies controlled structured assignment observations. Both policies resolve
source authority, revision freshness and conflicting assignments before the same
retained guest execution. Events are host-injected fixture data, not new user
prompts, OS file watching or real telemetry. The text goal/context remain host
responsibilities; guest required sets, jobs/cache and publication remain real.
The native screen still shows the last accepted guest target during clarification.

Freeze three development observations and expected outputs before a three-call
Meta probe. This input-boundary increment does not establish an advantage on
complex unseen tasks. No kernel/corpus expansion; deadlines and provenance limits
are unchanged. Finish evidence and protect rehearsal/video/submission time.


## November 2: portability and writable guest files

[PORTABLE_RUN.md](PORTABLE_RUN.md) packages a common Linux host environment for
Docker users, with a platform-independent VNC display route. The QEMU board/CPU
contract remains unchanged. Actual platform test results must be reported; Linux
containers on macOS do not establish physical Windows validation.

[GUEST_FILES.md](GUEST_FILES.md) adds a separate bounded volatile guest store:
runtime byte import, list/read/copy/move, complete-upload publication and generation
checks. The native Files view renders real state. It currently runs at EL1, has
no persistent disk, and is NOT an isolated filesystem service or AI file manager.
Next design bounded whole-plan file actions with preview and atomic publication;
preserve originals and reject stale/colliding plans. Existing comparisons remain
unchanged; stop framing tiny per-operation inference as a speed optimization.

Adding the 5,216-byte owned RAM store exposed a debug-only stack regression on
coord.start. The dispatcher now routes coordinator and other system operations
through separate helpers, so unrelated match-arm temporaries are not retained on
the same call path. Observed debug frames: old combined command executor 20,832
bytes; new router 144 plus coordinator helper 9,808 (system helper 12,720 on its
separate path). The console run frame is 23,376 bytes. The 64 KiB stack is unchanged.
Actual headless/native context cycles and scheduling fault diagnostics pass;
these frame measurements are not a complete worst-case stack bound or guard.


## November 2–3: guest-native prompt workspace

[PROMPT_WORKSPACE.md](PROMPT_WORKSPACE.md) adds a complete bounded interaction:
keyboard prompt, one Meta file-plan proposal, native preview, explicit confirmation
or cancellation, and actual guest copy/move results. The RFB adapter transports
QEMU pixels and UART keys; it is not a host-rendered UI or guest keyboard driver.
The prompt/review/file modules currently share EL1 privilege, not isolated services.

All plan checks precede writes. Sources must exist and be distinct; destinations
must be new and distinct; copy slots must fit. Chains, overwrite and deletion are
unsupported. Three actions maximum; one generation increment for a confirmed plan.
The guest rechecks after review, and cancellation invalidates late request IDs.
Meta receives current text file contents through the host Rust provider; one call
per prompt, one in-flight request, eight calls maximum per interactive session.
Semantic correctness is still a review/evaluation concern, not a kernel guarantee.

The unoptimized native-display development build exceeded the fixed first-2-MiB
RAM mapping when including the 1-MiB page pool. Development now uses opt-level=1
with debug symbols. The linker assertion, map, pool and 64-KiB stack are unchanged.
Earlier opt-level=0 frame measurements remain historical evidence, not current
bounds. Debug/release workspace and coordinator checks and debug MMU/scheduler
fault checks cover this configuration; a whole-kernel stack bound is still pending.

## November 3: declarative microapp runtime

[MICROAPPS.md](MICROAPPS.md) adds a bounded schema interpreter, app-local record
namespaces, form/list/filter controller and append-only schema migrations. Meta
proposes a full schema; explicit review and guest validation precede install.
The pure domain has no I/O or generated code. Host JSON conversion shares its
validation rules; guest execution additionally checks existing row values.
Only records.local is supported. The interpreter/renderer run at EL1, so namespace
separation is not process isolation. HTTPS and keyboard transport remain host-side.
A dark Ubuntu Mono framebuffer presentation is shared with the file workspace.

## November 3: saved-session adapter

[APP_STORAGE.md](APP_STORAGE.md) adds host-backed checkpoints after committed app
revision changes. The domain remains unaware of paths/files: the bridge reads
schemas/rows, a Rust validator reuses the domain, and the host adapter writes a
checksummed versioned JSON envelope via flush/atomic replace. An exclusive lock
prevents concurrent launcher writers. Fresh-guest restore validates all input
before boot and reads back exact data before input starts; failure preserves the
checkpoint. There is no guest block driver, storage capability for generated apps,
encryption or new process isolation. Runtime revisions are session-local.

## November 3: first declarative system-control capability

[DISPLAY_CONTROL.md](DISPLAY_CONTROL.md) adds display.brightness beside the existing
records.local capability. An installed control schema has one bounded required
numeric input, no records and no filter. Domain validation returns a checked
Brightness value; the controller verifies display availability; the display adapter
attenuates new RGB source colors. All views share the setting, with no cumulative
dimming on redraw. The user can reset to 100% from the launcher with F6.

Review exposes the requested permission. Capability cannot change during an app
update. Install/restore/update never actuate, and boot returns to 100%. The adapter
is a QEMU framebuffer implementation, not a phone backlight driver or a power
optimization. Shared EL1 privilege and trusted UART/host limitations remain.


## November 3: dynamic calculated fields

The app schema now carries bounded formulas. microapp_formula.rs compiles integer
expressions to a private fixed-size instruction array and evaluates checked i32
arithmetic. References resolve only to earlier numeric columns, so dependencies
are acyclic. Domain save/update materializes all derived values before mutation;
schema migration validates every row before its deterministic commit pass. No
unsafe, allocation, host expression evaluator, new capability or process boundary.
The guest controller skips computed fields during editing, while presentation
shows a local preview and the complete formula before approval. The host catalog
teaches Meta this generic vocabulary; replay fixtures live in a separate module
that the live branch never consults. See APP_FORMULAS.md for limits and checks.

## November 3: reviewed app data exchange

ImportBatch stages bounded cells outside installed records. Complete-batch checks
resolve the current schema, require every input, recompute formulas and optionally
compare supplied derived values. Runtime append validates the whole batch before
one revision update. Controller binds import to a selected app, request and base
revision and requires native approval. The host adapter owns CSV and the explicit
inbox/outbox; Meta receives neither paths nor records. Existing saved-session
persistence runs after the guest commit. No unsafe or new privilege boundary.
See APP_EXCHANGE.md and the reproducible workflow in DEMO_JOURNEY.md.


## November 3: bounded per-record workflows

The app schema optionally contains a reachable graph of up to four actions.
Generic domain code checks source state, revision and an optional numeric guard,
then changes one row state. Terminal records reject ordinary saves. State is kept
separately from editable values and persisted in validated snapshots; CSV imports
start new processes. Trusted fresh-device restoration is a distinct boot-only
controller phase, closed before interactive input. The host and saved-file owner
remain trusted; restore validates state membership, not historical action proofs.

No new unsafe, device operation or process isolation was added. The console
initializer was separated to release its large temporary stack frame before the
retained event loop handles actions. The linker stack size remains 64 KiB.
APP_WORKFLOWS.md and KERNEL_DECISION.md document the limits and tradeoffs.


## November 3: composable button programs

The app schema now includes up to two six-step programs triggered by explicit
user buttons. microapp_behavior.rs is a pure typed interpreter: set, calculate,
toggle, require and forward-only if. It has no I/O, pointers or ambient authority.
Steps read/write one candidate row and refresh derived fields; all references and
branches are checked before installation. Preview returns a candidate without
mutating runtime state. Confirmation re-evaluates against the same current
revision and commits one row once; any error leaves row/revision unchanged.
Terminal workflow state remains enforced. Programs may change through reviewed
compatible schema updates; existing workflow definitions remain immutable.

The model receives the versioned catalog and strict JSON schema; the host checks
proposals using the shared domain, then the guest validates actual state. App
controller/UI stages every behavior review and shows before/after values. The
host only transports definitions, input and checkpoints. This adds no background
events, external side effects, cross-row transactions or isolated process. The
interpreter/controller still share EL1 with the kernel. Kernel boot/MMU/interrupt/
IPC mechanisms are the existing foundation, not a claim of a finished OS.

Increasing schema size exposed a nested debug constructor stack issue even though
the first QEMU behavior run passed: retained console plus constructor and caller
used 75904 known frame bytes versus the 65536-byte stack. Explicit const evaluation
of file/workspace/app initial values reduces the constructor frame from 35808 to
3520 and the known startup sum to 43616. No new unsafe, stack-size or MMU change.
Known behavior action frames sum to 58464 before parser/indirect callees; this is
not a whole-kernel worst-case bound. Real debug/release regressions remain essential.
See APP_BEHAVIORS.md and the evidence/2026-11-03-app-behaviors stack report.


## November 3: bounded proposal correction

app_repair.rs owns the narrow repair policy. It reproduces the rejected proposal's
arithmetic error using the original request and masks only calculated expressions
when comparing structure. All other fields, permissions, guards and branch/step
structure must match before full validation can accept a correction. Clarification
or refusal remains possible. Provider code sends one HTTPS request per invocation;
it treats prior proposal strings as untrusted data, not new instructions.

The Python Studio coordinates at most three attempts per goal and eight per
session. A read-only app.planning diagnostic exposes only request/base/current
revision and mode. The bridge checks it before every attempt and delivery;
app.progress validates the same binding and shows a bounded attempt count.
Cancellation discards late results and prevents later calls, but an already sent
call can still incur provider usage. Per-attempt evidence retains both rejected
and accepted definitions. The guest's ordinary proposal validation and human
approval remain the installation boundary. No new unsafe or kernel mechanism.

This verifies permitted syntax and structure, not the meaning of arbitrary user
goals. The app controller/interpreter still share EL1, and the host is trusted.
See [APP_REPAIR.md](APP_REPAIR.md) for bounds, stopping conditions and reproduction.
