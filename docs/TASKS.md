# First cooperative kernel task

Implemented October 26, 2026. One bounded word-count task runs inside the kernel
while its console continues handling requests. This is a cooperative job in the
same address space and privilege level, not a process, thread, interrupt-driven
scheduler, AI agent, or performance demonstration.

## Try it

Run `sh scripts/panel.sh`, select Start kernel, and use the Kernel task tab.
Start task processes the supplied text; Cancel task targets the displayed task ID.
The progress bar uses processed/total bytes from the guest. A cancelled count is
labeled partial; only a completed task displays a final result. The Connection
check tab retains the existing echo control.

The default text has 149 ASCII bytes and 27 words under the rule below. A native
UI check cancelled task 1 at 32/149 bytes, then completed task 2 with 27 words.
See [captured panel](evidence/2026-10-26-task.png). These are actual observations.

Terminal commands, available through both direct UART and the Rust host console:

```text
task.start One small task, in Rust.
task.status
task.cancel 1
```

The example cancellation succeeds only while task 1 is still running. If it has
finished, task_not_running is the correct response. A new boot starts a new session
and resets IDs. Build host and guest together: the readiness banner is now
`Console ready: type help for commands`, and the panel expects the new operations.

## Exact behavior

- One slot, with at most 192 bytes of owned source text. No allocation or retained
  pointer into the reusable console input buffer. ASCII printable characters and
  spaces only; the guest repeats validation independently of the host.
- Words are maximal ASCII `[A-Za-z0-9]+` runs. Spaces and punctuation separate
  words, so `RAI-OS` counts as two. Chunk boundaries do not split a word. This is
  a deterministic counting exercise, not linguistic or semantic interpretation.
- Each step examines at most 8 bytes, updates its position and count, and returns.
  It preserves whether the previous byte was inside a word.
- The educational cadence permits one step every 100 ms of guest time, starting
  100 ms after acceptance. A delayed loop does not execute catch-up batches.
  The panel labels this deliberately slow learning pace. It is not an efficiency
  result; counting such a tiny text does not inherently require this much time.
- Empty text immediately completes with zero words. Otherwise the lifecycle is
  RUNNING -> COMPLETED or RUNNING -> CANCELLED. Snapshots remain available until
  another task is accepted or QEMU is restarted. No resume or result archive exists.
- Starting while running returns task_busy without replacing the current source.
  Successful starts consume increasing nonzero u32 IDs. Exhaustion rejects new
  work rather than reusing an ID. Cancel requires the exact current ID; stale
  commands cannot cancel a replacement task. Cancel after completion is rejected.
- Cancellation stops future steps. It does not undo already processed bytes or
  claim a partial count is final. A task can finish before a cancel request is
  parsed; the resulting rejection does not disconnect the panel.

## Cooperative loop

The console owns a Task value and repeatedly:

1. Samples guest time and gives the task at most one due step.
2. Checks UART for at most one received byte without waiting for one to arrive.
3. Updates line framing and executes a command only after a complete valid line.
4. Repeats, including during an unfinished or rejected input frame.

`src/task.rs` owns pure job state and transitions. `src/command.rs` invokes the
allowed operations. `src/console.rs` supplies time and interleaves work with input.
`src/uart.rs::try_get` replaces the blocking receive loop. No new unsafe code was
needed for the task; MMIO access retains its documented board assumptions.

This remains a busy polling loop. Transmit still waits for UART readiness and
can delay processing; no strict real-time latency or CPU/energy saving is claimed.
The later EL0 timer deadline does not preempt this cooperative kernel job.
Scheduling it alongside guest services still requires separate design and tests.

## Wire contract

These are additive operations within the existing @rai1 protocol. Examples show
literal escape notation for actual TAB/LF bytes:

```text
@rai1\t1\ttask.start\tOne small task.\n
@rai1\t2\ttask.status\n
@rai1\t3\ttask.cancel\t1\n
```

Start and cancel require exactly four fields; status requires three. Empty start
text is valid. Cancel's argument is a nonzero decimal u32; no signs or whitespace.
Each successful reply contains the same five-field payload in this order:

```text
task_id=1; state=running; processed=0; total=15; words=0
```

States are idle/running/completed/cancelled. Initial idle has ID and all counts
zero. Completed requires processed == total. Counts while running/cancelled are
partial. New errors: task_too_large, invalid_task_text, task_busy, stale_task,
task_not_running, task_ids_exhausted; malformed arguments use bad_args. Existing
frame limits/error handling are unchanged. Every response still matches its
request ID; task IDs are a separate namespace with no deduplication semantics.

The panel polls task status approximately every 250 ms independently of its
one-second uptime polling. It validates state/counter bounds before rendering.
Rejected operations are reported without treating a healthy transport as broken.
Disconnect/Stop clears live task readings. The host does not execute the count.

## Verification and next step

```sh
python3 scripts/verify_tasks.py
python3 scripts/verify_tasks.py --release
python3 scripts/verify.py
python3 scripts/verify.py --release
python3 scripts/verify_panel.py
```

Three host unit tests compile the pure Rust task module directly with rustc's
native test harness: chunk boundaries/pacing, cancellation/stale IDs, and
empty/bounded inputs/ID exhaustion. No simulated hardware is needed for them.
The task integration script also performs 28 real-QEMU exchanges (25 structured and three direct-shell) in each profile,
including completion during an incomplete UART frame, independent exact counting,
responsive echo/uptime, cancellation stability, busy/size/argument rejections,
and reuse after cancellation. The existing 37 protocol checks pass in both
profiles. Six panel tests include task parsing and live task/cancel/completion
through the worker; terminal integration and native UI controls were also checked.

Raw transcripts are in target/verification/tasks-debug.log and tasks-release.log.
Escaped [debug](evidence/2026-10-26-tasks-debug.txt) and
[release](evidence/2026-10-26-tasks-release.txt) captures are retained in Git.

The subsequent [small host coordinator](COORDINATOR.md) now sequences a bounded
queue using observed completion. Multiple active guest tasks and Meta model
integration remain pending. Review-pack fixtures/verifiers remain
separate from this introductory word-count task.
