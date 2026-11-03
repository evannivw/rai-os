# App Git verification — 2026-11-03

Offline evidence from real Git and AArch64 QEMU on macOS (debug/release) and Linux
ARM64 Docker with network disabled. All three reports have verified=true. These
checks use disclosed fixture app definitions and scripted tool choices; no Meta
request occurred. Git history in the reports contains real commit IDs. Raw Git
stdout, request/pending choices, guest UART, checkpoints and native captures are
retained. Repository object directories are excluded; rerun verify_app_git.py to
create fresh real Git stores.

The records are fictional LOCAL_ONLY test values. Definitions alone were committed,
rollback preserved current records, and reboot recovered the restored program/data.
See each report for rejection/concurrency checks. Rust tests cover additional
adapter boundaries and 64-version capacity. Regression reports remain separate.

source-hashes.json identifies the changed executable files tested before the source
freeze commit. linux-build-info.json identifies the container build environment;
the container has no project .git directory, so its report cannot infer a source
commit. Native Windows was not tested. Live evidence, if run, is recorded separately.
