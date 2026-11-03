# Live Meta generation of composed pages

Clean source 0d436bc62a390bd9163766582533a3ecfd71cc5d made exactly one Meta call.
HTTP 200, matching model, stop completion; provider latency 7715 ms and 4489 tokens
(3685 input / 804 output). No retry, correction or replacement fixture.

Meta returned Name Amount Report with actual Summary and Report pages: sum metric,
signed bars, literal text and a two-column table. The verifier reviewed each page
before installation. Eight fictional test records were then entered locally. Their
amounts sum to 36; editing -5 to 9 produces 50. Page navigation leaves state intact,
and fresh QEMU boot recovers the exact generated definition and current records
without inference. The table puts Amount before Name, as in the actual declaration;
no generated schema was rewritten to fit the demonstration.

The app definition and pages came from Meta. The records and arithmetic expectations
are independent disclosed verifier inputs, not model-generated data. Actual proposal,
diagnostics, request, native screenshots, UART, report and checkpoint are retained.
Offline negative-bar pixel, filter, rejection and unrelated-app checks are separate
evidence. Provider latency is not an OS performance benchmark.

For inspection, copy device.json to a new untracked file and start Studio in replay
mode using that copy; see APP_PAGES.md. This reopens a saved model result and does
not make another API call.
