#!/usr/bin/env python3
"""Guest-native prompt workspace; host relays input and one bounded Meta proposal.
Default source is live Meta. --source replay is an explicitly scripted offline demo.
"""
import argparse
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import threading
sys.dont_write_bytecode=True
from files_demo import Guest,ROOT,build
from workspace_vnc import serve
SEED={
 '/inbox/aurora.txt':b'Project Aurora. Approved release notes: offline handoff and tested power budget. Owner: Maya.\n',
 '/inbox/field.txt':b'Project Aurora. Approved field checklist: confirm cable, firmware version and recovery procedure.\n',
 '/inbox/draft.txt':b'Project Aurora. DRAFT ONLY. Unapproved ideas for next quarter; not ready for handoff.\n',
 '/inbox/weekend.txt':b'Personal weekend grocery list: rice, tomatoes, coffee. Unrelated to Aurora.\n',
}
EXAMPLE='Collect approved Aurora notes in /work. Keep originals.'
class Workspace:
    def __init__(self,kernel,binary,output,source='meta',vnc=False,graphics=False):
        self.output=output;output.mkdir(parents=True,exist_ok=False)
        password=os.environ.get('RAI_VNC_PASSWORD')
        if password and (not password.isascii() or not 1<=len(password)<=8):raise ValueError('RAI_VNC_PASSWORD must be 1 to 8 ASCII characters')
        self.guest=Guest(kernel,graphics,vnc=vnc,vnc_password=password if vnc else None);self.binary=binary;self.source=source
        self.lock=threading.RLock();self.threads=[];self.closed=False
        try:
            for path,data in SEED.items():self.guest.upload(path,data)
            self.guest.request('ui.open');self.guest.request('ui.source',source)
        except BaseException:self.guest.close();raise
    def press(self,key):
        with self.lock:
            if self.closed:return
            response=self.guest.request('ui.key',str(key))
            if response.startswith('submit='):
                fields=dict(p.split('=',1) for p in response.split('; '));rid=int(fields['submit']);goal=bytes.fromhex(fields['prompt']).decode('ascii')
                if any(t.is_alive() for t in self.threads) or len(self.threads)>=8:
                    self.guest.request('ui.error',f'{rid}:A request is still running or this session reached its eight-request limit.')
                    return response
                catalog=self.guest.fields('fs.list');files=[]
                for entry in catalog['files'].split(','):
                    if entry=='-':continue
                    path,size=entry.rsplit(':',1);data=self.guest.read(path)
                    try:text=data.decode('utf-8')
                    except UnicodeDecodeError:text=None
                    files.append(dict(path=path,bytes=int(size),text=text))
                request=dict(request_id=rid,generation=int(catalog['generation']),goal=goal,files=files)
                thread=threading.Thread(target=self.plan,args=(request,),daemon=False);self.threads.append(thread);thread.start()
            return response
    def plan(self,request):
        folder=self.output/str(request['request_id']);folder.mkdir()
        path=folder/'request.json';path.write_text(json.dumps(request,indent=2)+'\n');rid=request['request_id']
        cmd=[str(self.binary),str(self.guest.process.args[2]),'--file-plan',str(path)]
        if self.source=='replay':
            # A disclosed fixed proposal, not a natural-language interpreter.
            if request['goal']==EXAMPLE:
                p=dict(request_id=rid,generation=request['generation'],decision='plan',actions=[dict(kind='copy',source=src,destination='/work/'+src.rsplit('/',1)[1]) for src in ['/inbox/aurora.txt','/inbox/field.txt']],reason='Copy the two approved Aurora documents. Leave draft and personal notes unchanged.')
            else:p=dict(request_id=rid,generation=request['generation'],decision='clarify',actions=[],reason='Offline replay supports the documented example only. Use Meta for your own instruction.')
            supplied=folder/'supplied.json';supplied.write_text(json.dumps(p)+'\n');cmd[2]='--file-plan-replay';cmd.append(str(supplied))
        try:
            env=dict(os.environ);env.pop('RAI_VNC_PASSWORD',None)
            result=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=40)
            (folder/'provider.stdout.jsonl').write_text(result.stdout);(folder/'provider.stderr.jsonl').write_text(result.stderr)
            if result.returncode:raise ValueError('provider_rejected')
            events=[json.loads(line) for line in result.stdout.splitlines()];plans=[e['plan'] for e in events if e.get('event')=='file_plan'];assert len(plans)==1
            plan=plans[0]
            with self.lock:
                if self.closed:return
                if plan['decision']!='plan':self.guest.request('ui.notice',f"{rid}:{plan['reason']}")
                else:
                    self.guest.request('ui.plan',f"{rid}:{plan['generation']}")
                    for a in plan['actions']:self.guest.request('ui.action',f"{rid}:{a['kind']}:{a['source']}:{a['destination']}")
                    self.guest.request('ui.ready',f"{rid}:{plan['reason']}")
                self.guest.capture(folder/'proposal.ppm')
        except (ValueError,AssertionError,subprocess.TimeoutExpired,OSError):
            with self.lock:
                if not self.closed:
                    # Cancellation invalidates this id; rejection is expected for late responses.
                    try:self.guest.request('ui.error',f'{rid}:No valid plan was applied. Check the connection or refine your instruction.')
                    except AssertionError:pass
    def close(self):
        with self.lock:self.closed=True
        for t in self.threads:t.join(timeout=45)
        (self.output/'uart.log').write_text('\n'.join(self.guest.log)+'\n');self.guest.close()

def host_build(debug=False):
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*([] if debug else ['--release'])],cwd=ROOT,check=True)
    return ROOT/f'target/host-meta/{host}/{"debug" if debug else "release"}/rai-meta'

def run(kernel,binary,output,source='meta',port=5900,bind='127.0.0.1'):
    app=Workspace(kernel,binary,output,source,vnc=True);keys=queue.Queue(maxsize=256)
    def press(key):keys.put_nowait(key)
    def keyboard():
        while True:
            key=keys.get()
            if key is None:return
            try:app.press(key)
            except (AssertionError,OSError):print('Guest rejected input.',flush=True)
    thread=threading.Thread(target=keyboard,daemon=True);thread.start()
    try:serve(app.guest.vnc_socket,press,port,bind)
    finally:
        keys.put(None);thread.join(timeout=10);app.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['meta','replay'],default='meta');p.add_argument('--output',type=Path,required=True);p.add_argument('--port',type=int,default=5900);p.add_argument('--debug',action='store_true');a=p.parse_args()
    try:run(build(a.debug),host_build(a.debug),a.output.resolve(),a.source,a.port)
    except KeyboardInterrupt:pass
