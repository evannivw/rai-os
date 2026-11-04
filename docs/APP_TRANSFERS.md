# Atomic transfers between app records

A generated records app may declare one reusable transfer rule. The selected
saved row supplies a quantity and a destination key. RAI-OS previews both balances,
then applies both changes after explicit confirmation. The operation does not call
Meta. Field names, labels and app identities come from the declaration.

This adds a checked same-app lookup and two-row mutation, not a general relational
database, external payment system or arbitrary code execution.

## Generate and use

Use live mode and your own key as described in [DEMO_READY.md](DEMO_READY.md):

```text
App: name, balance, amount, to. Transfer amount from this balance to name=to atomically. Page: sum balance and bars by name.
```

Review the fields, transfer rule and page before installation. Enter two records:

| name | balance | amount | to |
| --- | ---: | ---: | --- |
| Warehouse | 12 | 4 | Dispatch |
| Dispatch | 5 | 1 | Warehouse |

Select Warehouse in records, tap `move >` or press F12. Preview must show 12 → 8
and 5 → 9. Apply commits both; Esc cancels. F11 opens pages: total stays 17 and the
bars show the new distribution. Change the source quantity to 99 and try again:
the guest rejects the operation and preserves both balances. These are fictional
example data, not built-in app types or real inventory transactions.

## Declaration

`transfer` is optional/null for older or unrelated schemas. When present it has
exactly these properties:

```json
{
  "id": "move",
  "label": "Move stock",
  "balance_field": "balance",
  "amount_field": "amount",
  "target_field": "to",
  "key_field": "name"
}
```

IDs and keys use the existing 1..12 lowercase identifier rule; label is 1..22
printable ASCII bytes. All four field bindings must be distinct editable fields.
Balance must be a required number, quantity a number, destination text, and lookup
key required text. Only records.local apps may declare transfers. There is one
rule per app, using the existing six-field/eight-record limit.

At preview and commit, the destination text must exactly equal one saved row's key.
Matching is case sensitive; missing, ambiguous and self destinations are rejected.
Keys are ordinary editable values, not stable foreign IDs: renaming a record can
break a later lookup. Duplicate keys are allowed as data but cannot resolve an
ambiguous transfer. Empty/whitespace-only destination is rejected.

Quantity must be positive i32; both balances nonnegative; source must be sufficient;
destination addition must fit i32. Terminal workflow state on either record blocks
the operation. Both complete candidate rows, including calculated fields, must
validate before any write. Success increments the global revision once. Failure
changes neither rows nor revision. Applying again requires a new preview; quantity
is not cleared automatically. Unrelated records and ordinary fields stay unchanged.

The controller requires a preview at the matching revision; leaving the preview
cancels its authority. The domain re-evaluates the operation rather than trusting
candidate values from the host. The current app runtime has one writer and no
interrupt writer. After validation, neither row assignment can fail. This is atomic
visibility in guest RAM, not power-loss durable storage or process isolation.
Host checkpoint failure is reported separately; a failed save cannot undo a completed
RAM operation. App interpreter and renderer still share EL1 with other modules.

## Boundaries and recovery

- Domain: `src/microapp_transfer.rs`, no I/O, allocation, new unsafe or device access.
- App controller: stages/reviews the rule; saved-row preview/confirmation and F12.
- Renderer: shows both original and proposed balances in the guest framebuffer.
- Meta: catalog v9 and strict JSON schema advertise only this implemented operation.
- Host: schema transport, checkpoint restoration and Git retain the rule. Definition
  rollback preserves current records and is subject to normal compatibility checks.
- No loops, arbitrary queries, cross-app transactions, background transfers,
  network payment APIs or native executable generation.

Protocol additions (all ordinary transport framing rules still apply):

```text
app.move.rule <request>:<id>:<label_hex>:<balance>:<amount>:<target>:<key>
app.move.preview <base_revision>
app.move.run <base_revision>
```

`app.schema` includes optional `move=id~label_hex~balance~amount~target~key`.
Existing schemas without a transfer keep their prior encoding. Install/evolution
reviews the rule after behaviors and before pages; recovery acknowledges the
already-saved definition without running it.

## Verification and demo

```sh
python3 -B scripts/verify_app_transfer.py --debug --output target/transfer-debug-new
python3 -B scripts/verify_app_transfer.py --output target/transfer-release-new
# Only from a clean committed source tree, with your own configured key:
python3 -B scripts/verify_app_transfer.py --source meta --output target/transfer-live-new
```

Offline verification uses a disclosed scripted proposal, real Rust validation and
real QEMU execution. Live mode permits exactly one provider call, no repair retry
or fixture replacement, and retains failures. Both save a report, UART traces,
screenshots and a pre-operation `showcase.json` with fictional data. Reopen a copy:

```sh
python3 -B scripts/demo.py saved --checkpoint target/transfer-live-new/showcase.json --output target/transfer-rehearsal-new
```

Saved mode performs no generation and never modifies the source checkpoint. A new
output directory is required. Resume via `scripts/demo.py resume --output PATH`.
See [DEMO_FINAL_RUNBOOK.md](DEMO_FINAL_RUNBOOK.md) for the recording sequence.


## Actual generated example

The [frozen-source live run](evidence/2026-11-03-app-transfer/live/README.md) passed
with one actual Meta call, no retries: Transfer Balances has four fields, the
transfer rule and a sum/bar page. Provider latency was 8119 ms, not an OS speed
comparison. The fictional data and arithmetic oracle were supplied independently.

From the repository root, reopen its pre-operation checkpoint with no key:

```sh
python3 -B scripts/demo.py saved --checkpoint docs/evidence/2026-11-03-app-transfer/live/showcase.json --output target/transfer-showcase-1
```

Connect a VNC viewer to localhost:5900. Open Transfer Balances > records > Warehouse
> move. The preview is 12→8 and 5→9; Apply, then F11 shows total 17. Select Warehouse,
replace Amount with 99, save and try move to see rejection. Use a new output directory
for each rehearsal; resume an existing one explicitly.

For Docker on Windows/Linux/macOS, build the image using [PORTABLE_RUN.md](PORTABLE_RUN.md). Copy
showcase.json into a fresh host state directory as device.json, mount that directory
at /state and run apps-replay; the same guest restores the saved app. The fixture
is data, not a source-code app implementation. Linux offline verification passed;
this specific saved actual result was verified on the local Mac QEMU run.
