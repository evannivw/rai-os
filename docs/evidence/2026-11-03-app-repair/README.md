# Bounded app correction: offline evidence

Recorded November 3, 2026. These checks used the changed working tree based on
a20bf4716be151643eae22f87a705649ef45a8a5. Reports intentionally retain dirty=true;
source.json records tested code hashes and tool versions. No Meta calls were made.
The subsequent source-freeze commit, not the parent hash, contains this increment.

- Debug and release QEMU: corrected proposal -> review -> approval -> local
  execution on three independently checked rows -> exact reboot recovery.
- Both profiles passed 13 adversarial cases: exhausted three attempts, guard
  removal, cancellation of invalid/valid results, replacement revision, synthetic
  transport failure, session budget, clarification, unsupported capability,
  malformed stdout/null, missing plan, malformed diagnostic array and synthetic
  timeout with retained partial output. The deadline is configured, not measured
  by waiting 110 seconds. Synthetic adapter failures are explicitly disclosed.
- 90 Rust host/shared-domain tests passed. Kernel Clippy with/without display,
  host Clippy, formatting and Python syntax checks passed.
- Existing apps/RFB, behavior runtime, evolving app journey and saved/live entry
  checks passed. Console debug/release each passed 37 protocol checks and boot.
- Linux ARM64 Docker, network disabled: the same correction verifier and existing
  saved demo passed. This does not establish native Windows or physical phone support.

Debug retains all per-attempt logs, feedback, supplied proposals and UART captures.
Release retains its report, success/recovery transcripts and captures; Linux retains
its report, build manifest and saved-demo output. Regression summaries are separate.
The supplied proposals are test data, not model-generated evidence or production
application templates. Raw log bytes are preserved.

Earlier development runs exposed stale test expectations: diagnostic key count,
repair error name and malformed app_plan counting. Those tests were corrected;
the retained final runs passed. No production validation was relaxed to pass them.

Reproduce using docs/APP_REPAIR.md. Live integration will be recorded separately
from clean committed source, with at most one goal and three calls. No live success
is established by this offline report.
