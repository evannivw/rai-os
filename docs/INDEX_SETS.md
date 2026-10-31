# Complete versioned index sets

Implemented October 30, 2026. A retained index session can now require up to four
exact document revisions and publish one inline artifact containing their source
bytes and individual indexes. Changing the requirement immediately invalidates
publication of the previous set for the new goal. Committed indexes for unchanged
revisions remain available; only missing revisions need worker execution.

This is the next correctness slice toward the review-pack contract. It does not
implement the timed release/outage/resource-budget scenarios, optional previews,
a separate pack-version/reference format, or all R1-R6 acceptance gates.
`state=complete` and the native SET COMPLETE label mean every required revision
has a committed index. They are not an independent semantic verification verdict.
The separate checker verifies exact source bytes and token frequencies afterward.

## Watch and inspect

From the repository root:

```sh
. "$HOME/.cargo/env"
sh scripts/bundle.sh --graphics
# Or run without the window and intentional presentation pauses:
sh scripts/bundle.sh
```

The Rust host walkthrough starts real QEMU, sets each requirement, asks the guest
which records are missing, submits ordinary per-record proposals for those records,
and retrieves the complete artifacts. It is a scripted deterministic walkthrough,
not Meta inference or a performance comparison. It does not combine source data
or construct an index on the host. The native screen comes from the unchanged
framebuffer path inside RAI-OS, with a new observed set-progress view.

The demo creates a new timestamped directory under target containing goal-1.json,
goal-2.json and goal-3.json. Each is the exact guest payload and passes the independent
checker before the demo reports publication. Existing directories/files are never
overwritten. The window closes after explicit session cleanup; snapshots remain
available in the output directory. There is no new touch interface.

| Requirement in this fresh run | Existing valid entries | New worker jobs | Total worker jobs |
| --- | --- | --- | --- |
| brief@1, empty@1, notes@1 | 0 | 3 | 3 |
| Add checklist@1 | 3 | 1 | 4 |
| Replace notes@1 with notes@2 | 3 | 1 | 5 |

The empty document remains in every required set and artifact. notes@1 remains
an immutable cached revision but cannot substitute for notes@2 in the final set.
The other three entries in that final set match their previous bytes and indexes.
These job counts demonstrate reuse in this fixture, not an AI scheduling advantage.

## Console commands

Use `cargo run --locked`, `sh scripts/console.sh` or `sh scripts/desktop.sh`.
A fresh boot can reproduce the first set with:

```text
coord.start index
coord.goal brief@1,empty@1,notes@1
coord.bundle.status
coord.observe brief@1
coord.propose 2:brief@1
coord.observe empty@1
coord.propose 3:empty@1
coord.observe notes@1
coord.propose 4:notes@1
coord.bundle
```

`coord.goal <comma-separated-keys>` replaces the required set and returns its
new goal_version, state, canonical required list, ready_records, total_records,
session completed-job count and missing list. `missing=-` means no missing record.
`coord.bundle.status` reads the same status without running either EL0 task.
`coord.bundle` returns an artifact only when all required cache slots are present;
otherwise it returns coord_bundle_incomplete, never an earlier completed set.
These commands require an active index-mode coordinator session.

Continue the example with `coord.goal brief@1,empty@1,notes@2`. It reports two
ready entries and only notes@2 missing. Observe notes@2, use the returned ticket
in its proposal, then retrieve coord.bundle again. Set changes and observations
share the same monotonic u32 issue counter. The numbers in the example assume
exactly the displayed commands in a fresh boot. Always use reported tickets in
other sessions; rejected/duplicate observations can also consume issue numbers.
The full three-stage walkthrough has goal versions 1, 5 and 7. Those are runtime
generations, distinct from the reference fixture's goal identifiers 1, 2 and 3.

A set contains one to four allowed revisions, at most one revision per logical
document. Input order is normalized lexically. Empty sets, duplicate keys,
conflicting notes revisions, denied/unknown records and extra arguments fail
without replacing the active requirement. Once a set is active, observations or
proposals outside it return coord_not_required without executing B. Setting even
an identical set creates a fresh generation and clears the active per-record
goal. A delayed old proposal gets no_goal before a new observation, stale_proposal
after a newer ticket is active, or not_required if its revision is excluded.
No stale proposal executes. Only stop/start returns to unrestricted index mode.

## Ownership and publication

EL0 coordinator A retains the required-ID bitset in its own execution state. A
new host input kind 32 carries the issue number and validated bitset. A clears
its active per-record goal while retaining its completed-revision set, then
attests the new requirement through coordinator-only SVC 15 and publishes event 9
(goal_changed). A refuses non-required record inputs with event 10 (not_required).
The existing individual index construction, cache policy and IPC chunks remain.

SVC 15 takes x0=generation, x1=required mask, x2=completed revision mask; it returns
zero in x0-x3 through the existing scalar ABI. The kernel requires exact agreement
with the pending goal input and its committed cache slots. Duplicate/unsolicited
attestations are rejected. The requirement becomes visible only after one matching
publication and both tasks waiting. Trace G denotes this attestation; a normal
set change makes three A calls and zero B calls, with AG,AO,AH apart from possible
timer preemptions. No new user pointer, writable mapping or process is introduced.

The kernel stores the validated requirement metadata and computes readiness from
its committed slots. It also refuses a worker dispatch outside that set. Serialization
and the bounded copied artifact run at EL1. Per-record index work remains at EL0;
linked source bytes are read-only public fixture data. This is not a new isolated
pack builder or general-purpose filesystem service. The host supplies requirements
and chooses missing-job order; automatic Meta planning for an entire set remains
future work. Existing Meta count/index modes do not send coord.goal and still work.

A successful artifact is an owned copy of at most four committed 384-byte indexes,
plus goal metadata. Serialization also reads the fixed read-only source data.
There is no heap allocation. This enlarges the bounded reply variant; the explicit
Clippy allowance documents its use on the existing 64 KiB kernel stack. It is not
a measured whole-kernel stack-peak or memory-efficiency claim. The six-slot cache,
private EL0 stack pages, twelve owned MMU table pages and 244 free pool pages remain.

Set changes retain cache entries but discard eligibility of old publications for
the new generation. Fatal session errors/stop clear the requirement, cache and
saved tasks. The issue counter survives stop/start and never wraps. Reboot resets
it. Previously exported snapshots remain inspectable historical artifacts; the
checker rejects them against a different current requirement or generation.
Bundle reads do not increment the per-record cache_hits counter or worker jobs.

## Artifact and independent checker

The single-line rai-index-set-v1 JSON has exactly format, goal_version, required
and records fields. required is a lexically sorted list of exact keys. Each records
entry has exactly record, bytes, source_hex and index. Entries are in the same
order and correspond one-to-one with required. Source bytes use canonical lowercase
hex to preserve LF, punctuation and empty content without injecting UART framing.
Indexes retain the existing canonical token:count representation; `-` is empty.
There are no external/dangling references or hidden optional entries in this format.

```sh
python3 scripts/check_bundle.py target/your-demo-directory/goal-3.json \
  --required brief@1,checklist@1,empty@1,notes@2 --goal-version 7 --find safety
python3 scripts/verify_bundle.py
python3 scripts/verify_bundle.py --release
```

The optional `--find safety` lookup reports `checklist@1=1, notes@2=1` for the
final fixture goal. Lookup runs on the host against the verified guest artifact;
the index entries were produced by the EL0 worker.

The checker takes the expected set and generation separately. It bounds input to
4096 bytes, rejects duplicate JSON fields, requires the exact record set/order,
checks fixture hashes and every source byte, and independently recalculates all
index frequencies. It returns combined token-to-record postings for comparison
with the existing expected.json reference. Tests cover all three reference goals,
unchanged entries, empty documents, missing/duplicate/stale/changed data, altered
indexes and generation mismatch. This checks content integrity/completeness, not
the unimplemented timing, release, outage or simulated-resource contract.

Real-QEMU tests additionally cover all 45 valid subsets of one to four public
revisions with no conflicting document revisions, incomplete publication, record
scope, old proposals, cache reuse/reset and original memory ownership. Dedicated
images reject a forged EL0 goal attestation and a truncated notes@2 stream after
an earlier complete set. Native captures distinguish building, complete, revision
change, explicit incomplete-output error, restored status and stop. No live Meta
request is necessary for this increment.


## Recorded evidence

Actual [debug UART](evidence/2026-10-30-bundle-debug.txt) and
[release UART](evidence/2026-10-30-bundle-release.txt) captures preserve the full
normal tests. The three complete [initial](evidence/2026-10-30-bundle-goal-1.json),
[expanded](evidence/2026-10-30-bundle-goal-2.json) and
[revised](evidence/2026-10-30-bundle-goal-3.json) artifacts are exact guest outputs.
[Pending revision](evidence/2026-10-30-bundle-revision-pending.png) and
[complete set](evidence/2026-10-30-bundle-final-complete.png) screenshots were
captured from QEMU; those frames matched byte-for-byte between debug and release.
A separate run of the public graphical walkthrough also exported and independently
verified all three files, then stopped its own guest session/window.


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

## October 31: native publication distinction

Active required sets now use the [publication view](PUBLICATION_DISPLAY.md).
Completeness by itself does not display CURRENT; only an acknowledged publication
does. The original captures and bundle exports above retain their actual meaning.
Use the new publication walkthrough for prepare/check/commit, historical output
and failed-update presentation.
