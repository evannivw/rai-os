# Submission package — November 4, 2026

Participant and representative: **Jarrol Villatoro**, individual entry.
Recommended track: **Indie Developer**. Problem statement: **Push the Frontier**.

## What to submit

Based on the supplied Participant Guide:

1. Public repository: https://github.com/evannivw/rai-os
2. An accessible **three-minute demo video URL** showing work built during the event.
3. One Cerebral Valley project entry, identifying the participant, original work,
   project purpose and Meta integration. Use [submission copy](SUBMISSION_DRAFT.md).

The public event page is https://cerebralvalley.ai/e/meta-ai-developer-hackathon.
It is not the authenticated project submission form. Exact required form fields
have not been inspected. No separate APK, slide deck or hosted website requirement
appears in the supplied guide. Follow any additional fields in the actual form.
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

## Before pressing Submit

- Watch the complete video with audio; replace reference narration if desired.
- Upload the final MP4 and verify its link works without requesting access.
- Check public repository access and paste the video URL into the project form.
- Use the individual participant details above; review the description and limits.
- Submit before the deadline and save the actual confirmation.

Preparing local artifacts does not upload the video or submit the project.
