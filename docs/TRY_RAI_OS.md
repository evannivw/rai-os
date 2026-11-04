# Try RAI-OS

**Use the saved apps without an API key, or supply your own Meta key to create new
ones. The author's credential is never included.** All commands below start at the
repository root. The screen is the actual Rust guest in QEMU; VNC carries pixels
and input. This is not a browser application or a physical-phone installer.

## 1. Start without a key

The portable route uses Docker with Linux containers on Windows, Linux or macOS:

```text
git clone https://github.com/evannivw/rai-os.git
cd rai-os
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm -it -p 127.0.0.1:5900:5900 -v rai-demo:/state rai-os-demo demo
```

Open a VNC viewer at **localhost:5900**. This port is not a web address. Some viewers,
including macOS Screen Sharing, may require a VNC password: set your local
RAI_VNC_PASSWORD environment variable (1–8 ASCII characters) and add
`-e RAI_VNC_PASSWORD` to the Docker run command. This local display password is
separate from the Meta key. Keep the published port bound to 127.0.0.1 as shown.

Already have the pinned Rust toolchain, Python and QEMU? Use the local route:

```sh
python3 -B scripts/demo.py saved --output target/judge-saved-1
```

Setup details: [GETTING_STARTED.md](GETTING_STARTED.md) and
[PORTABLE_RUN.md](PORTABLE_RUN.md). Linux container testing does not establish a
physical Windows-host test. No API/network access is needed for app use after the
image has been built. The Docker build itself downloads tools/dependencies.

### A two-minute hands-on check

The saved device contains three actual previously Meta-generated apps and fictional
data. Their [provenance](../fixtures/microapps/v1/demo-provenance.json) identifies the
original declarations/checkpoints. The `saved` label means recovery, not generation.

1. Open **Transfer Balances**, then **records**, then **Warehouse**.
2. Tap **move >**. Preview shows 12 → 8 and 5 → 9. **Apply both changes**.
3. Tap **pages >** or F11. Total stays **17** and the bars show **8 / 9**.
4. Return to records, open Warehouse, select Amount, replace 4 with 99, and Save.
   Open Warehouse again and tap move: insufficient balance is rejected; neither
   balance changes. Correct the amount before trying another transfer.
5. Home also offers **Repair Shop** (guarded process transitions) and **Quote App**
   (local arithmetic). Use [DEMO_READY.md](DEMO_READY.md) for their guided checks.

Navigation: **Home** opens apps; **Apps** searches; **Create** requests a new app.
Inside an app, **edit app >** or F2 requests a compatible update. Create and edit
use Meta only in live mode; saved mode offers the disclosed offline examples only.
The prompt accepts 128 printable ASCII characters. Use your computer keyboard;
there is no on-screen keyboard. Esc cancels. F3 records, F5 table, F11 pages, F12
transfer preview. Native labels are English.

Ctrl-C in the launch terminal stops and saves. Repeat the Docker command to resume
the same volume. Local users resume explicitly:

```sh
python3 -B scripts/demo.py resume --output target/judge-saved-1
```

A new local output directory or a differently named Docker volume starts a fresh
copy. Existing sessions are never silently reset. Existing older saved sessions
retain their apps rather than being automatically upgraded to the three-app kit.

## 2. Create new apps with your own Meta key

A key belongs to the evaluator's own Meta Model API account. Access, credit and
model permissions depend on that account. Never send the key to us, put it in an
app prompt, commit it, or show it in a recording.

### Local setup

```sh
python3 -B scripts/demo.py configure
python3 -B scripts/demo.py check-meta
python3 -B scripts/demo.py live --output target/judge-live-1
```

`configure` asks for the key with hidden terminal input. It validates the format
with Rust and creates an ignored .env.local with owner-only permissions. It refuses
to overwrite an existing file, follow a symlink, or fall back to visible input.
If a file already exists, edit it privately. If MODEL_API_KEY is set in your
shell, it takes precedence; change/unset that setting rather than editing a file
that would not be used. No key is placed in command arguments or reports.

`check-meta` makes **one small inference request** through the same model/HTTPS
adapter. It may use API credits. It sends a fixed connection-check message, no
application or record data. Success says **Connected to Meta** and reports the
observed HTTP/model result. It installs nothing. A local format check never claims
that authentication succeeded. `live` checks local format before launch; it does
not make an extra connection request automatically.

### Docker setup

Copy the public `config/meta.env.example` to a private checkout-root `.env.local`.
Use a text editor to put your own key after `MODEL_API_KEY=`. Use a single plain
setting, without shell `export` syntax. Then:

```text
docker run --rm --env-file .env.local rai-os-demo check-meta
docker run --rm -it --env-file .env.local -p 127.0.0.1:5900:5900 -v rai-live:/state rai-os-demo demo-live
```

The key is supplied at container runtime, never copied into the image. Do not pass
its value as a command argument. The live volume is separate from the saved demo.
Runtime environment variables are available to the trusted host/container operator;
this is not a hardware-backed secret store.

### Try this prompt

```text
App: name, balance, amount, to. Transfer amount from this balance to name=to atomically. Page: sum balance and bars by name.
```

Review the actual returned fields, transfer rule and page before installing. Enter
the fictional values above. Meta may clarify or decline; valid syntax does not
prove the intended app is correct. Creation uses the documented bounded correction
loop: up to three attempts per goal, eight per session, no silent model/fixture
fallback. Each successful installed tool runs its ordinary operations without
further inference. Your input/schema is sent for creation/evolution; record values
are not included in those app-generation requests.

## 3. Troubleshooting

| Message / symptom | Meaning and next step |
| --- | --- |
| No Meta key found | Configure your own key or run saved mode. |
| Key format invalid | Check the private value and environment precedence. Nothing was sent. |
| Meta rejected authentication (401) | Check/replace your own credential. |
| Meta denied access (403) | Verify account/model permissions; not proof the key is malformed. |
| Usage limit / credit (429 / 402) | Check your account quota/billing or retry later. |
| Timeout / secure connection failed | Check network, curl and system certificates. |
| HTTP succeeded but response rejected | The response did not meet the model/output contract. Not an invalid-key diagnosis. |
| Existing output / session already used | Resume it, or choose a fresh directory/volume. |
| localhost:5900 does not open in browser | Use a VNC viewer. Check the launch terminal and loopback port. |
| Fourth app cannot install | Current capacity is three. Start a separate fresh live session. |

Configuration and service errors are also shown as readable guest notices when a
creation request fails. Reports retain safe diagnostic codes, not credentials or
raw authentication headers. Generated app proposals and user prompts are retained
locally for debugging: use fictional data for shared demos.

## Verification and scope

```text
docker run --rm --network none rai-os-demo demo-verify
```

This boots QEMU, exercises the saved apps, checks actual results and reboots with
exact recovery. The current slice has three apps, six fields and eight records per
app; two pages with two blocks each; one checked transfer rule. Apps/UI share EL1.
HTTPS, Git, checkpoints and input relay use a host bridge. No general native-code
compilation, XLSX/PPTX, phone drivers, unrestricted AI control or Linux efficiency
win is claimed. See [APP_LIMITS.md](APP_LIMITS.md) and the
[recording runbook](DEMO_FINAL_RUNBOOK.md).
