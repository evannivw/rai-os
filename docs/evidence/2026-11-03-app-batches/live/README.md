# Live whole-table generation and evolution

On November 3, 2026, clean source e384bcae83820704010977a81a880f78c6fde39d
completed two real Meta goals in exactly two calls. Both passed on their initial
attempt. No correction, additional retries or fixture fallback occurred; the
six-call bound was not reached. All proposals and UART events remain retained.

1. Create: `App: name, stock, moved. Button Move all: for every row, if stock>=2, stock=stock-2 and moved=moved+2. Defaults 0.`
2. Evolve: `Change Move all: for each row, if stock>=1, stock=stock-1 and moved=moved+1. Keep fields and data.`

The first call took 5437 ms provider latency, HTTP 200, 3869 tokens (3108 input /
761 output). The second took 3800 ms, HTTP 200, 3784 tokens (3272 input / 512 output).
Both reported a matching model and stop completion. These are provider timings,
not local execution benchmarks or evidence of superiority over another OS.

Meta created Stock Move (stockmove), with one all-scoped Move all program. Its
first instruction conditionally skips two calculations. The second response
preserved the same app, fields and button, changing threshold 2 to 1 and both
arithmetic amounts from 2 to 1. The host and guest admitted both definitions;
the verifier explicitly approved the fields and program on each installation.
There is no production branch for these IDs or arithmetic values.

Eight fictional rows were entered locally and never sent to Meta. Preview and
cancellation preserved data. First confirmation matched independently calculated
results with one revision change. The update preserved all eight rows before its
new action ran. Second confirmation matched a second independent oracle, again
with one revision change. In particular Item 4 began at stock=1 and was skipped by
the first program, then changed to stock=0 by the revised program. Each local
execution and the final reboot required zero further inference. The exact final
schema/rows/checkpoint and native screenshots are retained in this directory.

The live run demonstrates dynamic program generation and reviewed evolution on
this disclosed case. The separate offline tests establish failure in the final
row, all-or-nothing rollback, unrelated schemas and terminal locks. Do not present
those scripted adversarial proposals as live model responses. This remains an
eight-row bounded interpreter, shared EL1 runtime and host-backed checkpoint,
not arbitrary native-code generation or physical phone support.
