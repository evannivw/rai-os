#!/usr/bin/env python3
"""Prompt-to-microapp bridge. QEMU owns schemas, records, UI and installation."""
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
from workspace import host_build
from workspace_vnc import serve
from app_storage import Checkpoint,collect,restore
from app_exchange import Exchange
def schema(g,app_id):
    row=g.fields('app.schema',app_id);fields=[]
    for item in row['fields'].split(','):
        key,kind,required,label,default,formula=item.split('~')
        fields.append(dict(key=key,kind=kind,required=required=='1',label=bytes.fromhex(label).decode('ascii'),default=bytes.fromhex(default).decode('ascii')))
        if formula:fields[-1]['formula']=bytes.fromhex(formula).decode('ascii')
    return dict(id=row['id'],title=bytes.fromhex(row['title']).decode('ascii'),capability=row['capability'],fields=fields,filter_key=row['filter'],filter_label=bytes.fromhex(row['filter_label']).decode('ascii'))

def records(g,app_id):
    data=g.fields('app.export',app_id)
    rows=[] if data['count']=='0' else [[bytes.fromhex(v).decode('ascii') for v in row.split(',')] for row in data['rows'].split('|')]
    assert len(rows)==int(data['count'])
    return rows

def deliver(g,p):
    rid=p['request_id']
    if p['decision']!='install':g.request('app.notice',f"{rid}:{p['reason']}");return
    s=p['spec'];g.request('app.begin',f"{rid}:{p['base_revision']}:{s['id']}:{s['title'].encode().hex()}:{s['capability']}")
    for f in s['fields']:
        g.request('app.field',f"{rid}:{f['key']}:{f['kind']}:{int(f['required'])}:{f['label'].encode().hex()}:{f['default'].encode().hex()}")
        if f.get('formula'):g.request('app.formula',f"{rid}:{f['key']}:{f['formula'].encode().hex()}")
    g.request('app.filter',f"{rid}:{s['filter_key']}:{s['filter_label'].encode().hex()}")
    g.request('app.ready',f"{rid}:{p['reason']}")

class Studio:
    def __init__(self,kernel,binary,output,source='meta',vnc=False,state=None,exchange=None):
        self.output=output;output.mkdir(parents=True,exist_ok=False)
        password=os.environ.get('RAI_VNC_PASSWORD')
        if password and (not password.isascii() or not 1<=len(password)<=8):raise ValueError('RAI_VNC_PASSWORD must be 1 to 8 ASCII characters')
        self.exchange=Exchange(exchange) if exchange else None
        self.checkpoint=Checkpoint(state,binary,kernel) if state else None
        self.saved_revision=None;self.storage_error=None
        self.lock=threading.RLock();self.threads=[];self.closed=False
        self.binary=binary;self.kernel=kernel;self.source=source
        try:
            saved=self.checkpoint.load() if self.checkpoint else None
            self.guest=Guest(kernel,vnc=vnc,vnc_password=password if vnc else None)
            self.guest.request('app.open');self.guest.request('app.source',source)
            if saved is not None:restore(self.guest,saved,deliver,schema,records)
            self.saved_revision=self.guest.fields('app.catalog')['revision']
            self.guest.request('app.storage','restored' if saved is not None else 'RAM only')
        except BaseException:
            if hasattr(self,'guest'):self.guest.close()
            if self.checkpoint:self.checkpoint.close()
            raise
    def persist(self):
        if not self.checkpoint:return
        revision=self.guest.fields('app.catalog')['revision']
        if revision==self.saved_revision:return
        self.guest.request('app.storage','unsaved')
        try:
            self.checkpoint.save(collect(self.guest,schema,records))
            self.saved_revision=revision;self.storage_error=None;self.guest.request('app.storage','saved')
        except (OSError,ValueError,subprocess.TimeoutExpired):
            self.storage_error='Checkpoint failed; current changes remain in RAM.'
            self.guest.request('app.storage','save failed');print(self.storage_error,flush=True)
    def press(self,key):
        return self.dispatch('app.key',str(key))
    def dispatch(self,name,arg):
        with self.lock:
            if self.closed:return
            reply=self.guest.request(name,arg)
            if reply.startswith('transfer='):
                event=dict(p.split('=',1) for p in reply.split('; '))
                try:
                    if self.exchange is None:raise ValueError('No exchange folder selected. Start with --exchange PATH.')
                    self.exchange.transfer(self.guest,event,schema,records)
                except FileNotFoundError:
                    self.guest.request('app.transfer.notice',f"{event['request']}:Place {event['app']}.csv in the selected exchange inbox, then request Import again.")
                except FileExistsError:
                    self.guest.request('app.transfer.notice',f"{event['request']}:This export already exists. Existing files are never overwritten.")
                except ValueError as error:
                    self.guest.request('app.transfer.notice',f"{event['request']}:{str(error)[:160]}")
                except (OSError,AssertionError):
                    self.guest.request('app.transfer.notice',f"{event['request']}:Transfer rejected. Check file access, values, formulas and available row slots. No rows were imported.")
            if reply.startswith('submit='):
                state=dict(p.split('=',1) for p in reply.split('; '));rid=int(state['submit'])
                if any(t.is_alive() for t in self.threads) or len(self.threads)>=8:
                    self.guest.request('app.notice',f'{rid}:A request is running or this session reached its eight-request limit.');return reply
                catalog=self.guest.fields('app.catalog');ids=[];count=0
                for entry in catalog['apps'].split(','):
                    if entry=='-':continue
                    app_id,version,rows,title=entry.split(':');ids.append(app_id)
                    if app_id==state['target']:count=int(rows)
                request=dict(request_id=rid,base_revision=int(state['revision']),goal=bytes.fromhex(state['prompt']).decode('ascii'),target=None if state['target']=='-' else schema(self.guest,state['target']),app_ids=ids,record_count=count)
                thread=threading.Thread(target=self.plan,args=(request,),daemon=False);self.threads.append(thread);thread.start()
            self.persist()
            return reply
    def tap(self,x,y):
        with self.lock:
            if self.closed:return
            # Pointer geometry belongs to the guest. Submission has the same shape
            # as a key reply; a click on Create needs the same planning path.
            return self.dispatch('app.tap',f'{x}:{y}')
    def plan(self,r):
        folder=self.output/str(r['request_id']);folder.mkdir();path=folder/'request.json';path.write_text(json.dumps(r,indent=2)+'\n');rid=r['request_id']
        cmd=[str(self.binary),str(self.kernel),'--app-plan',str(path)]
        if self.source=='replay':
            from app_replay import propose
            plan=propose(r)
            supplied=folder/'supplied.json';supplied.write_text(json.dumps(plan)+'\n');cmd[2]='--app-plan-replay';cmd.append(str(supplied))
        try:
            env=dict(os.environ);env.pop('RAI_VNC_PASSWORD',None)
            result=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=40)
            (folder/'provider.stdout.jsonl').write_text(result.stdout);(folder/'provider.stderr.jsonl').write_text(result.stderr)
            if result.returncode:raise ValueError('provider_rejected')
            events=[json.loads(line) for line in result.stdout.splitlines()];plans=[e['plan'] for e in events if e.get('event')=='app_plan'];assert len(plans)==1
            with self.lock:
                if self.closed:return
                deliver(self.guest,plans[0]);self.guest.capture(folder/'proposal.ppm')
        except (ValueError,AssertionError,subprocess.TimeoutExpired,OSError):
            with self.lock:
                if not self.closed:
                    try:self.guest.request('app.notice',f'{rid}:No app installed. The proposal was unavailable, invalid or outside this runtime. See the local session log.')
                    except AssertionError:pass # A cancelled request cannot be revived.
    def close(self):
        with self.lock:self.closed=True
        for t in self.threads:t.join(timeout=45)
        try:
            with self.lock:
                if self.guest.process.poll() is None:self.persist()
            (self.output/'uart.log').write_text('\n'.join(self.guest.log)+'\n')
        finally:
            self.guest.close()
            if self.checkpoint:self.checkpoint.close()

def run(kernel,binary,output,source='meta',port=5900,bind='127.0.0.1',state=None,exchange=None):
    app=Studio(kernel,binary,output,source,vnc=True,state=state,exchange=exchange);keys=queue.Queue(maxsize=256)
    if app.exchange:print(f'CSV exchange folder: {app.exchange.root}',flush=True)
    def press(key):keys.put_nowait(key)
    def keyboard():
        while True:
            key=keys.get()
            if key is None:return
            try:
                if isinstance(key,tuple):app.tap(*key)
                else:app.press(key)
            except (AssertionError,OSError):print('Guest rejected input.',flush=True)
    worker=threading.Thread(target=keyboard,daemon=True);worker.start()
    try:serve(app.guest.vnc_socket,press,port,bind,tap=lambda x,y:keys.put_nowait((x,y)))
    finally:keys.put(None);worker.join(timeout=10);app.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['meta','replay'],default='meta');p.add_argument('--output',type=Path,required=True);p.add_argument('--port',type=int,default=5900);p.add_argument('--debug',action='store_true');p.add_argument('--state',type=Path,default=ROOT/'target/device.json');p.add_argument('--temporary',action='store_true');p.add_argument('--exchange',type=Path,default=ROOT/'target/exchange');a=p.parse_args()
    try:run(build(a.debug),host_build(a.debug),a.output.resolve(),a.source,a.port,state=None if a.temporary else a.state,exchange=a.exchange)
    except KeyboardInterrupt:pass
