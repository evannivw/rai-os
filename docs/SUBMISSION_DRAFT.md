# RAI-OS — Submission description

## Project name

RAI-OS

## Short description

Create and evolve your own tools by prompt inside a Rust operating-system prototype,
with Meta-generated app definitions and checked local execution.

## Project description

RAI-OS explores a mobile operating system where people shape their own tools around
the work they need to do. A user describes an application, reviews what Meta proposes,
and opens the resulting tool in a phone-shaped interface running inside our own
AArch64 Rust kernel in QEMU.

Meta composes applications from an implemented capability catalog: forms, calculated
fields, guarded workflows, conditional actions, report pages and checked transfers
between records. Rust validates each definition before installation and enforces
operation rules during use. Once installed, ordinary app operations run locally
without another model request. A later prompt can extend a supported application
while preserving compatible records.

The three-minute demo follows a freshly generated Ledger app. We enter two locations,
transfer quantities while preserving their total, reject a transfer with insufficient
balance, add a report page through a second prompt, and recover the evolved app and
its records after restarting QEMU. Both generation requests used Meta Model API.
The recording uses scripted inputs, edited timing, fictional records and labelled
synthetic English narration.

The long-term goal is to let individuals and small teams create focused software
for their own workflows. Today's implementation is a bounded declarative prototype:
apps and UI share kernel privilege, and a portable host bridge supplies HTTPS,
checkpoints and Git history for app definitions. It does not yet compile arbitrary
native applications or run on physical phones. The public saved demo works without
an API key; evaluators can use their own Meta key to generate new supported tools.

## Meta integration

RAI-OS uses muse-spark-1.3 through Meta Model API to turn a user goal into a structured
application definition. The model receives a versioned capability catalog and, for
updates, the current definition and structural metadata. Stored record values are
not included in these app-generation requests.

Shared Rust validation checks supported operations, types and bounds. The user
reviews the proposal before installation, and the guest runtime validates operations
against current state before committing changes. A bounded correction loop can
repair eligible expression errors; it cannot expand the runtime's capabilities.
The final recorded demo accepted both requests on their first attempt with repairs
disabled. Requests, responses and verification results are retained in the repository.

## Verification

The final code review passed 114 Rust tests, formatting and static analysis, real
QEMU transfer tests in debug and release, saved-session recovery, and rebuilt Linux
container demo checks with network disabled. These checks establish the demonstrated
behavior; no performance advantage over existing operating systems is claimed.

- [Final review and test evidence](evidence/2026-11-04-final-review/README.md)
- [Recorded Meta requests and demo evidence](evidence/2026-11-04-submission/README.md)

## Participant and project links

- Participant and representative: Jarrol Villatoro.
- Participation: individual.
- Proposed track: Indie Developer.
- Problem statement: Push the Frontier.
- [Public repository](https://github.com/evannivw/rai-os)
- [Three-minute demo video](https://drive.google.com/file/d/18e2NPFsIktJhBC-T7ZY4ANAfQhD9PvaO/view?usp=sharing)
- [Instructions: saved demo or your own Meta key](TRY_RAI_OS.md)

## Development provenance

The repository includes an initial README and minimal Rust scaffold dated October
13. Kernel boot, UART and early host tooling were committed on October 26. The
prompt-generated declarative app runtime and the app capabilities shown in the
final demo were developed from November 3, with recording and final fixes on
November 4. Git history and [WORKLOG.md](WORKLOG.md) preserve the actual sequence.

The supplied guide distinguishes the October 26 kickoff from an October 27–November
4 building period. Eligibility of October 26 work has not been confirmed with the
organizers; that foundation is identified separately from the later app demo work.

Submission logistics and confirmation status are maintained in the separate
[submission checklist](SUBMISSION_PACKAGE.md).
