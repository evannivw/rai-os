#!/usr/bin/env python3
"""Real-QEMU atomic-transfer oracle; optional single actual Meta generation."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
from apps import Studio, build, host_build, records, schema, deliver
from app_storage import collect
from verify_apps import submit
from verify_app_behaviors import field
from verify_app_pages import block
from verify_app_repair import ScriptedStudio

GOAL='App: name, balance, amount, to. Transfer amount from this balance to name=to atomically. Page: sum balance and bars by name.'

def definition():
    target=field('to','text');target['required']=False
    return dict(id='locations',title='Locations',capability='records.local',fields=[field('name','text'),field('balance','number','0'),field('amount','number','0'),target,field('double','calculated',formula='$balance*2')],filter_key='',filter_label='',transfer=dict(id='send',label='Move stock',balance_field='balance',amount_field='amount',target_field='to',key_field='name'),pages=[dict(title='Distribution',blocks=[block('metric','Total','balance',aggregate='sum'),block('bars','Locations','balance','name')])])

def verify(kernel,binary,output,source='offline'):
    try:
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
        dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except (FileNotFoundError,subprocess.CalledProcessError):commit=dirty=None
    if source=='meta':assert commit and not dirty,'Freeze sources before live generation'
    output.mkdir(parents=True,exist_ok=False);state=output/'device.json';app=None
    report=dict(verified=False,source=source,source_commit=commit,source_worktree_dirty=dirty,provider_calls=0,checks=[])
    try:
        app=Studio(kernel,binary,output/'session','meta',state=state) if source=='meta' else ScriptedStudio(kernel,binary,output/'session',proposals=[definition()],state=state)
        app.repair_enabled=False;g=app.guest
        app.press(1002)
        for c in GOAL:app.press(ord(c))
        app.press(13)
        for thread in app.threads:thread.join(timeout=45);assert not thread.is_alive()
        report['provider_calls']=app.provider_calls if source=='meta' else 0
        events=[json.loads(line) for f in (output/'session').glob('*/provider.stdout.jsonl') for line in f.read_text().splitlines()]
        plans=[e['plan'] for e in events if e.get('event')=='app_plan'];assert len(plans)==1,events
        spec=plans[0]['spec'];rule=spec['transfer'];aid=spec['id'];assert rule and len(spec['pages'])==1
        assert [b['kind'] for b in spec['pages'][0]['blocks']]==['metric','bars']
        assert spec['pages'][0]['blocks'][0]['value']==rule['balance_field'] and spec['pages'][0]['blocks'][0]['aggregate']=='sum'
        def approve(s):
            before=g.request('app.catalog');assert g.request('app.key','0')=='state=review';app.press(13)
            if s.get('workflow'):app.press(13)
            for _ in s.get('behaviors',[]):app.press(13)
            assert g.request('app.key','0')=='state=move_review' and g.request('app.catalog')==before
            g.capture(output/'01-rule.ppm');app.press(13)
            for _ in s.get('pages',[]):assert g.request('app.key','0')=='state=page_review';app.press(13)
            assert schema(g,s['id'])==s
        approve(spec)
        keys=[f['key'] for f in spec['fields']];bi=keys.index(rule['balance_field']);ai=keys.index(rule['amount_field']);ti=keys.index(rule['target_field'])
        for row,(name,balance,amount,to) in enumerate([('Warehouse','12','4','Dispatch'),('Dispatch','5','1','Warehouse')]):
            if row:app.press(110)
            values={rule['key_field']:name,rule['balance_field']:balance,rule['amount_field']:amount,rule['target_field']:to}
            for i,f in enumerate(spec['fields']):
                if f['kind']!='calculated':g.request('app.value',f"{i}:{values.get(f['key'],f['default']).encode().hex()}")
            app.press(13)
        def select_source():
            g.request('app.open',aid);app.press(1003);app.tap(90,225)
        def revision():return int(g.fields('app.catalog')['revision'])
        baseline=collect(g,schema,records);base=revision();app.persist()
        (output/'showcase.json').write_bytes(state.read_bytes())
        select_source();app.tap(290,125);assert g.request('app.key','0')=='state=move_preview'
        g.capture(output/'02-preview.ppm');assert collect(g,schema,records)==baseline and revision()==base
        app.press(27);g.request('app.move.run',str(base),error='move_preview_required');assert collect(g,schema,records)==baseline
        select_source();app.tap(290,125);app.tap(240,720)
        after=records(g,aid);assert [r[bi] for r in after]==['8','9'] and revision()==base+1
        if source=='offline':assert [r[4] for r in after]==['16','18']
        app.press(1015);assert g.fields('app.page.values',f'{aid}:0')['metrics']=='17,-';g.capture(output/'03-distribution.ppm')
        report['checks'].append('Reviewed dynamic rule; preview/cancel write nothing; pointer Apply changes both balances with one revision; sum conserved at 17; derived values recomputed')
        select_source();g.request('app.value',f'{ai}:3939');app.press(13);select_source()
        before=collect(g,schema,records);base=revision();g.request('app.tap','290:125',error='move_insufficient');g.capture(output/'04-blocked.ppm')
        g.request('app.move.run',str(base),error='move_preview_required');assert collect(g,schema,records)==before and revision()==base
        # Stale approval cannot be replayed after another edit.
        select_source();g.request('app.value',f'{ai}:31');app.press(13);select_source();app.press(1016);base=revision()
        app.press(1003);app.tap(90,225);g.request('app.value',f'{ai}:32');app.press(13)
        g.request('app.move.run',str(base),error='move_preview_required');assert [r[bi] for r in records(g,aid)]==['8','9']
        report['checks'].append('Insufficient transfer rejected without writes/revision; cancelled and outdated UI approval cannot execute')
        if source=='offline':
            # Exact target lookup failures leave state intact; no index guessed by host.
            for target,error in [('Missing','move_target_missing'),('Warehouse','move_same_row')]:
                select_source();g.request('app.value',f'{ti}:{target.encode().hex()}');app.press(13);select_source();before=collect(g,schema,records);base=revision()
                g.request('app.move.preview',str(base),error=error);assert collect(g,schema,records)==before and revision()==base
            select_source();g.request('app.value',f'{ti}:{"Dispatch".encode().hex()}');app.press(13)
            bad=copy.deepcopy(spec);bad['transfer']['target_field']=rule['key_field'];p=submit(g,'Invalid transfer binding',aid);p['spec']=bad
            # Stage normally, assert the actual guest admission error at ready.
            from verify_app_behaviors import expect_error
            expect_error(lambda:deliver(g,p),'move_bad_binding');app.press(27);assert schema(g,aid)==spec
            report['checks'].append('Missing/self destination and aliased field binding rejected by actual guest')
        app.persist();final=collect(g,schema,records);report['final_state']=final;app.close();app=None
        app=Studio(kernel,binary,output/'restored','replay',state=state);assert collect(app.guest,schema,records)==final and not app.threads
        app.guest.request('app.open',aid);app.press(1015);app.guest.capture(output/'05-reboot.ppm')
        report['checks'].append('Exact rule, pages and edited records restored on fresh QEMU boot without inference')
        from demo import prepare
        seed=(output/'showcase.json').read_bytes()
        prepare(output/'rehearsal',binary,kernel,showcase=output/'showcase.json')
        assert (output/'showcase.json').read_bytes()==seed and (output/'rehearsal/device.json').read_bytes()==seed
        report['checks'].append('Saved-demo entry validates and copies the supplied showcase without modifying it')
        report['verified']=True
    except BaseException as e:report['error']=str(e) or type(e).__name__;raise
    finally:
        if app:app.close()
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
