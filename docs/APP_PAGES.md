# Composable visual pages

RAI-OS apps can declare two pages, each containing one or two ordered blocks:
text, metric, bars or a two-column table. The same guest renderer and data evaluator
serve every app. Pages are read-only views over that app's records; ordinary forms,
calculated fields, workflow actions and behavior buttons remain available.

This is a foundation for small dashboards, reports and data-backed presentations.
It is not Excel/PowerPoint compatibility, arbitrary native apps, a general layout
engine or a promise that every requested application is expressible. Page text is
part of the app definition and changes through reviewed evolution, not a slide editor.

## Try it

Open a live Studio using your own key as described in [DEMO_READY.md](DEMO_READY.md).
These prompts fit the current 128-character ASCII input limit:

```text
App: name, amount. Two pages: sum amount and bars by name; text "My report" and a name/amount table. No workflow.
App: sample, reading. Pages: min and max reading; bars by sample and a sample/reading table.
App: item, qty, unit_cents, total=qty*unit_cents. Pages: sum total and bars by item; notes and an item/total table.
```

Read the proposed fields, process/buttons if present, and each proposed page before
approval. Enter data using the existing forms. Tap `pages >` under the app heading
or press F11. Previous/Next (or Left/Right) switch pages. Records/Table remain in
the app toolbar. Editing a record updates aggregates, bars and tables locally,
without another model request. Empty input has an explicit representation.

The model may clarify or decline. A valid declaration is not proof of the user's
intent; check actual results. Generation keeps the existing bounded correction
policy. Corrections may fix eligible formula expressions but cannot change page
structure to bypass a failure. Saving/restoring with [Git](APP_GIT.md) includes the
pages in the definition and preserves current records.

## Contract

The optional `pages` array defaults to empty for existing app definitions. Each
page has `title` (1..24 printable ASCII bytes) and `blocks` (1..2). Page titles are
unique within the app. Every block has all six properties below:

| Property | Meaning |
| --- | --- |
| kind | text, metric, bars or table |
| title | A nonempty display label, at most 18 ASCII bytes |
| value | Literal text for text blocks; a field key for numeric metrics, bars and tables; empty for count |
| label_field | A field key for bars/tables; empty otherwise |
| aggregate | count, sum, min or max for metrics; empty otherwise |
| scope | all or pending; text requires all |

`text` content is at most 64 printable ASCII bytes and is displayed literally.
There is no markup, executable expression or network URL interpretation. `metric`
requires a number/calculated field for sum/min/max. Count counts selected records,
including records with blank numeric values. Numeric aggregates omit blank cells.
Sum/count on empty input are zero; min/max without numeric values display
unavailable, never a fabricated zero. At most eight i32 values sum into i64.

Bars require a numeric value field and any existing label field. A zero baseline
separates negative and positive values. Lengths use the maximum absolute value of
the selected records; actual signed numbers are printed beside every bar. This
scale changes with the data and is not a fixed cross-page comparison scale. Very
small nonzero values may round to zero pixels; their numeric labels remain visible.
Tables bind two existing fields and show all selected rows. The full values remain
available in the record form; long table/chart labels may be clipped visually.

`pending` requires the app's existing checkbox filter and selects false values.
A page block declares its scope explicitly; it does not silently inherit the
currently selected list filter. Pages cannot access another app, write records,
trigger actions, call Meta or acquire display/device permissions. Brightness apps
cannot declare pages. Record editing and process locks keep their existing rules.

Example reusable declaration (the field keys must exist in the owning app):

```json
{
  "title": "Overview",
  "blocks": [
    {"kind":"metric","title":"Total","value":"amount","label_field":"","aggregate":"sum","scope":"all"},
    {"kind":"bars","title":"By item","value":"amount","label_field":"name","aggregate":"","scope":"all"}
  ]
}
```

## Validation, updates and persistence

Meta receives catalog v8 and a strict output schema. Host and guest independently
validate page counts, labels, field references, types, aggregates, scope and app
capability. Each page has a native review step after fields/workflow/behaviors.
No page or app is installed until the final approval. Cancellation retains the
previous definition. Compatible changes may replace/remove pages while retaining
current data and the established field/workflow/capability rules.

Checkpoint recovery validates the complete definition and follows the same guest
admission path. Host-driven recovery of a previously saved device supplies its
review acknowledgments as it already does for other saved app components. This is
trusted host restoration, not a model bypass of live user approval. Old schemas
omit pages and retain their previous wire format and behavior.

## Code and protocol

- `src/microapp_pages.rs`: pure composition, binding and aggregate semantics.
- `src/app_ui.rs`: staging, page review, navigation and input hit regions.
- `src/display/app_pages.rs`: actual guest framebuffer rendering.
- `host/meta/src/apps.rs` and `provider.rs`: JSON contract and model context.
- `scripts/apps.py` and `app_storage.py`: transport and explicit host persistence.

Staging uses `app.page request:title_hex` followed by
`app.block request:page_index:kind:title_hex:value_hex:label_field:aggregate:scope`.
Field/filter declarations are checked with the complete spec at app.ready.
`app.schema` adds pages only for nonempty declarations. Titles/text are hex encoded
so punctuation cannot become wire delimiters. The read-only diagnostic
`app.page.values app_id:page_index` reports metrics in block order (`-` for a
nonmetric block; `empty` for unavailable min/max). It does not mutate state.

The renderer/interpreter still share EL1. These are modular responsibilities,
not isolated processes. No new unsafe, device driver or kernel memory mapping is
introduced. The development profile uses one compilation unit without incremental
code generation to keep code inside the existing MMU window; symbols and opt-level
1 remain. Schema output borrows its large Spec rather than copying it onto the
fixed stack. Exact observed size/frame information belongs in retained evidence.

## Verification

```sh
python3 -B scripts/verify_app_pages.py --debug --output target/pages-debug
python3 -B scripts/verify_app_pages.py --output target/pages-release
docker run --rm --network none rai-os-demo apps-pages-verify
# Separate clean-source integration check: at most one real Meta call.
python3 -B scripts/verify_app_pages.py --source meta --output target/pages-live
```

Offline proposals are disclosed fixtures. The verifier checks all four block kinds,
independent sums/filter results, eight records, signed bar pixels, navigation without
mutation, editing/recomputation, cancellation, reviewed evolution, invalid bindings,
an unrelated min/max app and fresh-guest recovery. Live mode uses the actual model
proposal, one call and no retry/fixture substitution. Actual outcomes belong in
WORKLOG and evidence; these commands alone do not establish a successful live run.
