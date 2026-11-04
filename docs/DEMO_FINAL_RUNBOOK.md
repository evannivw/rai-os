# Final three-minute recording runbook

Uploaded video: [Watch the three-minute demo](https://drive.google.com/file/d/18e2NPFsIktJhBC-T7ZY4ANAfQhD9PvaO/view?usp=sharing).

The claim: describe a bounded tool, review the generated application, then use it
locally inside our own Rust/QEMU OS. Meta creates the declaration; deterministic
Rust code validates and executes it. This is not a claim that model latency beats
a scheduler, that all apps are expressible, or that the prototype replaces Android.

## Completed recording

The uploaded Ledger video was recorded on November 4 from source
94e9efbac8960190740a3ab292c17ab6872d81c0. It uses two actual Meta requests: creation
and compatible page evolution, both accepted on their first attempt. Provider
times were 8892 ms and 9284 ms. The recording uses sampled real QEMU screens,
scripted inputs, edited timing, fictional records and labelled synthetic English
narration. [Evidence and source manifest](evidence/2026-11-04-submission/README.md)
retain the actual outcomes. Jarrol subsequently reported submitting the project;
see the separate [submission status](SUBMISSION_PACKAGE.md).

## Uploaded video sequence

| Time | Picture and demonstrated behavior |
| --- | --- |
| 0:00–0:12 | Saved three-app Home introduces the project. |
| 0:12–0:30 | A fresh session receives the transfer-app request. |
| 0:30–0:52 | Actual Meta generation, guest validation and review controls. |
| 0:52–1:10 | Fictional Warehouse and Dispatch records are entered through guest forms. |
| 1:10–1:30 | Preview and commit 12/5 -> 8/9; total stays 17. |
| 1:30–1:48 | An insufficient transfer is rejected without changing balances. |
| 1:48–2:10 | A second real Meta request adds a report page, preserving fields/rule/records. |
| 2:10–2:22 | The generated table and dispatch note show existing data. |
| 2:22–2:38 | A fresh QEMU guest recovers the evolved app and exact records. |
| 2:38–2:52 | Guest/host responsibilities and shared EL1 privilege are explained. |
| 2:52–3:00 | Public repository and saved/live reproduction instructions. |

The exact final checkpoint can be reopened without inference:

```sh
python3 -B scripts/demo.py saved --checkpoint docs/evidence/2026-11-04-submission/showcase.json --output target/recorded-ledger-new
```

This is saved recovery, not fresh generation. The earlier Transfer Balances
rehearsal remains documented in [APP_TRANSFERS.md](APP_TRANSFERS.md); its 8119 ms
creation request is a separate run and is not the uploaded Ledger creation time.
Use fresh output directories, retain unsuccessful attempts and label any edit or
saved result. Neither offline verifier proposals nor ordinary record operations
are new Meta requests. Historical negative performance experiments remain intact.

## Checklist for any recording or submission update

- Rehearse the three-minute sequence and confirm readable screen capture/audio.
- Confirm repository and video links are publicly accessible; list all contributors.
- Explain host responsibilities: HTTPS/model access, QEMU launch, Git and checkpoints.
  The app UI, values, validation and execution are in the guest; app modules are
  currently shared EL1, not isolated applications.
- Specify which footage is actual generation, saved recovery or fictional test data.
- Explain the implemented bounds: three apps, six fields, eight records per app,
  two pages with two blocks each, one transfer rule. No XLSX/PPTX, arbitrary native
  apps, physical-phone support or unrestricted hardware control.
- Preserve actual commit dates. The supplied guide has Oct 26 kickoff and Oct 27
  build-period start; organizer clarification of Oct 26 eligibility is still pending.
- Deadline from the supplied guide: Nov 4, 2026 at 20:00 Guatemala; internal target
  18:00. Current upload and participant-reported submission status are recorded above.

The recorded capability is verified. Further changes should preserve its reproducible
behavior and retain the original video evidence. Future capability
families: stable record relations, richer text/date expressions, reusable layouts,
and separately permissioned event services. They are future work, not demo claims.
