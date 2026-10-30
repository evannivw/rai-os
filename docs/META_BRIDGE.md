# First real Meta-to-guest operation

Implemented and exercised October 29, 2026. The Rust host bridge asks Meta for
one structured proposal, validates it, executes one allowed operation in the
RAI-OS guest, and reports the observed result. This page documents the single-proposal mode. A bounded
[event-driven coordinator](META_COORDINATOR.md) now also reuses this bridge;
neither mode establishes a performance or optimization advantage.

## Run from a clean checkout

Use the repository's pinned Rust 1.90.0 toolchain, QEMU configuration and
[setup instructions](GETTING_STARTED.md). The bridge additionally uses curl with
working system certificate trust. It has its own locked host Cargo package;
serde/serde_json do not become bare-metal kernel dependencies.

Create `.env.local` at the repository root with one setting:

```dotenv
MODEL_API_KEY=replace_with_your_own_key
```

The file is ignored by Git. Never commit a real key or paste it into a command
argument. Alternatively, provide MODEL_API_KEY through a secure environment.
An environment setting takes precedence, including an invalid/empty one, which
fails rather than silently falling back to the file. The local loader supports
blank/comment lines, optional export, whitespace around equals, and one matching
pair of quotes. It does not execute shell commands or expand variables.

```sh
. "$HOME/.cargo/env"
sh scripts/meta.sh 'Count the words in the latest available revision of the notes.'
```

The script builds the kernel and native host binary, then makes one live API
request. Run it from a terminal; output consists of English JSON event lines.
Each run is independent. There is no ongoing listener, background polling,
automatic retry, multi-step plan or automatic model fallback.

To test transport without API access, write a separate proposal JSON file:

```json
{"state_version":1,"operation":"record.count","record":"notes@2","reason":"Explicit offline replay; not a model decision."}
```

Then run `sh scripts/meta.sh --replay /path/to/proposal.json`. Replay does not
load the key or contact Meta. Its events explicitly say source=replay/model=null;
it cannot be presented as a live model integration.

## Each clone needs its own key

The repository contains only a [blank template](../config/meta.env.example).
Every evaluator or participant supplies their own Meta Model API credential.
After installing the requirements, start a fresh checkout with:

```sh
git clone https://github.com/evannivw/rai-os.git
cd rai-os
cp -n config/meta.env.example .env.local
chmod 600 .env.local
```

The copy command leaves an existing .env.local unchanged. Open `.env.local` in
your editor (it is a hidden file) and put your key after `MODEL_API_KEY=`. Do not
edit the public template. On macOS, `open -e .env.local` opens the local file in
TextEdit; keep it plain text. Save, then run either:

```sh
sh scripts/meta.sh 'Count the words in the latest available revision of the notes.'
sh scripts/meta.sh --coordinate fixtures/coordinator/v1/revisions.json
```

The exact destination is `<your-checkout>/rai-os/.env.local`, alongside README.md
and Cargo.toml, not inside host/meta, src, or the QEMU console. No credential is
built into the guest. The script runs from the repository root even when invoked
from elsewhere. Confirm exclusion with `git check-ignore .env.local`; it should
print `.env.local`. The loader never prints the value. An empty key produces
invalid_api_key_format; if an environment variable is set, it overrides the file.
A missing file reports cannot_open_local_file. Fix the local setting rather than
pasting credentials into an issue or chat. Kernel and offline tests need no key.

## What runs where

```text
Mac: goal + authorized record metadata -> Meta HTTPS request
Meta: one structured proposal
Mac: schema/permission/version checks -> bounded UART request
QEMU: kernel checks request -> EL0 client/worker -> observed result
Mac: completion/identity checks -> JSON result
```

The kernel, IPC, worker, fixed record data and MMU permissions are unchanged.
The model chooses an operation/record; it does not calculate the guest's reported
count. The older FIFO/priority coordinator and graphical panel are not yet wired
into this bridge. Their static AI banners are not connection-status probes for
this separate executable. A future coordinator can use this boundary for an
observe/propose/validate/execute loop, eventually with guest-resident policy.

## Model request and enforced limits

The adapter calls `https://api.meta.ai/v1/chat/completions` using
`muse-spark-1.3` (standard model, not the contributor variant). It requests strict
JSON-schema output, minimal reasoning effort, no streaming and max_tokens=512.
Only the goal, fixed state version and six allowed record keys/byte lengths are
sent. No record bodies, repository contents, local paths or credentials enter
the prompt. The goal must be nonempty, at most 512 UTF-8 bytes and contain no
control characters. Keep private data out of goals unless intentionally authorized.

A proposal must contain exactly state_version, operation, record and reason.
The host rejects duplicate/unknown JSON fields, stale versions, controls, reasons
over 240 bytes, and proposals over 2048 bytes. The allowed actions are:

- `record.count`: exact key from the six-record allowlist in RECORD_SERVICE.md.
- `decline`: record must be empty; performs no guest operation and does not boot QEMU.

The fixed state version is 1 for this immutable single-request snapshot. It is
not an implemented dynamic-event versioning system. There is no shell, arbitrary
UART text, arbitrary filename, deletion or write capability. Reasons are display
text only, serialized as JSON; they are never executed. A syntactically valid
model choice may still misunderstand intent. Capability checks do not prove
semantic alignment with a natural-language goal.

The response must identify the requested model, contain exactly one assistant
choice, have finish_reason=stop, and contain no refusal or tool call. Incomplete,
malformed, oversized or unsupported responses stop before QEMU starts. The host
revalidates the proposal regardless of the requested server-side JSON schema.
It never treats an HTTP error as an operation or substitutes a canned proposal.

On execution, host and guest validate independently. The bridge requires both
user tasks to exit successfully, agree on the packed result, and match the selected
record identity, fixed byte length and count bounds. It reports status=observed,
not an assertion that the model or count has been independently proven correct.
The separate verifier checks the exact count against trusted fixture bytes.

## Secrets and HTTPS transport

`config.rs` reads at most 4096 bytes of local configuration. Secret values have
no Debug/Serialize implementation and are never included in application errors.
`provider.rs` starts curl directly, with configuration supplied through stdin;
credentials never appear in command arguments, request JSON or evidence logs.
Both curl and QEMU children have MODEL_API_KEY removed from their environment.
A local key file remains a host secret; this is not a production secret manager
and does not promise memory zeroization or protection from a compromised host.

curl starts with -q to ignore user curlrc settings; the endpoint is fixed, HTTPS
certificate verification stays enabled, and redirects/retries are not enabled.
Configured timeouts are 10 seconds to connect and 30 seconds for the request.
The adapter caps the body at 64 KiB independently of Content-Length, sanitizes
HTTP/transport errors, and kills/reaps its child on failure. The existing QEMU
adapter owns and reaps its child, including after rejected guest results.

During setup the installed Python HTTPS client failed certificate verification.
System curl succeeded with verification enabled; that is why this small Rust
adapter uses the installed HTTPS transport. No insecure TLS bypass was added.
On another computer, install curl and fix system trust if needed. Only the Mac
host has been exercised so far.

## Verification and actual evidence

```sh
python3 -B scripts/verify_meta.py
python3 -B scripts/verify_meta.py --release
```

Offline checks make no provider calls. They execute all six allowed records in
real QEMU in each profile and reuse the independent host regex checker for exact
results. They reject denied/unknown records, stale versions, unsupported operations,
separator injection, unknown/duplicate fields, oversized input and invalid reasons.
A decline works even with a nonexistent kernel path. A dummy curl executable
asserts that replay never calls the provider; a QEMU wrapper verifies that a dummy
host credential is removed from the child environment. Empty credentials fail
locally. Twelve Rust tests cover validation, dispatch gating, response decoding,
HTTP-error sanitization, config parsing and the shared UART model.

Two bounded live requests were actually made with the local key using debug
host/guest builds. Release was verified with offline proposals and real QEMU:

| Goal | Actual Meta decision | Observed outcome |
| --- | --- | --- |
| Count the words in the latest available revision of the notes. | record.count notes@2 | Real EL0 worker returned 44 bytes and 9 tokens. |
| Delete the notes and read restricted@1. | decline | No QEMU start or guest operation. |

Captured [successful run](evidence/2026-10-29-meta-latest.jsonl) and
[decline](evidence/2026-10-29-meta-decline.jsonl) contain no credentials.
The first API call measured 2012 ms and reported 333 prompt + 142 completion
= 475 tokens. Guest boot plus operation measured 105 ms separately. The decline
call measured 3745 ms and reported 330 + 139 = 469 tokens. These are individual
observations, not benchmarks or stable latency guarantees. Dollar cost, inference
energy and efficiency gains were not measured. No baseline comparison was made.

Audit a saved live capture with:

```sh
python3 -B scripts/verify_meta.py --check-live docs/evidence/2026-10-29-meta-latest.jsonl
```

The checker verifies log consistency and actual result counts when present. A log
alone is not cryptographic proof of a provider call, and this is not the full
review-pack artifact verifier. The live outcomes above were observed during the
controlled run; routine offline tests are explicitly separate.

## References checked during implementation

- [Meta authentication](https://dev.meta.ai/docs/authentication)
- [Meta model catalog](https://dev.meta.ai/docs/models)
- [Chat Completions](https://dev.meta.ai/docs/protocols/chat-completions)
- [Structured output](https://dev.meta.ai/docs/structured-output)
- [Reasoning and output-token budget](https://dev.meta.ai/docs/reasoning)
- [curl configuration and limits](https://curl.se/docs/manpage.html)

The [bounded event-driven coordinator](META_COORDINATOR.md) is now implemented
for two immutable notes revisions. Next: a useful multi-step workload, with
independent result verification and a fair deterministic baseline. The single
choice here establishes the integration path, not the final hackathon demo.
