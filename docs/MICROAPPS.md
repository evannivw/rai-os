# Prompt-generated microapps

RAI-OS interprets bounded app descriptions in its own kernel image. Meta proposes
an app schema; the guest checks it, displays a review, and installs only after
Enter. Forms, local record edits and filtered lists execute in QEMU without
further inference. This is a useful, extensible interpreter slice, not arbitrary
code generation, an app store, a complete microkernel or measured AI superiority.

## Start and use

From the repository root with Rust, QEMU and Python available:

```sh
python3 -B scripts/apps.py --source meta --output target/apps-session-1
```

Set your own MODEL_API_KEY as described in [META_BRIDGE.md](META_BRIDGE.md).
Do not put it in prompts, source, screenshots or commits. The Rust provider alone
uses it for HTTPS; neither QEMU nor the VNC viewer receives it. Output paths must
be new. Ctrl-C stops the session. A maximum of eight proposals and one in-flight
request bounds each session. There are no automatic retries or silent fallbacks.

Connect a VNC viewer to localhost:5900. On macOS set a temporary
RAI_VNC_PASSWORD (1–8 ASCII characters) in the launch environment first, then
`open vnc://127.0.0.1:5900`. See [PROMPT_WORKSPACE.md](PROMPT_WORKSPACE.md) for
viewer authentication and input details. Keep the port on loopback. Standard
RFB clients on Windows/Linux can use the same packaged Linux environment:

```text
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm -it -p 127.0.0.1:5900:5900 --env-file .env.local rai-os-demo apps
```

Pass `-e RAI_VNC_PASSWORD` too if set on the host. Docker Desktop supplies Linux
on macOS/Windows. This is portability of the host tooling, not a native Windows
kernel build or installation onto a phone. The guest is our Rust AArch64 kernel,
not the Linux container kernel. It still needs host input and HTTPS adapters.

To try the disclosed offline script without credentials:

```sh
python3 -B scripts/apps.py --source replay --output target/apps-replay-1
```

Or use `apps-replay` instead of `apps` in Docker, omitting --env-file. The screen
explicitly says REPLAY / OFFLINE. It recognizes these two exact prompts only:

```text
Create an inspection app with equipment, notes, a done checkbox and a pending view.
Add a required integer priority field with default 1. Preserve all existing fields, records and the pending view.
```

The second prompt is an update: open the installed app, press F2, then type it.
Other offline prompts produce a clarification. Live Meta can propose other apps
within the contract, such as an inventory, checklist or small expense log. It
cannot implement unavailable operations; inspect the proposed fields carefully.

| Screen | Controls |
| --- | --- |
| Home | Type or F2: create. Tab/Down: select. Enter: open. |
| Prompt | ASCII input (128 chars), Backspace, Enter: propose. |
| Review | Enter or bottom button: install. Esc: discard. |
| Form | Tab/Down/Up: field. Type/Backspace: edit. Space: checkbox. Enter: save. F3: records. |
| Records | Up/Down/Tab: select. Enter: edit. N: new. Space: toggle configured status. F4: pending/all. |
| Form or records | F2: evolve this app, preserving records. |
| Any screen | Esc or F1: home and invalidate outstanding response. |

On keyboards that reserve function keys, use Fn with the key. Clipboard paste
accepts bounded ASCII; newlines do not submit. This is a keyboard-first graphical
UI, not a terminal emulator. Clicks are limited to the main action, footer/home
and the existing Tab hotspot; text fields are selected with Tab, not a mouse.

## Contract and boundaries

- Three installed apps, six ordered fields each, eight records per app.
- Values/defaults: at most 32 printable ASCII bytes. Text, signed 32-bit integer
  (optional fields may be empty), or checkbox string `true`/`false`.
- Stable lowercase/digit/underscore app and field IDs: 1–12 bytes; title up to
  28, field label up to 22. Required fields are checked on complete-record save.
- One optional filter references a checkbox and shows false records (pending).
  Its label is at most 18 bytes. The first field is the record-list headline;
  open a row to see/edit all fields. No sort/formulas/loops/deletion yet.
- Only `records.local` exists. No file, location, network, timer, native code or
  arbitrary memory capability. Schemas cannot name another app as a data source.
- Updates retain every old field's ID, position and type, adding fields only.
  Existing values are checked against the new constraints before any write.
  Added fields get valid defaults; incompatible migration changes nothing.
- Request IDs, target app and global revision bind review to the intended
  installation. Duplicate/cancelled/stale responses cannot install silently.
- All app schemas and records are **volatile guest RAM**, lost on QEMU shutdown.
  No persistence/import/export UX is claimed. Diagnostic app.export exposes
  records to the trusted host harness and UART console, not generated apps.

The pure runtime owns bounded arrays and validates before mutation. A declarative
app is **not a process**: runtime, controller and renderer currently share EL1
privilege and address space with the kernel. App namespaces and denied operations
are interpreter controls, not MMU isolation, confidentiality from the host, or
protection against a kernel/runtime bug. EL0 worker mechanisms elsewhere in this
repo are not yet wired to this runtime. Hardware-isolated services, persistent
storage and real device drivers remain future work.

Meta receives only the prompt, existing schema if updating, installed app IDs,
record count and revision. Record values are not included in that request. Session
logs contain prompts/schemas and the local UART verifier may include record data;
use fictional demo data. Host validation shares the guest domain rules. JSON
rejects unknown fields, and generated content is never evaluated as code.

## Code map and wire contract

- `src/microapp.rs`: pure bounded schemas, namespace selection and atomic migration.
- `src/app_ui.rs`: prompt/review/form/list state and user confirmation.
- `src/display/apps.rs`, `terminal.rs`: guest framebuffer presentation only.
- `host/meta/src/apps.rs`, `provider.rs`: strict JSON and one model proposal.
- `scripts/apps.py`: host input/API relay; `workspace_vnc.py`: actual QEMU pixels.

The existing @rai1 UART framing carries app.open [id], app.source meta|replay,
app.key code, app.catalog, app.schema id and app.export id. A submitted prompt
returns submit/request ID, revision, target ID or '-', and hex ASCII prompt.
Staging is app.begin `request:revision:id:title_hex:records.local`, app.field
`request:key:kind:required_0_or_1:label_hex:default_hex`, optional app.filter
`request:key:label_hex`, then app.ready `request:reason`. No staged state is
installed until app.key 13. app.notice gives a clarification; Esc invalidates
pending responses. Text fields are hex framed so punctuation cannot inject an
operation. app.* is a trusted diagnostic surface, not an untrusted-process ABI.

## Verification and delivery

```sh
python3 -B scripts/verify_apps.py --debug --output target/apps-check-debug
python3 -B scripts/verify_apps.py --output target/apps-check-release
# From a clean committed checkout only; up to TWO real paid calls, no retries:
python3 -B scripts/verify_apps_meta.py --output target/apps-live
# Offline portable check:
docker run --rm --network none rai-os-demo apps-verify
```

The offline test drives a real RFB connection, receives guest pixels, types a
prompt and clicks approval; checks exact records before/after schema evolution;
and rejects malformed, stale, cross-app and destructive proposals. The live test
independently checks the disclosed prompt's expected fields before confirmation,
then verifies real records survive an additional generated priority field. It is
an integration smoke test, not an unseen benchmark. Record actual observations
and source commit in WORKLOG; retain failed runs without post-hoc prompt tuning.

Next extensions should each add one checked capability, executor and real guest
check. Priorities: durable app/record snapshots, richer record views, then an EL0
service boundary. Do not offer location upload until location, transport, explicit
consent and enforceable capability boundaries exist. Preserve time for the Nov 4
video/submission. The current demo story is prompt -> reviewed app -> usable
records -> prompt update -> same records, not an unmeasured speed claim.
