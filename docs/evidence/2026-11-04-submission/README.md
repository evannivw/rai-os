# Actual submission recording — November 4, 2026

Source: 94e9efbac8960190740a3ab292c17ab6872d81c0, clean working tree.
One actual Meta recording attempt; two requests, both accepted on their first
attempt, with repairs disabled. Request 3: HTTP 200, 8892 ms, 4784 tokens.
Request 4: HTTP 200, 9284 ms, 4958 tokens. Both reported matching configured model.
These are measured request times, not a claimed speed advantage.

All four recording assertions passed: preview without mutation and 12/5 -> 8/9
transfer, rejection without balance/revision mutation, schema evolution preserving
records, exact restoration in a fresh QEMU guest. The recording contains 11 scenes.
Synthetic offline rehearsal preceded the live take; its first verifier failure
(directory ordering instead of request id) is documented in WORKLOG, not hidden.

recording.json preserves sampled frame hashes/times and scripted input events.
The local PNG sources remain in target/submission-live-20261104; large media is
not committed. video-manifest.json preserves source identity, timing transformations,
English narration and final MP4 hash. The MP4 is 1920x1080 with audio, nominally
180 seconds (container duration includes about 23 ms of audio padding). Full decode
passed; mean audio volume -20.0 dB, peak -4.5 dB. Eleven rendered scene proof frames
were visually inspected; four are included here. The voice is synthetic Samantha.
No claim of a human listening review; the participant should watch with audio.

The narration is an edited walkthrough, not an uncut live recording. Review controls
are exercised by scripted input; a real user operates the same review UI. Record
values are fictional. Generated apps remain bounded declarations sharing EL1.
The final showcase checkpoint can be opened without a key from repository root:

```sh
python3 -B scripts/demo.py saved --checkpoint docs/evidence/2026-11-04-submission/showcase.json --output target/submission-playback-new
```

Video upload and actual platform submission are still pending.
