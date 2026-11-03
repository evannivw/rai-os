# Generated apps with checked processes

A records.local app can now carry an optional workflow: an initial state and up
to four user-triggered actions. Each saved row has its own process state, separate
from its editable fields. A model defines this graph as part of the proposed app;
the Rust runtime checks it and executes transitions locally. There is no workshop
name, payment rule or state vocabulary built into the executor.

## Workshop capability demonstration

The declared request is:

```text
Repair shop: device, job, quote, paid, balance=quote-paid in cents. Initially received; start, finish, deliver if balance<=0.
```

The independent verifier expects five fields and a three-action chain. It imports
eight fictional orders, then leaves two delivered, one ready but unpaid, one in
repair and four received. It checks that premature completion, unpaid delivery,
stale actions and repeat completion change nothing. Recording a payment recomputes
the balance and allows the final transition. Final-state records cannot be edited
through normal forms, saves or check toggles. Export and reboot recovery are checked.

```sh
python3 -B scripts/verify_app_workflow.py --debug --output target/workflow-debug
python3 -B scripts/verify_app_workflow.py --output target/workflow-release
# Clean committed source; ONE paid Meta request, no retries/fallback:
python3 -B scripts/verify_app_workflow.py --source meta --output target/workflow-live
# Rebuild the portable image first:
docker run --rm --network none rai-os-demo apps-workflow-verify
```

Offline mode supplies a disclosed schema in the verifier only. Live mode sends the
prompt to Meta, checks the actual generated fields/graph/guard before installation,
and uses returned IDs for every operation. Inspect WORKLOG for observed results;
a successful development example does not establish general model reliability.

## Using a process app

- Review fields/formulas first, then review the complete action graph and guards
  on a second screen before installing. Esc cancels either review.
- Save/import a record. New records always start in the workflow's initial state.
- Open a saved record or select one in the list/table, then click **actions** or F9.
- Click an action, or select with Tab/arrows and Enter. The screen shows the
  current state, source/destination states, numeric condition and actual value.
- Use F3/records to return. The list displays each record's current state.
- A final state has no outgoing actions and locks normal editing. A declared
  outgoing action can explicitly reopen a state; there is no hidden reset button.

No inference is used for action clicks, calculations, ordinary edits, export or
restore. This is a richer app type, not autonomous background scheduling.

## Schema and execution contract

The optional workflow object has initial and actions. Each action has:

| Property | Meaning |
| --- | --- |
| id, label | Unique action identifier (12 bytes max), printable label (22 max) |
| from, to | Different state identifiers (12 max), with every source reachable from initial |
| guard_field | Empty for unconditional action, otherwise an existing numeric/calculated field |
| guard_op | none, eq, le or ge |
| guard_value | Signed i32 constant; unconditional actions require 0 |

The same identifier rules as app/field IDs apply. No shell, files, arbitrary code,
network or hardware operation is available through workflow actions. A guard can
read only its own row. All actions check the current app, row, revision, source
state and guard before mutation. Successful actions increment the runtime revision
once. There are at most four actions and five reachable states. Manual cycles are
allowed, but no automatic loops or scheduling exist.

Existing workflow definitions are immutable on schema updates, including adding
or removing a workflow; create another app for a different process. Compatible
field additions/formula updates still undergo the existing reviewed migration
checks. A terminal lock blocks ordinary data edits, not an explicitly reviewed
schema migration that can recompute derived fields. Guards are checked at action
time, not permanent data invariants or proof that a real-world repair/payment
happened. This prototype trusts user-entered data.

Capacity is unchanged: three apps, six fields, eight records per app, 32 ASCII
bytes per value, integer-only calculations. There is no customer database,
linked inventory, payment integration, multiple users, transaction history or
production backup service. This demonstrates process depth, not large-scale data
handling or an enterprise-ready workshop system.

## Persistence and authority

Saved-device JSON stores a states array for workflow apps. Validation requires
one known state per row, validates the full schema/data, and rejects missing or
unknown states. Fresh guest recovery uses a one-time restore phase, allowed only
before any runtime revision changes; the restore entry closes before interaction.
The saved-file owner and host remain trusted. This restores recorded state, not a
cryptographically verified action history; guards are not replayed at restore.
Normal app actions cannot use the restore entry. App namespaces and enforcement
still share EL1 with the kernel; this is not hardware/process isolation.

CSV contains ordinary and calculated fields only. It does not back up process
states, and reimported rows start a new process at initial. Use the saved-device
JSON to recover an ongoing process. No workflow data is added to Meta requests:
creation receives the goal; updates receive schema, count and revision, not rows.

## Code and wire contract

- microapp_workflow.rs: bounded action graph, numeric conditions, reachability.
- microapp.rs: per-row states, action admission/commit, terminal edit lock.
- app_ui.rs/display/apps.rs: two-stage review, native actions and row-state view.
- host/meta/src/apps.rs/provider.rs: strict untrusted workflow schema and catalog.
- snapshot.rs/app_storage.py: complete state validation and fresh-device recovery.

Proposal staging adds app.flow `request:initial` and app.action
`request:id:label_hex:from:to:guard_field:guard_op:guard_value` before app.ready.
app.schema adds initial/actions; app.export adds states. app.workflow.run
`revision:action_id` operates on the selected saved row in the actions view.
app.restore.begin/end `1` and app.restore.state `app_id:row:state` are trusted
boot-only checkpoint diagnostics, not generated-app capabilities. Update guest
and host together. Old snapshots without workflows remain compatible.
