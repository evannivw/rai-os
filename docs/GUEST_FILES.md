# First writable guest file store

This increment replaces no existing fixture/index APIs. It adds a separate,
small volatile file store inside RAI-OS with runtime imports, listing, exact
binary reads, copy and move. The bytes are held and manipulated in the QEMU guest,
not in a host directory disguised as an OS filesystem. The host sends commands
and verifies the returned bytes.

The native Files view displays actual guest paths and sizes. It is read-only
presentation; input still uses UART. It does not yet offer clicks, folders, search
or a full file browser. The new [prompt workspace](PROMPT_WORKSPACE.md) adds a
separate interactive view with model proposals and explicit confirmation. The file code currently runs at EL1 as a kernel module, not as an
isolated EL0 service. No new unsafe code, heap allocator or dependency was added.

## Limits and lifetime

- Eight occupied slots, each at most 512 bytes; one pending upload.
- Names are absolute ASCII paths up to 48 bytes, with letters/digits/_/./-/slashes.
  Empty, dot and dot-dot components are rejected. Spaces, colons and controls are
  not valid names. These are flat path-like names, not real directory objects.
- No links, overwrite, delete, compression, deduplication, disk driver, permissions
  model, journal or reboot persistence. Closing QEMU loses the files by design.
- All clients of this UART API have the same authority. Generation/ticket checks
  protect freshness and completeness; they are not authentication or a general
  security boundary against an untrusted host.
- Operations are sequential on one CPU with the existing masked console path;
  no concurrent filesystem access or crash-durable transaction is claimed.

The store is owned by the Console value, not a global mutable buffer. Stage/data
and replies have fixed arrays. Whole-kernel stack bounds remain unproven; actual
debug/release boot, file operation and retained-coordinator regressions are tested.
The complete OS memory footprint or power savings are not established by this work.

## UART/shell commands

All commands also work as @rai1 frames, with operation and argument tab-separated.
Use fs.list to read the current generation and fs.begin's returned upload ticket.
Do not predict a ticket from a generation: they are independent counters.

```text
fs.list
fs.begin 1:5:/inbox/hello.txt
fs.append 1:0:68656c6c6f
fs.commit 1
fs.read /inbox/hello.txt
fs.copy 2:/inbox/hello.txt:/work/hello.txt
fs.move 3:/work/hello.txt:/archive/hello.txt
fs.list
```

The example starts from an empty boot. Read output includes exact bytes as hex;
empty files are allowed. append accepts 1–64 bytes encoded as hex and an exact
byte offset. At most 256 input bytes per frame, excluding newline, remains the
existing transport limit. read can return the bounded full 512-byte file as hex.

begin requires the current filesystem generation, a final size and unused name.
Partial files are invisible to read/list. append validates ALL hex before touching
stage data. commit requires the right ticket, complete bytes, unchanged generation,
unused destination and free slot, then publishes the entry. A failing operation
never exposes a partial file or overwrites an existing destination.

copy/move likewise require current generation and an unused destination. Each
committed creation, copy or move increments generation. A move/copy during an upload makes its
commit stale; fs.abort <ticket> discards that pending upload. Failed operations
preserve committed files. Each operation is atomic in this sequential runtime;
the [prompt workspace](PROMPT_WORKSPACE.md) now adds all-or-nothing plans of up to
three independent copy/move actions.

## Import, demonstrate and verify

From a native development environment:

```sh
python3 -B scripts/files_demo.py --verify --output target/files-check-new
python3 -B scripts/files_demo.py --debug --verify --output target/files-debug-new
python3 -B scripts/files_demo.py --graphics --hold --output target/files-view-new
# Optional real local input, at most 512 bytes; no arbitrary directory scan:
python3 -B scripts/files_demo.py --import-file path/to/small-file.txt --output target/files-import-new
```

The optional file replaces the demonstration brief and is imported as
/inbox/brief.txt. Nothing modifies the input file. The deterministic demo also
uploads a 512-byte binary payload containing every byte value and checks its
roundtrip. It preserves the original brief, copies that brief to /work, then moves
the copy to /archive. These are actual
guest operations; the demo makes no Meta calls and must not be labeled AI planning.
The [portable container](PORTABLE_RUN.md) provides the same demonstration in Linux.

The verifier checks exact returned content, generation freshness, destination
collisions, traversal/oversize rejection, hidden partial uploads, malformed hex,
offset/ticket/overflow rejection, cancellation, full capacity, and empty state
after a second boot. UART logs, source hashes, outcome JSON and actual QMP screen
captures are saved under the new output directory. Source/returned byte equality
is checked independently by Python, not asserted solely by the kernel.

## Bounded file plans

[PROMPT_WORKSPACE.md](PROMPT_WORKSPACE.md) adds a native prompt and review surface.
A plan contains up to three independent copies/moves against one guest generation.
Every source, destination and required slot is checked before any write; a stale
or invalid plan changes nothing. Confirmation increments the generation once.
The host model proposes and the guest validates/executes. Existing uploads and
single-file commands remain available separately.

This is a useful execution surface, not a measured semantic advantage or complete
filesystem. Evaluate requested outcomes, ambiguity and unintended changes against
a competent baseline, including real model overhead. Do not credit ordinary copies
or cache reuse as AI optimization. Preserve the November 4 delivery constraints.
