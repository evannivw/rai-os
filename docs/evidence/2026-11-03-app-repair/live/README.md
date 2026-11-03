# Actual Meta generation with composed behavior

On November 3, 2026, clean source 5984a204aa20fe5a3c412099b93e1aa3b6ed5350 ran
one goal with a maximum of three calls. Exactly one call occurred, and the first
proposal passed host and guest validation. No correction, retry or fallback was
needed. The loop.json and aggregate/per-attempt logs retain that distinction.

Goal: `App: name, stock, moved. Add Move: require stock>=1, stock=stock-1, moved=moved+1. Numeric defaults 0.`

The configured Meta model returned HTTP 200 with matching model, stop completion,
4232 ms provider latency and 3495 tokens (2934 input / 561 output). This latency is
not a kernel execution time, a performance comparison or a promise for future calls.
The actual model proposal named the app Stock Mover (stockmove), with a require
condition and two calculate steps. No production code recognizes this app ID.

The QEMU verifier confirmed no installed app before both field/program approvals.
Preview changed no rows or revision. Confirming Move changed Widget from stock=3,
moved=0 to 2,1 with one revision increment, and Other from 7,4 to 6,5. Empty stock
0,0 failed behavior_condition_failed and remained unchanged. Restart restored the
same complete schema and every row without another model request. Captures show
the guest-owned display; report.json retains the exact final state.

This establishes actual generation, admission, local behavior execution and
checkpoint recovery for this disclosed example. Automatic correction is verified
separately using invalid scripted responses; this live run did not exercise it.
It is not evidence of arbitrary app generation, semantic correctness for unseen
requests, physical hardware control or isolated application processes.

All JSON/JSONL, UART and PNG evidence from this run is retained here. No keys,
real user data, fallback fixtures or additional calls are included. The earlier
failed live request under a237959 remains preserved in the separate behavior
increment; this result does not erase or reconstruct its discarded proposal.
