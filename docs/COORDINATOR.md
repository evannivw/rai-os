# Small host coordinator

Implemented October 26, 2026. A deterministic priority queue sequences the existing
[word-count task](TASKS.md) in the real QEMU kernel. Equal priorities use FIFO
(first in, first out). This is a host service beside the panel, not a kernel scheduler, isolated
process, AI agent, independent result verifier, or performance optimization.

## Try it

Run `sh scripts/panel.sh` from the repository root. Open **Queue**, enter text,
select **Low**, **Normal**, or **High**, and click **Add job** for each job. Start the kernel and click **Run queue** once.
The worker submits one job and waits for a matching completed guest snapshot
before starting the next. The panel displays guest IDs, processed bytes, and
observed counts. Only completed jobs label their count as a result.

- Choose the highest priority pending job: High, then Normal (the default), then
  Low. Equal priorities keep insertion order. Priority is captured when adding;
  changing the selector affects future additions, not existing jobs.
- A newly added high-priority job waits for active work to complete. No preemption,
  CPU-priority changes, deadlines, aging, or measured throughput improvement.
  With this finite eight-entry list, all pending jobs can drain if left running.
- Rows stay in insertion order so their numbers remain stable. Guest IDs and
  Activity reveal execution order; row number is not execution rank.
- Maximum eight entries, including finished history, each at most 192 printable
  ASCII bytes. Empty input is allowed and completes with zero words.
- **Pause queue** stops new submissions once handled by the worker; an active
  job finishes normally. It is not cancellation and cannot undo a submission
  already sent. **Run queue** resumes pending work explicitly.
- **Clear queue** removes the in-memory list only when paused with no active job.
  Additions are allowed while running; pending selection uses their priorities.
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

- `host/panel/src/coordinator.rs`: bounded inputs, priority selection with FIFO ties, ownership, pause,
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
execution, and observation would remain explicit. Resource budgets,
multiple job kinds, permissions, isolation, and actual Meta integration need their
own increments; this queue does not implement those guarantees.

## Why the coordinator is currently on the host

The Mac hosts this temporary development service because the guest currently has
one cooperative operation and no isolated service runtime. The placement lets us
exercise policy, real execution, and observation without making all coordination
part of the kernel's privileged code. It is not a requirement that coordination
remain on macOS or that the eventual OS depend on a Mac.

Target responsibilities:

- Agents interpret goals/events and propose allowed work. A model is one possible
  planning component; it does not get unrestricted execution authority.
- A coordinator validates and orders requests, tracks ownership and outcomes.
- Executors implement specific operations and return observable results. Today's
  word counter is a privileged kernel operation, not an isolated executor service.
- The kernel supplies execution mechanisms and enforces actual isolation and
  permissions once those mechanisms exist. Job dispatch priority here is distinct
  from a future kernel CPU scheduler.

To run the coordinator as an isolated service inside RAI-OS we still need resumable exception
handling, memory/address-space management, a user-mode execution path, scheduling,
system calls, and IPC. Copying it into a kernel module would be possible as an
intermediate experiment, but would share kernel memory/privileges, not supply
service isolation. The current host implementation uses String/Vec and std; moving
it requires adapting storage/runtime and transport, not merely moving a file.
A guest-native bounded queue could be a smaller explicit milestone if autonomous
execution without the host becomes the next priority. No such migration happened
in this increment. [Fatal EL1 diagnostics](EXCEPTIONS.md), added October 27, are
a first kernel prerequisite; they do not yet support returning to a service. The model API bridge remains on the host for the initial demo.

## Verification

`python3 scripts/verify_panel.py` passes fourteen tests: ten pure host tests and
four real-QEMU tests. Six coordinator tests cover priority/FIFO selection, an urgent
arrival during active work, pause/resume, FIFO waiting, mismatched
IDs, unknown submission outcomes, bounded/invalid text, manual ownership, and empty
completion. The FIFO live test checks ordered guest IDs and counts (2, 3, 0), manual
mutation rejection, pause without starting the next job, Stop during work, reboot,
and explicit continuation without replaying interrupted work. Existing lifecycle,
clock, echo, task cancellation, and missing-kernel checks still pass. A priority
live test enqueues Low/High/Normal/High and observes guest IDs 4/1/3/2 in those rows,
with independently specified expected counts 1/2/3/4.

Debug/release panel builds, formatting, and all-target panel Clippy passed. Native
window review checked Add, Run, Pause, Resume, observed results, and cleanup.
No kernel changes required repeating the unchanged kernel test suites.

Retained [test output](evidence/2026-10-26-coordinator-tests.txt) and
[native window capture](evidence/2026-10-26-coordinator.png).

Priority increment evidence: [test output](evidence/2026-10-26-priority-tests.txt)
and [native window](evidence/2026-10-26-priority.png). The earlier FIFO evidence
above is retained unchanged.
