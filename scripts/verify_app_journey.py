#!/usr/bin/env python3
"""Rehearse create -> import -> evolve -> export -> reboot in QEMU."""
import argparse
import copy
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,deliver,schema,records,build,host_build
from verify_apps import submit
from verify_app_exchange import example

CREATE='Create a quote app: item, integer quantity, integer unit cents, and calculated total cents = quantity * unit cents.'

def verify(kernel,binary,out,source='offline'):
    commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    if source=='meta':assert not dirty,'Freeze before the two live calls'
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json';exchange=out/'exchange'
    report=dict(source=source,source_commit=commit,source_worktree_dirty=dirty,maximum_live_calls=2 if source=='meta' else 0,retries=0,development_example=True,verified=False,kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),plans=[])
    app=Studio(kernel,binary,out/'session','meta' if source=='meta' else 'replay',state=state,exchange=exchange);g=app.guest;app.repair_enabled=False # Preserve this verifier's original paid-call cap.
    def propose(goal,offline,target=None):
        if source=='meta':
            g.request('app.open',target);app.press(1002)
            for c in goal:app.press(ord(c))
            response=app.press(13);rid=dict(p.split('=',1) for p in response.split('; '))['submit']
            for t in app.threads:t.join(timeout=45);assert not t.is_alive()
            events=[json.loads(line) for line in (out/'session'/rid/'provider.stdout.jsonl').read_text().splitlines()]
            plans=[e['plan'] for e in events if e.get('event')=='app_plan'];assert len(plans)==1
            plan=plans[0];assert plan['decision']=='install' and g.request('app.key','0')=='state=review'
        else:
            plan=submit(g,goal,target);plan['spec']=offline;deliver(g,plan)
        report['plans'].append(dict(goal=goal,plan=plan));return plan['spec']
    try:
        spec=propose(CREATE,example());assert spec['capability']=='records.local'
        assert [f['kind'] for f in spec['fields']]==['text','number','number','calculated']
        g.capture(out/'01-create-review.ppm');app.press(13);app_id=spec['id'];app.press(27);g.capture(out/'02-launcher.ppm')
        stream=io.StringIO(newline='');writer=csv.writer(stream);writer.writerow([f['key'] for f in spec['fields'][:3]])
        rows=[['Cable','3','125','375'],['Adapter','2','80','160']]
        writer.writerows([row[:3] for row in rows]);(exchange/'inbox'/f'{app_id}.csv').write_text(stream.getvalue())
        g.request('app.open',app_id);app.tap(310,160);assert g.request('app.key','0')=='state=import_review'
        g.capture(out/'03-import-review.ppm');assert records(g,app_id)==[];app.tap(240,720);assert records(g,app_id)==rows
        total=spec['fields'][3]['key'];goal=f'Add integer cost_cents default 100 and calculated profit = ${total} - $cost_cents. Keep existing fields and records.'
        offline=copy.deepcopy(spec);offline['fields'] += [dict(key='cost_cents',label='Cost cents',kind='number',required=True,default='100'),dict(key='profit',label='Profit cents',kind='calculated',required=True,default='',formula=f'${total}-$cost_cents')]
        updated=propose(goal,offline,app_id)
        assert len(updated['fields'])==6 and updated['fields'][:4]==spec['fields']
        assert [f['kind'] for f in updated['fields'][4:]]==['number','calculated'] and updated['fields'][4]['default']=='100'
        assert records(g,app_id)==rows;g.capture(out/'04-evolve-review.ppm');app.press(13)
        evolved=[row+['100',str(int(row[3])-100)] for row in rows];assert records(g,app_id)==evolved
        app.press(1007);app.tap(400,650);g.capture(out/'05-profit-table.ppm')
        # Local edit after generation, including exact derived arithmetic.
        app.tap(150,255);g.request('app.value','4:313530');app.press(13);evolved[0][-2:]=['150','225']
        assert records(g,app_id)==evolved
        totals=dict(item.split(':') for item in g.fields('app.totals',f'{app_id}:all')['totals'].split(','));assert totals[updated['fields'][5]['key']]=='285'
        app.press(1007);app.tap(400,650);g.capture(out/'06-edited-profit.ppm');app.tap(405,160)
        files=list((exchange/'outbox').glob('*.csv'));assert len(files)==1
        assert list(csv.reader(io.StringIO(files[0].read_text(),newline='')))==[[f['key'] for f in updated['fields']]]+evolved
        g.capture(out/'07-export-receipt.ppm');assert len(app.threads)==(2 if source=='meta' else 0)
        report['final_rows']=evolved;report['total_profit_cents']=285;report['export_file']=files[0].name
    except BaseException as e:report['error']=str(e) or type(e).__name__;raise
    finally:app.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    restored=Studio(kernel,binary,out/'restored','replay',state=state,exchange=exchange)
    try:
        assert records(restored.guest,app_id)==evolved and schema(restored.guest,app_id)==updated and not restored.threads
        restored.guest.request('app.open',app_id);restored.press(1003);restored.press(13);restored.guest.capture(out/'08-restored-app.ppm');report['verified']=True
    finally:restored.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
