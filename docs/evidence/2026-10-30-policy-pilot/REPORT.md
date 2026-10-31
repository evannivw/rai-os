# RAI-OS policy pilot results

Source commit: `c649e592040a643ee3a00e5f8fa0b95f13b59713`. Started: 2026-10-31T03:57:00.871771+00:00.
Development scenarios; this small sample is not a held-out or statistically powered evaluation.
Real QEMU indexes and wall times; credits are host-simulated accounting, not battery or CPU measurements.

| Scenario | Policy | Passed | On-time score range | Median completion ms (passed only) | Credits range | Decisions | Reported model tokens |
| --- | --- | --- | --- | --- | --- | --- | --- |
| steady | edf | 2/2 | 13–13 | 159.5 | 4–4 | 6 | 0 |
| steady | value | 2/2 | 13–13 | 155.5 | 4–4 | 6 | 0 |
| steady | meta | 1/2 | 0–9 | 8591 | 1–4 | 5 | 3660 |
| priority-change | edf | 2/2 | 16–16 | 2575.5 | 4–4 | 6 | 0 |
| priority-change | value | 2/2 | 16–16 | 2572.0 | 4–4 | 6 | 0 |
| priority-change | meta | 0/2 | 0–0 | — | 1–3 | 6 | 3558 |
| revision-quota | edf | 2/2 | 13–13 | 3584.5 | 6–6 | 8 | 0 |
| revision-quota | value | 2/2 | 13–13 | 3587.0 | 6–6 | 8 | 0 |
| revision-quota | meta | 0/2 | 0–0 | — | 0–1 | 3 | 1049 |

All trials, including failed and unauditable ones, remain in results.json and raw JSONL/stderr files.
Completion includes boot, decisions/inference, UART and independent artifact checking. Build time and the later evidence audit are outside each trial. Provider draining/cleanup is in total_elapsed_ms.
The native display is disabled for every timed trial. No simultaneous QEMU competitors.

Meta versus the stronger fixed result for each paired scenario/repetition: 0 wins, 0 ties, 6 losses, 0 unauditable pairs.
Ranking: valid final completion first, then weighted on-time score, then completion time, then credits. This is not a comparison with Android or evidence of general OS superiority.

## Per-trial outcomes

| Trial | Passed | Reason | Completion ms | Score | Credits | Stale choices |
| --- | --- | --- | --- | --- | --- | --- |
| 1-steady-edf | True | verified | 163 | 13 | 4 | 0 |
| 1-steady-value | True | verified | 152 | 13 | 4 | 0 |
| 1-steady-meta | False | meta_incomplete_or_refused | — | 0 | 1 | 0 |
| 1-priority-change-value | True | verified | 2573 | 16 | 4 | 0 |
| 1-priority-change-meta | False | meta_incomplete_or_refused | — | 0 | 3 | 1 |
| 1-priority-change-edf | True | verified | 2576 | 16 | 4 | 0 |
| 1-revision-quota-meta | False | meta_incomplete_or_refused | — | 0 | 0 | 0 |
| 1-revision-quota-edf | True | verified | 3586 | 13 | 6 | 0 |
| 1-revision-quota-value | True | verified | 3589 | 13 | 6 | 0 |
| 2-steady-value | True | verified | 159 | 13 | 4 | 0 |
| 2-steady-meta | True | verified | 8591 | 9 | 4 | 0 |
| 2-steady-edf | True | verified | 156 | 13 | 4 | 0 |
| 2-priority-change-meta | False | meta_incomplete_or_refused | — | 0 | 1 | 0 |
| 2-priority-change-edf | True | verified | 2575 | 16 | 4 | 0 |
| 2-priority-change-value | True | verified | 2571 | 16 | 4 | 0 |
| 2-revision-quota-edf | True | verified | 3583 | 13 | 6 | 0 |
| 2-revision-quota-value | True | verified | 3585 | 13 | 6 | 0 |
| 2-revision-quota-meta | False | meta_incomplete_or_refused | — | 0 | 1 | 0 |
