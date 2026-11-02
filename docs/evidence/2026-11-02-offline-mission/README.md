# November 2 offline mission evidence

Offline checks use an explicit oracle-supplied replay or dummy curl provider;
none is a live Meta result. `source: meta` inside the two provider-fixture reports
identifies the adapter under test; `transport: dummy_curl` identifies fake I/O.
Dummy token counts and latency are not real inference measurements.

Debug and release each passed 54 unit tests, Clippy with warnings denied,
eleven rejected plans, overwrite protection, eight corrupted packages and two
offline provider cases. The release semantic regression also passed real-QEMU
checks for the previous small corpus; it does not execute the new documents.

Only selected logs/plans/reports are archived. Deliberately corrupted directories
and symlinks are not archived. Valid replay artifact bytes are reproducible using
`scripts/offline_mission.py --source replay --output <new-directory>`.
A separate live/ folder will record the single frozen live probe if performed.

## Frozen live probe

`live/` records one actual Meta call from clean source
303d781e256974ed9de32e7f7e94e70457f60d46, observed at
2026-11-02T18:36:27.252054+00:00. No retries, fallback or answer tuning.
The model selected all nine expected files, five exact evidence quotations and
two pending actions. Independent checking passed: 8620 exact source bytes,
zero external reader resources. Provider latency was 3558 ms; complete planning,
checking and ZIP packaging took 4379 ms, excluding compilation. Usage was 2863
prompt + 769 completion = 3632 total tokens. This one development case establishes
neither superiority over a resolver nor generalization to unseen work.

Open `live/run/package/published/index.html` or unpack
`live/run/aurora-offline.zip`. The entire package is a HOST artifact; guest execution
is false. The integrated browser blocked file URLs; visual/disconnected-browser
review remains pending. Archive audit rechecked fixture hashes, exact package/ZIP
bytes and same-call usage. Raw logs retain original output paths and timestamps.
