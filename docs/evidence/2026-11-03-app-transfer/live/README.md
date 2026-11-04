# One actual Meta-generated transfer app

Clean source ce219caf694953d3a13cd4394e39469b8a989a0e made exactly one Meta call:
HTTP 200, matching model, stop completion, 8119 ms, 4832 tokens (4066 input / 766
output). No correction, retry or fixture replacement. Actual app: Transfer Balances.

Meta supplied the four fields, optional transfer rule and Balances page containing
sum and bars. The declaration is preserved exactly. Two independently entered
fictional records had balances 12 and 5; moving 4 produced 8 and 9 while total 17
remained. Preview/cancel changed nothing. Quantity 99 was rejected. Fresh QEMU boot
restored the exact rule, page and current records without another inference call.
The supplied-checkpoint demo entry validated and copied the pre-operation state.

The model generated no calculated field in this run. The raw report's first check
ends with the overly broad phrase "derived values recomputed": that clause was
asserted only in the offline branch, not this live run. The report is preserved
verbatim; the verifier's future annotation has been corrected. Here the independently
checked derived result is the page sum, not a calculated record column. Offline
unit/QEMU tests separately cover both-row calculated fields and late formula failure.

Provider latency is not an OS performance benchmark. UI screenshots are real guest
captures, records are test data, apps still share EL1, and persistence/HTTPS use the
host. showcase.json is the pre-operation checkpoint; device.json contains final
post-verification edits. Reopen a COPY using demo.py saved --checkpoint; never claim
saved recovery is fresh generation. The three-minute video is still to be recorded.
