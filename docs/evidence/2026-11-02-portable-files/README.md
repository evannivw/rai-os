# Portable runtime and guest files — November 2, 2026, Guatemala

This is an implementation/portability check, not an AI performance experiment.
No live model calls were made. Source was an uncommitted implementation snapshot
based on the commit in manifest.json; Linux build-info files contain exact source
and compiled-binary hashes, independent of a Git directory inside the image.

- files-debug-final / files-release-final: actual guest UART, reboot, result and
  framebuffer PNG. Each verification sent 76 requests and independently checked
  file bytes and failed-operation behavior. PPM intermediates are not archived.
- files-cocoa: native macOS window execution and actual guest framebuffer capture.
- linux-arm64 / linux-amd64: offline container file checks and prior three-event
  rules-context lifecycle, raw input/output, artifacts, reports and source hashes.
  The AMD64 Linux environment ran through Docker emulation on an ARM Mac.
- autocrlf-checkout.json: all 254 staged build inputs retain identical bytes with
  core.autocrlf=true, including bundled font notices/licenses. No Windows host ran.
- shell-example.json: the documented shell example executed against real QEMU.
- rai-portable-vnc.json: loopback RFB greeting and normal guest exit; no viewer or
  browser was used. The screen is guest-rendered, not HTML or the host panel.
- Context checks/logs cover existing behavior in debug/release and native debug;
  scheduler debug logs also cover normal, corrupt-register and user-fault cases.
- Clippy and console logs are JSON-wrapped to retain their exact textual output,
  including CRLF/blank lines, without Git whitespace normalization.

Initial failures are deliberately retained: initial-debug-failure and its runner
log show the coordinator failure after adding the owned RAM store; the fixed
headless/native checks passed after command-frame separation. The original Linux
run log shows the missing ramfb ROM/ASCII decoder failure. Firmware installation
and UTF-8 diagnostic decoding corrected that packaging issue. Another build
failed because .dockerignore was omitted from a source-hash input allowlist; it
was added explicitly. No failure was converted into successful evidence.

Observed debug frames were 20,832 bytes for the pre-fix combined command executor,
then 144 for the router plus 9,808 for the coordinator helper; the other system
path uses 12,720. Console::run retains 23,376. The stack remains 64 KiB. This is
not a complete call-graph bound, guard-page implementation or proof that every
future operation fits. Frame output is retained in rai-stack-frames.json.

Storage is volatile, sequential, guest EL1: eight files of up to 512 bytes, one
pending upload, path-like names, complete-upload publication and version checks.
No persistent disk, user permissions, isolated file service, mouse/touch browser,
multi-action atomic plan or AI file planning is claimed. No efficiency advantage
is established. Existing negative Meta timing comparisons remain unchanged.
