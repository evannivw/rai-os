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
