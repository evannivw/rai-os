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
