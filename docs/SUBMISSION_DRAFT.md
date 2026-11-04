# Submission copy — review before submitting

Project: RAI-OS

One-line description: Describe a tool, review the AI-generated application, and use
it locally inside a Rust operating-system prototype.

## Project description

RAI-OS explores an operating system where people create their own small tools by
prompt. Meta proposes a declarative application using an implemented capability
catalog. Rust validates the proposal, the user reviews it, and the guest executes
its forms, calculations, guarded processes, conditional actions, data pages and
checked transfers. Ordinary use continues without a model round trip.

Our demonstration creates an app, uses its actual records, rejects an invalid
operation and recovers work after reboot. The public saved kit includes three
actual Meta-generated examples, requiring no API key to try. Evaluators can supply
their own key to generate new supported apps.

We built an AArch64 Rust kernel and guest-rendered phone-shaped interface for QEMU.
A portable host bridge handles Meta HTTPS, input, Git definition history and
checkpoints. Current apps share EL1; this is a bounded declarative prototype, not
isolated native applications or a physical-phone replacement.

## Meta integration

The configured model is muse-spark-1.3 through Meta Model API. Requests provide the
user's goal, a versioned capability catalog and the current app definition for an
update. Shared Rust validation, user review and guest execution enforce implemented
bounds. A bounded correction loop can repair eligible expression errors without
changing permissions/guards. Evidence retains actual responses and outcomes.

## Links and facts to fill before submission

- Repository: https://github.com/evannivw/rai-os
- Reproduction: docs/TRY_RAI_OS.md in that repository.
- Video URL: pending upload; use the final narrated three-minute recording.
- Team members and representative: enter the actual participant details.
- Track and selected problem statement: confirm in the official platform.
- Contribution period: preserve actual commits/worklog; distinguish any earlier work.
- Supplied guide discrepancy: kickoff Oct 26; build period Oct 27–Nov 4. Organizer
  clarification of Oct 26 eligibility remains unresolved; do not claim it is settled.

Deadline in the supplied guide: Nov 4, 2026, 20:00 Guatemala. Internal target: 18:00.
This file is draft submission copy, not evidence of an uploaded video or completed
entry. Save the actual platform confirmation after submission.
