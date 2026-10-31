# First coordinator inside RAI-OS

Implemented October 30, 2026. Two linked Rust programs now remain alive at EL0:
coordinator A retains a goal and admits matching proposals; worker B counts
immutable records. Each has a private writable stack page, and both exchange
kernel-routed scalar IPC. The coordinator's policy/state executes in the guest,
not in the host terminal or in an ordinary EL1 module.

The initial kernel increment prepared and verified this boundary with manual
proposals. The subsequent [guest Meta bridge](GUEST_META_BRIDGE.md) now connects
real provider responses to `coord.*`; older bridge modes retain their batch path.
HTTPS, credentials and JSON/provider parsing remain host responsibilities.
The original kernel captures below use manual proposals, not model inference.

## Run a complete cycle

From the repository root run `sh scripts/console.sh` (or `cargo run --locked`),
then enter these commands in a fresh QEMU boot:

```text
coord.start
coord.observe notes@1
coord.propose 1:notes@1
coord.status
coord.stop
```

Start reports idle. Observe reports `awaiting_proposal`, `ticket=1`, and
`record=notes@1`. The matching proposal executes at EL0, then reports
`state=completed; event=executed; ticket=1; record=notes@1; bytes=30; tokens=7;
completed=1`. The report is one line and also contains real per-task dispatch
counts and a trace. `coord.status` reads the last validated report without
running either service. The second count field is a session completion counter,
not the record's word count.

For stale-response handling, use another fresh boot:

```text
coord.start
coord.observe notes@1
coord.observe notes@2
coord.propose 1:notes@1
coord.propose 2:notes@2
coord.stop
```

The old proposal returns `coord_stale_proposal`; worker B gets no turn.
The current proposal returns 44 bytes/9 tokens and one completed task.
Always read the returned ticket rather than assuming consecutive values:
every admitted observation consumes a ticket, including duplicate observations,
although a duplicate retains the coordinator's existing goal ticket.

## The actual execution path

1. Kernel runtime starts A in task map A and B in task map B, once per session.
   A publishes idle and waits for host input; B blocks waiting for IPC.
2. UART parsing accepts an allowed record key. A bounded scalar observation
   resumes A's host wait. A stores the goal ID/ticket in its own execution state,
   then publishes a proposal request and waits again.
3. A proposal resumes A. A checks that a goal exists, the ticket/key match and
   the goal is not already completed. A stale, wrong-key or duplicate proposal
   produces a rejection with no IPC send and no B execution.
4. A sends kind 20/count to B. The kernel wakes B, restores its saved private
   context, and resumes it. B sends packed result kind 21 and sequence kind 23.
5. A receives both messages, checks their sender/kind, record ID and sequence,
   retains the result and advances its completion count. Its next publication
   is validated against the result/sequence actually sent by B. Both programs
   must be waiting again before the UART command returns success.

The proposal currently selects the sole supported action, count, for the current
record. This is deterministic admission and coordination, not model inference
inside the guest. There is one current goal, not a general agent framework or
multi-goal history/cache. A repeated observation of that goal preserves its
pending/completed state; observing another record replaces it. Returning to an
older record after replacement creates new work. All six public fixture records
are allowed; this path does not implement release schedules or file watching.

## Ownership, budgets and failure

The pair holds one exclusive user-RAM lease throughout its lifetime. Standalone
`worker.*` sessions and legacy EL0 batches cannot coexist with it. Their entry
commands return `user_memory_busy`; stopping an inactive alternative session
cannot release this pair's lease. Read-only and EL1 diagnostics remain available.
There are still twelve owned page-table pages and 244 free pool pages.

Both tasks run round-robin on one CPU, skipping blocked contexts. Each admitted
start/observe/propose gives each task at most six timer expirations and sixteen
syscall attempts; attempt sixteen is not served. Budgets span all resumes within
that command. A slice is 20 ms and restarts after a syscall; this is not a 120 ms
aggregate command deadline. Commands are synchronous: host events are processed
between executions, not concurrently during worker calculation. Waiting consumes
no service turns, but UART still polls and no low-power improvement is established.

A fault, invalid context/syscall/report, budget exhaustion, missing publication
or closed-batch IPC deadlock retires the whole interdependent pair. Both saved
continuations and inboxes are discarded, user RAM is cleared, and the lease is
released. Failed status retains the last confirmed goal/count plus the failure
code and failing turn's trace; partial execution is not presented as completion.
Stop/start explicitly recovers. Stop is idempotent and resets service counters.

The kernel-issued u32 ticket counter survives stop/start within the same boot,
so a delayed pre-restart proposal cannot satisfy a new goal. Exhaustion returns
`coord_ticket_limit`; tickets never wrap. Reboot resets everything and requires
a new host transport session. Tickets are correlation/version numbers, not
secrets, authentication or permission tokens. The UART host is trusted to submit
allowed goals; the kernel does not grant arbitrary memory or executable access.

## Scalar ABI and transport

Existing IPC SVC 10/send and 12/receive-wait are reused with endpoints A=1, B=2,
routes 1->2 and 2->1, and two-entry inboxes. Kernel dispatch binds sender identity.
New calls are accepted only from A in this runtime:

- SVC 13: host wait. On delivery x0=0, x1=kind (30 observation, 31 proposal),
  x2=ticket, x3=validated record ID. No pointer or byte buffer is shared.
- SVC 14: publish. Inputs x0=state (0 idle, 1 awaiting, 2 completed), x1=ticket,
  x2=packed record/result, x3=session completed count, x4=event code. x0-x3 return
  zero; other GPRs are preserved. Publication is provisional until quiescence
  and validation. Multiple publications per command are rejected.

Event codes: 0 started, 1 observed, 2 duplicate, 3 executed, 4 stale_proposal,
5 wrong_record, 6 already_completed, 7 no_goal. Pending record representation
has ID in bits 32..63 and zero result fields. Completed representation uses the
existing ID/bytes/tokens layout. Metadata/sequence checks are not a duplicate
semantic token scan; Python verifies exact counts independently.

The additive UART operations are `coord.start`, `coord.status`, `coord.stop`
(exactly three fields), `coord.observe` (four fields, key argument), and
`coord.propose` (four fields, `<nonzero-u32-ticket>:<key>` argument). Example:
`@rai1\t7\tcoord.propose\t2:notes@2\n`. Request IDs and goal tickets are separate.
Human commands use a space before the same argument. Extra fields are rejected.

The normal status fields are coordinator, worker, state, event, ticket, record,
bytes, tokens, completed, a_calls, b_calls, a_slices, b_slices and trace. States
are stopped, idle, awaiting_proposal, completed or failed. No goal uses
`record=none`. Rejected proposals return an outer error; `coord.status` retains
the guest's rejection event and unchanged goal/count.

Trace pairs start with A/B. S=send attempt, R=immediate IPC receive, D=IPC wait,
W=message wake (not a CPU dispatch), H=host wait, O=publication, P=preemption,
F=fault/unsupported syscall, L=syscall budget exhaustion. The trace has at most
96 pairs and resets per admitted command; status does not reset it. In the normal
count, A makes five calls and B makes three. Invalid proposals make two A calls
and zero B calls. No trace is a claim of throughput superiority.

Input errors retain bad_args/unknown_record/record_denied. Lifecycle errors are
coord_not_running/coord_not_stopped. Admission rejections are coord_no_goal,
coord_stale_proposal, coord_wrong_record and coord_already_completed. Contained
session errors include coord_task_fault, coord_bad_syscall, coord_context_invalid,
coord_slice_limit, coord_call_limit, coord_bad_report, coord_extra_report,
coord_missing_report, coord_ipc_closed, coord_deadlocked and coord_trace_limit;
existing timer/MMU errors propagate and retire the pair as well.

## Verification and code map

```sh
python3 scripts/verify_guest_coordinator.py
python3 scripts/verify_guest_coordinator.py --release
```

Twelve real-QEMU images per profile exercise normal coordination, five A fault
modes, B attempting A's host gateway, and five B fault modes. Normal checks cover
all six fixture counts, independent IPC trace replay, duplicate/stale/future/wrong
proposals, 19 completions before restart, old-ticket rejection after restart,
retained state across EL1 diagnostics, framing rejection, ownership exclusion
and recovery. A fabricated completion disguised as a duplicate is rejected by the kernel;
mutation checks also ensure forged counts/traces fail the independent audit.
Full captures go to `target/verification/guest-coordinator-*`. Selected actual
[debug](evidence/2026-10-30-guest-coordinator-debug.txt) and
[release](evidence/2026-10-30-guest-coordinator-release.txt) responses are retained in Git.

- `schedule/coordinator_program.rs`: EL0 goal/admission/dispatch/result policy.
- `schedule/worker_program.rs`: the same persistent EL0 counting worker.
- `schedule/coordinator.rs`: kernel-owned two-task session, gateway, bounded
  dispatch, report validation and cleanup. This module itself runs at EL1.
- `command.rs` / `protocol.rs`: validated console requests, with no AI provider.
- `host/shared/model.rs`: safe request encoding used by the host terminal.

This is an initial linked guest coordinator, not a complete microkernel, dynamic
process loader, on-device inference engine or native graphical desktop. Meta
integration is now available as `--guest`: it reads the pending goal/ticket,
asks Meta for a supported proposal and submits it through this boundary while
keeping the guest authoritative over admission and observed completion.
See [GUEST_META_BRIDGE.md](GUEST_META_BRIDGE.md) and the current [delivery plan](DEMO_PLAN.md).

## Retained index extension

The count-mode behavior above remains available unchanged. `coord.start index`
now selects exact indexing, bounded scalar output transfer and six immutable
revision slots. `coord.result` retrieves the active completed artifact, and
observing a previously completed revision produces `event=reused` without a
worker turn. See [RETAINED_INDEX.md](RETAINED_INDEX.md) for the different mode
budgets, metadata ABI, ownership, native views and verification. The default
Meta bridge remains count-only; [`--guest-index`](META_INDEX_BRIDGE.md) selects
this new path.
