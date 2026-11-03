#!/usr/bin/env python3
"""Verify actual guest RGB dimming, installed authority, recovery and safe restore."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,BRIGHTNESS,SPEC,deliver,schema,records,build,host_build,ROOT
from files_demo import Guest
from verify_apps import submit,stage

def pixels(path):return path.read_bytes().split(b'\n',3)[3]
def patch(g,path,expected):
    g.capture(path);rgb=pixels(path);offset=(460*480+100)*3
    assert rgb[offset:offset+3]==bytes([expected]*3),(rgb[offset:offset+3],expected)

def verify(kernel,binary,out,source='replay'):
    head=subprocess.run(['git','rev-parse','HEAD'],capture_output=True,text=True);commit=head.stdout.strip() if head.returncode==0 else None
    dirty=bool(subprocess.run(['git','status','--porcelain'],capture_output=True,text=True).stdout.strip()) if commit else None
    if source=='meta':assert commit and dirty is False,'Freeze source before the single live call'
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json';checks=[]
    report=dict(source=source,maximum_live_calls=1 if source=='meta' else 0,retries=0,source_commit=commit,source_worktree_dirty=dirty,kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),mode='guest software RGB attenuation; not physical backlight',verified=False,checks=checks)
    app=Studio(kernel,binary,out/'session',source,state=state);g=app.guest
    try:
        for c in BRIGHTNESS:app.press(ord(c))
        app.press(13)
        for t in app.threads:t.join(timeout=45);assert not t.is_alive()
        assert g.request('app.key','0')=='state=review'
        plan=[e['plan'] for p in (out/'session').glob('*/provider.stdout.jsonl') for e in map(json.loads,p.read_text().splitlines()) if e.get('event')=='app_plan'];assert len(plan)==1
        spec=plan[0]['spec'];report['plan']=plan[0]
        assert spec['capability']=='display.brightness' and len(spec['fields'])==1
        f=spec['fields'][0];assert f['kind']=='number' and f['required'] and f['default']=='50' and spec['filter_key']==''
        assert g.fields('app.display')==dict(available='true',brightness='100',mode='software_rgb') and not state.exists()
        g.capture(out/'permission-review.ppm');app.tap(240,720)
        app_id=spec['id'];assert schema(g,app_id)==spec and records(g,app_id)==[]
        assert g.fields('app.display')['brightness']=='100';saved=state.read_bytes();app.press(27);g.capture(out/'home-100.ppm')
        g.request('app.open',app_id);app.tap(240,720);assert g.fields('app.display')['brightness']=='50';patch(g,out/'control-50.ppm',127)
        app.press(27);g.capture(out/'home-50.ppm')
        base=pixels(out/'home-100.ppm');dim=pixels(out/'home-50.ppm');errors=[abs(b-a//2) for a,b in zip(base,dim)]
        assert max(errors)<=2 and sum(dim)<sum(base)*.55 and sum(dim)>sum(base)*.4
        report['pixel_check']=dict(max_channel_difference_from_half=max(errors),mean_channel_difference_from_half=sum(errors)/len(errors),total_rgb_ratio=sum(dim)/sum(base))
        for _ in range(3):g.request('app.key','0')
        g.capture(out/'home-50-repeat.ppm');assert pixels(out/'home-50-repeat.ppm')==dim
        checks.append('Review and install do not actuate; Apply changes actual QEMU pixels including the home screen; repeated redraws do not compound dimming')
        g.request('app.open',app_id);g.request('app.value','0:3230');assert g.fields('app.display')['brightness']=='50';app.press(13);patch(g,out/'control-20.ppm',51)
        for value in ['0','19','101','-1','999','oops','']:
            g.request('app.value','0:'+value.encode().hex());g.request('app.run',error='display_brightness_range');assert g.fields('app.display')['brightness']=='20'
        app.tap(240,350);assert g.fields('app.display')['brightness']=='100';app.press(27);g.capture(out/'home-reset.ppm');assert pixels(out/'home-reset.ppm')==base
        assert state.read_bytes()==saved and records(g,app_id)==[]
        checks.append('20 percent has exact 51/255 white pixels; invalid inputs have no effect; reset recovers the exact original image; actions do not create records or change saved schemas')
        # This second app is an explicit fixture for denied-authority testing,
        # not an additional generated app or a second provider call.
        record=submit(g,'Verifier record fixture');record['spec']=copy.deepcopy(SPEC);deliver(g,record);g.request('app.key','13');app.persist()
        g.request('app.run',error='app_capability_denied');assert g.fields('app.display')['brightness']=='100'
        before=g.request('app.catalog');saved=state.read_bytes()
        bad=submit(g,'Unauthorized capability update','inspection');bad['spec']=copy.deepcopy(spec);bad['spec']['id']='inspection';stage(g,bad);g.request('app.ready',f"{bad['request_id']}:Escalate",error='app_capability_change')
        bad=submit(g,'Invalid default',app_id);bad['spec']=copy.deepcopy(spec);bad['spec']['fields'][0]['default']='0';stage(g,bad);g.request('app.ready',f"{bad['request_id']}:Invalid default",error='display_brightness_range')
        g.request('app.begin',f"{bad['request_id']}:{bad['base_revision']}:screen:53637265656e:system.all",error='app_capability_denied')
        g.request('app.key','27');g.request('app.ready',f"{bad['request_id']}:Late",error='app_stale_request')
        assert g.request('app.catalog')==before and state.read_bytes()==saved and schema(g,app_id)==spec
        g.request('app.open',app_id);g.request('app.value','0:3530');app.press(13);assert g.fields('app.display')['brightness']=='50'
        checks.append('Records-only app cannot actuate display; capability escalation, invalid default, unknown permission and late response rejected without changing installed state')
    except (AssertionError,ValueError,OSError) as e:report['error']=str(e) or type(e).__name__;raise
    finally:app.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    restored=Studio(kernel,binary,out/'restored','replay',state=state)
    try:
        assert schema(restored.guest,app_id)==spec and not restored.threads
        assert restored.guest.fields('app.display')['brightness']=='100'
        restored.guest.request('app.open',app_id);patch(restored.guest,out/'restored-100.ppm',255)
        restored.press(13);assert restored.guest.fields('app.display')['brightness']=='50'
        restored.press(1010);assert restored.guest.fields('app.display')['brightness']=='100'
        checks.append('Saved control schema restores without inference or actuation; user can apply again and recover with F6')
        report['verified']=True
    finally:restored.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

def headless(binary,out):
    subprocess.run(['cargo','build','--locked','--bin','rai-os','--target-dir','target/no-display'],cwd=ROOT,check=True)
    kernel=ROOT/'target/no-display/aarch64-unknown-none-softfloat/debug/rai-os';g=Guest(kernel,expect_display=False)
    try:
        from apps import BRIGHTNESS_SPEC
        p=submit(g,'Headless control');p['spec']=BRIGHTNESS_SPEC;deliver(g,p);g.request('app.key','13')
        assert g.fields('app.display')==dict(available='false',brightness='100',mode='software_rgb')
        g.request('app.run',error='display_unavailable');g.request('app.key','1010',error='display_unavailable')
        assert g.fields('app.display')['brightness']=='100';print('PASS: actual headless QEMU rejects display actions')
    finally:(out/'headless-uart.log').write_text('\n'.join(g.log)+'\n');g.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--headless',action='store_true');p.add_argument('--source',choices=['meta','replay'],default='replay');p.add_argument('--output',type=Path,required=True);a=p.parse_args();binary=host_build(a.debug);verify(build(a.debug),binary,a.output.resolve(),a.source)
    if a.headless:headless(binary,a.output.resolve())
