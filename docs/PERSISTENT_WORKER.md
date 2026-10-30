# One persistent guest worker

Implemented October 30, 2026. The kernel can retain one linked EL0 worker across
console requests. This is a kernel/service-lifetime increment, independent of
Meta inference. The host Meta coordinator still uses `record.count` batches;
it does not yet use this new session or live inside the guest.

## Try it

Run `sh scripts/console.sh` from the repository root, then enter:

```text
worker.start
worker.count notes@1
worker.count notes@2
worker.status
worker.count empty@1
worker.stop
```

The direct UART shell (`cargo run --locked`) accepts the same commands. No API
key or network is needed. Real QEMU debug/release observations include:

```text
worker=EL0; state=waiting; completed=0; reason=none; calls=1; slices=0
record=notes@1; bytes=30; tokens=7; worker=EL0; state=waiting; completed=1; reason=none; calls=3; slices=0
record=notes@2; bytes=44; tokens=9; worker=EL0; state=waiting; completed=2; reason=none; calls=3; slices=0
```

Timer placement can change `slices`. `completed` is checked against a sequence
sent by the EL0 worker, whose local counter survives each suspended receive.
It resets after stop/start. Persistence means guest RAM across requests, not disk
storage or survival across QEMU restarts.

## What remains alive

1. `worker.start` takes exclusive ownership of the fixed user RAM, clears it,
   and enters the worker once at its initial PC with a fresh private B stack.
2. The worker calls receive-wait (SVC 12). With no request it stops receiving
   CPU turns. Its saved registers, next PC, SP and stack bytes remain owned.
   The console resumes under the base page-table root.
3. `worker.count` admits one validated record ID from UART. Kernel ingress uses
   endpoint 1; the worker is endpoint 2. Endpoint 1 is not an EL0 client here.
   The runtime completes the pending receive in x0-x3 and resumes the original
   continuation, preserving other registers and stack memory.
4. The worker scans immutable fixture bytes, sends the packed result and its
   own sequence counter, then waits again. The console returns a result only
   after validating both replies and observing the next wait.
5. `worker.stop` retires the continuation, clears owned user RAM and mailboxes,
   and releases the lease. It is idempotent. A fresh start begins at zero.

This reuses private task map B, the integer context switch, timer slices and
bounded IPC. There is one worker and one admitted request at a time. Dispatch is
synchronous inside the command handler; UART requests do not run concurrently
with worker computation. Waiting for an external host message is a valid idle
state, unlike the closed-batch mutual-wait deadlock in `ipc.wait`.

## Ownership and safety

`user_memory::Lease` is a non-clonable exclusive token backed by an atomic flag.
Every existing user-entry path acquires it before accessing user RAM. The worker
retains it even while suspended. Legacy `user.run`, `user.spin`, `schedule.*`,
`ipc.*` and `record.count` return `user_memory_busy` during a worker session;
none can clear its stack. Stop the worker before running those commands.

Read-only status, echo, memory-pool diagnostics and EL1 context/timer diagnostics
remain available. The reserved page pool does not overlap the user stack.
No new page tables are allocated: twelve owned table pages and 244 free pool
pages remain. The lease covers the whole existing 8 KiB user region, although
this worker uses only its B half. This deliberately excludes simultaneous legacy
batches instead of pretending to support dynamic address spaces.

Before each entry and after each exception, the runtime validates user PC,
private stack bounds/alignment and EL0 PSTATE. The lease preserves the memory
ownership assumption of `run_turn`. A single masked kernel CPU, no DMA,
non-nested handlers and base-map restoration remain required. User code and
constants are shared read-only; kernel metadata and A's stack deny EL0 access.
No raw pointers are accepted over IPC. Ordinary kernel modules still share EL1
privilege; this is not a complete microkernel or arbitrary process loader.

## Wire contract and errors

Argument-free operations have exactly three fields:
`@rai1\t1\tworker.start\n`, `worker.status`, `worker.stop`.
Counting has exactly four: `@rai1\t2\tworker.count\tnotes@1\n`.
Examples use escaped TAB/LF notation. Same record allowlist, packed result and
ASCII token semantics as [RECORD_SERVICE.md](RECORD_SERVICE.md).

Status payload fields: `worker=EL0; state=<stopped|waiting|failed>;
completed=<u64>; reason=<code|none>; calls=<u8>; slices=<u8>` (one line).
Count prepends `record=<key>; bytes=<u64>; tokens=<u64>; `.
There is an internal ready state while running; requests are serialized so it
is not externally observable. Counts/calls/slices are observations, not battery
or CPU-efficiency metrics. Calls/slices describe the last admitted operation;
status and rejected inputs do not reset them.

- `worker_not_stopped`: start requires stopped, including after a failure.
- `worker_not_waiting`: count requires a live waiting worker.
- `worker_sequence_limit`: completed reached u64::MAX; stop/start to reset.
- `bad_args`, `unknown_record`, `record_denied`: input rejected before wakeup.
- `worker_fault`, `worker_bad_syscall`, `worker_context_invalid`: retire the
  worker for a fault, unsupported syscall/exit, or invalid saved context.
- `worker_slice_limit`, `worker_call_limit`: retire after the request's budget.
- `worker_reply_rejected`: malformed/missing/extra result or wrong sequence.
- `worker_ingress_failed`, `worker_unexpected_reply`: internal protocol violation.
- Existing timer/MMU errors propagate and also retire this worker session.

Failure retains diagnostic status, clears user RAM, discards the continuation,
and releases the lease immediately. Other EL0 diagnostics can then run. Explicit
stop/start is needed before this worker can accept new work; no silent retry.
Previously confirmed completed count survives failure for diagnosis, not as a
claim that the failing request completed.

IPC inboxes retain capacity two and routes 1->2 / 2->1. Request kind 20 carries
record ID; result kind 21 carries ID/bytes/tokens; kind 23 carries the sequence.
Kernel-bound sender identity, result ID, byte length, token upper bound, exact
next sequence and absence of extra messages are checked. The kernel does not
repeat the token scan to prove its semantic correctness. The independent Python
check computes expected counts from fixture bytes.

Each start/count receives a fresh budget: at most six 20 ms timer expirations
and sixteen syscall attempts (the sixteenth is not served). A valid count uses
two sends and one new wait. Completing an old blocked receive is not an extra
syscall attempt. A syscall yield starts another timer slice, so six slices are
not a 120 ms whole-request deadline. Both budgets together bound execution;
they are not reset on each resume within the admitted operation.
No timer runs for a suspended worker. UART still polls, so idle worker state
does not establish low power consumption of the whole guest.

## Verification and limits

```sh
python3 scripts/verify_worker.py
python3 scripts/verify_worker.py --release
```

Each profile boots six real images: normal, deliberate forbidden kernel read,
infinite computation, syscall flood, wrong result ID and wrong sequence. Normal
runs serve 25 records per session, stop/restart, and repeat. Independent regex
counts cover all six fixture records, including empty data. Tests interleave
other user-entry attempts and EL1 diagnostics, check retained counts, malformed
requests, explicit recovery, unchanged page-table roots/pool accounting, and
legacy operation after stop/failure. Captures are in `target/verification/worker-*`.
The host-console integration also exercises start/count/status/stop. Selected real
[debug](evidence/2026-10-30-worker-debug.txt) and
[release](evidence/2026-10-30-worker-release.txt) responses are retained in Git.

This establishes service lifetime. It does not add arbitrary file access,
multiple persistent workers, background event dispatch, a guest coordinator,
permissions for AI-generated code, or a demonstrated advantage over other OSes.
The next scoped step can be guest-side coordination over this lifetime boundary;
credentials and HTTPS should remain on the host for the hackathon.
