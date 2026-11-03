# Composable whole-table actions

A generated button can now declare `scope: "all"` and apply its bounded program
to every saved record in its application. The same `set`, `calculate`, `toggle`,
`require` and forward `if` operations compose its behavior. There is no production
branch for an app ID, prompt, business domain or the verifier's example.

The visible product change is describe -> review scope/program -> preview each
row -> apply the entire table. A later reviewed prompt can change the program
without replacing the application or losing its saved records. This extends the
app interpreter; it adds no new kernel mechanism, driver or isolation boundary.

## Scope and transaction rules

- `row` is the default and preserves all older saved applications. It operates on
  the selected saved row. The new Meta response contract requires an explicit scope.
- `all` means every saved row of that app, including rows hidden by the pending
  filter. It does not mean all installed apps or all OS files. Use `if` to select
  which rows change, with the existing typed comparisons and forward skip bound.
- Each program reads only its current row. Derived values refresh after each
  write. There are no cross-row references, aggregates, joins or inter-app effects.
- Every row must pass before any row changes. A requirement failure, arithmetic
  error or terminal workflow row rejects the whole batch and preserves revision.
- An empty batch is rejected. A successful confirmation increments revision once,
  including a valid conditional no-op. Preview never changes data or revision.
- A row-only button cannot run as a batch; an all-row button cannot silently run
  through the single-row commit API. Scope changes require ordinary app review;
  the formula-repair loop cannot change scope to make a failed proposal pass.

Bounds remain three apps, six fields, eight rows per app, two buttons and six
instructions per button. This is a bounded whole-table transformation, not a
scalable database or unrestricted programming language.

## Atomic execution without a second table copy

`Runtime::preview_behavior_all` evaluates every row and counts those that change.
It retains only the current candidate. `run_behavior_all` repeats that validation
at confirmation, then evaluates and writes each row in a deterministic second
pass. Only after the last write does it advance the revision.

This is sound because the current language is pure and row-local: it has no
external effects, cross-row reads, random source, time source or concurrent writer.
Writing an earlier row cannot alter a later computation. Existing terminal states
are checked before the commit pass. No interrupt handler mutates this runtime.
If those assumptions change, this transaction strategy must be replaced before
exposing the new operations. The commit pass is not a generic transaction engine.

The fixed 64 KiB kernel stack is unchanged. This implementation avoids retaining
all candidate rows on that stack. Measured assembly frames and real QEMU tests
are retained with the evidence; they do not establish a whole-program stack bound.
No new unsafe code is introduced. App runtime and UI still share EL1 privilege.

## Native review and persistence

Installation displays whether the button covers one row or ALL saved records.
The button list repeats that scope. Whole-table preview shows before/after fields,
changed-row count and a row pager. Click Previous/Next near the bottom or use
Up/Down/Tab to inspect other rows; confirmation is explicitly `Apply entire table`.
A failed preview reports its one-based row and instruction. Leaving the screen
cancels the preview; stale revisions cannot be committed. Reviewing every page is
possible but not mandatory, so the scope label must remain visible at confirmation.

Confirmed schemas and records use the existing host checkpoint. Recovery restores
the button scope without executing it. Meta receives schema/request metadata,
not record contents. The kernel still has no persistent disk-backed app database.

## Contract and transport

The catalog is `rai-declarative-7`. A button has id, label, event=press, scope and
steps. The host accepts missing scope as row for backward compatibility and omits
that default when serializing older-compatible saved schemas. Unknown scope
strings fail `behavior_bad_scope`; they are not formula-repair candidates.

The existing `app.button` command creates a row-scoped button. During proposal
staging only, `app.button.scope REQUEST:BUTTON:all` sets its batch scope; request
binding still applies. `app.schema` appends `~all` to the encoded button only when
needed, preserving the old four-part encoding for row buttons. Existing preview
and run commands dispatch according to the installed scope; callers do not grant
scope by passing a runtime argument. `app.behavior.status` adds `row=N` for a batch
failure, preserving its earlier row-only response otherwise.

## Independent verification

```sh
python3 -B scripts/verify_app_batches.py --debug --output target/batches-debug
python3 -B scripts/verify_app_batches.py --output target/batches-release
docker run --rm --network none rai-os-demo apps-batches-verify
# Separate live generation and evolution: at most 3 attempts per goal, 6 total.
python3 -B scripts/verify_app_batches.py --source meta --output target/batches-live
```

Offline mode supplies disclosed schema bytes through the real Rust validator and
real QEMU. Eight rows exercise both condition branches, unchanged preview/cancel,
one-revision commit, hidden rows, final-row failure, schema evolution preserving
data, stale admission and exact restart recovery. A second app uses unrelated
text/check fields on the same executor. Host tests also cover the last row failing
on division/overflow, terminal workflow locks, empty batches, invalid scope,
wrong commit scope, old schemas and denied scope expansion during formula repair.

Live mode makes two goals from clean committed source. The first generates a
whole-table conditional action. The second changes its threshold/arithmetic on
the same saved records. Each goal allows at most three attempts, with no fixture
fallback or additional run-until-success. The independent oracle calculates the
expected result without reading the generated expressions. All attempts and
failures remain in the output. A failed second goal does not turn the first into
a successful evolution demonstration. See WORKLOG and retained evidence for
observed results; the verifier's presence alone is not evidence of a live pass.

## Suggested demonstration beat

Show the prompt, generated program and ALL-records scope. Preview the first and
last rows, then apply and show the table. Change the program through a second
prompt and run it over the same data. Finish by recovering the application after
restart. Explain that inference constructs the program; subsequent local actions
need no model call. Keep the existing workflow/CSV showcase as a separate example.
Do not present an offline fixture as a live model response or claim performance
superiority from this small prototype.
