#!/usr/bin/env python3
"""Produce a checked portable host package. Does not boot QEMU or process files in the guest."""
import argparse
from datetime import datetime,timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import zipfile
sys.dont_write_bytecode=True
from check_offline_pack import ROOT,FIXTURE,load,check_package


def build(debug=False):
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*([] if debug else ['--release'])],cwd=ROOT,check=True)
    return ROOT/f'target/host-meta/{host}/{"debug" if debug else "release"}/rai-meta'


def replay_plan():
    r=load(FIXTURE/'request.json');e=load(FIXTURE/'expected.json')
    return dict(request_goal=r['goal'],request_version=r['request_version'],catalog_version=1,permissions_version=1,decision='publish',selected=e['selected'],evidence=[dict(source=v['source'],quote=v['contains']) for v in e['required_evidence']],pending=e['pending'],reason='Explicit oracle-supplied development replay; not live model inference.')


def run(binary,source,out):
    out.mkdir(exist_ok=False);request=FIXTURE/'request.json'
    command=[str(binary),'host-only','--mission-plan',str(request)];env=dict(os.environ)
    if source=='replay':
        env['MODEL_API_KEY']='';path=out/'replay.json';path.write_text(json.dumps(replay_plan(),indent=2)+'\n');command[2]='--mission-plan-replay';command.append(str(path))
    started=time.monotonic();result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=40)
    (out/'plan.stdout.jsonl').write_text(result.stdout);(out/'plan.stderr.jsonl').write_text(result.stderr)
    report=dict(source=source,execution_location='host',guest_execution=False,plan_exit=result.returncode,published=False)
    items=[json.loads(s) for s in result.stdout.splitlines()]
    diagnostics=[e for e in items if e['event']=='provider_diagnostic']
    if diagnostics:report['provider']=diagnostics[0]['diagnostic']
    if result.returncode==0:
        p=next(e['plan'] for e in items if e['event']=='mission_plan');path=out/'plan.json';path.write_text(json.dumps(p,indent=2)+'\n')
        if p['decision']=='publish':
            result=subprocess.run([str(binary),'host-only','--mission-build',str(request),str(path),str(out/'package')],cwd=ROOT,env=dict(env,MODEL_API_KEY=''),capture_output=True,text=True,timeout=30)
            (out/'build.stdout.jsonl').write_text(result.stdout);(out/'build.stderr.jsonl').write_text(result.stderr);report['build_exit']=result.returncode
            if result.returncode==0:
                package=out/'package/published';report['checked']=check_package(package);report['published']=True
                with zipfile.ZipFile(out/'aurora-offline.zip','x',compression=zipfile.ZIP_DEFLATED) as z:
                    for file in sorted(package.rglob('*')):
                        if file.is_file():z.write(file,Path('aurora-offline')/file.relative_to(package))
                with zipfile.ZipFile(out/'aurora-offline.zip') as z:
                    assert set(z.namelist())=={'aurora-offline/'+str(p.relative_to(package)) for p in package.rglob('*') if p.is_file()}
                    for name in z.namelist():assert z.read(name)==(package/name.removeprefix('aurora-offline/')).read_bytes()
                report['archive_sha256']=hashlib.sha256((out/'aurora-offline.zip').read_bytes()).hexdigest()
        else:report['decision']=p['decision']
    report['end_to_end_ms']=round((time.monotonic()-started)*1000);(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['replay','meta'],default='replay');p.add_argument('--output',type=Path,required=True);p.add_argument('--debug',action='store_true');a=p.parse_args()
    out=a.output.resolve();assert not out.exists(),'Output must be new';binary=build(a.debug)
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    manifest=dict(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()),observed_utc=datetime.now(timezone.utc).isoformat(),source=a.source,model='muse-spark-1.3' if a.source=='meta' else None,max_tokens=2048,reasoning_effort='minimal',schema='rai_offline_mission',maximum_calls=1 if a.source=='meta' else 0,retries=0,fallback=False,binary_sha256=sha(binary),fixture_hashes={str(p.relative_to(FIXTURE)):sha(p) for p in sorted(FIXTURE.rglob('*')) if p.is_file()},execution_location='host',guest_execution=False)
    # Create a sibling session directory only inside the new user-requested output.
    out.mkdir();(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    report=run(binary,a.source,out/'run');print(json.dumps(report,indent=2))
    if not report['published']:raise SystemExit(1)
