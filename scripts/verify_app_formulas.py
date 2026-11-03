#!/usr/bin/env python3
"""Two unrelated arithmetic apps in real QEMU. Live mode: two frozen calls, no retries."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,deliver,schema,records,build,host_build
from verify_apps import submit,stage

# Test cases only: these prompts/specs are not loaded by the application bridge.
CASES=[
 dict(goal='Create a quote app: item, integer quantity, integer unit cents, and calculated total cents = quantity * unit cents.',id='quotes',title='Quotes',keys=['item','qty','unit','total'],formula='$qty*$unit',rows=[['Cable','3','125','375'],['Adapter','2','80','160']],edit=['Cable','4','125','500']),
 dict(goal='Create a travel budget app: trip, integer budget cents, integer spent cents, calculated remaining cents = budget - spent.',id='trips',title='Travel budget',keys=['trip','budget','spent','remaining'],formula='$budget-$spent',rows=[['Lake','5000','1250','3750'],['City','800','950','-150']],edit=['Lake','6000','1250','4750'])]

def fixture(c):
    return dict(id=c['id'],title=c['title'],capability='records.local',filter_key='',filter_label='',fields=[dict(key=k,label=k.title(),kind=['text','number','number','calculated'][i],required=True,default='',**({'formula':c['formula']} if i==3 else {})) for i,k in enumerate(c['keys'])])

def fill(g,s,row):
    for i,f in enumerate(s['fields']):
        if f['kind']!='calculated':g.request('app.value',f'{i}:{row[i].encode().hex()}')

def verify(kernel,binary,out,source='offline'):
    try:
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True,stderr=subprocess.DEVNULL).strip()
        dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except (FileNotFoundError,subprocess.CalledProcessError):commit=None;dirty=None
    if source=='meta':assert commit and dirty is False,'Freeze source before two live calls'
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json'
    report=dict(source=source,maximum_live_calls=2 if source=='meta' else 0,retries=0,source_commit=commit,source_worktree_dirty=dirty,kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),development_cases=True,ai_advantage_measured=False,verified=False,apps=[])
    app=Studio(kernel,binary,out/'session','meta' if source=='meta' else 'replay',state=state);g=app.guest;app.repair_enabled=False # Preserve this verifier's original paid-call cap.
    try:
        for n,c in enumerate(CASES):
            if source=='meta':
                app.press(27);app.press(1002)
                for char in c['goal']:app.press(ord(char))
                app.press(13)
                for t in app.threads:t.join(timeout=45);assert not t.is_alive()
                assert g.request('app.key','0')=='state=review','No installable Meta proposal'
                plans=[e['plan'] for p in sorted((out/'session').glob('*/provider.stdout.jsonl')) for e in map(json.loads,p.read_text().splitlines()) if e.get('event')=='app_plan']
                assert len(plans)==n+1;s=plans[-1]['spec']
            else:
                p=submit(g,c['goal']);s=fixture(c);p['spec']=s;deliver(g,p)
            assert s['capability']=='records.local' and [f['kind'] for f in s['fields']]==['text','number','number','calculated']
            assert s['fields'][3].get('formula') and not s['filter_key']
            assert len(report['apps'])==n
            g.capture(out/f'{n}-review.ppm');app.press(13);app_id=s['id'];assert schema(g,app_id)==s
            for rowno,row in enumerate(c['rows']):
                if rowno:app.press(110)
                fill(g,s,row)
                if rowno==0:
                    g.capture(out/f'{n}-form.ppm')
                    # Tapping the result cannot redirect typing into the derived field.
                    app.tap(150,420);app.tap(150,200)
                    for _ in range(3):app.press(9)
                    app.press(ord('X'));app.press(8)
                g.request('app.value','3:393939',error='app_read_only')
                app.press(13)
            assert records(g,app_id)==c['rows'],'Independent expected arithmetic differs'
            app.press(1007);app.tap(150,255);fill(g,s,c['edit']);app.press(13)
            expected=[c['edit'],c['rows'][1]];assert records(g,app_id)==expected
            total=s['fields'][3]['key'];totals=dict(v.split(':') for v in g.fields('app.totals',f'{app_id}:all')['totals'].split(','))
            assert int(totals[total])==sum(int(row[3]) for row in expected)
            app.press(1007);app.press(1008);g.capture(out/f'{n}-table.ppm')
            report['apps'].append(dict(goal=c['goal'],spec=s,rows=expected))
        if source=='offline':
            # All-or-nothing rejection on a later existing row, not only the first.
            c=report['apps'][0];app_id=c['spec']['id'];g.request('app.open',app_id);g.request('app.key','1003');g.request('app.key','13')
            fill(g,c['spec'],['Cable','2147483647','2','']);before=g.request('app.catalog');saved=records(g,app_id)
            g.request('app.key','13',error='formula_overflow');assert records(g,app_id)==saved and g.request('app.catalog')==before
            p=submit(g,'Change calculation',app_id);p['spec']=copy.deepcopy(c['spec']);p['spec']['fields'][3]['formula']='$qty / ($unit - 80)'
            stage(g,p);g.request('app.formula',f"{p['request_id']}:total:{p['spec']['fields'][3]['formula'].encode().hex()}")
            g.request('app.ready',f"{p['request_id']}:Change calculation",error='formula_div_zero')
            assert records(g,app_id)==saved and g.request('app.catalog')==before
            for expr,error in [('$total','formula_unknown_field'),('$missing','formula_unknown_field'),('run(1)','formula_syntax')]:
                g.request('app.formula',f"{p['request_id']}:total:{expr.encode().hex()}");g.request('app.ready',f"{p['request_id']}:Invalid",error=error)
                assert records(g,app_id)==saved and g.request('app.catalog')==before
            g.request('app.key','27');g.request('app.formula',f"{p['request_id']}:total:31",error='app_stale_request')
            # Compatible update recalculates existing rows, retaining every input.
            p=submit(g,'Subtract instead',app_id);p['spec']=copy.deepcopy(c['spec']);p['spec']['fields'][3]['formula']='$qty-$unit';deliver(g,p);app.press(13)
            c['spec']=p['spec'];c['rows']=[row[:3]+[str(int(row[1])-int(row[2]))] for row in saved]
            assert records(g,app_id)==c['rows'];report['rejections_and_atomic_update']=True
        app.press(27);g.capture(out/'home.ppm');assert len(app.threads)==(2 if source=='meta' else 0)
    except BaseException as e:
        report['error']=str(e) or type(e).__name__;raise
    finally:
        app.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    restored=Studio(kernel,binary,out/'restored','replay',state=state)
    try:
        for c in report['apps']:
            assert schema(restored.guest,c['spec']['id'])==c['spec'] and records(restored.guest,c['spec']['id'])==c['rows']
        assert not restored.threads
        restored.guest.request('app.open',report['apps'][0]['spec']['id']);restored.press(1003);restored.press(13);restored.guest.capture(out/'restored.ppm')
        report['verified']=True
    finally:
        restored.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
