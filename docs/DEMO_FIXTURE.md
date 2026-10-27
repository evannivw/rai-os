# Review-pack correctness fixture v1

Created October 26, 2026. These are original synthetic English records under the
repository license, with no personal data or credentials. This is a tiny public
correctness fixture, not a performance benchmark or held-out evaluation set.
No policy, job engine, Meta integration, or independent candidate-result verifier
is implemented by this increment. The existing kernel remains help/info/uptime/echo.

## Files and reference results

The fixture lives in [fixtures/review_pack/v1](../fixtures/review_pack/v1).

- [catalog.json](../fixtures/review_pack/v1/catalog.json): identities, immutable
  revisions, permissions, paths, exact byte lengths, and SHA-256 digests.
- `records/`: seven source revisions across six document IDs, including an empty
  required document, two revisions of notes, one optional document, and one denied
  synthetic document. Text uses ASCII and LF, including the final LF except for
  the truly empty document. Never normalize bytes when loading or comparing.
- [expected.json](../fixtures/review_pack/v1/expected.json): exact required sets,
  complete index maps for three goals, one exact 64-byte optional preview with
  weight 1, and state expectations after each event. These are reference data,
  not captured successful runs, manifests, or publication evidence.
- `scenarios/`: ordinary, changing, and blocked traces, all with a 60,000 ms
  deadline. The blocked trace intentionally never restores the missing revision.

Keys such as `notes@2` denote `(record_id="notes", revision=2)`. They are fixture
identifiers, not a proposed UART protocol. Catalog revision 1 remains immutable
when revision 2 arrives. An empty source must remain in the required set even
though it contributes no index postings. The denied record must never enter a
required pack, preview, or model prompt.

Goal 1 requires `brief@1`, `empty@1`, and `notes@1`. Goal 2 adds `checklist@1`.
Goal 3 replaces `notes@1` with `notes@2`. Every goal permits the same optional
`appendix@1` preview. Other catalog records are not implicitly eligible outputs.

Manual reference anchors: `Pump A: pump 7.` in notes revision 1 gives `pump: 2`,
`a: 1`, and `7: 1`; revision 2 instead includes `b` and `8`. In goal 3, `safety`
has one occurrence each in `notes@2` and `checklist@1`. The empty record contributes
no tokens. Full expected maps use the acceptance contract's ASCII tokenization.

## Changing trace

Times are offsets from scenario start, not pauses for the fixture checker.

| Time | Atomic event | Expected contract state |
| --- | --- | --- |
| 0 s | Release initial records; activate goal 1 and initial limits. | Three required records; 64-byte scratch cap. |
| 10 s | Release checklist and activate goal 2. | Four required records; an earlier pack is stale. |
| 20 s | Release notes revision 2 as unavailable; activate goal 3. | Revision 1 cannot satisfy goal 3; new required content cannot yet be read. |
| 25 s | Lower scratch cap to 16 bytes. | Future work must use smaller bounded buffers. |
| 30 s | Restore access to notes revision 2. | Required inputs are now available; this does not itself mean READY. |
| 40 s | Lower cumulative simulated work cap from 16,384 to 8,192 units. | Work already spent remains charged. |
| 60 s | Evaluate deadline. | Only verified goal 3 output can pass. |

The ordinary trace activates goal 1 at time zero without interruptions. The
blocked trace matches the changing trace but omits recovery at 30 s. Goal 3 then
cannot be completed through legal reads. A final success claim must be rejected;
report the unavailable required revision instead. Do not remove this case from
reports or make it solvable only for one policy.

Reference snapshots describe inputs/constraints, not policy progress. An outage
means the read adapter fails for that revision; it does not revoke permission or
corrupt immutable bytes. In these traces the new revision is unavailable from its
first release, so no legal earlier cached copy exists. General future outages
need not invalidate already validated copies unless permissions/revisions change.

## Harness visibility and event order

The trusted future harness/verifier can read the full fixture. Policies must see
only released metadata, current goal, current limits, and observations available
at that time. Catalog paths, future bodies/events, and expected results are never
included in policy prompts. A denied record may appear as denied metadata, never
as a body. Keep policy access behind the same adapter for both strategies.
This is a test-harness boundary, not OS isolation from a malicious host process.

Each event applies its release, availability changes, goal, and limits as one
state transition. Goal versions increase only when goals change; the future engine
must increment state_version for every event. At a shared timestamp, apply all due
events before accepting another operation or publishing. Old goal/state proposals
must be rejected or revalidated by the engine. There is no future-event lookahead.

The checker replays events instantly without sleeping. A future live runner must
use elapsed monotonic time, including model latency; it must not pause events
while awaiting a model. A deterministic virtual-time test must label its clock and
cannot report its simulated duration as measured speed. Record final goal
activation (20 s for changing/blocked) separately from completion since start.

## Resource model for the forthcoming runner

These fixture values are test constraints, not measurements of battery or RAM.
The following accounting must be implemented and tested before claiming R6:

- Scratch cap: sum of live temporary job buffers. A job processes a bounded chunk
  and releases scratch at each cooperative boundary. If an event becomes due
  during a chunk, that chunk may finish under its admitted limit and release its
  scratch; apply due events before admitting the next chunk or publication. No
  worker may retain a buffer across a boundary to evade the lower cap.
- Work cap: cumulative simulated units, not a replenishing balance. Charge one
  unit per source or derived payload byte scanned/copied by each operation and
  one unit per attempted operation. Copying and subsequently indexing the same
  source charges both passes. Preview work charges only consumed prefix bytes.
  Validation/rework also charge their actual payload reads. Reserve the complete
  bounded step charge before execution. Denied/invalid attempts consume one unit
  without reading protected data when that unit can be reserved. If even the
  attempt charge cannot be reserved, log admission rejection without executing
  or charging the operation. No operation may overdraw remaining units.
- A lower work cap never resets usage or retroactively makes earlier legal work
  illegal. If usage already meets/exceeds the new cap, admit no more charged work;
  report budget exhaustion when unfinished requirements cannot complete. Include
  old/new caps and usage in the log. A valid already-published pack may remain valid.
- Output cap: fixed 16,384 bytes for retained job result payloads, including indexes,
  manifests, previews, partial results, and cached/previous pack payloads. Charge
  the exact serialized byte length and count duplicate retained payloads separately;
  reject growth before allocation. Source fixture storage is read-only and excluded.
  This does not measure allocator/object overhead or all kernel memory. The next
  artifact-format increment must define canonical serialization before execution.
- Host/model resources are outside these guest job budgets and must be reported
  separately, including wall latency, calls/tokens, and known API cost. Real hosted
  inference energy is unknown, not zero. Verifier audit cost is reported separately
  from the engine's charged validation work.

A source outage, low scratch limit, and exhausted work budget have distinct causes.
Neither strategy may waive limits or silently change required output. These rules
are a specification only; the current checker does not execute jobs or audit usage.

## Check the fixture

From the repository root, using Python 3's standard library:

```sh
python3 -B scripts/check_review_fixture.py
python3 -B -m unittest discover -s scripts/tests -v
```

The checker validates source digests/lengths, permissions of goal references,
exact expected indexes/previews, event ordering, release-before-use, and expected
state transitions. Its byte scanner is separate from the regular-expression
calculation used to initially prepare index references. The reference anchors
above make representative counts reviewable. This is useful consistency evidence,
not proof that the specification itself has no mistakes.

Six tests cover the checked-in fixture and five corruptions: same-length changed
source, wrong index count, missing release, wrong outage checkpoint, and an event
at the deadline (events must precede evaluation). Tests mutate disposable copies,
never committed sources. A valid publication timestamp may equal the deadline;
results published later do not qualify.

Keep v1 identifiable in Git. Intentional future contract changes need a new fixture
version. Benchmark-scale data, varied seeds, adversarial/held-out cases, calibrated
budgets, and model/baseline comparisons remain separate future work.

## Small implementation commits after this fixture

Each increment must be checked, documented, committed, and pushed separately:

1. **Define candidate artifacts and audit records.** Use Rust host types for packs,
   manifests, references, publication events, and execution records. Specify exact
   byte encodings, version fields, timestamps, and size/work accounting. Include a
   hand-authored valid candidate and a minimal audit trace; neither is kernel output.
2. **Implement the independent result verifier (R1-R4).** Separate Rust host tool
   reads trusted inputs and untrusted candidate output. Check the exact required
   set, bytes, index, and same-pack manifest references. Do not import the guest's
   tokenizer, pack validator, or policy success flag. Test missing/duplicate/extra
   records, stale revision, changed byte, wrong index, and dangling/cross-pack links.
3. **Verify publication and execution evidence (R5-R6).** Replay goal/state changes,
   availability, permissions, charged work, and buffer allocation/release from
   engine-owned logs. Check source snapshots and deadlines. Reject partial/stale
   READY, missing evidence, forbidden successful reads, and over-budget steps.
   Logs supplied by an untrusted engine alone cannot prove hidden accesses never
   happened; the controlled harness must observe access/resource boundaries.
4. **Connect the deterministic host executor.** Produce actual candidate results
   and audit traces for all three scenarios, then run the verifier. Ordinary and
   changing should be feasible; blocked must never report final success. Capture
   evidence. This is host pilot evidence, not a QEMU implementation.
5. **Connect guest jobs and policy comparison incrementally.** Verify real UART/QEMU
   execution before claiming OS behavior. Freeze competitive baselines, then add
   the Meta coordinator with identical allowed actions and observations. Compare
   only after correctness gates pass; publish overhead, failures, and held-out
   outcomes as specified in [DEMO_EXPERIMENT.md](DEMO_EXPERIMENT.md).

A graphical UI can display the verifier's structured verdicts later. A green
indicator must represent actual checked evidence, never model-written confidence.
