# Final three-minute recording runbook

Uploaded video: [Watch the three-minute demo](https://drive.google.com/file/d/18e2NPFsIktJhBC-T7ZY4ANAfQhD9PvaO/view?usp=sharing).

The claim: describe a bounded tool, review the generated application, then use it
locally inside our own Rust/QEMU OS. Meta creates the declaration; deterministic
Rust code validates and executes it. This is not a claim that model latency beats
a scheduler, that all apps are expressible, or that the prototype replaces Android.

## Ready-to-rehearse actual example

One actual Meta generation of Transfer Balances passed creation, approval, movement,
rejection and reboot. Use the saved checkpoint command in [APP_TRANSFERS.md](APP_TRANSFERS.md)
to rehearse the exact data path without another API call. Its raw proposal and
provider diagnostics are retained; generation took 8119 ms. The video itself and
submission have not been completed by these verification runs.

## Recording sequence

| Time | Picture | Explain |
| --- | --- | --- |
| 0:00–0:20 | Real boot and native guest Home | Educational AArch64 Rust kernel in QEMU; phone-shaped guest UI. |
| 0:20–0:55 | Type the transfer-app prompt; show actual proposal and approvals | Meta receives a versioned capability catalog. RAI-OS checks bindings and bounds. Keep actual waiting time visible or label an edit/time cut. |
| 0:55–1:25 | Open the generated app; enter Warehouse 12/4/Dispatch and Dispatch 5/1/Warehouse | User data creates the example; this is not a special compiled inventory app. A preloaded data cut must be labelled. |
| 1:25–1:55 | Preview 12→8 and 5→9, Apply, then page total 17 and bars | Two records change together; computed views update locally without another model call. |
| 1:55–2:15 | Set quantity 99 and attempt transfer | The OS rejects an invalid operation even though the app was proposed by AI. Both balances stay intact. |
| 2:15–2:40 | Show actual prior app history/evolution or saved reboot | Definitions and data can be recovered. Say whether showing Git definition rollback or checkpoint reboot; they are different. |
| 2:40–3:00 | Source architecture and public reproduction commands | What was built during the event, current limits, and the mobile direction. |

The transfer-generation prompt is in [APP_TRANSFERS.md](APP_TRANSFERS.md). Rehearse
from a fresh output directory. A successful recorded actual proposal can be reopened
with `demo.py saved --checkpoint PATH`; label it saved/recovered, never live generation.
Do not present offline verifier proposals as Meta output. A model decline is a real
outcome; use the previously verified saved example with transparent labelling.

If time is tight, record one coherent create/use/reject/recover story. Pages,
formulas, workflow rules and Git support that story; a feature list is secondary.
Record only features that actually worked. Existing evidence includes negative
performance experiments and should remain in the repository.

## Before submission

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
  18:00. These instructions do not mean the video or submission has been completed.

After this capability is verified, prioritize rehearsal, final screen polish,
reproduction and recording over another unverified subsystem. Future capability
families: stable record relations, richer text/date expressions, reusable layouts,
and separately permissioned event services. They are future work, not demo claims.
