# Native navigation and evaluator-setup evidence

Recorded late November 3 Guatemala / November 4 UTC, on uncommitted changes based
on c86c01a. source-hashes.json identifies final source content. No actual provider
request is part of these offline checks. Native authentication failure is a labelled
synthetic 401; no invalid real key was sent to Meta.

- 112 Rust tests; host/guest Clippy and formatting; Python syntax checks passed.
- QEMU debug/release: compact three-app navigation, correct edit targets, separate
  global Create, case-insensitive search, row gaps, bounded 128-character input,
  safe native auth notice and unchanged records after rejection/cancellation.
- Saved demo debug/release: workshop guard, calculated quote edits, CSV, the actual
  generated transfer app, conserved sum 17 and exact fresh-boot recovery.
- Generic app UI regression passed. Local and Linux final-image CLI tests verified
  VNC startup, saved resume, empty live mode with a synthetic format-only key,
  zero model requests and clean shutdown. Linux saved-demo test passed offline.
- Setup: missing/invalid/precedence checks; exclusive 0600 secret file creation;
  existing files and symlinks preserved; simulated 401/403/429/timeout/transport.
- Console/protocol 37 checks in both profiles. The debug mapping still has 4096
  bytes remaining; presentation added no memory-layout/driver/privilege changes.

Initial failures are retained. The first navigation oracle incorrectly searched
inbound UART responses for outbound notice text; the screen was correct and the
verifier now observes the notice call. CLI testing reproduced a real Python
KeyboardInterrupt/Thread.start race. The VNC server now signals stop intent and
unwinds normally; the unchanged start/stop test then passed locally and in Linux.

The default three app schemas/data came from earlier recorded Meta runs. The
polish checks do not count as new AI generation. Source mode live/saved is not an
authentication indicator. The later explicit connection probe has separate evidence.
Screen images are actual guest captures; the video/submission are still pending.
