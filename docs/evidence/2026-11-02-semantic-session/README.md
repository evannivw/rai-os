# Retained semantic session evidence — November 2

Offline replay/rules use real QEMU execution. Fake provider sessions have source
meta because they exercise the Meta adapter, but transport is dummy_curl and all
provider latency/token values there are synthetic. They are NOT live inference.

Debug and release passed 55 unit tests and Clippy with warnings denied. Both
profiles passed replay/rules continuity, five corrupted-trace rejections, eight
invalid-stream rejections and two fake-provider sessions. The deliberately wrong
middle answer was rejected without changing the prior guest publication; the
later valid event still completed. Native-display release replay/rules passed.
The previous recorded sequence passed in debug; the full previous comparison
verification passed in release (12 rules cases with real guest work and corruption
checks). Archived new traces were re-audited after copying.

The strict stop test initially exposed ignored unknown fields in a unit enum
variant. A strict empty struct variant fixed that before final passing tests.
No failed result is presented as a passing run.

Live results are not included in offline/. The frozen three-call probe is planned
from clean committed source; a later live/ directory will preserve its outcome.


## Frozen live result

`live/` contains the actual three-call Meta probe and rules session from clean
source 948ae1926d31c1ab80f294cc37482dfa68c2cf98. Observed start:
2026-11-02T19:15:56.029591+00:00. No retries/fallback or post-answer tuning.
Each policy made three correct decisions, produced two verified publications,
executed four guest worker jobs, reused two exact indexes on update and preserved
previous bytes. Ambiguity dispatched no work. Meta asked which operational note
was intended (Pump A or Pump B); the next controlled event specified current.

Whole-session times excluding build: rules 661 ms, Meta 7121 ms. Meta interpretation
process times: 2805, 2229, 1920 ms; three requests used 2910 total reported tokens.
No accuracy advantage or speed win is shown. These are known small development
cases and one pass in fixed order, not a generalization or statistical study.
Guest execution is real; event delivery and independent fixed-case intent checking
are host controlled. Both traces and all provider outcomes are preserved. The
archive was re-audited, including same-call diagnostic/response metadata.
