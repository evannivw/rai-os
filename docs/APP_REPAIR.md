# Bounded application correction

The app bridge can send a rejected arithmetic definition back to Meta with the
validator's exact error. This is a small propose -> validate -> correct loop,
not permission to expand the runtime or weaken the user's reviewed rules.
Successful correction still produces an ordinary app proposal that the user must
review before installation. Running the installed app needs no further inference.

## Boundaries and responsibilities

Meta receives the original goal, runtime catalog, original schema/request metadata,
the rejected proposal and a structured diagnostic. Actual record rows, API keys
and host file access are not part of this app contract. A rejected proposal is
untrusted data, including any instructions embedded in its strings.

The Rust host validator checks each candidate using the shared app domain code.
The Python bridge coordinates the bounded attempts and transports accepted
proposals. The guest independently checks the current request and revision,
validates installation and presents the existing field/button approval screens.
It never stages a rejected candidate as an installed application.

The kernel's boot, memory, exceptions and IPC foundations remain underneath this
work. This is an app-generation coordination improvement, not a new scheduler,
driver or isolation mechanism. The app interpreter/controller still share EL1
privilege; the host bridge remains trusted. This does not make arbitrary native
code safe to execute or establish a complete microkernel.

## What may be repaired

Only these decoded-proposal validation failures are eligible:

- `formula_syntax`: an arithmetic expression does not follow the grammar.
- `formula_unknown_field`: an expression references an unavailable numeric field.
- `formula_limit`: an expression exceeds the bounded arithmetic language limits.

The validator recomputes the rejected proposal's error against the original
request. Feedback cannot simply label a different failure as repairable.
A repair may change only a calculated field's `formula`, a `calculate` step's
`value`, and the human-readable proposal reason. Original field identities,
types/defaults, capabilities, workflow, guards, button definitions, step order,
branch spans and all other values must remain identical. A candidate that removes
a `require` guard to avoid an error fails `app_repair_scope_denied`.

The model may instead return `clarify` or `decline` with no app specification.
Malformed responses, unavailable capabilities, incompatible migrations, stale
requests, transport failures and changed permissions do not trigger this cycle.
A new user request is needed to reconsider the application's structure.

Arithmetic validity alone does not prove that a formula implements the user's
intention. For example, two different well-formed expressions may compute
entirely different results. The runtime enforces its rules; user review and
independent expected-result checks still establish the demonstrated behavior.
There is no general natural-language semantic verifier or guarantee of success.

## Attempt limits, cancellation and approval

Each goal allows at most three provider calls: one original proposal and at most
two corrections. A Studio session allows eight provider attempts total, including
corrections and failed calls. Each provider process has a 40-second timeout; its
HTTPS request is separately bounded. The request has a 110-second planning
window, and another call starts only when a complete 40-second call fits inside
the remaining window. Validation failure is not an unlimited automatic retry.

Every attempt and final delivery checks the original request ID, base revision,
current runtime revision and the guest's active planning state. Cancellation,
closing the session, a replacement request or intervening mutation prevents the
next attempt and prevents a late candidate from reaching review. A call already
sent to the provider may finish and incur usage; cancellation is not a refund or
a claim that a remote request was stopped. Its result is discarded locally.

The native screen distinguishes initial generation from correction and shows the
attempt count. A valid candidate proceeds through field review and each declared
button review. No app is installed merely because the model or host validator
accepted it. Existing preview/confirmation checks govern later button execution.

Ordinary offline replay remains a single attempt. The independent verifier
explicitly enables correction around disclosed scripted proposals while keeping
the production Rust validation and QEMU guest real.

## Logs and reproduction

Each request folder retains `request.json`, `loop.json`, any `feedback-02.json`
and `feedback-03.json`, and `attempt-01` through `attempt-03` as applicable.
Completed processes have their own stdout/stderr logs; a timed-out process
retains captured partial output in its attempt folder. The request's aggregate
JSONL logs tag complete events with their attempt number. Earlier validation failures are
preserved even if a later proposal succeeds. A successful loop emits only one
accepted `app_plan`; a cancelled late result may appear in provider logs while
`loop.json` records its discard. Logs can contain goals and app definitions, so
review them before publishing. Do not put credentials in goals or definitions.

The command-line boundary accepts the original request plus explicit feedback:

```sh
# One Meta call; the bridge is responsible for the bounded loop.
rai-meta KERNEL --app-repair REQUEST.json FEEDBACK.json
# No Meta call: validate a supplied correction against the same feedback.
rai-meta KERNEL --app-repair-replay REQUEST.json FEEDBACK.json PROPOSAL.json
```

Feedback has exactly the next attempt number, validation code and rejected
proposal, for example:

```json
{"attempt":2,"code":"formula_syntax","proposal":{"request_id":1,"base_revision":1,"decision":"install","reason":"Example","spec":{}}}
```

The abbreviated empty specification above is not a valid executable example;
actual feedback retains the complete rejected definition. The CLI recomputes its
failure and scope rather than trusting the supplied code.

## Independent verification

```sh
python3 -B scripts/verify_app_repair.py --debug --output target/app-repair-debug
python3 -B scripts/verify_app_repair.py --output target/app-repair-release
# Optional bounded live check; requires clean committed source and credentials.
python3 -B scripts/verify_app_repair.py --source meta --output target/app-repair-live
```

The offline verifier scripts only provider proposal bytes. Every supplied
proposal goes through the real Rust CLI and every guest interaction runs in real
QEMU. The transport-failure case is explicitly a synthetic adapter failure, not
an experiment against an unavailable remote service. No live calls are made in
offline mode.

Its disclosed example starts with a malformed stock expression and corrects it
without changing a guard. The independent oracle expects stock/moved values
`3,0 -> 2,1` and `7,4 -> 6,5` on separate rows, with an empty-stock row rejected
unchanged. It checks no installation before both approvals, preview without
mutation, exact revision change and recovered data after reboot. This is an
example assembled in the verifier, not an app-specific production implementation.

Adversarial cases cover three invalid attempts with no fourth call, guard removal,
cancellation after an invalid response, a valid late response after cancellation,
a replaced request/revision, transport failure, session-budget exhaustion,
clarification, an unsupported capability, malformed adapter event envelopes and
a synthetic process timeout with partial output retained. The success case must retain the
first failure while accepting only the later valid proposal. These tests verify
orchestration and execution, not every possible model output or a wall-clock
measurement of the full timeout window.

Live mode uses the same previously rejected generic goal, allows at most three
calls and checks the actual accepted app with the independent data oracle. It
does not substitute an offline fixture after a live failure or retry until it
passes. A first-attempt success establishes generation, but does not demonstrate
that the live correction path was needed. Attempt logs distinguish those cases.

Observed results belong in retained reports and WORKLOG. The existence of this
verifier does not by itself establish successful offline or live execution.

## Observed offline results, November 3

The final local debug and release verifiers passed with real QEMU and the real
Rust replay validator: two attempts for the corrected app, independent three-row
execution and reboot checks, plus all thirteen adversarial cases listed above.
Both runs made zero Meta calls. Partial synthetic timeout output was retained.
This establishes the bounded offline path; successful live generation or repair
requires separate provider evidence and is not implied by these results.
