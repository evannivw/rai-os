# Git tools for generated applications

Meta can select a Git operation for the currently opened app: status, history,
show, diff, save or restore. The bridge executes a validated operation against a
dedicated local bare repository. Git is real; it runs on the host, not inside the
AArch64 guest or the kernel. Generated app programs still run locally in QEMU.

## Use it in the native app screen

Live demo sessions now select their own `app-git` directory. For a standalone
session, opt in explicitly:

```sh
python3 -B scripts/apps.py --source meta --output target/git-session \
  --state target/git-device.json --git-store target/git-definitions
```

Open an installed app, choose **edit app >** (F2), and enter a prompt beginning with
`git:`. That prefix selects the Git tool contract; Meta chooses the operation and
an observed revision, rather than the bridge matching application-specific phrases.
Examples:

```text
git: save this definition as version one
git: show the saved history
git: compare the previous saved version with the current definition
git: restore the previous saved version, keeping current records
```

A normal prompt without the prefix still creates/evolves the app. After changing
and reviewing a program, save its new definition explicitly if you want it in Git.
There is no silent auto-commit on every row edit or every model response.

Status/history produce a native notice; the notice shows up to three recent
entries, while the session log retains the complete bounded result. Show/diff
retain the full definition or actual Git diff in the session log; their native
notice is a summary, not a graphical diff viewer. A restore opens the usual native
field/program approval screens. Running the restored app needs no inference.

Each Git prompt makes at most one Meta call, with no automatic retry or fallback.
Git and app-generation calls share the existing eight-attempt Studio session cap.
Offline replay does not invent Git intentions: explicit scripted selections belong
to the verifier or the low-level replay CLI. Git must be installed on the host;
the portable Docker image includes it. Native Windows remains unverified; use the
existing Linux container path on Windows.

## What is versioned, and what rollback means

Each installed app has a separate `<app-id>.git` bare repository under the selected
root. Its only committed file is `app.json`: fields, formulas, workflows, buttons,
scopes and other schema properties. Current record rows and workflow states remain
in the device checkpoint and are never added to Git or sent to Meta by this tool.
Definitions/defaults themselves may contain user-authored text; do not put secrets
in prompts or app definitions. API credentials are never Git content or Git child
environment variables.

Restore selects an exact observed commit and produces an ordinary app proposal.
Host validation checks identity, capabilities and schema compatibility; the guest
validates against actual current records and then requires approval. Only after
approval does the definition change. Existing editable values stay; calculated
fields are recomputed by the ordinary migration rules. Restoring a definition
that would remove/reorder fields, change field types, change capabilities or
replace an existing workflow is rejected. A formula incompatible with current
values is also rejected by the guest. No data rollback is claimed.

Restoring does not reset Git HEAD or erase commits. If the restored program is
saved again, Git appends a new commit whose parent is the latest saved version.
This preserves the intervening history. Identical current/saved definitions do not
create duplicate commits. The normal UI still allows the user to cancel a restore
proposal; merely asking Meta to restore is not installation.

## Contract and execution boundaries

`app_git.rs` validates a closed operation enum, ASCII labels/reasons and exact
revision IDs from the observed history. `git_store.rs` owns Git I/O. The response
contains no shell command, executable path, arguments, file name or repository URL.
There are no clone, push, fetch, reset, config, hook, arbitrary-file or kernel-source
operations. Current app identity and the history root come from the host session,
not the model. Existing ordinary repositories are never adopted by this adapter.

Planning is read-only: the bridge gathers real Git context, then asks Meta for one
operation. After the response, Python rechecks the guest request and current/base
revision under its input lock. Local execution rechecks Git HEAD under a per-app
lock; saving also reobserves state before its no-op path. Git's compare-and-swap
reference update protects against a newer head appearing before publication.
Cancellation/stale guest state prevents tool execution. A provider call already
sent can still incur usage; its late result is discarded. A local operation that
has already crossed the locked execution boundary completes before later input.

The Git child has a cleared environment with only explicit runtime settings. It
uses argument arrays, no shell, no pager, disabled hooks/signing/fsmonitor,
disabled external diff/textconv and no allowed network protocol. Global/system
Git config is ignored; local store config/owner/HEAD must match the dedicated
format. Symlinks, alternate object stores, grafts and shared-repository redirection
are rejected. The host user, filesystem and Git binary remain trusted: this is not
an isolation boundary against another process running as the same account.

Limits: 64 saved versions per app; Meta sees the latest 16; native history summary
shows three. Definitions are at most 16 KiB, tool output at most 64 KiB, inspected
store entries at most 2048. Each Git subprocess has a two-second deadline; bridge
planning is bounded at 40 seconds and local application at 25 seconds. A lock left
by an interrupted process fails closed as busy; inspect whether another session
is active before manually removing it. There is no automatic lock stealing.
Diff hashes the current definition into a temporary/unreferenced Git blob for
comparison, but does not change committed history. A failed CAS can also leave
unreferenced objects; no automatic history pruning or garbage collection runs.

## Verification and reproduction

```sh
python3 -B scripts/verify_app_git.py --debug --output target/git-debug
python3 -B scripts/verify_app_git.py --output target/git-release
docker run --rm --network none rai-os-demo apps-git-verify
# Separate bounded check: exactly three goals, at most one call each.
python3 -B scripts/verify_app_git.py --source meta --output target/git-live
```

The independent verifier installs disclosed example definitions, saves two real
Git commits, changes data under the second program, requests the earlier program,
requires native approval, and checks the restored operation on current records.
It also checks reboot. These input app definitions are verifier fixtures in both
modes; live mode tests Meta's actual Git choices, not fresh app generation.

Offline tests additionally exercise history/diff, canceled save, append-after-
restore, concurrent Git change, session cap, incompatible field removal and stale
guest state. Rust tests cover real Git CAS/no-op semantics, the 64-version limit,
ordinary repository rejection, altered config, alternate stores, symlinks and a
reference-transaction hook that must not run. Previous generated-app/repair/demo
regressions remain separate. Actual observed results belong in WORKLOG/evidence;
the commands above are not themselves evidence. The recorded [live run](evidence/2026-11-03-app-git/live/README.md) passed two actual Meta-selected saves and a reviewed restore in exactly three calls.

## Toward updating RAI-OS itself

This increment versions app definitions only. It does not let Meta rewrite the
Rust interpreter, coordinator, renderer, host bridge, kernel or build files.
Interface definitions, settings and isolated service packages can become further
versioned artifacts with their own validators. They must not inherit arbitrary
source-write authority from app Git tools.

Before executable self-update, define a package boundary, reproducible build and
checks, restricted candidate execution, compatible state migration, atomic
activation and an independently bootable previous version. A Git commit is code
history; it does not restore a crashed process, reverse a destructive data migration
or guarantee that a machine can boot. See [KERNEL_DECISION.md](KERNEL_DECISION.md)
for the mechanisms the kernel will eventually need to support those boundaries.

Git references: [configuration and hook controls](https://git-scm.com/docs/git-config),
[creating commits from trees](https://git-scm.com/docs/git-commit-tree), and
[conditional reference updates](https://git-scm.com/docs/git-update-ref.html).
