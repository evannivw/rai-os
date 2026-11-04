# Building with the RAI-OS app toolbox

The creation mechanism is a small language for applications. Meta receives its
versioned [catalog](../fixtures/microapps/v1/runtime.json), the user's request and
any existing app definition. It proposes a complete declaration. Shared Rust
validation admits only implemented combinations; the guest reviews, installs and
executes them. New capabilities belong in the language, evaluator, renderer and
validators together. Adding an item to a model prompt does not implement it.

## Pieces you can combine

| Layer | Reusable pieces | Examples of combinations |
| --- | --- | --- |
| Inputs | Text, signed integers, checkboxes, required/default values | Costs, quantities, names and completion flags |
| Calculations | Derived fields, checked + - * /, same-row dependencies | Integer-cent balances, subtotals, remaining units |
| Behavior | Button programs, assignments, arithmetic, conditions, branches, row/all scope | Conditional updates across the saved table |
| Cross-record operation | Checked lookup and atomic transfer with preview | Move quantities between user-defined locations or allocations |
| Processes | States, guarded transitions, terminal edit locks | Intake/repair/delivery or other declared processes |
| Presentation | Text, count/sum/min/max metrics, signed bars, two-column tables, two pages | Live reports and small data presentations |
| Data continuity | Reviewed CSV import/export, compatible evolution, host checkpoints | Use a generated tool and keep its current records |
| Definition history | Scoped Git status/history/show/diff/save/restore | Change a tool and recover its earlier behavior |
| System action | Explicit software framebuffer brightness capability | A bounded screen-control tool |

Names such as workshop or expense tracker are examples, not production app IDs
recognized by the interpreter. Use arbitrary supported identities/fields/programs.
For a complete inventory see [APP_CAPABILITIES.md](APP_CAPABILITIES.md), for exact
bounds see [APP_LIMITS.md](APP_LIMITS.md), for pages see [APP_PAGES.md](APP_PAGES.md), and for two-record operations see
[APP_TRANSFERS.md](APP_TRANSFERS.md).

## How to experiment

1. Describe one useful tool and its data. Ask for a supported calculation or action.
2. Inspect the generated declaration/review before installing. Enter fictional test
   data and verify expected results, including a rejected action where relevant.
3. Ask for pages that present those same fields. Edit data and observe the local
   views update. Use integer cents when representing currency.
4. Save the definition with `git: save this definition`. Request a compatible change,
   review it, then save another version. Restore an earlier definition if needed;
   current records remain and incompatible schema restoration is rejected.
5. Keep actual provider responses and test results when presenting a live example.
   Saved recovery is useful but must not be described as fresh generation.

## Toward spreadsheet and presentation tools

Today's foundations include a bounded table, formulas, composed actions and live
presentation blocks. A fuller spreadsheet needs scalable row/column storage,
stable references, richer numeric/date/text types, cross-cell formulas, editing
and recalculation policies, import fidelity and a larger layout system. A fuller
presentation editor needs editable document pages, text layout, images/assets,
positioning, export and document persistence. Neither XLSX nor PPTX is supported.

Next useful primitive families include record identity/relations, richer typed
expressions, reusable layouts/assets and explicit event-driven services. Each
needs tests for failure and recovery as well as a working example. More capacity
requires stack/storage/layout measurements, not only larger constants.

An appliance restricted to one or two installed tools is also a product direction.
A proper creation lock needs an explicit owner/admin policy enforced across the
guest controller, model route and installation/update paths, with a recovery path.
Hiding Create alone is not permission enforcement. Such a lock is not implemented
by this increment. Executable OS self-update remains deferred.

The current guest is small because its functionality is narrow. There is no
comparable workload/memory/performance benchmark proving superiority over Linux,
and no claim of a complete phone platform or arbitrary application support.
