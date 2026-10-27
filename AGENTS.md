# RAI-OS collaboration rules

## Language and context

- All project documentation, code comments, UI text, and new commit titles AND
  bodies must be in English. Continue teaching the user in Spanish in chat.
- Preserve historical evidence and existing Git history verbatim; do not translate
  captured bytes or rewrite old commits to conceal their original content/date.
- Educational Rust AArch64 kernel for QEMU, with a long-term microkernel direction.
  Read docs/WORKLOG.md, docs/HACKATHON_PLAN.md, and docs/ARCHITECTURE.md before
  changing scope. The user is learning Rust and systems programming.

## Teaching and scope

- Explain what each step will execute and why, in concise Spanish, before acting.
- Teach one idea per increment. Avoid advancing several phases without explanation.
- Report changes, actual verification results, limitations, and the next step.
- The terminal console and graphical Device Lab panel are host tools. The
  project targets a future mobile OS; do not describe the host-rendered portrait
  view as a native kernel desktop or support for a physical phone.
- Show real QEMU results. Clearly label simulated budgets and unimplemented AI
  or telemetry. Do not fabricate battery, GPU, memory usage, or benchmark wins.

## Git: standing user authorization

- Commit each completed, verified increment with a coherent scope, and push it
  to the authorized origin remote. No repeated permission question is needed
  for authorized RAI-OS work; still honor tool sandbox requirements.
- Review diffs, included files, and relevant checks before committing. Stage
  explicit paths. Never commit credentials, target/, or unrelated user changes.
- Use clear English titles and bodies describing what changed, why, and the
  checks actually run. Do not invent test results or earlier development steps.
- Verify push completion and local/remote HEAD agreement. Report failures and
  resolve without force-pushing or deleting history.
- Preserve real dates. Label accumulated snapshots honestly.

## Proportional Clean Architecture

- Keep domain operations independent of UART, QEMU, terminal UI, and Meta APIs.
- Separate parsing/validation, use-case coordination, and I/O adapters.
  Dependencies point toward domain rules.
- The kernel must not depend on the host UI or an AI provider. Presentation
  renders results; it does not own domain policy or access MMIO.
- Keep responsibilities explicit. Add traits only for real boundaries; avoid
  empty abstraction layers and premature frameworks.
- Keep unsafe small and document concrete preconditions.
- Current kernel modules share memory and privilege; they are not isolated processes.

## Verification and hackathon provenance

- Kernel/protocol changes: formatting, Clippy, and relevant real-QEMU tests;
  check release when executable code or build configuration changes.
- Documentation: consistency, local links, and git diff --check. Do not repeat
  tests without relevant changes or unresolved concerns.
- Record versions and limits; distinguish measured results, configuration,
  simulation, and planned features.
- The supplied Participant Guide requires a public repository, identifiable new
  work, real Meta model integration, and a three-minute demo video.
- Kickoff: Oct 26; guide build period: Oct 27-Nov 4. Eligibility of Oct 26 and
  pre-existing work in the Indie track still needs organizer clarification.
- Deadline: Nov 4, 2026, 20:00 Guatemala; internal target: 18:00.
- Do not claim performance/security superiority or eligibility without evidence.
