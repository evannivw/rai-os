# Saved app sessions

The native app launcher now checkpoints confirmed apps and saved records to an
explicit host file. It restores them into a fresh QEMU instance without inference.
This preserves user work across guest shutdown while a guest disk driver remains
future work. It is **host-backed persistence**, not an on-device filesystem.

## Use

```sh
python3 -B scripts/apps.py --source meta --output target/session-1
# Stop with Ctrl-C, then use a new log directory and the same saved device:
python3 -B scripts/apps.py --source meta --output target/session-2
```

The default state is `target/device.json`; it is ignored by Git. Use
`--state /absolute/path/my-device.json` to choose another device, or `--temporary`
for a RAM-only session. A separate output folder holds each session's logs.
Changing the model source to replay does not change stored apps. Restore never
contacts Meta. Each installed app retains its schema and exact saved record bytes;
transient drafts, UI selection, request IDs and runtime revision numbers are not
persistent and are rebuilt for the new session.

For Docker, mount a **named volume** so the file survives container removal:

```text
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm -it -p 127.0.0.1:5900:5900 -v rai-device:/state --env-file .env.local rai-os-demo apps
```

Use the same volume on the next run. `apps-replay` needs no key; add
`-e RAI_VNC_PASSWORD` when using macOS Screen Sharing, as in MICROAPPS.md.
A named volume persists locally; moving to another computer requires deliberately
copying the state file. Keys are not part of it. The volume has no guest disk
interface and generated apps cannot choose arbitrary host filenames.

## Commit and recovery

The bridge checks the guest revision after an action. If confirmed state changed,
it reads bounded app schemas/records and validates them with the same Rust domain
used by the guest. It writes a temporary file with owner-only permissions, flushes
it, atomically replaces the previous checkpoint, then flushes the parent directory.
The UI says `saved` only after that sequence succeeds. `save failed` means current
changes remain in RAM; a later action retries. Do not shut down while relying on
an unsaved change. Unsubmitted prompts and unapproved proposals are not installed
or checkpointed. This is not a transactional disk subsystem inside RAI-OS.

One advisory host lock prevents two launcher instances from writing the same
saved device. Startup checks a versioned JSON envelope and SHA-256 checksum,
then validates the entire schema/row set with Rust before starting QEMU. The
fresh guest independently validates installation and each complete record. Every
restored schema and value is read back before accepting user input. A restore
failure aborts startup, preserves the original file and does not silently reset
user data. Startup restoration is never applied over a populated guest.

SHA-256 detects accidental corruption; it is not a signature or encryption.
The local host and saved-file owner remain trusted. App state and diagnostic logs
may contain user records: use fictional demo data and do not commit private files.
A corrupted checkpoint is rejected, not repaired; a separately copied backup is
still needed for recovery from storage damage. Durability relies on the host
filesystem honoring flush/replace semantics. Native Windows Python is not this
runner's target; Windows uses the Linux Docker environment.

## Implementation and tests

- `scripts/app_storage.py`: envelope, lock, atomic replacement and fresh-guest restore.
- `host/meta/src/snapshot.rs`: strict bounded validation through microapp::Runtime.
- `scripts/apps.py`: revision observation, checkpoint status and startup lifecycle.
- Guest `app.value index:hex` stages one form value; complete-record Save still
  validates type/required constraints. This is trusted UART diagnostics, not a
  generated-app capability. `app.storage` reports a host acknowledgment only.

```sh
python3 -B scripts/verify_app_storage.py --debug --output target/storage-debug
python3 -B scripts/verify_app_storage.py --output target/storage-release
docker run --rm --network none rai-os-demo apps-storage-verify
```

The test checks exact schemas/rows over fresh QEMU boots, evolution after restore,
exclusive writer locking, failed replacement preservation and retry, malformed
form input, corrupt checksums and unsupported capabilities. The release run also
kills QEMU abruptly before restoring its last committed state. No live inference
is needed or used by these persistence checks.
