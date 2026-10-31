# Development policy pilot v1

These three scenarios are frozen before live measurement. Rules, credit formula,
fixed-policy tie breaks, timing and scoring are specified in
../../../docs/POLICY_PILOT.md. All policies see only the currently activated event.
They use the same six existing public fixture records and real EL0 index worker.

Priority weights and cumulative credit limits are experiment inputs, not hardware
telemetry. The final required set must verify before any score counts. The quota
is enforced by a shared host controller, not by a new guest kernel scheduler.

These are development scenarios. Do not call them held-out cases, redesign them
after observing model performance, or treat one favorable result as superiority.
The separate original review_pack/v1 fixture and its contract remain unchanged.
