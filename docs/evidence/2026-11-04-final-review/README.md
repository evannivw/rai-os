# Final repository review — November 4, 2026

Reviewed the current demo path across Rust domain rules, the host/Meta boundary,
checkpoint/Git adapters, evaluator onboarding and source readability. The base
commit and hashes of changed files are in summary.json. Tests ran on that working
tree before the review commit; historical demo evidence and video stay unchanged.

## Correctness fix

Text<N> derived equality compares the entire backing array. Backspace previously
reduced its visible length without clearing the deleted byte. Renaming Dispatch
to B could leave a hidden suffix, so destination lookup failed despite displaying
B. Text::pop now clears the removed byte and retains the existing canonical zero
suffix invariant. Layout, capacity and public API are unchanged; no unsafe added.
Unit tests cover partial deletion, delete/retype, empty and zero-capacity backspace,
and a transfer to the shorter key. The QEMU verifier edits through actual keyboard
input: balances 8/9 become 6/11, derived values 12/22, one revision, exact recovery.
Both debug and release passed. Both retain 4096 bytes in the fixed MMU window.

## Other corrections

- Refuse optimized Python in the shared guest adapter, since host protocol and
  recovery checks depend on assertions. Normal mode and -B remain supported.
- Correct Git's missing-app message to edit app/F2 and describe optional VNC auth.
- Expand credential-setup control flow and document format-only vs real connection
  checks, preserving existing behavior. Tests use a synthetic key, never the author key.
- Include build.rs in Docker's allowed sources and source-hash manifest so the
  linker-script rebuild dependency is retained in portable builds.
- Update tested-platform documentation and distinguish the retained workshop
  rehearsal from the completed, uploaded Ledger video.

## Verification

114 Rust tests passed. Rust formatting and host/kernel Clippy passed with warnings
denied; all 88 tracked Python sources parse. Real QEMU transfer tests passed in
both profiles, and the saved three-app kit passed use/export/recovery. Native
navigation, invalid console input and setup-error handling also passed. Optimized
Python startup was rejected for -O, -OO and PYTHONOPTIMIZE=1.

Rebuilt the Linux image and ran saved-demo and transfer checks with network disabled.
Image identity and build inputs are retained here. Docker was initially stopped;
a first build waited on its credential helper, and an isolated-config retry lacked
the daemon socket. Starting Docker and specifying the local socket with a temporary
empty credential configuration resolved the environment issue. Failed attempt logs
remain in target/final-review-20261104; they are not successful test results.
No live model requests or changes to sharing permissions occurred. The author's
private Meta key file was not accessed.

## Remaining scope

No additional demo-blocking issue was found in the reviewed paths. This is not a
formal security audit or a claim of unrestricted app support. Current apps share
EL1, host services supply HTTPS/persistence/Git, and physical Windows/phone hardware
was not tested here. Some Python orchestration remains compact; broad refactoring
and new capabilities are deliberately deferred to preserve the verified demo.
The uploaded video records the earlier source revision identified by its manifest;
these fixes do not alter its recorded outcomes. Platform submission remains separate.
