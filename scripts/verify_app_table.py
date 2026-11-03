#!/usr/bin/env python3
"""Create a table app, verify exact guest totals and recover it after reboot."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,TABLE,records,schema,build,host_build

def verify(kernel,binary,out,source='replay'):
    head=subprocess.run(['git','rev-parse','HEAD'],capture_output=True,text=True)
    commit=head.stdout.strip() if head.returncode==0 else None
    dirty=bool(subprocess.run(['git','status','--porcelain'],capture_output=True,text=True).stdout.strip()) if commit else None
    if source=='meta':assert commit and dirty is False,'Freeze before the single live call'
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json'
    report=dict(source=source,maximum_live_calls=1 if source=='meta' else 0,retries=0,development_fixture=True,ai_advantage_measured=False,created=False,verified=False,source_commit=commit,source_worktree_dirty=dirty,kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest())
    app=Studio(kernel,binary,out/'session',source,state=state);g=app.guest
    try:
        for c in TABLE:app.press(ord(c))
        app.press(13)
        for t in app.threads:t.join(timeout=45);assert not t.is_alive()
        assert g.request('app.key','0')=='state=review';assert not state.exists()
        plans=[e['plan'] for p in (out/'session').glob('*/provider.stdout.jsonl') for e in map(json.loads,p.read_text().splitlines()) if e.get('event')=='app_plan'];assert len(plans)==1
        s=plans[0]['spec'];report['plan']=plans[0]
        assert [f['kind'] for f in s['fields']]==['text','number','check'] and s['filter_key']==s['fields'][2]['key']
        assert 'cent' in s['fields'][1]['label'].lower(),'Currency unit should be explicit'
        g.capture(out/'review.ppm');app.press(13);app_id=s['id'];assert schema(g,app_id)==s;report['created']=True
        rows=[['Cable','1250','false'],['Adapter','750','true'],['Refund','-200','false']]
        for index,row in enumerate(rows):
            if index:app.press(110)
            for field,value in enumerate(row):g.request('app.value',f'{field}:{value.encode().hex()}')
            app.press(13)
        assert records(g,app_id)==rows
        numeric=s['fields'][1]['key'];app.press(1007);g.capture(out/'table-all.ppm')
        assert g.fields('app.totals',f'{app_id}:all')['totals']==f'{numeric}:1800'
        app.press(1004);g.capture(out/'table-unpaid.ppm')
        assert g.fields('app.totals',f'{app_id}:pending')['totals']==f'{numeric}:1050'
        assert records(g,app_id)==rows and len(app.threads)==1
        # Edit the first visible row through the same guest-owned table hit target.
        app.tap(180,254);g.request('app.value','1:31333530');app.press(13);rows[0][1]='1350'
        assert records(g,app_id)==rows and g.fields('app.totals',f'{app_id}:all')['totals']==f'{numeric}:1900'
        app.press(1007);g.capture(out/'table-updated.ppm');app.press(27);g.capture(out/'home.ppm')
        report['final_rows']=rows;report['totals']=dict(all=1900,unpaid=1150)
    except (AssertionError,ValueError,OSError) as e:report['error']=str(e) or type(e).__name__;raise
    finally:
        app.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    restored=Studio(kernel,binary,out/'restored','replay',state=state)
    try:
        assert records(restored.guest,app_id)==rows and not restored.threads
        assert restored.guest.fields('app.totals',f'{app_id}:pending')['totals']==f'{numeric}:1150'
        restored.tap(132,270);restored.press(1007);restored.guest.capture(out/'restored-table.ppm');report['verified']=True
    finally:restored.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['meta','replay'],default='replay');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
