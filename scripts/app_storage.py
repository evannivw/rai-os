#!/usr/bin/env python3
"""Explicit host checkpoint adapter. No model calls; no guest disk-driver claim."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

MAX_FILE=65536

def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode('ascii')
def unique(pairs):
    result={}
    for k,v in pairs:
        if k in result:raise ValueError('Duplicate snapshot key')
        result[k]=v
    return result

class Checkpoint:
    def __init__(self,path,binary,kernel):
        self.path=Path(path).resolve();self.binary=binary;self.kernel=kernel;self.closed=False
        self.path.parent.mkdir(parents=True,exist_ok=True)
        self.lock=os.open(str(self.path)+'.lock',os.O_CREAT|os.O_RDWR|os.O_NOFOLLOW,0o600)
        try:fcntl.flock(self.lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except BaseException:os.close(self.lock);raise ValueError('This saved device is already in use')
    def check(self,state):
        data=canonical(state)
        if len(data)>32768:raise ValueError('Snapshot too large')
        # A temporary input file contains app data, never an API credential.
        with tempfile.NamedTemporaryFile(prefix='rai-state-',suffix='.json') as file:
            file.write(data);file.flush()
            result=subprocess.run([str(self.binary),str(self.kernel),'--app-state-check',file.name],capture_output=True,text=True,timeout=15)
        if result.returncode:raise ValueError('Snapshot rejected by the Rust validator')
    def load(self):
        if not self.path.exists():return None
        with self.path.open('rb') as file:data=file.read(MAX_FILE+1)
        if len(data)>MAX_FILE:raise ValueError('Snapshot too large')
        envelope=json.loads(data,object_pairs_hook=unique)
        if not isinstance(envelope,dict) or set(envelope)!={'format','sha256','state'} or envelope['format']!='rai-device-1':raise ValueError('Unsupported snapshot envelope')
        state=envelope['state']
        if hashlib.sha256(canonical(state)).hexdigest()!=envelope['sha256']:raise ValueError('Snapshot checksum mismatch; original file preserved')
        self.check(state);return state
    def save(self,state):
        self.check(state)
        envelope=dict(format='rai-device-1',sha256=hashlib.sha256(canonical(state)).hexdigest(),state=state)
        data=canonical(envelope)+b'\n'
        if len(data)>MAX_FILE:raise ValueError('Snapshot too large')
        fd,temp=tempfile.mkstemp(prefix='.rai-checkpoint-',dir=self.path.parent)
        try:
            with os.fdopen(fd,'wb') as file:file.write(data);file.flush();os.fsync(file.fileno())
            os.replace(temp,self.path)
            directory=os.open(self.path.parent,os.O_RDONLY)
            try:os.fsync(directory)
            finally:os.close(directory)
        finally:
            if os.path.exists(temp):os.unlink(temp)
    def close(self):
        if not self.closed:os.close(self.lock);self.closed=True

def collect(g,schema,records):
    entries=g.fields('app.catalog')['apps']
    apps=[]
    for item in entries.split(','):
        if item=='-':continue
        app_id=item.split(':')[0];app=dict(spec=schema(g,app_id),rows=records(g,app_id))
        if app['spec'].get('workflow'):app['states']=[] if not app['rows'] else g.fields('app.export',app_id)['states'].split(',')
        apps.append(app)
    return dict(version=1,apps=apps)

def restore(g,state,deliver,schema,records):
    assert g.fields('app.catalog')['apps']=='-','Restore is only allowed in a fresh guest'
    g.request('app.restore.begin','1')
    for app in state['apps']:
        g.request('app.open');g.request('app.key','1002')
        for c in 'Restore saved app':g.request('app.key',str(ord(c)))
        reply=dict(p.split('=',1) for p in g.request('app.key','13').split('; '))
        deliver(g,dict(request_id=int(reply['submit']),base_revision=int(reply['revision']),decision='install',spec=app['spec'],reason='Restore a previously saved app.'))
        g.request('app.key','13')
        if app['spec'].get('workflow'):g.request('app.key','13')
        for _ in app['spec'].get('behaviors',[]):g.request('app.key','13')
        for _ in app['spec'].get('pages',[]):g.request('app.key','13')
        for rowno,row in enumerate(app['rows']):
            if rowno:g.request('app.key','110')
            for index,value in enumerate(row):
                if app['spec']['fields'][index]['kind']!='calculated':g.request('app.value',f'{index}:{value.encode("ascii").hex()}')
            g.request('app.key','13')
            if app['spec'].get('workflow'):g.request('app.restore.state',f"{app['spec']['id']}:{rowno}:{app['states'][rowno]}")
        if app['spec'].get('workflow'):
            actual=[] if not app['rows'] else g.fields('app.export',app['spec']['id'])['states'].split(',')
            assert actual==app['states'],'Restored states differ'
        assert schema(g,app['spec']['id'])==app['spec'] and records(g,app['spec']['id'])==app['rows'],'Restored bytes differ'
    g.request('app.restore.end','1')
    g.request('app.open')
