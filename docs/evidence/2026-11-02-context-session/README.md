# Constant-goal context cycle evidence — November 2

Debug/release/graphics-release JSON archives preserve relative paths and exact
text of each offline output in raw_files. They use real guest execution with
replay, rules or a dummy curl provider. source=meta in dummy-provider reports
identifies the adapter under test, not real inference. Synthetic usage is not a
live measurement. The manifest/summary of a later live probe belong in live/.

All archived new traces were audited against the context fixture. The unchanged
previous live semantic-session archive still passes the shared auditor. 57 host
unit tests and Clippy passed in debug/release. Context suites passed in both
profiles; native display replay/rules and previous semantic-session regression
passed in release. See checks.json for raw command logs. No visual redesign.

Each headless context suite also checked seven invalid observation inputs before
provider access and altered-context evidence, in addition to five lifecycle
corruptions, eight invalid stream suffixes and two fake-provider sessions. The
constant goal and exact current observation are inspected by the fake transport.

To inspect raw_files, decode a JSON archive and read values directly. Reconstruct
only these trusted repository paths in a fresh temporary directory when rerunning
the auditor. Do not treat an arbitrary external archive as trusted filesystem paths.
