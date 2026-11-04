# RAI-OS demo kit

**Describe the tool you need. Review it. Use it locally. Keep your work.**

For the shortest evaluator setup, including private key entry and connection
checking, start with [TRY_RAI_OS.md](TRY_RAI_OS.md). The saved kit contains Repair
Shop, Quote App and Transfer Balances: three actual previously Meta-generated apps
with fictional records. Their schemas/data are assembled unchanged in
fixtures/microapps/v1/demo-device.json; the adjacent provenance file identifies
the original sources/hashes. Assembly/restoration is not a new model generation.
The final [three-minute video](https://drive.google.com/file/d/18e2NPFsIktJhBC-T7ZY4ANAfQhD9PvaO/view?usp=sharing)
is recorded and uploaded. It follows a fresh Ledger app through creation, checked
transfer, rejection, evolution and recovery, with English captions and synthetic
reference narration. See [final recording evidence](evidence/2026-11-04-submission/README.md)
and the [submission checklist](SUBMISSION_PACKAGE.md). The workshop script below
is retained as an earlier rehearsal option, not the final video's shot list.

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
The screen says saved: app use is real guest execution, but no model runs.
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

Use `python3 -B scripts/demo.py configure` for hidden key entry and
`python3 -B scripts/demo.py check-meta` for one explicit connection request.
See [TRY_RAI_OS.md](TRY_RAI_OS.md) for Docker and error meanings. Never show
the private file in a recording. Then:

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
saved mode with an explicit explanation. There is no silent fallback. Eligible expression errors have bounded, logged correction attempts.
The source meta label means generation mode, not inference on every action.

For a new creation/evolution story, use the two prompts and checks in
[DEMO_JOURNEY.md](DEMO_JOURNEY.md). Existing workshop process graphs cannot change
on update. The quotation evolution is a separate demonstrated app, not evidence
that a workshop workflow can be rewritten.

## Earlier workshop rehearsal sequence and English narration

This earlier workshop script is retained for reproducible alternative rehearsals.
It describes separate workshop and quote examples, not the uploaded Ledger video.

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
recording and has no voice track. The final linked video instead uses sampled
real QEMU interaction and synthetic reference narration.

## Rehearsal checks and recording assets

```sh
python3 -B scripts/demo.py verify --output target/demo-check-1
# Optional FFmpeg, with drawtext enabled; no extra Python dependencies:
python3 -B scripts/render_demo_preview.py --check target/demo-check-1 --output target/demo-preview-1
```

The verifier restores all three exact public apps, performs actual guest pointer/key
input, checks denied unpaid delivery, exact payment, profit, CSV, app separation
and reboot. It rejects reuse of the same output directory and retains failure
reports. It makes no model call. The preview uses these native captures plus the
prior live creation/evolution evidence, produces a 180-second silent MP4 and a
scene manifest with English narration. Its source labels remain visible.

## Submission record and later updates

Jarrol reported submitting the project on November 4. The repository does not
independently verify the platform receipt; [SUBMISSION_PACKAGE.md](SUBMISSION_PACKAGE.md)
tracks that distinction. The checks below apply to any later entry/video update.

- Watch the uploaded three-minute video with audio; replace reference narration if desired.
- Check public repository and video access from a logged-out session.
- Keep the platform entry, contributor list and model/provider identification
  consistent with actual registration and the retained provider metadata.
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
result and labels the session saved; it is not a new generation. For a fresh live
creation, use the existing live entry point with your own key and the two prompts
in the linked evidence. Keep generation, human review, local execution and saved
recovery visibly distinct in the three-minute narration.


Live demo sessions now enable [Git tools](APP_GIT.md) in their own app-git folder.
Open an installed app, select edit app, and request `git: save this definition`.
Save a second reviewed definition before requesting
`git: restore the previous saved version, keeping current records`.
The restored program requires the usual approvals. Show that the program changes
while the current values remain, and explain that Git retains both versions.
This is host-backed app-definition history, not executable OS self-update.

### Verified Git recovery segment

[Live Git evidence](evidence/2026-11-03-app-git/live/README.md) records three actual
Meta tool selections and real commits: save, save, restore. The input app definitions
are verifier fixtures in this segment; do not describe them as newly generated by
Meta. Show the restore approval and preserved current records, then execute the
restored behavior. This demonstrates reversible app evolution, not arbitrary OS
source editing or model-speed superiority. Use the normal live demo to create/evolve
a supported app, then issue the git: prompts described in [APP_GIT.md](APP_GIT.md).

### Future direction: adaptable system services

The user explicitly deferred executable RAI-OS self-modification on November 3.
It may appear as a brief future-direction statement, separate from the working
app-definition versioning demonstration. Suggested English narration:

> Today, RAI-OS creates and evolves supported apps with reviewed changes and Git
> history. Our next research direction is applying that approach to system services,
> with isolation, tested updates and recovery. Executable self-update is not enabled.

This is a technically plausible direction, not demonstrated readiness: the current
app runtime and UI share EL1, and Git alone cannot recover an unbootable system or
reverse data migration. Keep the kernel outside the proposed model-editable scope.
See [KERNEL_DECISION.md](KERNEL_DECISION.md) and [APP_GIT.md](APP_GIT.md).

For the visual polish decision, see [DESIGN_DIRECTION.md](DESIGN_DIRECTION.md).
The interactive concepts are proposals; record the actual guest implementation.


### Composed live-data presentation

[APP_PAGES.md](APP_PAGES.md) provides an actual one-call Meta-generated two-page
report and a safe saved-session launch. It combines metrics, signed bars, text and
a table over the same records. Edit an amount and show the report update without
inference. Keep this a capability example: no spreadsheet/presentation file-format
compatibility or arbitrary layout is implied. The conversational Column redesign
remains separate from this actual guest page renderer.
