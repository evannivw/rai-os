#!/usr/bin/env python3
"""Record sampled real QEMU interaction and independent assertions; at most two Meta calls."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import threading
import time
from apps import Studio,build,host_build,records,schema
from app_storage import collect
from demo import FIXTURE
from verify_app_transfer import GOAL
from verify_app_repair import ScriptedStudio
from verify_app_pages import block
from files_demo import ROOT

class Recorder:
    def __init__(self,output,source):
        self.output=output;self.source=source;self.app=None;self.scenes=[];self.events=[];self.errors=[]
    def key(self,k):
        self.events.append(dict(time=time.time(),input='key',value=k));return self.app.press(k)
    def tap(self,x,y):
        self.events.append(dict(time=time.time(),input='tap',x=x,y=y));return self.app.tap(x,y)
    def text(self,value):
        for c in value:self.key(ord(c));time.sleep(.025)
    def hold(self,n=1):time.sleep(n)
    def scene(self,name,action):
        directory=self.output/name;directory.mkdir();frames=[];stop=threading.Event();started=time.monotonic()
        def sample():
            app=self.app
            if app is None:return
            with app.lock:
                if app.closed:return
                p=directory/f'{len(frames):05d}.ppm';app.guest.capture(p);p.unlink()
                png=p.with_suffix('.png');frames.append(dict(path=str(png.relative_to(self.output)),time=time.monotonic()-started,sha256=hashlib.sha256(png.read_bytes()).hexdigest()))
        sample()
        def loop():
            while not stop.wait(1/6):
                try:sample()
                except BaseException as e:self.errors.append(type(e).__name__);return
        t=threading.Thread(target=loop);t.start()
        try:action();self.hold(.5)
        finally:
            stop.set();t.join(timeout=15);assert not t.is_alive();sample()
            self.scenes.append(dict(name=name,actual_seconds=time.monotonic()-started,frames=frames))
            assert not self.errors,self.errors
    def g(self):return self.app.guest
    def wait_plan(self):
        for t in self.app.threads:t.join(timeout=45);assert not t.is_alive()
        assert self.g().request('app.key','0')=='state=review','Model did not produce a reviewable app'
        events=[json.loads(line) for p in self.app.output.glob('*/provider.stdout.jsonl') for line in p.read_text().splitlines()]
        return max((e['plan'] for e in events if e.get('event')=='app_plan'),key=lambda p:p['request_id'])['spec']
    def approve(self,s):
        before=self.g().request('app.catalog');self.key(13);self.hold(1)
        if s.get('workflow'):self.key(13);self.hold(1)
        for _ in s.get('behaviors',[]):self.key(13);self.hold(1)
        if s.get('transfer'):
            assert self.g().request('app.key','0')=='state=move_review';assert self.g().request('app.catalog')==before
            self.hold(1);self.key(13);self.hold(1)
        for _ in s.get('pages',[]):
            assert self.g().request('app.key','0')=='state=page_review';self.key(13);self.hold(1)
        assert schema(self.g(),s['id'])==s

def record(kernel,binary,output,source):
    output.mkdir(parents=True,exist_ok=False);r=Recorder(output,source)
    commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip())
    if source=='meta':assert not dirty,'Commit the recording harness before actual Meta calls'
    report=dict(verified=False,source=source,source_commit=commit,dirty=dirty,actual_provider_calls=0,checks=[],narration='not yet rendered')
    try:
        intro=output/'intro.json';intro.write_bytes(FIXTURE.read_bytes());r.app=Studio(kernel,binary,output/'intro-session','replay',state=intro)
        r.scene('01-home',lambda:r.hold(3));r.app.close();r.app=None
        state=output/'device.json'
        seed=json.loads((ROOT/'docs/evidence/2026-11-03-app-transfer/live/showcase.json').read_text())['state']['apps'][0]['spec']
        evolved=copy.deepcopy(seed);rule=seed['transfer'];evolved['pages'].append(dict(title='Dispatch report',blocks=[block('table','Balances',rule['balance_field'],rule['key_field']),block('text','Next step','Ready for dispatch')]))
        r.app=Studio(kernel,binary,output/'live-session','meta',state=state) if source=='meta' else ScriptedStudio(kernel,binary,output/'offline-session',proposals=[seed,evolved],state=state)
        r.app.repair_enabled=False
        def compose():r.tap(400,775);r.text(GOAL);r.hold(2)
        r.scene('02-prompt',compose)
        spec=None
        def create():
            nonlocal spec
            r.key(13);spec=r.wait_plan();assert spec.get('transfer') and len(spec.get('pages',[]))==1
            r.hold(1);r.approve(spec);r.hold(1)
        r.scene('03-create',create)
        rule=spec['transfer'];aid=spec['id'];keys=[f['key'] for f in spec['fields']];bi=keys.index(rule['balance_field']);ai=keys.index(rule['amount_field'])
        assert spec['pages'][0]['blocks'][0]['kind']=='metric' and spec['pages'][0]['blocks'][0]['aggregate']=='sum' and spec['pages'][0]['blocks'][0]['value']==rule['balance_field']
        def set_input(i,value):
            r.tap(150,220+i*72)
            for _ in range(32):r.key(8)
            r.text(value)
        def data():
            for row,(name,balance,amount,target) in enumerate([('Warehouse','12','4','Dispatch'),('Dispatch','5','1','Warehouse')]):
                if row:r.key(110)
                values={rule['key_field']:name,rule['balance_field']:balance,rule['amount_field']:amount,rule['target_field']:target}
                for i,f in enumerate(spec['fields']):
                    if f['kind']!='calculated':set_input(i,values.get(f['key'],f['default']))
                r.hold(.5);r.key(13);r.hold(.5)
            assert [v[bi] for v in records(r.g(),aid)]==['12','5']
        r.scene('04-data',data)
        def select():r.key(1003);r.tap(90,225)
        def transfer():
            select();before=collect(r.g(),schema,records);base=int(r.g().fields('app.catalog')['revision'])
            r.tap(290,125);r.hold(2);assert collect(r.g(),schema,records)==before
            r.tap(240,720);assert [v[bi] for v in records(r.g(),aid)]==['8','9'];assert int(r.g().fields('app.catalog')['revision'])==base+1
            r.key(1015);assert r.g().fields('app.page.values',f'{aid}:0')['metrics']=='17,-';r.hold(2)
        r.scene('05-transfer',transfer);report['checks'].append('Preview writes nothing; transfer changes 12/5 to 8/9 once, total 17')
        def reject():
            select();set_input(ai,'99');r.key(13);select();before=collect(r.g(),schema,records);base=r.g().request('app.catalog')
            with r.app.lock:r.g().request('app.tap','290:125',error='move_insufficient')
            assert collect(r.g(),schema,records)==before and r.g().request('app.catalog')==base;r.hold(3)
        r.scene('06-reject',reject);report['checks'].append('Insufficient transfer rejected without mutation')
        before=records(r.g(),aid);new_spec=None
        def evolve():
            nonlocal new_spec
            r.tap(60,125)
            prompt=f'Keep all fields and rules. Add page: table {rule["key_field"]}/{rule["balance_field"]}; text "Ready for dispatch".'
            assert len(prompt)<=128;r.text(prompt);r.hold(.7);r.key(13);new_spec=r.wait_plan()
            assert new_spec['transfer']==spec['transfer'] and len(new_spec['pages'])==2
            assert new_spec['fields']==spec['fields'] and new_spec['pages'][0]==spec['pages'][0]
            blocks=new_spec['pages'][1]['blocks']
            assert any(b['kind']=='text' and b['value']=='Ready for dispatch' for b in blocks)
            assert any(b['kind']=='table' and {b['value'],b['label_field']}=={rule['key_field'],rule['balance_field']} for b in blocks)
            r.approve(new_spec);assert records(r.g(),aid)==before
        r.scene('07-evolve',evolve)
        def report_page():r.key(1015);r.tap(370,720);r.hold(3)
        r.scene('08-new-page',report_page);report['checks'].append('Second proposal adds a page, retains the rule and all current records')
        final=collect(r.g(),schema,records);report['final_state']=final;report['actual_provider_calls']=r.app.provider_calls if source=='meta' else 0
        assert r.app.provider_calls==2 if source=='meta' else len(r.app.invocations)==2
        def restart():
            old=r.app;r.app=None;old.close();r.hold(.5)
            r.app=Studio(kernel,binary,output/'restored-session','replay',state=state)
            assert collect(r.g(),schema,records)==final
            r.hold(1);r.tap(150,250);r.key(1015);r.tap(370,720);r.hold(2)
        r.scene('09-restart',restart);report['checks'].append('Fresh guest restores the exact evolved app and rows; no new inference')
        r.scene('10-architecture',lambda:r.hold(2))
        def outro():r.key(27);r.hold(2)
        r.scene('11-try',outro);report['verified']=True
    except BaseException as e:
        report['error']=str(e) or type(e).__name__
        if r.app and source=='meta':report['actual_provider_calls']=max(report['actual_provider_calls'],r.app.provider_calls)
        raise
    finally:
        if r.app:r.app.close()
        report['scenes']=r.scenes;report['inputs']=r.events
        (output/'recording.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({k:v for k,v in report.items() if k not in ['scenes','inputs','final_state']},indent=2))
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    record(build(),host_build(),a.output.resolve(),a.source)
