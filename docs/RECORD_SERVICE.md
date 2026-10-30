# First bounded guest record service

Implemented October 29, 2026. This is a useful execution boundary for the demo:
a host request causes a protected EL0 worker to scan an immutable record and
return its byte length and ASCII token count. It is one closed two-task batch
per request, not a persistent daemon, general filesystem or autonomous agent.

## Try it

```sh
sh scripts/console.sh
```

Enter `record.count notes@1`, `record.count notes@2`, `record.count empty@1`,
and `record.count restricted@1`. The direct guest console from `cargo run --locked`
accepts the same commands. No API key or network access is needed.

Observed results: notes revision 1 has 30 bytes and 7 tokens; revision 2 has
44 bytes and 9 tokens; the empty record has zero bytes/tokens. The restricted
record returns `record_denied`. A token is a maximal run of ASCII letters or
digits. Punctuation, whitespace and every other byte are separators. This counts
total occurrences, not distinct words, and does not yet produce an index.

## Contract and boundaries

UART uses `@rai1\t<request-id>\trecord.count\t<key>\n` (literal TAB/LF).
The kernel preserves the UART request ID in its response. Exactly one key is
required. The allowed keys and fixed batch-local scalar IDs are:

| Scalar ID | Immutable fixture key |
| --- | --- |
| 1 | brief@1 |
| 2 | notes@1 |
| 3 | notes@2 |
| 4 | checklist@1 |
| 5 | empty@1 |
| 6 | appendix@1 |

The original [fixture](DEMO_FIXTURE.md) is unchanged. Allowed source bytes are
compiled into shared user read-only data, padded to a 128-byte maximum per record;
the actual lengths exclude padding. Oversized sources fail at compile time.
Restricted source bytes are not linked into the kernel. Unknown revisions are
not substituted. All six allowed revisions are available in this small service;
scenario releases, outages and changing goals are not implemented here.

Successful payload prefix: `record=<key>; bytes=<n>; tokens=<n>; worker=EL0`.
It is followed by the existing bounded [task/message trace](IPC_WAIT.md), which
shows completion, exit values, timer slices, send/receive counts and wakeups.
The host independently checks results; a success string alone is not evidence.

| Error | Meaning |
| --- | --- |
| bad_args | Missing/empty key, extra UART field, or whitespace in the key. |
| unknown_record | Key/revision is not in the fixed allowed catalog. |
| record_denied | The explicitly denied synthetic record was requested. |
| service_request_rejected | Worker exited with the invalid-request result. |
| service_reply_rejected | Client rejected response status, sender, kind or record ID. |
| service_failed | Task fault/limit, other unsuccessful exit, or mismatched exit values. |
| service_bad_result | Successful exits supplied an invalid ID/length/count shape. |

Underlying runtime errors can also propagate. Errors do not return a partial
count as success. Every invocation starts with fresh records, cleared private
stacks and empty inboxes, then retires both endpoints. UART processes one command
at a time; it does not deliver new commands to a running batch. Request IDs are
transport correlation only, with no authentication or deduplication.

## Execution

1. Shell/UART parsers create a validated RecordId, rejecting denied/unknown input.
2. The kernel starts task A (client) with only that scalar ID in x0 and task B
   (worker) with zeroed registers. Each has its own writable stack map.
3. A sends B message kind 20 (`COUNT`), value = record ID, then waits for a reply.
4. B receives from its own inbox, validates sender and kind, checks ID bounds,
   and scans the selected read-only bytes at EL0.
5. B sends kind 21 (`RESULT`), then exits. A wakes after B's send, validates
   the response identity, and exits with the same packed value.
6. The service adapter requires both successful exits and matching values. It
   checks record identity, exact expected source length, and tokens <= bytes.
   UART serializes those observed values and the runtime report.

IPC values pack the record ID into bits 32..63, bytes scanned into bits 16..31,
and token count into bits 0..15. With at most 128 bytes, both counts fit.
Worker validation errors use kind 22 (`ERROR`), value 1, and exit code 4.
Client response rejection uses exit code 2. The existing kernel binds the sender
ID; users cannot choose it. Only A-to-B and B-to-A routes are enabled.

Normal observed trace: `AS,BR,AD,BS,AW,AE,BE`: client sends, worker receives,
client waits, worker sends, client wakes, client exits, worker exits. Real timer
preemptions may add P events. Each task has the existing six-preemption limit
with 20 ms slices and a 16-attempt IPC cap (attempt 16 terminates unserved).
Neither request size nor service logic can increase these limits.

## Code and safety

- `src/records.rs`: immutable data, allowed identity parsing and bounded token
  counting, independent of UART, scheduling and Meta. The count function and
  source table are explicitly linked into user executable/read-only regions.
- `src/schedule/service_programs.rs`: linked EL0 client and worker using the
  existing inline scalar syscall helpers. No user pointers cross the boundary.
- `src/service.rs`: validated request -> execution -> checked result. It checks
  completion and result shape; it does not recompute the semantic count at EL1.
- `src/schedule/runtime.rs`: selects linked entries, initializes the scalar input,
  reuses private-stack dispatch, IPC, waiting and failure containment. New jobs
  do not change the underlying queue/lifecycle policy.
- `host/shared/model.rs`: host frame encoding and separator/size checks.

During debug bring-up, core's is_ascii_alphanumeric helper was emitted into EL1
text, so a nonempty record faulted while the empty record succeeded. The worker
now uses an explicit byte pattern with the same semantics, within its user text.
Permissions were retained. Debug and release are both exercised because code
placement can differ. The new data/counting code uses safe Rust; existing syscall
assembly and unsafe linker attributes retain explicit preconditions.

All linked user code and permitted fixture bytes are shared read-only. Private
stack mappings do not provide per-record secrecy from another malicious EL0
program. Denied bytes are absent, and the API has a fixed allowlist; this is not
an authenticated multi-user storage system. The two tasks are fixed linked
programs, not dynamically loaded applications or a complete microkernel.

## Verification

```sh
python3 -B scripts/verify_service.py
python3 -B scripts/verify_service.py --release
python3 -B scripts/verify_console.py
```

Each profile runs five actual QEMU images: normal, invalid scalar ID, unknown
service kind, wrong record identity in a reply, and a worker forbidden read.
Each image receives the six allowed keys twice; the normal image also executes
a direct shell request (61 service invocations per profile). The host reads exact
fixture bytes, checks catalog lengths/digests and computes expected token counts
with a separate Python regex. It checks both task exits, packed results and
scheduler transitions, including no dispatch while waiting. Deliberately altered
counts, record IDs, lengths and completion states are rejected by that checker.

Denied keys, malformed/extra arguments, unknown revisions/operations, repeated
fresh invocations and post-failure operation are covered. Subsequent user, waiting,
private-stack, MMU and page-pool checks confirm kernel continuity and unchanged
page ownership: twelve table pages, 244 free pool pages. The denied source body
is checked absent from each ELF. Host console tests also exercise the command.

Actual captured output: [debug](evidence/2026-10-29-service-debug.txt) and
[release](evidence/2026-10-29-service-release.txt). This is an independent checker
for this small operation, not the full [demo artifact verifier](DEMO_ACCEPTANCE.md).
No model integration, resource optimization, energy measurement or comparative
performance advantage is established by these results.

## Next step

One real Meta proposal is now connected through the [host bridge](META_BRIDGE.md)
to an allowlisted request and its observed result. Credentials/HTTPS remain on
the host. Next, extend the
useful workload and event loop alongside the independent result verifier and a
fair deterministic baseline. Persistent task lifetime, external wake sources and
additional executors are explicit future increments, not prerequisites to show
this first model -> validated operation -> guest result path.

## Separate persistent mode (October 30)

`record.count` retains its original closed-batch contract. New `worker.*`
commands reuse counting/IPC with a retained EL0 continuation; see
[PERSISTENT_WORKER.md](PERSISTENT_WORKER.md). While that session holds the fixed
user RAM lease, `record.count` returns `user_memory_busy`. Stop the worker before
using this batch mode or the existing Meta bridge. The bridge is not migrated
by this kernel increment.
