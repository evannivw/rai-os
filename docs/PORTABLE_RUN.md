# Run RAI-OS from Windows, Linux or macOS

The kernel targets AArch64 QEMU virt-9.2, not macOS. A host computer runs QEMU and
an optional Rust API bridge. This increment provides a common Linux container
with Rust 1.90.0, the cross target, Python, curl and QEMU. No host Rust/Python/QEMU
installation is needed for this route. Windows uses Docker Desktop's Linux
container backend; this is NOT a native Windows executable or phone installation.

## Build and verify

Install Git and a Linux-container Docker runtime. For Windows, follow the
[official Docker Desktop instructions](https://docs.docker.com/desktop/setup/install/windows-install/).
Then, in PowerShell, a Linux shell or macOS Terminal:

```text
git clone https://github.com/evannivw/rai-os.git
cd rai-os
docker build -t rai-os-demo -f portable/Dockerfile .
docker run --rm --network none rai-os-demo verify
```

The image builds the actual kernel and native host bridge for its Linux CPU
architecture. QEMU uses software emulation (`-accel tcg`), one Cortex-A53 CPU,
128 MiB configured RAM and no guest NIC. It does not require nested virtualization
or /dev/kvm. The [versioned QEMU virt board](https://www.qemu.org/docs/master/system/arm/virt.html)
keeps the board contract fixed; host CPU architecture and guest architecture are
different concerns. A container is the host environment, not the RAI-OS kernel.

Verification boots real QEMU, imports binary/text bytes into guest RAM, checks
copy/move/read and failures, confirms empty storage on reboot, and runs the existing
constant-goal rules cycle in another guest session. No API key or network is used.
A failure returns a nonzero exit code. Test assertions require normal Python;
do not run with -O/PYTHONOPTIMIZE.

To inspect the UART console:

```text
docker run --rm -it --network none rai-os-demo console
```

Type `fs.list`, `help` or other guest commands. The console currently accepts
commands, not clicks. End the container session with Ctrl-C.

## See the actual guest display

```text
docker run --rm -it -p 127.0.0.1:5900:5900 rai-os-demo desktop
```

Connect a VNC viewer to localhost port 5900. The displayed pixels are drawn by
RAI-OS into its QEMU framebuffer. The desktop command loads sample bytes, copies
and moves a file in the guest and holds the session open. Press Enter in the
terminal to close it. VNC shows the screen; it is not yet a mouse/touch-operated
file browser. The UART terminal remains the control channel.

Publish the VNC port to loopback as shown; the container listener has no password.
This is not a web page or a macOS panel. The host qemu.sh adapter also chooses
Cocoa on macOS and GTK on Linux; RAI_DISPLAY supports cocoa/gtk/sdl/vnc explicitly.
The container desktop defaults to VNC and needs no host GUI toolkit or X server.
Debian's [seabios package](https://packages.debian.org/trixie/all/seabios/filelist)
provides the ramfb ROM that a minimal QEMU package alone does not install.

## Optional real Meta calls

Create your own checkout-root .env.local using the public format described in
[META_BRIDGE.md](META_BRIDGE.md), then explicitly run:

```text
docker run --rm --env-file .env.local rai-os-demo context-meta
```

This calls Meta up to three times for the existing constant-goal context example.
It does NOT connect Meta to the new file operations yet. `context-rules` performs
that context example without inference. The key is supplied at runtime, never
built into the image. Do not include shell `export` syntax in Docker's env file.
The default verify action does not read .env.local; no key is needed to try RAI-OS.

## Reproduction and scope

The Docker build context is denied by default, with explicit source/fixture/tool
allowlists. .env files, private key suffixes, .git and host targets are excluded.
Git attributes retain LF line endings for Linux scripts and fixture inputs even
when a Windows clone enables core.autocrlf. The official Rust multi-architecture
base manifest is pinned by SHA-256. Debian
packages use the stable repository and may receive updates; this is a reproducible
recipe with recorded versions, not a claim of bit-identical future apt installs.

`target/portable-build-info.json` in the image records included source-file hashes,
a combined source digest and both compiled binary hashes. To retain full raw
verification evidence, mount a dedicated empty output directory and pass
`verify --output /evidence/run` (the run directory must not already exist).
For example, after creating `evidence-output` in the checkout:

```text
docker run --rm --network none --mount type=bind,source=<absolute-evidence-output-path>,target=/evidence rai-os-demo verify --output /evidence/run
```

The development image includes compilers/tools and is much larger than the kernel.
Its download size is not an OS memory measurement. The guest's RAM file store has
8 files of at most 512 bytes each; persistence, an isolated file service and AI
file planning are future work. See [GUEST_FILES.md](GUEST_FILES.md).

Verification status is recorded in evidence/2026-11-02-portable-files. Linux
container checks on a Mac do not constitute testing Docker Desktop on a physical
Windows machine. Native Windows/WSL GUI setup and physical-phone support must not
be claimed from those results.


Recorded verification for this increment:

| Environment | Result |
| --- | --- |
| Native macOS ARM64 + QEMU | Debug/release UART and file tests passed; native screen checked |
| Linux ARM64 container on Docker Desktop for Mac | Offline file/reboot and retained rules-context tests passed |
| Linux AMD64 container, emulated on the same ARM Mac | Same offline file/reboot and rules-context tests passed |
| Git checkout with core.autocrlf=true | All 254 included source/input files retained identical bytes |
| Container VNC | Loopback RFB 3.8 greeting and clean guest shutdown passed |
| Physical Windows host / Windows VNC viewer | Not tested |

These runs establish portability of the tested Linux environment and exact guest
operations. They are not cross-platform performance benchmarks or proof of phone
compatibility. Full traces and source hashes are in the linked evidence directory.


## Interactive prompt workspace

Use `workspace` with your runtime env file for Meta, or `workspace-replay` without
credentials for the explicitly scripted offline example. Both use the guest's
native prompt UI through standard VNC keyboard/button events. `workspace-verify`
checks the real RFB input and guest transaction path without network access.
See [PROMPT_WORKSPACE.md](PROMPT_WORKSPACE.md) for complete commands and limits.
The earlier `desktop` action remains a static file demonstration.


The bounded app correction verifier is also available without credentials or
network access:

```sh
docker run --rm --network none rai-os-demo apps-repair-verify
```

It supplies disclosed invalid/corrected proposals to the real Rust validator and
QEMU guest. This checks correction limits, stale/cancelled requests, guarded
execution and recovery; it does not make live Meta calls. See
[APP_REPAIR.md](APP_REPAIR.md) for native commands and the separate live check.


`docker run --rm --network none rai-os-demo apps-batches-verify` checks the generic
whole-table programs, reviewed evolution and recovery using disclosed offline
schemas. See [APP_BATCHES.md](APP_BATCHES.md) for the separate bounded live run.


The image now includes Git. Live apps use /state/app-git for dedicated definition
history; keep the /state volume to preserve it. Run
`docker run --rm --network none rai-os-demo apps-git-verify` for real offline
Git/QEMU checks. [APP_GIT.md](APP_GIT.md) describes prompts, rollback and boundaries.
