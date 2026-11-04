# Repository-wide audit — November 4, 2026

Base: `1e88cdff509a22c5f071104f80ebef90fc887f64`, with the audit changes in
this commit. Jarrol reported submitting the project on November 4; no exact
submission time or independently inspected platform receipt is claimed.

## Scope and corrections

Reviewed the app domain/controller, schema/update paths, formulas, workflows,
transfers, persistence, Meta boundaries, kernel contracts, portable entry points
and maintained documentation. No additional confirmed production Rust defect was
found in this pass; that is not a guarantee that none remain.

- Replaced the accumulated README with a current evaluator entry point and a
  complete documentation index. Kept the previous README at its Git permalink.
- Finalized SUBMISSION.md, kept the old description URL as a compatibility pointer,
  and recorded the participant-reported submission. Dated evidence remains unchanged.
- Corrected stale formula, batch scope, navigation, storage and host/guest claims.
  Historical plans now identify their original scope. The runbook matches the
  uploaded eleven-scene, two-request Ledger recording.
- Demo startup now explains existing/missing/invalid paths and manifests without
  a traceback; a missing/non-file seed leaves no partial output. Resume tolerates
  missing optional START.txt while preserving checkpoint bytes.
- Direct legacy VNC now binds to loopback. The container explicitly selects its
  internal listener; documented Docker publication stays on host loopback.

## Verification actually performed

- `scripts/check_docs.py`: offline relative paths, heading anchors and index
  coverage across maintained documentation. See docs-check.log for the final count.
  This does not check external availability or factual accuracy.
- Python AST parsing for all 90 Python files; shell syntax and whitespace checks.
- Deterministic host controller stress: **1,000,000 actions without panic**.
  This is panic resistance, not complete semantic or security verification.
- macOS real-QEMU CLI: saved startup, resume, recovery without instructions,
  unchanged saved bytes, clean rejection paths and empty live startup/shutdown.
- Rebuilt Linux Docker image: saved app use/recovery, CLI startup/recovery and
  direct VNC checks with container networking disabled.
- Separate legacy container desktop: real guest file operations, 480x800 VNC
  handshake, loopback host publication, explicit internal listener and clean exit.

All actual listeners were inspected through QMP or the VNC handshake. Only new
verification sessions were stopped; existing user sessions were left untouched.
No real Meta request was made and no author's private key file was read. The live
startup test uses a synthetic format-only key and submits no prompt.

The initial Linux VNC verifier invocation used the wrong kernel path and failed
before startup. Its output is retained in docker-vnc-final.log. Repeating with
`target/display/aarch64-unknown-none-softfloat/release/rai-os` passed, as recorded
in docker-vnc-corrected.log. This was an audit invocation error, not a guest fix.

The earlier [114-test Rust review](../2026-11-04-final-review/README.md) and
[actual Meta recording](../2026-11-04-submission/README.md) retain their own source
revisions. Production Rust did not change in this pass; those tests and live
requests were not rerun. The uploaded video and its evidence were not rewritten.

## Public metadata and remaining boundaries

An unauthenticated GitHub API request confirmed the repository is public.
The external About field still contains the older sentence retained in
`github-public-metadata.json`. Changing that field was not possible in this
session: no authenticated GitHub API tool or browser was available, and native
computer access reported a locked Mac. Proposed replacement:

> Rust AArch64 OS prototype with Meta-generated apps, checked local execution, and a native QEMU interface.

Generated apps still share EL1 privilege. Native phone support, arbitrary native
apps and guest disk persistence remain future work. Linux containers on a Mac do
not establish execution on a physical Windows host. Check the current
[app limits](../../APP_LIMITS.md) before extending the demo claims.

Reports and raw logs below accompany `summary.json`. Portable build provenance
records exact source and binary hashes; local paths and ephemeral ports in logs
are observations from these runs, not required reproduction settings.

The Linux image was built before the final audit documentation and raw-log Git
attribute were added. All recorded executable/build sources match this tree;
only `.gitattributes` differs by the new audit-log preservation rule. The image
hash and its original build manifest are retained rather than relabelled.
