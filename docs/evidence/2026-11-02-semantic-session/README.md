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
