# Declarative app capabilities

See [APP_LIMITS.md](APP_LIMITS.md) for why source ownership is not unlimited app
capability, and [DEMO_READY.md](DEMO_READY.md) for the runnable presentation.

Meta receives the versioned machine-readable catalog at
[fixtures/microapps/v1/runtime.json](../fixtures/microapps/v1/runtime.json) with
each app request. The catalog describes implemented fields, views, operations and
limits; tests check its advertised capacities against the shared Rust domain.
A model description does not grant new authority. Guest validation still owns
installation, record changes and migrations. Full control for the user means
reviewing/choosing behavior within implemented capabilities, not allowing a model
to manufacture a location driver, network stack or arbitrary privileged code.

The [display control capability](DISPLAY_CONTROL.md) now permits a separate app
to apply software brightness to the QEMU framebuffer after explicit user action.
It is the first capability beyond records; physical backlight control is still
unavailable. An app chooses one capability, and updates cannot change it.

See [APP_TOOLBOX.md](APP_TOOLBOX.md) for composing these pieces and the path toward
spreadsheet/presentation tools.

## Available now

| Capability | What an app can do | Execution |
| --- | --- | --- |
| [Atomic transfers](APP_TRANSFERS.md) | Resolve a unique destination in the same app; preview and update both balances together | Guest Rust, checked i32 |
| [Composed visual pages](APP_PAGES.md) | Text, live count/sum/min/max metrics, signed bars and two-column tables in reviewed pages | Guest Rust |
| Software display brightness | Apply 20..100%; explicit permission, user action and reset | Guest Rust framebuffer |
| [Composed behavior](APP_BEHAVIORS.md) | User buttons combine typed writes, arithmetic, conditions and forward branches; preview and atomic row or whole-table commit | Guest Rust |
| [Processes](APP_WORKFLOWS.md) | Per-record states, up to four guarded actions, final-state editing lock | Guest Rust |
| Typed fields | Text, signed integer, checkbox; labels, required values and defaults | Guest Rust |
| [Calculated fields](APP_FORMULAS.md) | Per-app integer expressions over earlier numeric fields | Guest Rust, checked i32 |
| [CSV exchange](APP_EXCHANGE.md) | Reviewed atomic import and explicit export through a selected host folder | Guest validation + host I/O |
| Local records | Create and edit complete records in the current app | Guest Rust |
| Record view | Select/edit rows; toggle one configured status | Guest Rust |
| Pending filter | Show records whose configured checkbox is false | Guest Rust |
| Table | Three columns per page, up to six total; select row to edit | Guest Rust |
| Totals | Sum each numeric column over the visible rows | Guest Rust, i64 |
| Compatible update | Append fields while preserving existing field identity/types/values | Guest Rust after review |
| Launcher | Icon, name, case-insensitive search and pointer/keyboard navigation | Guest Rust |
| Saved session | Recover confirmed apps/records after a fresh boot | Explicit host adapter; guest revalidates |

The catalog is input to proposal generation, not a menu of unrestricted tool calls.
Meta returns the strict app schema, not executable shell/Rust/JavaScript. The
runtime interprets that schema locally. Creating a native executable, linking it
against a stable syscall ABI, loading it and isolating it are a separate future
pipeline. Do not call a schema proposal native machine-code compilation.

## Tables

From an app press F5 or tap `table`; F3/tap `records` returns to the list. Right or
the lower-right column-page indicator selects the second three-column page when needed; Left
returns to the first. Up/Down select rows. Enter or a row tap opens its form.
N adds a row; F4 toggles the configured pending filter. Totals follow that filter.
Long values/labels are clipped in table cells; the form shows the full values.

Numbers are signed i32 cells, optional empty cells are omitted from sums, and
at most eight values are accumulated into i64. This prevents overflow for the
available range without approximating decimal arithmetic. Currency examples use
explicit integer cents. Bounded same-row formulas are now available; no currency
conversion, arbitrary functions, cross-row references,
XLSX compatibility or Microsoft Excel behavior is claimed. Bounded CSV exchange
is now available through a user-selected host folder.

A disclosed offline prompt is:

```text
Create an expense app with item, integer amount in cents, paid checkbox and an unpaid view. I want a table and totals.
```

The live mode uses Meta; replay supplies a fixed schema and labels itself as such.
The table and arithmetic are available to any app with numeric fields, not just
this example. Edits update totals without another model request. Saved-session
recovery also performs no inference.

```sh
python3 -B scripts/verify_app_table.py --debug --output target/table-debug
python3 -B scripts/verify_app_table.py --output target/table-release
# Clean committed source; one live Meta call, no retry:
python3 -B scripts/verify_app_table.py --source meta --output target/table-live
```

The verifier independently checks 1250 + 750 - 200 = 1800 cents; filtering out the
paid 750 yields 1050. Editing the first row to 1350 yields 1900 total / 1150 unpaid.
It verifies exact rows and totals after reboot. The domain test also sums eight
maximum i32 values without overflow and checks optional blanks and filtering.

## Further capability directions

Calculated fields and [composed button programs](APP_BEHAVIORS.md) are implemented.
The latter extends the language, not a list of built-in business apps. Two buttons
with up to six steps each can target one saved row or all saved rows through
[explicit batch scope](APP_BATCHES.md). Each program reads only its current row;
the separate transfer rule supplies bounded cross-row lookup and a two-row commit.
Neither mechanism grants background execution or arbitrary hardware access.

1. **Guest-private app files:** scoped file handles and quotas, building on the
   existing guest file store. User-mediated CSV exchange is already available;
   it does not grant generated apps arbitrary file access.
2. **Events and background work:** declared triggers, queues, cancellation and
   execution budgets. Wire to an actual service boundary before claiming isolation.
3. **Optional in-app AI:** a declared action with explicit user consent, selected
   data only, cost/rate limits and unavailable/offline behavior. Apps do not get
   the provider key. Ordinary forms and calculations remain local.
4. **Native apps:** restricted build pipeline, stable ABI, checked loader, separate
   address spaces and capability-bearing syscalls. Only then expose generated
   native code as an app type. Phone drivers are another separate project.

These are capability directions; the current engineering sequence is maintained
in [APP_LIMITS.md](APP_LIMITS.md#engineering-sequence-after-the-submission-slice).
None of these future capabilities are available just because the model requests
them. Unsupported intent must produce clarification/decline, not a partial app
presented as complete. Protect Nov 4 demonstration/submission time; the current
product slice is create -> install -> use -> evolve -> recover, with measured
outcomes rather than unsupported speed or novelty claims.

## Live generation and a ready-to-open session

On November 3, frozen source d99eda5 made one live Meta request for the expense
example. HTTP 200, 3903 ms provider latency, 1312 reported tokens, no retry. Meta
returned Expense Tracker with explicit cents, a paid flag and an unpaid filter.
The verifier installed it only after checking the requested fields, entered the
three fictional rows, checked exact totals/edits, then restored the app in a new
QEMU instance without another inference request. This is a known development
integration example, not a comparative speed claim.

[Showcase state](../fixtures/microapps/v1/showcase.json) combines two actual
previously accepted Meta schemas: Inspection Log (source 2996d51) and Expense
Tracker (d99eda5), with their independently checked fictional records. The
showcase restores them offline; it does not pretend to generate them again.
Copy it to a writable, untracked device file before opening it:

```sh
mkdir -p target
cp -n fixtures/microapps/v1/showcase.json target/showcase-device.json
python3 -B scripts/apps.py --source replay --state target/showcase-device.json --output target/showcase-session-1
```

Then connect the VNC viewer as documented in MICROAPPS.md. `cp -n` preserves an
existing saved device instead of resetting it. Use a new output folder on each
launch. To create a new app with Meta, choose `--source meta` and supply your own
key; restored apps need no key. Home shows both app icons, Apps searches them,
and clicking an icon opens the actual guest app. Table is available inside each.
The fixture/checkpoint is deliberately public sample data; never replace it with
private records in Git.
