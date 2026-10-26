# Work log and provenance

Dates use America/Guatemala (UTC-06:00). Preserve real dates and distinguish
pre-existing work from eligible event contributions. Historical captured bytes
and already-published commit messages remain unchanged.

## Before October 26

- 46e681e: Oct 13, initial commit.
- 3e6d9c9: Oct 13, normal macOS Rust example, README, and Apache-2.0 license.
- The initial local inspection found a clean tree at
  /Users/jv/Development/Rust/rai-os on an ARM64 Mac, Darwin 25.3.0, with origin
  git@github.com:evannivw/rai-os.git. No AGENTS.md existed in the repo or ancestors.
- Original README/source are preserved in Git history. The host example remains
  in examples/host_hello.rs; current text/output were later translated to English.
  LICENSE was not changed.

## October 26: minimal boot, then milestone 1

Implemented Assembly entry, linker layout, no_std/no_main Rust, and UART output.
Verified the real boot banner before expanding to input, help/info/echo, bounded
structured requests, validation, and recovery. No external crate dependencies,
API keys, Meta integration, heap, userspace, or agent isolation.

Verification actually performed on this Mac:

- Rust/Cargo 1.90.0; target aarch64-unknown-none-softfloat; bundled rust-lld.
- file identified a statically linked AArch64 ELF.
- QEMU 11.1.2, virt-9.2, Cortex-A53, TCG, one CPU, 128 MiB.
- Real initial boot, followed by 28 console/protocol checks in debug AND release,
  plus boot in each profile. Includes editing, valid requests, bad versions/IDs,
  argument errors, unknown operations, NUL/escape/non-ASCII, exact 256-byte limit,
  overflow, recovery, CRLF, consecutive requests, and fragmented delivery.
- Formatting and Clippy with warnings denied passed.
- Interactive cargo run: echo and Ctrl-C exit verified.
- Original macOS example compiled and ran with its original Spanish output.

Historical evidence: [first boot](evidence/2026-10-26-first-boot.txt),
[debug UART](evidence/2026-10-26-debug-uart.txt),
[release UART](evidence/2026-10-26-release-uart.txt).
These are earlier actual captures, not regenerated translations. CR, TAB, and
Backspace are rendered as literal \r, \t, and \b; LF separates lines. Raw bytes
from subsequent runs are stored under target/verification. Echo fixture text
in historical evidence may be Spanish and must not be silently rewritten.

Historical checked ELF SHA-256 values (not promises about future rebuilds):

- debug: a483404dfd1f6b4f341efc369c8fcbce34e7ffc82f68c8b785b55b85ccea4a7c
- release: ab4cc476869087391f2a87e52cc39b4d4b228f2fd2f3d7cc22268e9a825b76fd

Not tested: physical hardware, Linux/Windows hosts, electrical UART errors,
CPU exception recovery, process isolation, or performance superiority.

## Tool installation

Homebrew installed QEMU/Rustup and dependencies. Homebrew Rustup 1.29.2 crashed
with SIGSEGV during component installation; official Rustup 1.28.2 successfully
installed the pinned compiler/components. A later self-update attempt failed
without preventing the verified builds. Shell startup files were not modified;
load `. "$HOME/.cargo/env"` in a new terminal.

## Event rules and publication

The user first supplied the event requirements, then pasted the Participant Guide
on Oct 26. It requires a public repo, identifiable new work, real Meta integration,
and a three-minute video. The guide lists Oct 27-Nov 4 as the build period, while
public event pages announce Oct 26 kickoff and workshops starting Oct 27.
Eligibility of Oct 26 work and the earlier base for Indie remains unresolved.
Keep the discrepancy visible; do not change dates. API access was requested by
the user; event acceptance and model access are separate unverified dependencies.

Deadline: Nov 4, 2026, 20:00 Guatemala (Nov 5, 02:00 UTC). See
[HACKATHON_PLAN.md](HACKATHON_PLAN.md) for sources, open questions, and checklist.
No Discord messages were sent and no terms were accepted for the user.
Public visibility was verified with unauthenticated GitHub REST on Oct 26:
GET /repos/evannivw/rai-os returned private: false.

Initially changes were local only. The user subsequently authorized regular
commits and pushes. Published increments:

- 75b69a5: standing collaboration rules in AGENTS.md.
- 3b846d7: honest snapshot of accumulated Oct 26 kernel work, not a fabricated
  sequence of earlier commits. Formatting, Clippy, and both QEMU profiles passed
  before publication. Common credential-pattern checks found no matches; this
  is not a security audit.
- 0dd0788: Rust host terminal console, validation/process/presentation boundaries,
  and instructions. Two validation unit tests, Clippy, and real-QEMU integration
  passed: info/echo, invalid input/recovery, missing ELF, and EOF cleanup.
  The console also passed from a clean Git export before publication.

Local and remote HEAD matched after these pushes. No graphical UI, telemetry,
Meta calls, agent processes, or battery measurements existed at that point.

## English-language policy — October 26

The user requested English for project artifacts and commit titles AND bodies.
Earlier titles were English but bodies and documentation were Spanish. Preserve
those published commits; do not rewrite or backdate history. Translate current
documentation, UI, and examples, retaining historical evidence verbatim.
Chat teaching remains Spanish. The current graphical UI and controlled comparison
are still proposals, not implemented benchmarks or performance results.

English migration checks: host protocol unit tests (2), host Clippy, real-QEMU
console integration, and both kernel verification profiles (28 cases each) passed.
Historical UART captures were retained unchanged. No scheduling or AI logic was
introduced by this language change.

## Comparison design — October 26

Added a proposed review-pack experiment with competitive baselines, equal inputs,
independent correctness checks, model overhead, held-out scenarios, and explicitly
simulated budgets. No experiment has run and no efficiency gain is claimed.
The next gate is a small policy pilot, not a complete new kernel or dashboard.

## Required demo result contract — October 26

Defined mandatory records, byte integrity, exact indexing, consistent manifests,
version-aware publication, and execution limits in DEMO_ACCEPTANCE.md. Optional
previews cannot compensate for missing required output. Events and constraints
apply equally to all policies. This increment defines acceptance only: no new
execution engine, model integration, resource measurements, or benchmark results.
Validation: specification consistency, local Markdown links, and git diff --check.
