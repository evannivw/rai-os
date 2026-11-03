#!/usr/bin/env python3
"""Independent composed-page checks in real QEMU; optional one live generation."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
from apps import Studio,build,host_build,deliver,records,schema
from app_storage import collect
from verify_apps import submit
from verify_app_behaviors import field,expect_error
from verify_app_repair import ScriptedStudio

GOAL='App: name, amount. Two pages: sum amount and bars by name; text "My report" and a name/amount table. No workflow.'
def block(kind,title,value='',label='',aggregate='',scope='all'):
    return dict(kind=kind,title=title,value=value,label_field=label,aggregate=aggregate,scope=scope)
def definition():
    return dict(id='visualdesk',title='Visual desk',capability='records.local',fields=[field('name','text'),field('amount','number','0'),field('total','calculated',formula='$amount*2'),field('done','check','false')],filter_key='done',filter_label='Pending',pages=[
        dict(title='Overview',blocks=[block('metric','Total','total',aggregate='sum'),block('bars','Amounts','total','name')]),
        dict(title='Report',blocks=[block('text','My report','Live data | safe ~ composition ^ no scripts'),block('table','Details','total','name')])])

def verify(kernel,binary,output,source='offline'):
    try:
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True,stderr=subprocess.DEVNULL).strip()
        dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except (FileNotFoundError,subprocess.CalledProcessError):commit=dirty=None
    if source=='meta':assert commit and not dirty,'Freeze sources before live generation'
    output.mkdir(parents=True,exist_ok=False);state=output/'device.json';app=None
    report=dict(verified=False,source=source,source_commit=commit,source_worktree_dirty=dirty,provider_calls=0,checks=[])
    try:
        app=Studio(kernel,binary,output/'session','meta',state=state) if source=='meta' else ScriptedStudio(kernel,binary,output/'session',proposals=[definition()],state=state)
        app.repair_enabled=False # One live call, no retry or fixture substitution.
        g=app.guest
        app.press(1002)
        for ch in GOAL:app.press(ord(ch))
        app.press(13)
        for thread in app.threads:thread.join(timeout=45);assert not thread.is_alive()
        report['provider_calls']=app.provider_calls if source=='meta' else 0
        events=[json.loads(line) for f in (output/'session').glob('*/provider.stdout.jsonl') for line in f.read_text().splitlines()]
        proposals=[e['plan'] for e in events if e.get('event')=='app_plan'];assert len(proposals)==1,events
        spec=proposals[0]['spec'];app_id=spec['id']
        assert len(spec.get('pages',[]))==2 and [b['kind'] for p in spec['pages'] for b in p['blocks']]==['metric','bars','text','table']
        def approve(s):
            old=g.request('app.catalog');assert g.request('app.key','0')=='state=review';app.press(13)
            for _ in s.get('workflow',{}).get('actions',[])[:1]:app.press(13)
            for _ in s.get('behaviors',[]):app.press(13)
            for i in range(len(s.get('pages',[]))):
                assert g.request('app.key','0')=='state=page_review' and g.request('app.catalog')==old
                if i==0:g.capture(output/'01-review.ppm')
                app.press(13)
            assert schema(g,s['id'])==s
        approve(spec)
        numeric=next(f['key'] for f in spec['fields'] if f['kind']=='number');label=next(f['key'] for f in spec['fields'] if f['kind']=='text')
        assert spec['pages'][0]['blocks'][0]['aggregate']=='sum'
        nums=[-5,0,10,20,7,2,3,-1];expected=[]
        for i,n in enumerate(nums):
            if i:app.press(110)
            row=[]
            for index,f in enumerate(spec['fields']):
                if f['kind']=='calculated':row.append(str(n*2));continue
                value=f'Item {i+1}' if f['key']==label else str(n) if f['key']==numeric else 'true' if f['kind']=='check' and i in [2,6] else 'false' if f['kind']=='check' else f['default']
                row.append(value);g.request('app.value',f'{index}:{value.encode().hex()}')
            app.press(13);expected.append(row)
        assert records(g,app_id)==expected
        before=collect(g,schema,records);revision=g.request('app.catalog');app.tap(400,125);assert g.request('app.key','0')=='state=pages'
        multiplier=2 if source=='offline' else 1
        metric=spec['pages'][0]['blocks'][0];assert metric['value']==('total' if source=='offline' else numeric) and metric['scope']=='all'
        assert g.fields('app.page.values',f'{app_id}:0')=={'metrics':f'{sum(nums)*multiplier},-'}
        g.capture(output/'02-overview.ppm')
        if source=='offline':
            # Independent raster check: first signed value -10, max absolute 40.
            data=(output/'02-overview.ppm').read_bytes().split(b'\n',3)[3]
            def pixel(x,y):return data[(y*480+x)*3:(y*480+x)*3+3]
            assert pixel(350,478)==bytes.fromhex('87bfff') and pixel(365,478)==bytes.fromhex('87bfff')
            assert pixel(349,478)!=bytes.fromhex('87bfff') and pixel(366,478)==bytes.fromhex('465366')
        app.tap(370,720);assert g.request('app.key','0')=='state=pages';g.capture(output/'03-report.ppm');app.tap(60,720)
        assert collect(g,schema,records)==before and g.request('app.catalog')==revision
        report['checks'].append('Four generic block kinds, two reviewed pages, signed/zero values, all eight records, real guest pixels, page navigation without data mutation')
        # Edit real input; derived values and presentation update without inference.
        app.press(1003);app.press(1005) # normalize to first row below via pointer
        app.tap(90,225)
        index=next(i for i,f in enumerate(spec['fields']) if f['key']==numeric)
        g.request('app.value',f'{index}:39');app.press(13);expected[0][index]='9'
        if source=='offline':expected[0][2]='18'
        assert records(g,app_id)==expected
        app.press(1015);assert g.fields('app.page.values',f'{app_id}:0')=={'metrics':f'{(sum(nums)+14)*multiplier},-'}
        report['checks'].append('Editing input recomputes live metrics and derived data without another model request')
        if source=='offline':
            # Cancel a visual change, then review both pages before publishing it.
            changed=copy.deepcopy(spec);changed['pages'][0]['blocks'][0]['scope']='pending';changed['pages'][0]['blocks'][1]['scope']='pending'
            old=collect(g,schema,records);p=submit(g,'Only show pending data','visualdesk');p['spec']=changed;deliver(g,p);app.press(13);assert g.request('app.key','0')=='state=page_review';app.press(27);assert collect(g,schema,records)==old
            p=submit(g,'Only show pending data','visualdesk');p['spec']=changed;deliver(g,p);approve(changed);spec=changed
            assert records(g,app_id)==expected;app.press(1015)
            assert g.fields('app.page.values',f'{app_id}:0')=={'metrics':'74,-'}
            g.capture(output/'04-filtered.ppm')
            # Host and guest independently reject references to nonexistent fields.
            bad=copy.deepcopy(spec);bad['pages'][0]['blocks'][0]['value']='unknown'
            p=submit(g,'Invalid binding test','visualdesk');p['spec']=bad
            expect_error(lambda:deliver(g,p),'page_unknown_field');app.press(27)
            assert schema(g,app_id)==spec and records(g,app_id)==expected
            # A different app reuses the same aggregation without app-specific code.
            other=dict(id='readings',title='Readings',capability='records.local',fields=[field('level','number','-20')],filter_key='',filter_label='',pages=[dict(title='Range',blocks=[block('metric','Low','level',aggregate='min'),block('metric','High','level',aggregate='max')])])
            p=submit(g,'A range report');p['spec']=other;deliver(g,p);approve(other);app.press(13)
            assert g.fields('app.page.values','readings:0')=={'metrics':'-20,-20'}
            assert records(g,app_id)==expected
            report['checks'].append('Cancel preserves definition; reviewed visual evolution retains rows; pending filter is exact; bad binding rejected; unrelated app shares min/max executor')
        final=collect(g,schema,records);report['final_state']=final;app.close();app=None
        app=Studio(kernel,binary,output/'restored','replay',state=state)
        assert collect(app.guest,schema,records)==final and not app.threads
        app.guest.request('app.open',app_id);app.press(1015);app.guest.capture(output/'05-reboot.ppm')
        report['checks'].append('Exact definitions, pages, rows and calculated values survive fresh QEMU boot without inference')
        report['verified']=True
    except BaseException as error:report['error']=str(error) or type(error).__name__;raise
    finally:
        if app:app.close()
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    return report
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
