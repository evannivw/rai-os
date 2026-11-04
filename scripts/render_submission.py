#!/usr/bin/env python3
"""Render a 180-second narrated edit from verified, actually sampled QEMU interaction."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import textwrap

SCENES=[
(12,'Your tools.\nYour OS.','Prompt -> review -> use -> evolve','SAVED SHOWCASE / REAL QEMU',
 'A small team should be able to shape its own tools. RAI OS explores that idea: describe a tool, review its behavior, and use it inside a Rust operating system.'),
(18,'Start with\na request.','An empty session. One concrete goal.','NEW SESSION / SCRIPTED KEYBOARD INPUT',
 'We start a new session and describe a tool for moving quantities between locations, with a total and a chart. These names and rules come from the application request. They are not special application types built into the kernel.'),
(22,'Meta proposes.\nYou decide.','Fields + behavior + presentation','ACTUAL META REQUEST / REVIEW BEFORE INSTALL',
 'This is an actual Meta request. The model receives the implemented capability catalog and proposes a declarative application. Rust validates its fields and operations. We review the transfer rule and the page before installation. A proposal alone cannot execute an operation.'),
(18,'Make it\nyour workspace.','Two locations. Fictional test records.','REAL GUEST FORM / SCRIPTED TEST INPUT',
 'The generated form accepts our test records: twelve units in Warehouse, and five in Dispatch. We enter a quantity of four and choose Dispatch as the destination. These are fictional demonstration records, entered through the same guest form controls.'),
(20,'Two records.\nOne checked change.','12 + 5 -> 8 + 9\nTotal preserved: 17','REAL GUEST EXECUTION / NO MODEL ROUND TRIP',
 'The preview shows both changes before we apply them. Warehouse falls from twelve to eight, while Dispatch rises from five to nine. The total remains seventeen. Rust validates and commits both records together, and the chart updates locally without another model request.'),
(18,'The boundary\nis visible.','Request 99. Reject. Preserve both balances.','REAL REJECTION / NO DATA MUTATION',
 'Now we request ninety nine units. The source does not have enough. The operation is rejected, and both balances stay unchanged. The same checks apply to every generated app using this capability. The model describes behavior; the runtime enforces its rules.'),
(22,'Evolve the tool\nby prompt.','Add a report. Keep the fields and rules.','SECOND ACTUAL META REQUEST / HUMAN REVIEW',
 'Next, we ask Meta to add a report page while keeping the fields and rules. The model sees the application definition, not the stored record values. We review the updated declaration before installation. Existing data stays in place.'),
(12,'New view.\nSame work.','A generated table and a dispatch note.','UPDATED APP / EXISTING RECORDS PRESERVED',
 'The new page presents the existing balances and a dispatch note. The tool has changed through a second prompt, while our records and transfer rule remain intact.'),
(16,'Restart.\nKeep your work.','Fresh QEMU boot. Exact recovery.','SAVED RECOVERY / NO INFERENCE',
 'We close the guest and start a fresh instance. The host checkpoint restores the evolved application and its exact records. This is saved recovery, not new generation. The application remains usable without another call to Meta.'),
(14,'Own kernel.\nClear boundaries.','Rust / AArch64 / QEMU\nGuest: UI, validation, execution\nHost: HTTPS, checkpoints, Git\nApps share EL1 today','BOUNDED PROTOTYPE / MOBILE DIRECTION',
 'Our Rust kernel runs on emulated ARM hardware. The guest draws and executes applications. The host provides HTTPS and persistence. Apps share kernel privilege today; isolated app services are future work.'),
(8,'Try it.\nMake it yours.','github.com/evannivw/rai-os\nStart with docs/TRY_RAI_OS.md','BUILT BY JARROL VILLATORO',
 'Try the public saved demo without a key. Bring your own Meta key to create new tools.'),
]

def probe(path):return json.loads(subprocess.check_output(['ffprobe','-v','error','-show_format','-show_streams','-of','json',str(path)],text=True))
def ffmpeg(args,cwd):subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-nostdin',*args],cwd=cwd,check=True)
def stamp(seconds):
 ms=round(seconds*1000);return f'{ms//3600000:02}:{ms//60000%60:02}:{ms//1000%60:02},{ms%1000:03}'

def render(capture,output):
    report=json.loads((capture/'recording.json').read_text())
    assert report['verified'] and report['source']=='meta' and report['actual_provider_calls']==2,'Do not narrate synthetic proposals as actual Meta'
    assert len(report['scenes'])==len(SCENES) and sum(s[0] for s in SCENES)==180
    output.mkdir(parents=True,exist_ok=False);manifest=dict(kind='edited_sampled_qemu_interaction',duration_seconds=180,source_commit=report['source_commit'],recording_sha256=hashlib.sha256((capture/'recording.json').read_bytes()).hexdigest(),actual_provider_calls=2,voice='macOS Samantha synthetic reference narration',timing='Source intervals preserved in capture; footage shortened or held to fit scenes, explicitly labelled',scenes=[])
    offset=0;subtitles=[];narration=[];clips=[]
    for i,((duration,title,body,origin,voice),recorded) in enumerate(zip(SCENES,report['scenes'])):
        prefix=f'{i+1:02}';frames=recorded['frames'];assert frames
        lines=[]
        for j,f in enumerate(frames):
            frame=capture/f['path'];assert hashlib.sha256(frame.read_bytes()).hexdigest()==f['sha256']
            seconds=frames[j+1]['time']-f['time'] if j+1<len(frames) else .2
            lines.extend([f"file '{frame.resolve()}'",f'duration {max(.02,seconds):.6f}'])
        lines.append(f"file '{(capture/frames[-1]['path']).resolve()}'")
        (output/f'{prefix}-frames.txt').write_text('\n'.join(lines)+'\n')
        (output/f'{prefix}-voice.txt').write_text(voice+'\n')
        subprocess.run(['say','-v','Samantha','-r','170','-f',f'{prefix}-voice.txt','-o',f'{prefix}-voice.aiff'],cwd=output,check=True)
        seconds=float(probe(output/f'{prefix}-voice.aiff')['format']['duration'])
        speed=max(1,seconds/(duration-1));assert speed<=1.3,'Narration needs editing rather than excessive speed'
        ffmpeg(['-i',f'{prefix}-voice.aiff','-af',f'atempo={speed:.6f},adelay=250|250,apad','-t',str(duration),'-ar','48000','-ac','2',f'{prefix}-voice.wav'],output)
        actual=frames[-1]['time']-frames[0]['time']+.2;ratio=min(1,(duration-.2)/actual)
        filters=[f'color=c=0x090b0f:s=1920x1080:r=24:d={duration}[bg]',f'[0:v]setpts={ratio:.8f}*(PTS-STARTPTS),fps=24,scale=576:960:flags=neighbor,tpad=stop_mode=clone:stop_duration={duration}[screen]',
                 '[bg]drawbox=x=1254:y=44:w=596:h=980:color=0x303743:t=1,drawbox=x=80:y=145:w=130:h=3:color=0x87bfff:t=fill[border]',
                 '[border][screen]overlay=1264:54:shortest=1[base]']
        texts=[('brand','RAI-OS / META HACKATHON 2026',30,80,'0x87bfff'),('title',title,62,210,'0xe7ebf1'),('body',textwrap.fill(body,43) if '\n' not in body else body,30,440,'0xa1adbc'),('origin',textwrap.fill(origin,49),24,885,'0x87bfff'),('note',f'{i+1:02}/11  |  EDITED QEMU INTERACTION / SCRIPTED INPUTS\nSYNTHETIC REFERENCE NARRATION',21,987,'0x9fa9b8')]
        font='fontfile=/System/Library/Fonts/Menlo.ttc'
        chain='[base]'
        draw=[]
        for name,text,size,y,color in texts:
            filename=f'{prefix}-{name}.txt';(output/filename).write_text(text)
            draw.append(f'drawtext={font}:textfile={filename}:expansion=none:fontsize={size}:fontcolor={color}:x=80:y={y}:line_spacing=15')
        chunks=textwrap.wrap(voice,88);spoken=seconds/speed
        for n,chunk in enumerate(chunks):
            start=.25+n*spoken/len(chunks);end=.25+(n+1)*spoken/len(chunks)
            filename=f'{prefix}-caption-{n}.txt';(output/filename).write_text(textwrap.fill(chunk,48))
            draw.append(f"drawtext={font}:textfile={filename}:expansion=none:fontsize=26:fontcolor=0xe7ebf1:x=80:y=740:line_spacing=12:enable='between(t,{start:.3f},{end:.3f})'")
            subtitles.append(f'{len(subtitles)+1}\n{stamp(offset+start)} --> {stamp(offset+end)}\n{chunk}\n')
        chain+=','.join(draw)+'[out]';filters.append(chain)
        clip=f'{prefix}-scene.mp4';clips.append(clip)
        ffmpeg(['-f','concat','-safe','0','-i',f'{prefix}-frames.txt','-i',f'{prefix}-voice.wav','-filter_complex',';'.join(filters),'-map','[out]','-map','1:a','-t',str(duration),'-c:v','libx264','-preset','veryfast','-crf','19','-pix_fmt','yuv420p','-c:a','aac','-b:a','160k',clip],output)
        manifest['scenes'].append(dict(start=offset,duration=duration,title=title,narration=voice,origin=origin,recorded_scene=recorded['name'],actual_seconds=recorded['actual_seconds'],footage_speed=1/ratio,voice_rate_multiplier=speed))
        narration.append(f'{stamp(offset)} — {stamp(offset+duration)} | {title.replace(chr(10)," ")}\n{voice}\n');offset+=duration
    (output/'clips.txt').write_text(''.join(f"file '{clip}'\n" for clip in clips))
    ffmpeg(['-f','concat','-safe','0','-i','clips.txt','-c','copy','-movflags','+faststart','rai-os-demo.mp4'],output)
    metadata=probe(output/'rai-os-demo.mp4');assert abs(float(metadata['format']['duration'])-180)<.15
    assert {s['codec_type'] for s in metadata['streams']}=={'audio','video'}
    video=next(s for s in metadata['streams'] if s['codec_type']=='video');assert (video['width'],video['height'])==(1920,1080)
    (output/'captions-en.srt').write_text('\n'.join(subtitles)+'\n');(output/'narration-en.txt').write_text('\n'.join(narration))
    manifest['video_sha256']=hashlib.sha256((output/'rai-os-demo.mp4').read_bytes()).hexdigest()
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');(output/'media-info.json').write_text(json.dumps(metadata,indent=2)+'\n')
    for i,s in enumerate(manifest['scenes']):
        ffmpeg(['-ss',str(s['start']+min(s['duration']-1,5)),'-i','rai-os-demo.mp4','-frames:v','1',f'proof-{i+1:02}.png'],output)
    print(output/'rai-os-demo.mp4')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--capture',required=True,type=Path);p.add_argument('--output',required=True,type=Path);a=p.parse_args();render(a.capture.resolve(),a.output.resolve())
