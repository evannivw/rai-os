#!/usr/bin/env python3
"""Make a labeled silent evidence storyboard, not a live-interaction recording."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import textwrap

ROOT = Path(__file__).resolve().parents[1]


def render(check, output):
    report = json.loads((check / 'report.json').read_text())
    if report.get('verified') is not True or report.get('source') != 'saved_actual_meta_results':
        raise ValueError('Run scripts/demo.py verify successfully first')
    if not shutil.which('ffmpeg') or not shutil.which('ffprobe'):
        raise ValueError('FFmpeg with drawtext and ffprobe are required for this optional preview')
    output.mkdir(parents=True, exist_ok=False)
    live = ROOT / 'docs/evidence/2026-11-03-app-workflows/live'
    quote = ROOT / 'docs/evidence/2026-11-03-app-exchange/live'
    # Duration, title, on-screen explanation, source image and honest origin label.
    scenes = [
        (15, 'Tools shaped\nby you.',
         'Describe a small application. Review its behavior. Use it inside RAI-OS.',
         check / '01-launcher.png', 'SAVED SHOWCASE / REAL QEMU'),
        (20, 'A request becomes\nan application.',
         'Meta generated fields and a balance formula for a repair workshop. One recorded request: 5.518 seconds provider time.',
         live / '01-fields.png', 'EARLIER LIVE META RUN / NOV 3'),
        (20, 'Review the process.\nThen install.',
         'The model defines stages and actions. Rust validates the proposal. The user reviews fields and rules before installation.',
         live / '02-process-review.png', 'EARLIER LIVE META RUN / NOV 3'),
        (20, 'Not paid?\nNot delivered.',
         'This repair is done, but its balance is 4000 cents. The declared delivery rule blocks the action without changing the order.',
         check / '03-blocked.png', 'SAVED APP / NEW QEMU REHEARSAL'),
        (20, 'Record payment.\nDeliver locally.',
         'Payment changes the balance to zero. Delivery succeeds. Ordinary edits are now locked. No inference for these actions.',
         check / '04-delivered.png', 'SAVED APP / NEW QEMU REHEARSAL'),
        (15, 'Your tool\ncan evolve.',
         'Separate recorded example: a second prompt added cost and profit to a quoting app while preserving its original records.',
         quote / '04-evolve-review.png', 'SEPARATE LIVE CREATE + UPDATE RUN'),
        (20, 'Change the cost.\nKeep the logic.',
         'Increase one cost from 150 to 175 cents. Total profit changes from 285 to 260 locally. The workshop stays unchanged.',
         check / '06-local-edit.png', 'SAVED APP / NEW QEMU REHEARSAL'),
        (15, 'Restart.\nKeep your work.',
         'Both apps, their records and process states recover after a fresh QEMU boot. Persistence uses an explicit host checkpoint.',
         check / '09-restarted.png', 'REAL QEMU RESTART / NO INFERENCE'),
        (15, 'Small kernel.\nExplicit boundaries.',
         'Rust guest: validation, execution, graphics. Host: HTTPS, input, saved sessions. Apps currently share EL1 privilege.',
         check / '02-orders.png', 'PROTOTYPE / 3 APPS, 6 FIELDS, 8 ROWS'),
        (20, 'Run it.\nCreate your own.',
         'Try the saved apps without a key. Use your own Meta key for new supported tools. Native code and physical phones remain future work.',
         check / '01-launcher.png', 'github.com/evannivw/rai-os'),
    ]
    manifest = dict(kind='silent_edited_still_frame_storyboard', duration_seconds=180,
                    live_video=False, voice_track=False, verification_report_sha256=
                    hashlib.sha256((check / 'report.json').read_bytes()).hexdigest(), scenes=[])
    clips = []
    for index, (duration, title, body, frame, origin) in enumerate(scenes):
        if not frame.is_file(): raise ValueError(f'Missing source capture: {frame}')
        scene = dict(start_seconds=sum(s['duration_seconds'] for s in manifest['scenes']),
                     duration_seconds=duration, title=title, narration=body, origin=origin,
                     frame=str(frame.relative_to(ROOT)) if frame.is_relative_to(ROOT) else str(frame),
                     frame_sha256=hashlib.sha256(frame.read_bytes()).hexdigest())
        manifest['scenes'].append(scene)
        texts = [('brand', 'RAI-OS / PROMPT TO APP', 38, 80, '0xc1f27d'),
                 ('title', title, 70, 200, '0xe6ece7'),
                 ('body', textwrap.fill(body, width=39), 38, 425, '0xe6ece7'),
                 ('origin', textwrap.fill(origin, width=44), 28, 795, '0xc1f27d'),
                 ('note', 'EDITED QEMU CAPTURES / SILENT PREVIEW\nNot continuous live interaction', 25, 940, '0x9ba9a2')]
        filters = [f'color=c=0x060908:s=1920x1080:r=12:d={duration}[bg]',
                   '[0:v]scale=576:960:flags=neighbor[screen]',
                   '[bg][screen]overlay=1250:60:shortest=1[base]']
        chain = '[base]'
        for n, (name, text, size, y, color) in enumerate(texts):
            filename = f'{index:02d}-{name}.txt'
            (output / filename).write_text(text)
            font = 'fontfile=/System/Library/Fonts/Menlo.ttc' if Path('/System/Library/Fonts/Menlo.ttc').is_file() else 'font=monospace'
            chain += f'drawtext={font}:textfile={filename}:expansion=none:fontsize={size}:fontcolor={color}:x=90:y={y}:line_spacing=15'
            chain += ',' if n + 1 < len(texts) else '[out]'
        filters.append(chain)
        clip = f'clip-{index:02d}.mp4'; clips.append(clip)
        subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-nostdin',
                        '-loop', '1', '-framerate', '12', '-i', str(frame),
                        '-filter_complex', ';'.join(filters), '-map', '[out]', '-t', str(duration),
                        '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '18', '-pix_fmt', 'yuv420p',
                        '-an', clip], cwd=output, check=True)
    (output / 'clips.txt').write_text(''.join(f"file '{clip}'\n" for clip in clips))
    subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-nostdin', '-f', 'concat',
                    '-safe', '0', '-i', 'clips.txt', '-c', 'copy', '-movflags', '+faststart',
                    'rai-os-preview.mp4'], cwd=output, check=True)
    metadata = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_format',
                          '-show_streams', '-of', 'json', str(output / 'rai-os-preview.mp4')], text=True))
    assert abs(float(metadata['format']['duration']) - 180) < .1
    assert len(metadata['streams']) == 1 and metadata['streams'][0]['codec_type'] == 'video'
    assert (metadata['streams'][0]['width'], metadata['streams'][0]['height']) == (1920, 1080)
    manifest['video_sha256'] = hashlib.sha256((output / 'rai-os-preview.mp4').read_bytes()).hexdigest()
    (output / 'scenes.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (output / 'media-info.json').write_text(json.dumps(metadata, indent=2) + '\n')
    print(output / 'rai-os-preview.mp4')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    render(args.check.resolve(), args.output.resolve())
