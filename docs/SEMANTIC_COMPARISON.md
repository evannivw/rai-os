# Frozen semantic development comparison

November 1, 2026. Compare a deterministic domain resolver with Meta on the same
bounded task. This is a small development comparison, not a held-out benchmark,
Android comparison, general language test or proof of commercial differentiation.

## Frozen protocol (before live measurement)

The [protocol](../fixtures/semantic_comparison/v1/protocol.json) fixes twelve cases,
two repetitions per policy, rotating rules/Meta order by case plus repetition.
Forty-eight sequential trials, at most 24 Meta requests; no retries/fallback or
stopping early because a policy is failing. Compile release before timing. A fresh
guest session is used for every accepted selection. No policy sees future events.
All outcomes and available usage are retained. Source and fixture hashes are
recorded before running; the runner requires a clean committed tree.

The first six goals/expectations are unchanged from the previous development
probe. Six additional cases cover unit references, review time, conflicting
revisions, checklist content, an unavailable unit and an explicit no-default
instruction. Conflict accepts either clarification or a nonexecuting decline;
choosing an arbitrary revision is not accepted. Other expected decisions/memberships
remain exact. Cases and resolver were designed by the same implementer, who has
seen the previous Meta results; this is NOT held-out evidence. No population
accuracy, statistical significance or broader language advantage follows.

## Competent scoped baseline, not a general language engine

resolver.rs consumes the same compiled public catalog as Meta: six documents,
279 source bytes, identical permissions and limits. It implements the same handoff
dependency rule. It does not read case IDs or expected.json. Its declared grammar:

- Explicit document keys, unknown-key rejection and conflicting-revision handling.
- Pump letters/numbers and review times extracted from document content.
- Current/latest/newest/most recent, previous/older/oldest and before-switch
  references, with conflicts requiring clarification.
- Handoff/turnover/hand-over dependency closure to overview, one note and checklist.
- Explicit “do not assume”/“don't assume” guards, and two declared handoff-negation
  phrases. This is not general negation understanding.
- Limited domain synonyms and content overlap for gate/weather or cables/seals;
  explicit overview/empty selections. Unknown language/ties request clarification.

This resolver is deliberately engineered for the small known domain, with tests
for references/conflicts. It is not a deliberately delayed baseline, case-answer
table or bare keyword search. Its language coverage is limited and disclosed.
A future claim about flexibility needs independently authored unseen language and
appropriate stronger baselines; these development cases cannot establish it.

Meta uses the unchanged rai_selection prompt/schema: muse-spark-1.3, minimal
reasoning, max_tokens 1024, one request per case. Both policies have the same exact
request text/content, dependency policy, allowed membership, independent intent
oracle, canonical execution order, per-job guest gates and publication checker.
Both batch, both execute through the SAME binary/worker; neither has a cache warm
start. Neither receives expected answers. The fixed-case oracle runs afterward,
blocking wrong meaning before guest work equally for both. A permitted selection
alone does not prove intent correctness.

## Metrics and audit

Primary: independently correct decision and exact membership, plus verified
execution/publication for select outcomes. Clarify/decline executes no work.
Wrong decisions, provider/format failures, missing dependencies or failed artifacts
remain failed trials. Source permissions and invalid selection handling retain the
previous offline adversarial tests.

End-to-end milliseconds include interpreter process startup, provider overhead
if used, intent check, local handoff, QEMU boot/execution/publication and cleanup.
Builds are excluded. The oracle load before the timer is identical for both.
Report per-case times/medians alongside correctness; a fast incorrect answer is
not a win. Two repetitions describe these runs, not a latency distribution.
Meta adapter latency and known/unknown token usage are reported separately. Do
not count provider_diagnostic and semantic_response as separate requests.

semantic_comparison.py reconstructs acceptance and execution from saved raw traces
and expected answers, not only claimed success summaries. Corrupt success flags,
selections, historical states and publication bytes are rejected in offline tests.
Logs are observed software evidence, not cryptographic attestation of time or API
access. The common trusted-host/independent-checker boundary is unchanged.

```sh
. "$HOME/.cargo/env"
# Offline resolver/real-QEMU tests, no key or network:
python3 -B scripts/verify_semantic_comparison.py --debug
python3 -B scripts/verify_semantic_comparison.py
# Full frozen LIVE comparison, at most 24 calls. Configure your own .env.local.
# Source must be committed/clean; output directory must be new.
python3 -B scripts/semantic_comparison.py --output target/comparison-new
# Re-audit an existing run without inference:
python3 -B scripts/semantic_comparison.py --audit-only --output target/comparison-new
```

A nonempty result directory is never overwritten. Do not change settings or
expected answers after observing performance and keep claiming the same freeze.
Results from a later experiment require a separate protocol/evidence record.

## Observable change of goal: separate recorded-decision replay

```sh
python3 -B scripts/semantic_sequence.py --graphics --output target/sequence-new
```

This independently rechecks two already archived real Meta selections (historical
and latest), then replays them in ONE actual guest session. It performs no live
inference and is not part of the timing comparison. All second-goal input exists
before playback; it is not evidence of live event-driven semantic replanning.
Graphics pauses (1500 ms per state) are explicit presentation delays.

Expected sequence: three first-goal worker jobs -> checked publication -> new goal,
previous bytes still readable/historical -> two exact cache hits -> only notes@2
needs a new job -> checked replacement. Total four jobs, two publications. The
auditor verifies actual UART operations, exact artifacts, unchanged prior bytes,
fresh generations/tickets and completion counts. A malformed second selection
rejects the entire playback before boot. Each replay ends with explicit stop.
No new kernel, driver, UI design or generic agent framework is introduced.

For the three-minute video, label recorded decisions and show this concrete
continuity behavior. Present the comparison's actual outcome even if rules win.
The custom kernel is an experimental execution boundary, not evidence that this
feature needs a new OS. Preserve contribution dates and the unresolved kickoff /
build-period eligibility issue. Finish rehearsal/video/submission before adding
infrastructure; deadline remains November 4, 20:00 Guatemala (internal 18:00).

## Verification status before measurement

Source and offline checks will be committed before the 24-call run. Live results
are pending and will be appended separately without changing this protocol.


Offline verification passed: 50 host tests and all-target Clippy in debug/release;
twelve rules cases with independent real-QEMU auditing per profile; headless and
native two-goal replays in both profiles; malformed second selection rejected
before boot; corrupted reports, history/artifacts and an omitted comparison trial
rejected. Existing semantic regression suites passed in both profiles (six replay,
20 pre-boot rejection and four offline provider cases each). Formatting, local
links and diff whitespace passed. New traces are under
evidence/2026-11-01-semantic-comparison. No live measurements occurred before the
source/protocol commit.
