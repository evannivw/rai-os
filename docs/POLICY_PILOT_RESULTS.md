# First policy pilot: fixed rules outperform this per-action Meta configuration

Measured October 30, 2026 in Guatemala (the manifest uses October 31 UTC).
[Protocol and inputs](POLICY_PILOT.md) were frozen and pushed in commit
c649e592040a643ee3a00e5f8fa0b95f13b59713 before these measurements. No policy,
prompt, token limit, scoring rule or scenario was tuned after observing results.
This is an operational test of the implemented integration, not a general test of
model intelligence, a comparison with Android or the planned held-out evaluation.

## Observed result

Eighteen sequential real-QEMU trials used three development scenarios, three
policies and two repetitions. Every trace passed the independent audit, including
failed outcomes. EDF and value ordering each passed 6/6; Meta passed 1/6.

| Scenario | EDF passes | Value passes | Meta passes | Fixed completion range | Meta completed result |
| --- | --- | --- | --- | --- | --- |
| Steady | 2/2 | 2/2 | 1/2 | 152–163 ms | 8591 ms; score 9/13 |
| Priority change | 2/2 | 2/2 | 0/2 | 2571–2576 ms | No qualifying final result |
| Revision and quota change | 2/2 | 2/2 | 0/2 | 3583–3589 ms | No qualifying final result |

Fixed policies earned the full on-time score in every run. The second and third
scenario times include waiting for the shared events at 2500 and 3500 ms. Do not
interpret those waits as time spent indexing. All passing final artifacts include
exact current revisions and independently checked source bytes/token indexes.

Five Meta trials ended with `meta_incomplete_or_refused`. The frozen adapter uses
that error for a non-stop completion, unexpected message role, refusal or tool-call
response. It does not preserve which branch caused these five failures. We cannot
conclude that all were refusals, blame a particular token limit, or diagnose a
model reasoning failure from that combined code. No raw provider body was logged.
The bridge stopped safely; no failed partial set was reported as complete.

The one successful Meta trial chose the same sensible order as EDF: notes@1,
brief@1, empty@1. They became available at 2952, 6661 and 8514 ms. Notes met its
3000-ms target, brief missed 6000 ms, and empty met 9000 ms, giving 8 + 0 + 1 = 9.
The final set passed at 8591 ms. Returned provider latencies totaled 8433 ms;
request-to-response observations totaled 8448 ms, including transport/dispatch
boundaries. Thus the dominant measured cost in this successful run was waiting
for model decisions, not a better or worse index implementation.

Both fixed policies and that successful Meta trial spent four simulated credits
for the steady set. Partial failed runs sometimes spent fewer credits, but those
are not efficiency wins: missing the required output fails the primary gate.
No battery, CPU-energy or phone-performance improvement was measured.

## Usage and diagnostic limits

There were 14 Meta requests across six trials. Usage was available for nine
responses, totaling 8267 reported tokens. Usage for the five rejected responses
was not retained by the existing adapter; this is a partial total, not total
billable usage or proof the failed calls cost nothing. No monetary cost or hosted
inference energy is inferred.

Summing each decision request timestamp to its corresponding response/drain
boundary gives 39,998 ms of observed decision waiting across all Meta trials.
The `inference_ms` field in results.json instead sums only returned provider
latencies and is incomplete on failed responses; zero there means no usable
latency was reported, not zero time spent. End-to-end trial times still include
those waits. All raw timing and failure evidence remains available.

The provider configuration was the existing Muse Spark 1.3 endpoint, minimal
reasoning and 512 maximum output tokens. Responses, failures and stale decisions
were kept; no retries, alternate provider, fixed fallback or rerun replaced a
failed trial. A separate diagnosis should preserve explicit finish/refusal
categories and available usage before testing any new provider configuration.

## What this changes about the next step

This pilot does not support using a remote model to select every tiny index job.
The successful model choices were reasonable, but their latency consumed the
short target windows. Provider reliability also prevented five comparisons from
isolating scheduling quality. Six trials and three small known fixtures are too
few and too narrow for a general conclusion about AI planning.

Keep deterministic execution, validation, caches and fast scheduling in RAI-OS.
A next hypothesis is to ask the model for a larger plan or a higher-level goal
translation, then let bounded deterministic code carry it out and consult the
model only for meaningful changes. That may amortize inference cost; it remains
a hypothesis requiring a new frozen experiment with equally capable baselines.
Do not weaken the baseline or choose only favorable interruptions to manufacture
a win. First improve rejection diagnostics; retain these original results.

## Evidence and reproduction

- [Generated report, including every trial](evidence/2026-10-30-policy-pilot/REPORT.md)
- [Metrics and timelines](evidence/2026-10-30-policy-pilot/results.json)
- [Frozen source, binary/input hashes and exact trial commands](evidence/2026-10-30-policy-pilot/manifest.json)
- [Successful Meta trace](evidence/2026-10-30-policy-pilot/2-steady-meta.jsonl)
- [One retained failed Meta trace](evidence/2026-10-30-policy-pilot/1-steady-meta.jsonl)

All 18 JSONL/stderr pairs and exact published artifacts are retained in the same
folder. Audit without any API request:

```sh
python3 scripts/compare_policies.py --audit docs/evidence/2026-10-30-policy-pilot
```

For fresh measurements, follow POLICY_PILOT.md with a new output directory and
your own local key. Logs and hashes support reproducibility; they are not
cryptographic proof of provider access or guaranteed timings on another host.
