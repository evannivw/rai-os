# Dynamic calculated fields

An app's schema can now describe its own arithmetic. The runtime does not know
about quotes, travel, inventory or budgets: it only knows fields and validated
operators. Meta chooses field names and formulas from the user's prompt. After
approval, editing, calculation, table totals and recovery need no model calls.

Use the live launcher with your own prompt:

```sh
python3 -B scripts/apps.py --source meta --output target/formula-session-1
```

For example, request an app with item, quantity, integer unit price in cents and a
calculated total. Another app can calculate budget minus spending. These are
examples of the same primitives, not extra applications built into the kernel.
The replay branch deliberately supports only its documented offline fixtures;
`scripts/app_replay.py` is imported only by that branch. Unknown replay prompts
produce a clarification, never a hidden live call or a guessed app.

## Contract

A calculated field has kind `calculated`, required `true`, default `""`, and a
`formula` string, for example `$quantity * $unit_price`. Ordinary fields omit the
formula or use an empty string. Older saved schemas remain valid.

- Literals from 0 through 2147483647; signed results and input values use i32.
- Operators `+ - * /`, unary minus, parentheses and spaces. Standard arithmetic
  precedence; integer division truncates toward zero. Use integer cents for money.
- References use `$key` and must name an earlier number or calculated field in
  the same row. No self/forward/cross-app references, cycles, functions or loops.
- Maximum 64 ASCII bytes, 32 compiled instructions and 8 parser depth levels
  (parentheses and unary minus consume a level). No arbitrary code or I/O.
- At least one editable input, at most six total fields, eight rows, three apps.
- Missing numeric input, division by zero or overflow rejects a whole save/update.

Review shows the full expression. Calculated cells show `= auto` and cannot be
edited; keyboard navigation skips them. A preview appears once the complete form
is valid; `--` means it cannot yet be calculated. Save recomputes every derived
value, rather than trusting any previously displayed/stored result. Table totals
include calculated columns using i64 accumulation.

A reviewed update keeps existing field identities/order/types and editable data.
It may change formulas or append calculated fields. All existing rows must remain
valid before any schema/data/revision changes. Derived values are intentionally
recomputed when a formula changes. The model receives schema/record count, not
private row values; the guest can therefore reject a proposed migration that the
model could not evaluate. Host snapshots independently recompute and reject any
stored derived result that does not match. Restore sends only editable inputs,
then reads back all saved data exactly.

## Implementation and protocol

- `src/microapp_formula.rs`: pure compiler and checked stack interpreter.
- `src/microapp.rs`: dependency resolution, derivation and atomic row migration.
- `src/app_ui.rs`: read-only fields and `app.formula request:key:expression_hex`.
- `src/display/apps.rs`: review expressions, computed preview and table rendering.
- `host/meta/src/apps.rs`, `provider.rs`: strict schema and model instructions.
- `host/meta/src/snapshot.rs`: independent saved-value recomputation.

The formula staging frame follows app.field and precedes app.ready, keeping each
UART request within the existing 257-byte bound. app.schema now emits six parts
per field: `key~kind~required~label_hex~default_hex~formula_hex`. Update host and
guest together; older five-part wire consumers are not compatible. Saved JSON
without a formula remains compatible. app.value rejects computed fields with
app_read_only. Formula staging still requires the current pending request.

This runtime still shares EL1 privilege with the kernel. Formula validation is
interpreter enforcement, not process isolation. It does not compile arbitrary
Rust/native apps and does not introduce network, location or background tasks.

## Verify

```sh
python3 -B scripts/verify_app_formulas.py --debug --output target/formulas-debug
python3 -B scripts/verify_app_formulas.py --output target/formulas-release
# From clean committed sources only: at most two paid calls, no retries.
python3 -B scripts/verify_app_formulas.py --source meta --output target/formulas-live
# Rebuild the portable image first; offline and no credentials:
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm --network none rai-os-demo apps-formulas-verify
```

The independent expected rows include 3*125=375, 4*125=500, 5000-1250=3750 and
800-950=-150. The live checker uses two declared prompts absent from the replay
catalog; it checks generated results, editing, totals and recovery without another
inference. Offline checks additionally cover read-only access, overflow, invalid
references/syntax, cancelled staging, failed migration on a later row, and atomic
recomputation after a compatible formula change. Domain tests cover chained
calculations, precedence, depth limits, missing inputs and snapshot tampering.
These are development integration checks, not general model reliability or a
performance comparison. WORKLOG records actual outcomes and source provenance.
