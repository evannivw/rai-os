# One rejected live generation, not a successful app demonstration

Clean source a23795927a456acb5d58fd8d8918b5e518d39eab made one Meta request.
The provider returned HTTP 200 with a decoded proposal, model identity matched,
6450 ms provider latency and 3576 reported tokens (2879 prompt / 697 completion).
Shared host validation rejected it with formula_syntax; execution=false. No app
was installed, no retry/fallback occurred. The final UART catalog remains
revision=1; apps=-. No device.json was created because nothing was committed.

The original verifier report says AssertionError because its install assertion
failed. The retained provider files show the actual rejection code. At this source
revision the rejected proposal itself was discarded, so its exact invalid
expression cannot be inspected or reconstructed from this evidence. Do not claim
to know which expression or instruction caused the failure.

A subsequent host-only diagnostic fix retains future rejected proposals, and the
verifier now includes the rejection code in its report. It was tested offline
using a clearly synthetic malformed expression. That test neither repairs this
live response nor establishes a successful live generation.
