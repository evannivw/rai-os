# User-mediated app data exchange

Every records.local app now has native **import** and **export** toolbar actions
(F7/F8). They work with any compatible installed schema, including calculated
fields. There are no app-specific importers. Meta does not see the file contents
or select paths; the user chooses a host exchange folder when starting the bridge.

```sh
python3 -B scripts/apps.py --source meta --state target/my-device.json --exchange target/my-exchange --output target/exchange-session-1
```

The default exchange folder is target/exchange. The launcher prints the selected
absolute path. Inside it:

- `inbox/<app-id>.csv`: the one file read when that app requests Import.
- `outbox/<app-id>-r<revision>.csv`: a new complete export; existing paths are
  never overwritten. These files are outside the saved-device checkpoint.

The app ID comes from its installed schema, not a model-supplied file path. For
an empty app, **Export first** produces a header-only template containing its
exact field keys. Fill a copy and save it under the app's inbox filename. File
selection is currently outside QEMU; the guest review and approval are native.

## CSV contract and review

The first row names every editable field, or every field including calculated
ones, exactly once. Header order may differ from schema order. Values remain
literal strings, including empty optional values, commas and escaped quotes.
Use normal CSV quoting. A header from one schema is not silently mapped onto a
different schema; export a fresh template after adding fields.

Import accepts 1..8 rows and at most 16384 bytes; an app still has eight total
record slots. Every cell is at most 32 printable ASCII bytes. Unicode, embedded
newlines, missing/extra/duplicate columns, invalid types and excess rows are
rejected. No truncation or partial import. An empty exported app is a template,
not a valid zero-row import.

For input-only CSV, the guest computes derived values. If the CSV includes
calculated columns, their values must exactly match canonical recomputation.
An incorrect result, missing input, overflow or division by zero rejects the
entire batch. This also catches bad data in a later row after valid earlier rows.

The guest stages the batch, validates all rows and shows one incoming row at a
time, including its computed values. Tab/Down or a tap on the row counter moves
forward; Up moves backward. Enter/the **Import rows** button appends all rows in
one revision change. Esc cancels without altering saved records. Imports append;
they do not replace, merge or deduplicate existing data. Importing the same file
again after a new request intentionally adds another copy if capacity permits.

Export includes all saved rows and columns, regardless of a pending filter,
visible table page or unsaved form draft. It uses CSV escaping and CRLF endings.
Exports preserve text exactly; when opening in a spreadsheet, import text columns
as text so that another program does not reinterpret them as formulas. No XLSX
or full spreadsheet-format compatibility is claimed.

## Boundaries and failure behavior

Guest import state is tied to the selected installed app, request and current
runtime revision. Duplicate cells, incomplete staging and cancelled/late messages
cannot commit. The domain validates all rows before changing any data. Import
approval triggers the existing checkpoint mechanism; `save failed` still means
only the running guest contains the new data. Resolve that before shutdown.

The host adapter parses bounded CSV and refuses symlink/non-regular input files.
Export writes a private temporary file, flushes it, publishes with an exclusive
hard link and flushes the directory. A prior destination is never replaced. A
host filesystem that does not support those operations rejects export. The host
and selected directory remain trusted; this is not a guest disk driver, arbitrary
file permission for generated apps, or hardware isolation. Keys are never part of
the data, and importing/exporting makes no inference calls.

Docker uses `/exchange`; mount a host folder separately from the device volume:

```sh
docker build -t rai-os-demo -f portable/Dockerfile .
mkdir -p target/exchange
docker run --rm -it -p 127.0.0.1:5900:5900 -v rai-device:/state --mount "type=bind,source=$(pwd)/target/exchange,target=/exchange" --env-file .env.local rai-os-demo apps
```

On Windows, supply the absolute host directory in Docker's mount syntax. A Linux
container provides the host adapter; native Windows Python is not this runner.

## Code and checks

- `src/microapp.rs`: bounded ImportBatch, canonical derivation, atomic append.
- `src/app_ui.rs`: request state, staging, review and confirmation.
- `src/display/apps.rs`: native preview, import/export controls and page navigation.
- `scripts/app_exchange.py`: CSV parsing and selected-folder I/O only.
- `scripts/apps.py`: transfer routing and existing saved-state checkpoint.

Diagnostic frames are app.import.begin `request:count`, app.import.cell
`request:row:field:value_hex`, app.import.ready `request`, then guest Enter to
commit. app.transfer.notice `request:message` reports host completion/errors.
These are trusted host/UART diagnostics, not an untrusted-process ABI.

```sh
python3 -B scripts/verify_app_exchange.py --debug --output target/exchange-debug
python3 -B scripts/verify_app_exchange.py --output target/exchange-release
docker run --rm --network none rai-os-demo apps-exchange-verify
```

The test restores a previously generated Meta schema (provenance in
fixtures/microapps/v1/exchange-example.json), then checks native review/cancel,
quoted/reordered CSV, exact exported bytes, no-overwrite behavior, invalid batches,
late/duplicate/incomplete staging and exact recovery after a fresh guest boot.
Restoring that example is explicitly not a new model invocation.

For [workflow apps](APP_WORKFLOWS.md), CSV covers fields only. Imported rows start
in the initial state; CSV does not preserve a process history or current state.
Use the saved-device JSON for full process recovery.
