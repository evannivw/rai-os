# Three-minute demo: an app that changes with your work

The concrete story: a small workshop needs a quoting tool today. The user creates
it from a prompt, brings existing records, then realizes they need profit tracking.
They evolve the same app and retain their data. The result is usable local software
inside our QEMU guest, with no inference for each form edit or calculation.

This emphasizes a visible finished workflow. Do not lead with scheduler benchmarks
or unsupported comparisons with Android. Do not claim novelty, general native code
compilation, phone deployment or hardware isolation. The supplied hackathon guide
weights impact 30%, Meta tooling 25%, execution 20% and demo 25%; one complete use
case should make all four assessable. Dates/contributions remain in WORKLOG, with
the October 26 kickoff versus October 27 build-period discrepancy disclosed.

## Suggested video sequence

| Time | Show | Explain |
| --- | --- | --- |
| 0:00–0:15 | Own Rust kernel boot and empty phone-shaped launcher | “Describe the tool you need, then use it here.” |
| 0:15–0:50 | Type a quoting-app prompt, actual Meta wait, review fields/formula, install icon | Meta constructs the schema from reusable capabilities. |
| 0:50–1:15 | Import a prepared fictional CSV, preview and approve | Real rows and calculated totals, not an animated mockup. |
| 1:15–1:55 | Ask to add cost/profit; show reviewed schema and same rows with profit | The app changes without discarding the user's work. |
| 1:55–2:20 | Edit a cost, show immediate local calculation, export CSV | No model round trip for ordinary app use. |
| 2:20–2:40 | Restart QEMU and open the saved app | Schemas and records survive through the disclosed host checkpoint. |
| 2:40–3:00 | Show source/run instructions and the architecture boundary | Rust guest owns validation/execution/UI; host carries HTTPS/input/files. |

The live API latency must remain visible or explicitly identify edited waiting
time. Captures from a replay are labeled replay; do not narrate them as live
creation. Use fictional records and a separate device file for rehearsal. The
internal submission target remains November 4, 18:00 Guatemala; official deadline
20:00. Protect recording/submission time instead of adding another subsystem.

## Reproducible rehearsal

```sh
# No API call: disclosed schemas, actual QEMU execution and independent checks.
python3 -B scripts/verify_app_journey.py --output target/journey-offline
# Clean committed source; two paid Meta calls, no retries or fallback.
python3 -B scripts/verify_app_journey.py --source meta --output target/journey-live
```

The live path creates a fresh app and evolves it using a second prompt. Imported
headers are assembled from the actual returned field keys. The same independent
oracle expects totals 375 and 160 cents; initial profits 275 and 60; changing the
first cost from 100 to 150 yields 225 and 60, total 285. Export and reboot must
preserve every value. Neither the kernel nor the normal bridge contains a quote-
app branch. The offline verifier supplies disclosed example schemas only.

Reports, model diagnostics, CSV output, device checkpoints and native screenshots
are in the chosen output directory. Use a new output directory for every attempt;
retain failed attempts. A passing development case is not a general reliability
or speed benchmark. Consult WORKLOG for the actual run outcome.

## Highest-value finishing work

1. Rehearse this exact path with the real viewer and a fresh device; verify the
   recorded text is readable at video size and keep the pointer movements clear.
2. Make the payoff visually obvious: the same two records before/after evolution,
   then a changed cost and profit total. Keep the current restrained black design.
3. Prepare a clearly labeled saved-session fallback if the API is unavailable.
   Let reviewers restore it without a key and generate their own apps with a key.
4. Finish the three-minute recording, accessible video link, public repo/run
   instructions and contribution dates. These are delivery tasks, not completed
   merely because this document exists.

After delivery, the next substantial product improvement is schema-aware data
mapping and a broader action vocabulary, followed by actual process isolation.
For this deadline, a reliable whole journey matters more than another simulated
hardware feature or a collection of disconnected screens.

## Observed live rehearsal, November 3

The complete live journey passed from clean source 0de2acf: two Meta calls, no
retries. Creation took 4682 ms provider time; evolution 4995 ms. Quote App used
actual generated keys including totalc, which the CSV/header and evolution prompt
followed dynamically. Imported rows survived adding cost/profit, local editing,
export and restart. The final independent profit sum was 285 cents. These are
measured development observations, not promised latencies for other requests.

[Report and proposals](evidence/2026-11-03-app-exchange/live/report.json) and
[native final screen](evidence/2026-11-03-app-exchange/live/08-restored-app.png)
are available with the recorded source provenance. The generated fictional state
can be inspected without a key; copy it before opening to preserve evidence:

```sh
mkdir -p target
cp docs/evidence/2026-11-03-app-exchange/live/device.json target/journey-device.json
python3 -B scripts/apps.py --source replay --state target/journey-device.json --exchange target/journey-exchange --output target/journey-inspect-1
```

This opens the prior result, not fresh generation. Use source meta and your own
key to create/evolve tools. File templates must use the current exported header,
including the new cost input. The video itself has not been recorded by this check.
