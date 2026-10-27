# Small host coordinator

Implemented October 26, 2026. A deterministic FIFO queue sequences the existing
[word-count task](TASKS.md) in the real QEMU kernel. FIFO means first in, first
out. This is a host service beside the panel, not a kernel scheduler, isolated
process, AI agent, independent result verifier, or performance optimization.

## Try it

Run `sh scripts/panel.sh` from the repository root. Open **Queue**, enter text,
and click **Add job** for each job. Start the kernel and click **Run queue** once.
The worker submits one job and waits for a matching completed guest snapshot
before starting the next. The panel displays guest IDs, processed bytes, and
observed counts. Only completed jobs label their count as a result.

- Maximum eight entries, including finished history, each at most 192 printable
  ASCII bytes. Empty input is allowed and completes with zero words.
- **Pause queue** stops new submissions once handled by the worker; an active
  job finishes normally. It is not cancellation and cannot undo a submission
  already sent. **Run queue** resumes pending work explicitly.
- **Clear queue** removes the in-memory list only when paused with no active job.
  Additions are allowed while running, at the tail, without reordering.
- Manual task start/cancel are blocked while the coordinator owns work. Echo,
  info, and status remain available. A pre-existing running manual task prevents
  the queue from starting; it is never taken over.
- **Stop kernel**, connection failure, or an unexpected task snapshot pauses the
  queue. An active or unacknowledged job becomes **Interrupted**, meaning its
  outcome is uncertain, not successful. Its last observation remains history.
- A fresh QEMU boot resets guest IDs. Pending jobs require an explicit Run;
  interrupted jobs are never automatically retried. Queue row numbers are local
  labels, not guest IDs. Historical guest IDs belong to their original session.
- Closing the panel discards all queue state and stops its QEMU child. No queue
  persistence, background daemon, or recovery journal exists yet.

The guest still uses its educational pace of eight bytes per 100 ms. Host status
polling is approximately every 250 ms, with worker command waits up to 100 ms.
These waits and serial transport add latency; they are not an efficiency result.

## Code responsibilities

- `host/panel/src/coordinator.rs`: bounded inputs, FIFO choice, ownership, pause,
  completion, and interrupted-state rules. No UI, threading, or I/O dependency.
- `host/panel/src/worker.rs`: owns the coordinator and QEMU on one thread. Reserves
  a job before sending, obtains a correlated protocol response, validates its
  task snapshot, then polls for progress. Subsequent observations must retain the
  same guest ID and total, with nondecreasing progress/count. Unexpected state,
  transport failure, or rejected automatic submission stops the session safely
  without a retry. The kernel's result is observed, not independently recomputed.
- `host/panel/src/app.rs`: sends user actions and renders worker snapshots. It
  neither chooses the next job nor computes results.
- Existing guest code validates and executes each word-count request. No kernel
  source, protocol, dependency, or privilege boundary changed in this increment.

The minimal loop is **choose pending job -> send -> observe -> complete -> choose**.
Later, a Meta-backed policy could propose a different allowed choice. Validation,
execution, and observation would remain explicit. Priorities, resource budgets,
multiple job kinds, permissions, isolation, and actual Meta integration need their
own increments; this queue does not implement those guarantees.

## Verification

`python3 scripts/verify_panel.py` passes eleven tests: eight pure host tests and
three real-QEMU tests. Four coordinator tests cover FIFO waiting, pause, mismatched
IDs, unknown submission outcomes, bounded/invalid text, manual ownership, and empty
completion. The new live test checks ordered guest IDs and counts (2, 3, 0), manual
mutation rejection, pause without starting the next job, Stop during work, reboot,
and explicit continuation without replaying interrupted work. Existing lifecycle,
clock, echo, task cancellation, and missing-kernel checks still pass.

Debug/release panel builds, formatting, and all-target panel Clippy passed. Native
window review checked Add, Run, Pause, Resume, observed results, and cleanup.
No kernel changes required repeating the unchanged kernel test suites.

Retained [test output](evidence/2026-10-26-coordinator-tests.txt) and
[native window capture](evidence/2026-10-26-coordinator.png).
