# RAI-OS demo kit

**Describe the tool you need. Review it. Use it locally. Keep your work.**

This is the current presentation entry point. The workshop is the main story;
the quoting tool demonstrates schema evolution and formulas. Both are actual
previously Meta-generated apps, with fictional records. Their schemas and data
are assembled unchanged in fixtures/microapps/v1/demo-device.json; the adjacent
provenance file identifies the two original evidence files and their hashes.
Assembly/restoration does not count as another model generation.

## Try the result without a key

From the repository root, with the existing Rust/Python/QEMU setup:

```sh
python3 -B scripts/demo.py saved --output target/demo-visit-1
```

Connect a VNC viewer to localhost:5900. For macOS Screen Sharing, first set a
local RAI_VNC_PASSWORD of 1–8 ASCII characters as explained in
[MICROAPPS.md](MICROAPPS.md). The display/input relay is still VNC, not a web page.
All paths below are relative to the checkout. Ctrl-C saves and stops. Resume:

```sh
python3 -B scripts/demo.py resume --output target/demo-visit-1
```

The first command refuses an existing folder instead of resetting it. Resume
preserves edits and uses a new session log each time. To start over, choose a new
folder. The tracked fictional fixture is never modified. The copied device.json
and exchange folder contain the visitor's records; do not publish private data.
The screen says replay: app use is real guest execution, but no model runs.
Create in this mode accepts only the existing disclosed offline examples.

For Docker users on Windows/Linux/macOS, build the image then run:

```text
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm -it -p 127.0.0.1:5900:5900 -v rai-demo:/state rai-os-demo demo
```

Pass `-e RAI_VNC_PASSWORD` if the viewer requires it. Repeating the command resumes
the same named volume. No API key is required. Docker provides a Linux host for
QEMU; it is not a phone port or a native Windows executable. For a completely
network-disabled automated check (no interactive VNC port):

```text
docker run --rm --network none rai-os-demo demo-verify
```

## Five-minute hands-on tour

1. Open **Repair Shop**, then **records**. There are eight orders in different
   process states. Select **Phone C**, then **actions**. Its repair is done, but
   its unpaid balance is 4000 cents. Click **Deliver**: the guest rejects it.
2. Return to **records**, open **Phone C**, select **Paid cents**, replace 1000
   with 5000, and Save. Open **actions** and Deliver: balance is zero, so it works.
   The record is now final; ordinary edits are locked.
3. Home -> **Quote App** -> **table**. The right column page shows cost and profit.
   The two-record profit total begins at 285 cents. Open Cable, change Cost cents
   from 150 to 175, Save and reopen the table: its profit is 200; total is 260.
4. Click **export**. Inspect the new CSV in the demo's exchange/outbox directory.
   It contains field data; full process recovery uses device.json, not CSV.
5. Stop and resume. Phone C remains delivered and the quote values remain changed.
   No inference is needed for these edits, actions, export or recovery.

These are deliberately tiny fictional datasets. The workshop is already full:
8 rows. There is no delete/archive operation yet. The complete limits and
engineering sequence are in [APP_LIMITS.md](APP_LIMITS.md).

## Generate something new with Meta

Put your own key in the checkout-root .env.local following
[META_BRIDGE.md](META_BRIDGE.md). Never show the file in a recording. Then:

```sh
python3 -B scripts/demo.py live --output target/demo-live-1
```

This starts with an empty device and uses real Meta for submitted prompts. Try:

```text
Repair shop: device, job, quote, paid, balance=quote-paid in cents. Initially received; start, finish, deliver if balance<=0.
```

Review both fields and process before installing. Enter 0 for no payment if the
model leaves the numeric field blank; formulas need numeric operands. Enter a new
fictional order using the actual returned field labels. Creation is not guaranteed:
show an error or clarification honestly, retain the attempt, and only switch to
saved mode with an explicit explanation. There is no silent fallback or retry.
The source meta label means generation mode, not inference on every action.

For a new creation/evolution story, use the two prompts and checks in
[DEMO_JOURNEY.md](DEMO_JOURNEY.md). Existing workshop process graphs cannot change
on update. The quotation evolution is a separate demonstrated app, not evidence
that a workshop workflow can be rewritten.

## Three-minute video sequence and English narration

The final video should show actual interaction and a readable pointer. This
script fits a three-minute cut with pauses for actions; rehearse the timing.

| Time | Show | Narration |
| --- | --- | --- |
| 0:00–0:15 | Empty launcher, then goal | “A small workshop needs software that fits its process. In RAI-OS, describe the tool you need, review it, and use it inside our Rust operating-system prototype.” |
| 0:15–0:40 | Actual Meta request, field review | “Meta turns this request into an application description: fields, a balance calculation, and a repair process. RAI-OS validates the proposal. We review it before installing.” |
| 0:40–0:55 | Process review, install, icon | “This is a generated declarative app, not a hardcoded workshop screen. The runtime supplies reusable capabilities; the model combines them.” |
| 0:55–1:25 | Disclosed fictional orders; blocked delivery | “This repair is done, but the customer still owes money. Delivery is blocked by Rust code using the app's declared rule. The model cannot bypass that rule by asking again.” |
| 1:25–1:45 | Enter payment; deliver | “Record the payment and the balance updates locally. Now delivery is permitted, and the completed record is locked against ordinary edits. These actions require no model round trip.” |
| 1:45–2:10 | Separate quote app, before/after evolution captures | “A second tool shows how applications can evolve. A new prompt added cost and profit while preserving the original records. Editing a cost recalculates the result locally.” |
| 2:10–2:35 | Export, close, reopen | “The data can be exported. Restarting QEMU restores both the application definitions and their records through our explicit host checkpoint adapter. Using a saved app needs no API key.” |
| 2:35–3:00 | Architecture and runnable repository | “The Rust guest draws the interface and validates and executes app behavior. A host bridge handles input, HTTPS and saved sessions. Today these are bounded apps sharing the kernel's privilege level, not isolated native programs or a phone-ready OS. Our next step is broader capabilities and isolated services. You can run this example, or provide your own Meta key and create your own supported tool.” |

The main workshop run and quote evolution are separate recorded examples. Label
cuts between runs, replay/saved state, prepared data and any shortened API wait.
The observed workshop provider latency was 5518 ms, not a promised response time.
Do not splice them into a claim of one continuous live session. The rough preview
renderer uses still QEMU captures and labeled cuts; it is not an interactive video
recording and has no voice track. Replace it with narrated interaction for the final.

## Rehearsal checks and recording assets

```sh
python3 -B scripts/demo.py verify --output target/demo-check-1
# Optional FFmpeg, with drawtext enabled; no extra Python dependencies:
python3 -B scripts/render_demo_preview.py --check target/demo-check-1 --output target/demo-preview-1
```

The verifier restores both exact public apps, performs actual guest pointer/key
input, checks denied unpaid delivery, exact payment, profit, CSV, app separation
and reboot. It rejects reuse of the same output directory and retains failure
reports. It makes no model call. The preview uses these native captures plus the
prior live creation/evolution evidence, produces a 180-second silent MP4 and a
scene manifest with English narration. Its source labels remain visible.

## Submission completion remains separate

- Record narration and real interaction; watch the final three-minute video.
- Check public repository and video access from a logged-out session.
- Fill the platform entry, contributor list and model/provider identification
  using actual registration and the retained provider metadata.
- Keep real commit dates; disclose October 26 kickoff versus October 27 build
  period and retain organizer clarification. Do not claim unresolved eligibility.
- Internal target: November 4, 18:00 Guatemala; supplied-guide deadline: 20:00.
- Retain the actual submission confirmation. A passing verifier is not submission.

No new performance superiority, general reliability, novelty or phone support is
claimed. Prior live evidence is in APP_WORKFLOWS and DEMO_JOURNEY; current-kit
verification outcomes are recorded in WORKLOG.


Fresh live app creation uses the [bounded correction loop](APP_REPAIR.md): at most
three attempts per goal and eight per session, with every response retained.
Eligible expression errors may be corrected; changed guards or permissions are
rejected, and a valid result still requires approval. The saved showcase remains
offline and unchanged. Do not present its recovery as a new model generation.


The [whole-table action extension](APP_BATCHES.md) adds a separate demonstration
beat: describe a conditional action, inspect different row previews, apply once,
then change the program by prompt while retaining data. Existing saved showcase
fixtures remain unchanged. Live evidence for this extension belongs to its own
reports, not the earlier saved workshop generation.


The new extension now has [actual two-prompt evidence](evidence/2026-11-03-app-batches/live/README.md):
Meta creates an all-row conditional action, then changes its threshold and
arithmetic on the same eight saved records. Both calls passed first attempt.
A useful visible contrast is Item 4: the first action skips its stock=1 value;
the revised action processes it. This is a change in generated program behavior,
not a hidden application-specific Rust branch or a benchmark claim.

To inspect that exact saved result without inference, copy its checkpoint to a
new local working file before opening it. Do not use tracked evidence as writable
session storage:

```sh
cp -n docs/evidence/2026-11-03-app-batches/live/device.json target/batch-showcase.json
python3 -B scripts/apps.py --source replay --state target/batch-showcase.json --output target/batch-showcase-session
```

Connect the VNC viewer as described above. This restores the actual prior model
result and labels the session replay; it is not a new generation. For a fresh live
creation, use the existing live entry point with your own key and the two prompts
in the linked evidence. Keep generation, human review, local execution and saved
recovery visibly distinct in the three-minute narration.


Live demo sessions now enable [Git tools](APP_GIT.md) in their own app-git folder.
Open an installed app, select Create, and request `git: save this definition`.
Save a second reviewed definition before requesting
`git: restore the previous saved version, keeping current records`.
The restored program requires the usual approvals. Show that the program changes
while the current values remain, and explain that Git retains both versions.
This is host-backed app-definition history, not executable OS self-update.
