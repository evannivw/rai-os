# Demo kit verification, November 3

These checks exercised the kit before its source commit. source-files.json records
exact new script/fixture bytes; reports record compiled kernel hashes. The kernel
and host Rust sources are unchanged from the earlier workflow implementation.

- debug/release: real QEMU; exact saved apps, unpaid delivery denied, payment and
  delivery accepted, quote profit 285 -> 260, CSV, app separation, restart.
- entry: actual CLI + RFB startup for saved/resume/live; saved bytes preserved;
  live starts empty; zero model requests; clean shutdown.
- linux-arm64.log: same real-QEMU kit check in Linux ARM64 Docker, network disabled.
- preview: source-frame manifest, inspected video frames and FFprobe metadata.
  A 1920x1080, 180-second silent edited still-frame preview was rendered. It is
  not a screen recording, voiceover or submitted final video. Regenerate it with
  render_demo_preview.py; large MP4/intermediate clips remain outside Git.

The live models were called in the two earlier evidence runs cited in the fixture
provenance, not in these checks. No physical Windows, Linux laptop or phone test
is inferred from Docker on this Mac. No capacity/driver/isolation expansion.
