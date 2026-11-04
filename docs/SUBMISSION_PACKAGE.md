# Submission record — November 4, 2026

Participant and representative: **Jarrol Villatoro**, individual entry.
Prepared track: **Indie Developer**. Problem statement: **Push the Frontier**.

**Status:** Jarrol reported that the project was submitted on November 4, 2026.
This records the participant's confirmation; no platform receipt has been
independently inspected. The video was uploaded and linked before submission.

## Submitted project materials

Based on the supplied Participant Guide:

1. Public repository: https://github.com/evannivw/rai-os
2. **[Three-minute demo video](https://drive.google.com/file/d/18e2NPFsIktJhBC-T7ZY4ANAfQhD9PvaO/view?usp=sharing)**, uploaded by the participant.
3. One Cerebral Valley project entry, identifying the participant, original work,
   project purpose and Meta integration. Use the [reviewed submission description](SUBMISSION.md).

The [project submission route](https://cerebralvalley.ai/e/meta-ai-developer-hackathon/hackathon/submit)
is separate from the [public event page](https://cerebralvalley.ai/e/meta-ai-developer-hackathon).
The authenticated form and receipt have not been independently inspected. The
participant supplied the form questions about partner feedback and Meta use.
No separate APK, slide deck or hosted website requirement appears in the supplied guide.
Never upload an API key or private environment file. Evaluators can run the saved
kit without a key, or use their own for generation: [try RAI-OS](TRY_RAI_OS.md).

Deadline: November 4 at 9 PM EST, **20:00 Guatemala**; internal target **18:00**.
The event page confirms the deadline. Preserve the supplied guide's distinction
between kickoff October 26 and building period October 27–November 4. Eligibility
of October 26 work remains unresolved; retain real history and identify it honestly.

## Recording and provenance

The recording harness samples real QEMU screens during scripted interaction and
checks the resulting guest state. Offline rehearsal uses labelled recorded fixtures;
only a verified fresh two-request Meta recording can feed the final renderer.
The edit uses held or shortened footage to fit 180 seconds; it is not a latency
benchmark or an uncut live session. Fictional records are entered through guest forms.
The English reference voice is synthetic macOS Samantha, not the participant's voice.

Narrative: prompt, review, enter data, transfer, reject an invalid transfer, evolve
by prompt, preserve records, reboot and recover. The generated apps are declarative;
the guest runtime and UI currently share EL1. Host services supply HTTPS, checkpoints
and Git. Physical-phone installation and isolated app services remain future work.

To reproduce on a Mac with the documented build prerequisites and ffmpeg installed:

```sh
python3 -B scripts/record_submission.py --source offline --output target/rehearsal-new
# Uses your configured key; requires clean sources; at most two actual requests:
python3 -B scripts/record_submission.py --source meta --output target/recording-new
python3 -B scripts/render_submission.py --capture target/recording-new --output target/video-new
```

Recording outputs include frame hashes, input events and state assertions. Video
outputs include narration, captions, rendering manifest and media metadata. These
large local artifacts remain outside Git. Failed takes must remain identifiable.

## Prepared local artifacts

The verified fresh recording and rendering are complete. See
[evidence and measured outcomes](evidence/2026-11-04-submission/README.md).
Local deliverables (ignored by Git):

- `target/submission-video-20261104/rai-os-demo.mp4`: three-minute 1080p edit.
- `target/rai-os-submission-20261104.zip`: video, captions, narration, copy and provenance.
- `target/submission-video-20261104/captions-en.srt`: optional external captions;
  the video already includes burned-in captions.

The public repo contains a saved checkpoint for the exact generated app from the
video, plus requests, responses and verification evidence. No API key is included.

## Submission follow-up

- Retain the platform confirmation and check that its repository/video links are correct.
- Keep video access enabled and verify playback with audio from a logged-out window.
- Keep the submitted description and dated evidence available while fixing documented issues.
- If changing form fields, use the platform's permitted editing flow; repository updates
  alone do not update the submitted form or the already-recorded video.

The participant supplied the uploaded Drive video link on November 4. An anonymous
page request returned the video title without an access-request page. Full anonymous
video playback and byte-for-byte identity with the local MP4 were not verified.
The participant has since confirmed project submission. Earlier pending statuses
in dated work-log/evidence entries describe the state when those records were made.
Local ZIP contents are a recording-time snapshot; use SUBMISSION.md for the latest
reviewed description.

## Repository About field

The November 4 audit confirmed public access. The external GitHub About field
still uses the earlier generic sentence; it is separate from this repository's
reviewed description. Suggested replacement:

> Rust AArch64 OS prototype with Meta-generated apps, checked local execution, and a native QEMU interface.

[Audit evidence](evidence/2026-11-04-repository-audit/README.md) records why this
external metadata change was not possible in that session. Repository pushes do
not update either GitHub About or the submitted Cerebral Valley form.
