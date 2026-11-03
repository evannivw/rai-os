#!/usr/bin/env python3
"""Portable Linux environment entry point. Never loads credentials for offline actions."""
import argparse
from contextlib import nullcontext
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
os.chdir(ROOT)

def binaries():
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    return ROOT/f'target/host-meta/{host}/release/rai-meta',ROOT/'target/display/aarch64-unknown-none-softfloat/release/rai-os'

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path);p.add_argument('action',choices=['build','verify','console','desktop','context-rules','context-meta','workspace','workspace-replay','workspace-verify','apps','apps-replay','apps-verify','apps-storage-verify'],default='verify',nargs='?');a=p.parse_args()
    binary,kernel=binaries()
    if a.action=='build':
        subprocess.run(['cargo','build','--locked','--bin','rai-os','--features','guest-display','--target-dir','target/display','--release'],check=True)
        host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
        subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta','--release'],check=True)
        paths=[]
        for name in ['src','host','scripts','fixtures','assets','.cargo','portable']:
            paths.extend(p for p in (ROOT/name).rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts)
        paths.extend(ROOT/name for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','linker.ld','.dockerignore','.gitattributes'])
        hashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
        metadata=dict(source_files=hashes,source_digest=hashlib.sha256(json.dumps(hashes,sort_keys=True).encode()).hexdigest(),kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),bridge_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
        (ROOT/'target/portable-build-info.json').write_text(json.dumps(metadata,indent=2)+'\n');return
    assert binary.is_file() and kernel.is_file(),'Build the image or run portable.py build first'
    if a.action=='console':
        raise SystemExit(subprocess.call(['sh','scripts/qemu.sh',str(kernel)]))
    from files_demo import run as files
    from semantic_session import run as context
    from context_session import FIXTURE
    if a.output:a.output.mkdir(parents=True,exist_ok=False)
    with (nullcontext(str(a.output.resolve())) if a.output else tempfile.TemporaryDirectory(prefix='rai-portable-')) as directory:
        output=Path(directory)
        (output/'build-info.json').write_bytes((ROOT/'target/portable-build-info.json').read_bytes())
        if a.action in ['apps','apps-replay']:
            from apps import run
            run(kernel,binary,output/'apps','meta' if a.action=='apps' else 'replay',bind='0.0.0.0',state=Path('/state/device.json'));return
        if a.action=='apps-storage-verify':
            from verify_app_storage import verify
            verify(kernel,binary,output/'storage');return
        if a.action=='apps-verify':
            from verify_apps import verify
            verify(kernel,binary,output/'apps');return
        if a.action in ['workspace','workspace-replay']:
            from workspace import run
            run(kernel,binary,output/'workspace','meta' if a.action=='workspace' else 'replay',bind='0.0.0.0');return
        if a.action=='workspace-verify':
            from verify_workspace import verify
            verify(kernel,binary,output/'workspace');return
        if a.action=='desktop':
            os.environ.setdefault('RAI_DISPLAY','vnc');files(kernel,output/'files',graphics=True,hold=True);return
        if a.action=='verify':
            report=files(kernel,output/'files',verify=True)
            decisions,checked=context(binary,kernel,'rules',output/'context',fixture=FIXTURE,contextual=True)
            result=dict(system=platform.system(),machine=platform.machine(),rust=subprocess.check_output(['rustc','-V'],text=True).strip(),qemu=subprocess.check_output(['qemu-system-aarch64','--version'],text=True).splitlines()[0],files=report,context=checked,context_report=decisions)
            (output/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2));return
        source='meta' if a.action=='context-meta' else 'rules'
        report,checked=context(binary,kernel,source,output/'context',fixture=FIXTURE,contextual=True)
        print(json.dumps(dict(report=report,audit=checked),indent=2))

if __name__=='__main__':main()
