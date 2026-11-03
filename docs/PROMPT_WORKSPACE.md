# Prompt-driven guest workspace

RAI-OS now draws a prompt workspace in its own 480x800 framebuffer. Users type a
request, receive a proposed file plan, inspect every source/destination, then
confirm or cancel. One Meta call proposes the whole plan. This is a bounded
prototype, not a general desktop, autonomous agent or measured AI advantage.

## Run it

Native macOS/Linux development environment, from the checkout root:

```sh
python3 -B scripts/workspace.py --source meta --output target/workspace-session-1
```

Then connect a standard RFB 3.3 or 3.8 VNC viewer to localhost:5900. The built-in
macOS Screen Sharing viewer requires a local VNC password: set RAI_VNC_PASSWORD
in the launch environment (1–8 ASCII characters), then open
`open vnc://127.0.0.1:5900` and enter that same password. Do not reuse your Meta
key or any account password. For Docker, also pass `-e RAI_VNC_PASSWORD` after
setting that environment variable on the host. Authentication is performed by
QEMU; the relay forwards its challenge/response and never reads the entered
password. This is legacy VNC authentication, not transport encryption. The viewer is a transport for pixels
and input: the interface is rendered by Rust inside QEMU, not a host web page.
Use an unused output directory each time. Ctrl-C in the launcher closes QEMU.

The portable route uses the same Linux container:

```text
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm -it -p 127.0.0.1:5900:5900 --env-file .env.local rai-os-demo workspace
```

Each participant supplies their own MODEL_API_KEY; see [META_BRIDGE.md](META_BRIDGE.md).
The image excludes keys. Only the Rust host provider reads the key; it is never
sent to the guest or VNC viewer. The model receives the typed prompt and the
current bounded guest catalog, including text contents. This demo imports four
disclosed fictional notes, not files from arbitrary host directories. The launch
itself does not call Meta: submission does. There is at most one in-flight request
and eight requests per session, with no retry or silent policy fallback.

Without a key, use the explicitly labeled offline replay:

```text
docker run --rm -it --network none -p 127.0.0.1:5900:5900 rai-os-demo workspace-replay
```

Or add `--source replay` to the native launcher. Replay accepts exactly this
instruction and supplies a recorded-in-code proposal; it is NOT inference:

```text
Collect approved Aurora notes in /work. Keep originals.
```

Other replay prompts produce a clarification. The live Meta mode accepts other
ASCII requests within the available operations. It is not guaranteed to interpret
every request correctly. Inspect the actual proposal before confirming.

## Interaction

- Type in the viewer; the prompt is always focused. Printable ASCII, maximum 128
  bytes; Backspace edits. Plain ASCII clipboard paste is supported up to 128 bytes,
  without translating pasted newlines into submissions. Unicode/IME is not ready.
- Enter or the large bottom button submits a prompt. During the request, files
  remain unchanged and the screen shows a waiting state.
- On review, Enter or the button confirms ALL listed actions. Esc cancels and
  invalidates any outstanding proposal. A late model response cannot revive it.
- Tab switches between the two file pages. Esc returns to editing; Enter after a
  result starts a new instruction. These are the currently wired controls, not a
  full mouse-driven file browser. The footer is also a cancellation click target.
- Keep the published VNC port on loopback as shown. Authentication is optional;
  this is a local developer demo, not a network service.

## Execution boundaries

The [RFB 3.8](https://www.rfc-editor.org/rfc/rfc6143.html) adapter forwards actual
QEMU pixels and translates bounded key/button events into UART ui.key messages.
QEMU's backend VNC listener uses a private Unix socket. There is no guest USB or
virtio keyboard driver yet; a physical-phone input stack is still future work.
The input relay and HTTPS transport remain host-side and work in the common Linux
container. The prompt, review state, rendering and file mutation live in the guest.
These guest modules currently share EL1 privilege; they are not isolated services.

The Rust host provider validates a strict request/plan schema and binds the plan
to request_id and filesystem generation. File contents are untrusted model input.
The provider has no direct execution path: the bridge sends ui.plan, up to three
ui.action messages, and ui.ready. The guest independently validates the whole plan
before review, and again on confirmation. This validates structure and authority,
not semantic correctness; review remains necessary.

Actions are independent copies or moves from existing sources to new path-like
names. No source can appear twice; no destination can already exist or repeat.
No chains, overwrites, deletion, directory objects or generated content. All copy
slots must fit before the first write. A concurrent staged upload blocks a plan.
All checks precede writes, then one generation increment publishes the complete
plan on the sequential single-owner path. A stale generation, invalid last action,
collision or capacity failure changes none of the plan's files. This is not a
crash-durable transaction or persistent filesystem. RAM limits remain eight files,
512 bytes each. The model defaults to preserving originals; explicit moves are
available and visible for review. A semantic preservation promise is not inferred
from free-form text by the kernel.

## Verify and reproduce

```sh
python3 -B scripts/verify_workspace.py --output target/workspace-check-1
python3 -B scripts/verify_workspace.py --debug --output target/workspace-debug-1
docker run --rm --network none rai-os-demo workspace-verify
```

The verifier performs an actual RFB handshake/framebuffer request, types the
example through the keyboard relay, proves preview made no changes, clicks the
apply button, and independently checks exact originals and output bytes. It also
checks invalid last actions, stale generations, cancelled requests and unsupported
replay prompts. It saves actual QMP screenshots, normalized UART and provider logs.
The built-in macOS Screen Sharing viewer was manually checked with optional VNC
authentication: typing, proposal review, confirmation and result paging worked.
Physical Windows and other viewer compatibility still need their own validation.

After freezing a clean source snapshot, the explicit live smoke command is:

```sh
python3 -B scripts/verify_workspace_meta.py --output target/workspace-live-1
```

This makes at most one new live call on the same disclosed development data,
independently checks the selected sources/destination scope before confirming,
then checks exact guest results. No retry, tuning to the response, benchmark or
unseen evaluation is implied. Prior negative timing evidence remains unchanged.


## Recorded result

The November 3 live smoke test on clean source a330bd3 passed: Meta proposed the
two approved-document copies, the independent check accepted their scope, and the
guest preserved all originals and returned both exact copied byte sequences after
one confirmed generation increment. One call, no retries; provider latency 3093 ms,
817 reported tokens. This known development example demonstrates the integration,
not superiority over another policy. See [raw evidence](evidence/2026-11-03-prompt-workspace/live/manifest.json).
Offline Linux ARM64 and emulated AMD64 runs also passed the actual RFB/guest flow.
