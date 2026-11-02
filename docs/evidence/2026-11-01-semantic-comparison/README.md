# Semantic comparison preparation — November 1, 2026

Offline archives preserve exact file text from resolver and recorded-selection
replay tests, including intentionally invalid inputs and corrupted-manifest checks.
Native archives use actual QEMU graphical playback with explicit presentation
pauses. Both use previously archived Meta selections, NOT new live inference.
The baseline reads no expected answers. checks.json lists verification outcomes.

The committed source/protocol defines a later 48-trial development measurement
(24 calls maximum). Live measurements are not yet in this source increment and
will be appended separately without replacing failures. See
[SEMANTIC_COMPARISON.md](../../SEMANTIC_COMPARISON.md).


## Live comparison

`live/` retains all 48 frozen trials from source 74cdecd, including 24 Meta calls,
with no retries or changed expectations. Rules 24/24, Meta 24/24; rules were faster
in every paired case/repetition. Fourteen selected publications and 34 real worker
jobs per policy; ten nonexecuting outcomes per policy. All 24 usage reports sum
to 22074 tokens. Re-audit without inference from the repository root:

```sh
python3 -B scripts/semantic_comparison.py --audit-only --output docs/evidence/2026-11-01-semantic-comparison/live
```

All archived traces, selections, artifacts and schedule/fixture checks passed.
This is a known-domain development comparison, not held-out or general OS evidence.
