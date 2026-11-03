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


## Frozen live result

Live probe source: d69c631de11e66946d575d3f5b745ff197b2ee69, clean tree.
Observed UTC start: 2026-11-03T04:47:03.792016+00:00 (November 2 in Guatemala).
Exactly three new Meta calls; no retries/fallback or post-answer tuning. Goal text
was identical throughout each policy session; current structured observations
changed. Both policies passed 3/3, published twice and executed four guest jobs,
reusing two indexes on update and preserving earlier bytes. The conflict event
performed no guest work. Live measurement was headless; native display was tested
separately offline. Source labels/events remain controlled fictional fixture data.

Whole-session times: rules 581 ms, Meta 11541 ms. Three Meta interpretation process
times: 3993, 3452, 3917 ms. Reported total usage: 2881 tokens across three requests,
counted once each. Both policies tie in correctness; rules are faster here. This
is not a harder unseen study and does not demonstrate a decision-quality win.

`live/` preserves manifest, unchanged-goal input streams, observations, all raw
provider/guest traces and the summary. Archive auditing rechecked exact fixture
hashes, original context, independent decision/guest acceptance and same-call usage.
