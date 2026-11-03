# Composable application behavior

An application can now declare small programs assembled from reusable operations.
This expands the language that Meta can use, rather than adding another built-in
business application. The same Rust executor handles every accepted definition.
The initial event is an explicit button press on an existing saved record.

The kernel's existing boot, memory, exceptions and execution foundations remain
the platform underneath this interpreter. This does not make the kernel complete
or frozen forever. App runtime, controller and renderer still share EL1 privilege;
this increment does not wire them into isolated EL0 processes or add device drivers.

## What a button can express

A records.local schema may have a behaviors array with at most two buttons. Each
button has id, label, event: press and one to six ordered steps. A step has op,
field, value, compare and skip. Keys reference fields in that same app and row.

| Operation | Meaning |
| --- | --- |
| set | Write a typed literal into an editable field. |
| calculate | Evaluate checked integer arithmetic and write an editable number. |
| toggle | Reverse a checkbox value. |
| require | Abort the entire button if a typed comparison is false. |
| if | If a typed comparison is false, skip the next N steps; otherwise continue. |

Numeric comparisons support eq, ne, lt, le, gt and ge. Text and checkbox
comparisons support eq and ne. Compare values are typed literals, not expressions.
set values must fit the field's ordinary 32-byte value limit and validation rules.
Calculated fields cannot be written directly. calculate targets an editable number
and accepts the existing 64-byte bounded arithmetic language. Unlike a calculated
column, a button expression can reference any numeric/calculated field in the
current row: this is sequential execution over existing values, not a new column
dependency cycle. Signed i32 overflow, missing input and division by zero fail.

For non-comparison operations compare is empty and skip is zero. toggle also has
an empty value. require has skip zero. if must skip 1..N following steps within
the same button; there is no backward jump, unbounded loop or recursion. Every
button must contain at least one mutation instruction. All instructions are
validated, including branches that would be skipped for current data.

## A generic composition example

The following is an illustrative button, not a built-in runtime case:

```json
{
  "id": "move",
  "label": "Move one",
  "event": "press",
  "steps": [
    {"op":"require","field":"stock","value":"1","compare":"ge","skip":0},
    {"op":"calculate","field":"stock","value":"$stock-1","compare":"","skip":0},
    {"op":"calculate","field":"moved","value":"$moved+1","compare":"","skip":0},
    {"op":"toggle","field":"flag","value":"","compare":"","skip":0},
    {"op":"if","field":"stock","value":"2","compare":"lt","skip":1},
    {"op":"set","field":"status","value":"low","compare":"","skip":0}
  ]
}
```

This moves one count between two fields in the same row. It is not a transfer
between records, apps or inventory tables. A calculated total = stock + moved
is refreshed locally. If stock becomes less than two, status becomes low;
otherwise that final set is skipped. This same vocabulary can express other
bounded button behavior without adding application IDs to the kernel.

## Preview, atomic execution and validation

Open a saved row and tap **do**, or press F10. Choose a button to preview its
effects; the program itself is reviewed at installation. A preview changes neither
data nor the runtime revision.
Confirm with the bottom button or Enter to execute; leaving the preview cancels
that approval. Direct execution without the matching preview is rejected.

Execution checks the app, saved row, button and revision. Each step operates on a
staged copy of the row; numeric derived columns are refreshed after each mutation,
so subsequent steps see consistent intermediate values. An error at any step
leaves all original values and the revision unchanged. A successful confirmation
commits the final row and advances the revision once. This transaction scope is
one row, not cross-record/database transactions or durable-disk atomicity.

The UI exposes the failing instruction number and error. Step numbers start at
one; admission failures (for example a stale revision or terminal workflow lock)
use zero. These are deterministic validation/execution reports, not an assertion
that a generated app fulfills every natural-language requirement. Reviewed tests
and expected results still matter.

Existing workflow states remain separate. Buttons do not write those states or
bypass their terminal edit lock; normal workflow actions still control them.
The prototype trusts manually entered data and the host checkpoint owner.

## Creation, updates and persistence

Meta receives the versioned runtime catalog and the structured output contract.
A proposal is validated on the host, then again against actual guest state.
Fields are reviewed first, then the optional workflow, then every behavior button
before installation. There is no auto-execution on installation or recovery.

A compatible app update can change behavior definitions after review while
retaining existing field identity, order and types. Existing workflow definitions
remain immutable. Saved rows are checked by the existing migration path before
commit. Snapshot recovery validates and restores the schema with its behaviors;
apps saved before this extension have an empty behavior collection by default.
No model call is needed for preview, execution, ordinary editing or recovery.

This slice adds no automatic repair loop. Rejected model proposals are not
silently rewritten or retried. A future bounded propose -> validate -> report ->
correct loop must preserve validation authority, request/revision binding and
clear attempt/cost limits. Formal validity alone cannot prove user intent.

## Wire and code map

- microapp_behavior.rs: generic bounded programs, typed conditions and evaluation.
- microapp.rs: schema validation, row preview and atomic confirmed execution.
- app_ui.rs/display/apps.rs: staged review, saved-row button selection and preview.
- host/meta/src/apps.rs/provider.rs: untrusted JSON, catalog and Meta contract.
- scripts/apps.py/app_storage.py: proposal transport, readback and checkpoints.

Proposal staging adds app.button `request:id:label_hex:press` and app.step
`request:button_id:op:field:value_hex:compare:skip` before app.ready. app.schema
adds behaviors: buttons separated by commas, each `id~label_hex~press~steps`;
steps separated by /, each `op^field^value_hex^compare^skip`. Empty means none.

app.behavior.preview `revision:button_id` operates on the selected saved row in
the behavior list. app.behavior.run requires the matching preview and revision.
app.behavior.status returns step and error. These UART diagnostics belong to the
trusted bridge/test harness; they do not grant a generated app arbitrary access.

## Independent verification

```sh
python3 -B scripts/verify_app_behaviors.py --debug --output target/behaviors-debug
python3 -B scripts/verify_app_behaviors.py --output target/behaviors-release
# Portable offline check after rebuilding the image:
docker run --rm --network none rai-os-demo apps-behaviors-verify
# Optional: clean committed source, one live request, no retry/fallback.
python3 -B scripts/verify_app_behaviors.py --source meta --output target/behaviors-live
```

Offline mode uses two disclosed verifier-only schemas. The first independently
checks stock 3 -> 2 -> 1 and moved 0 -> 1 -> 2, a conserved total of 3, both branch
outcomes, checkbox toggle and text set. Another button writes a staged value then
fails a later require; no value or revision may change. A reviewed update adds a
late overflowing calculation to verify the same rollback behavior. Static tests
reject invalid skipped instructions, derived-field writes, invalid branch spans
and malformed values. A different text/boolean app shares the executor and must
not modify the first app; closing its workflow blocks further row behaviors.
Fresh QEMU must recover exact definitions, values and workflow state.

The optional live path requests a smaller move program and checks actual returned
behavior, no installation before review, preview without mutation, one-revision
commit, required-condition failure and recovery. The goal and expected data are
fixed before the call; actual field/button identities are read from the proposal.
A failed request is retained rather than retried until it passes.

Observed execution results belong in WORKLOG and retained report.json files.
The existence of this verifier is not a claim that it has passed every environment.

## Remaining limits

Two buttons of six instructions, six fields, eight records, three apps, printable
ASCII and integer arithmetic remain bounded prototype choices. There is no
background event delivery, timer, loop, cross-record access, general layout
language, network, location, camera, native executable or new privilege boundary.
A records.local behavior cannot silently acquire display.brightness or another
capability. These primitives extend app behavior; they do not create drivers or
make arbitrary untrusted code safe to run with kernel privileges.
