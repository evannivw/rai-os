# Provider diagnostics before batch planning

Implemented October 31, 2026. This increment makes provider outcomes observable;
it does not implement batch planning or demonstrate an AI speedup. The
[first pilot](POLICY_PILOT_RESULTS.md) and its five ambiguous failures remain
unchanged. Their exact causes cannot be recovered from the old captures.

## Where the project stands

The guest already runs bounded workers, caches exact revisions, rejects stale
proposals, retains a prior verified publication and renders current/historical
state. The next hypothesis is one model proposal for multiple operations, followed
by fast deterministic execution. A later semantic fixture must require useful
interpretation/dependency decisions, not merely more tiny index tasks.

Current sequence: diagnose provider outcomes (this increment), implement bounded
batch proposals, independently specify the semantic workload, then freeze a fair
comparison including total model/transport time. Batching alone is an overhead
hypothesis, not proof that AI outperforms a good fixed policy.

## Structured outcome

For each consumed live provider outcome, the bridge emits one provider_diagnostic
JSON event with state_version, outcome (decoded or rejected), a fixed error code
when rejected, and diagnostic metadata. Replay emits no fabricated provider event.
The shared asynchronous source and the single-request CLI use the same reporter.

Decoded means envelope and proposal JSON parsing succeeded. Domain permissions,
revision admission and guest execution still run afterward and may reject it.
It does not mean the task completed, an operation was authorized or an artifact
was verified. A rejection cannot dispatch an operation.

Metadata contains elapsed provider-call milliseconds, pipeline stage, HTTP status
when parsed, curl exit code when observed, expected-model match, choice count,
normalized finish reason/role, refusal/tool-call presence, and available token
usage. Unknown strings become other. Response text, refusals, tool-call arguments,
headers, credentials and raw provider bodies are not logged in these diagnostics.
The error code is generated locally rather than copied from provider prose.

| Condition | Error category |
| --- | --- |
| finish_reason=length | meta_finish_length |
| finish_reason=content_filter | meta_finish_content_filter |
| finish_reason=tool_calls or function_call | meta_finish_tool_calls |
| Any other non-stop finish | meta_finish_other |
| Unexpected role | meta_unexpected_role |
| Non-null refusal | meta_refusal_present |
| Non-null tool-call field | meta_tool_calls_present |
| Wrong model / not exactly one choice | unexpected_meta_model / unexpected_meta_choices |
| Missing content / malformed proposal | missing_meta_content / invalid_proposal_json |
| Malformed envelope | invalid_meta_response |
| Non-200 HTTP response | meta_http_STATUS |
| curl timeout / other transport failure | meta_timeout / meta_transport_failed |

If several conditions coexist, rejection checks have a deterministic order:
typed envelope, model, choice count, finish reason, role, refusal, tool calls,
content and proposal parsing. Available flags remain visible even when an earlier
condition determines the code. Duplicate known envelope fields remain rejected
by typed deserialization from the original bytes.

## Accounting and limits

Usage is retained on rejected HTTP-200 envelopes when its numeric fields can be
read, including length/refusal responses. Missing/malformed usage is null, not
zero. Non-200, invalid JSON and interrupted transport may have no usable usage.
Numbers are provider-reported tokens, not independently measured billing or cost.

Legacy model_response/decision_response records summarize the same successful
call; do not add their usage to provider_diagnostic usage a second time. Within
one bridge process only one provider call is outstanding; diagnose each consumed
outcome once in event order with its original snapshot version. If a pending call
is joined and discarded during unrelated early teardown without being consumed,
this reporter does not claim to capture its result. A disconnected worker thread
is a separate adapter error, not a fabricated provider response.

latency_ms covers the provider method, including local request preparation,
transport and decoding, on success or failure. It is not pure model compute time
or total QEMU-task completion time. The elapsed timer now begins before local
request preparation; the original pilot timing records are not rewritten.

Model muse-spark-1.3, endpoint, prompts, strict one-operation schema, minimal
reasoning, max_tokens=512, response cap, TLS checks, timeouts and no-retry policy
are unchanged. The unchanged instructions/request-builder section SHA256 is
9e04542290f5df1dcc0c4e58d7f6357d5aacb80a2471ff77b0a0def88245781d.
A future batch schema or provider-setting change must be a separate experiment.

## Verification and one planned live probe

```sh
. "$HOME/.cargo/env"
python3 scripts/verify_provider_diagnostics.py
python3 scripts/verify_provider_diagnostics.py --release
```

Each profile passed 25 explicitly offline curl-fixture cases, with dummy keys and
real QEMU for valid execution/asynchronous cleanup. They cover specific refusal/
finish causes, partial metadata, missing usage, HTTP/transport errors, malformed
and duplicate envelopes, post-decode domain rejection, one-call/no-retry behavior,
and sentinel checks against leaking raw content or keys. Offline fixtures are not
Meta observations, even though they exercise the live adapter code path.

All 41 host tests and all-target host Clippy passed in debug/release. Release
regressions passed for single-operation replay and whole-set guest bridging.
No kernel/display/protocol implementation changed. Offline traces are retained in
[evidence](evidence/2026-10-31-provider-diagnostics/offline-debug.json).

After committing this code, make exactly one ordinary live request for the latest
notes count, with the unchanged settings and no retry or fallback. Preserve its
outcome, diagnostic and exact code/binary identity separately. This is a diagnostic
smoke test, not a repeat of the pilot or evidence about its five historical failures.
No live result is claimed by this pre-probe document section.

## October 31 live probe: normal completion, no historical diagnosis

After committing and pushing 95bd62810929d41562a61f3e1a068db5f96fe9ff, made the
single planned request with unchanged model/settings and no retry. Meta returned
HTTP 200, finish_reason=stop, one assistant choice, matching model, no refusal or
tool calls. Provider-method elapsed time was 1592 ms. Reported usage: 376 prompt,
153 completion, 529 total tokens. The CLI wall time was 1702 ms.

The admitted proposal selected notes@2. Real QEMU returned 44 bytes and 9 tokens;
its boot-plus-operation segment was 103 ms. The existing independent record
checker accepted the captured result. The diagnostic and legacy response summary
agree on usage/latency and describe the same single call, not two calls.

[Raw outcome](evidence/2026-10-31-provider-diagnostics/live/stdout.jsonl) and
[code/binary identities](evidence/2026-10-31-provider-diagnostics/live/manifest.json)
are preserved. This small single-record request differs from the earlier
multi-record scheduling trials; its time is not a measured speedup. The original
five failures remain undiagnosed because their detailed response metadata was not
retained. This probe validates instrumentation, not batch or semantic planning.
