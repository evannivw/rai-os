# Batch integration evidence — November 1, 2026

Offline tests only in this source increment. `offline-*` contains exact observed
wire events and artifacts, scenario definitions and exit codes. `transport` labels
replay versus dummy-key curl fixtures. Native files used the actual guest display;
no screenshot or performance inference is implied. `checks.json` records check
summaries. Delayed real-QEMU replies and delayed checker cases are explicit fault
fixtures, not runtime measurements. No key, raw provider body or API request is
included. Source ownership, limits and commands: [META_BATCH.md](../../META_BATCH.md).

The source commit containing these files defines the implementation. One live
three-record smoke probe is planned after committing, without retry or fallback;
its evidence will be stored separately and will not replace these records or the
original policy pilot. This fixture establishes integration, not semantic benefit.
