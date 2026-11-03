#!/usr/bin/env python3
"""Independent all-row behavior checks in real QEMU; optional bounded live goal."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
from apps import Studio, build, deliver, host_build, records, schema
from app_storage import collect
from verify_apps import submit
from verify_app_behaviors import field, step, expect_error
from verify_app_repair import ScriptedStudio

GOAL = 'App: name, stock, moved. Button Move all: for every row, if stock>=2, stock=stock-2 and moved=moved+2. Defaults 0.'

def example():
    return dict(id='batchdesk', title='Batch desk', capability='records.local',
        fields=[field('name','text'),field('stock','number','0'),field('moved','number','0'),
                field('flag','check','false'),field('status','text','ready'),field('total','calculated',formula='$stock+$moved')],
        filter_key='flag',filter_label='Pending',behaviors=[
            dict(id='move',label='Move all',event='press',scope='all',steps=[
                step('if','stock','2','ge',3),step('calculate','stock','$stock-2'),
                step('calculate','moved','$moved+2'),step('set','flag','true')]),
            dict(id='reject',label='Check all',event='press',scope='all',steps=[
                step('set','status','staged'),step('require','name','Blocked','ne')])])

def verify(kernel,binary,output,source='offline'):
    try:
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True,stderr=subprocess.DEVNULL).strip()
        dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except (FileNotFoundError,subprocess.CalledProcessError):commit=dirty=None
    if source=='meta':assert commit and not dirty,'Freeze sources before the bounded live goal'
    output.mkdir(parents=True,exist_ok=False)
    report=dict(verified=False,source=source,source_commit=commit,source_worktree_dirty=dirty,
                maximum_live_calls=6 if source=='meta' else 0,generation_calls=0,checks=[],
                kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest())
    checkpoint=output/'device.json';app=None
    try:
        app=Studio(kernel,binary,output/'session','meta',state=checkpoint) if source=='meta' else ScriptedStudio(kernel,binary,output/'session',state=checkpoint,proposals=[example()])
        g=app.guest
        for char in GOAL:app.press(ord(char))
        reply=dict(p.split('=',1) for p in app.press(13).split('; '))
        for thread in app.threads:thread.join(timeout=115);assert not thread.is_alive()
        folder=app.output/reply['submit'];events=[json.loads(line) for line in (folder/'provider.stdout.jsonl').read_text().splitlines()]
        report['generation_calls']=app.provider_calls if source=='meta' else 0
        report['loop']=json.loads((folder/'loop.json').read_text());report['events']=events
        accepted=[e['plan'] for e in events if e.get('event')=='app_plan']
        assert len(accepted)==1 and accepted[0]['decision']=='install',report['loop']
        spec=accepted[0]['spec'];app_id=spec['id']
        assert [f['key'] for f in spec['fields']][:3]==['name','stock','moved']
        assert [f['kind'] for f in spec['fields']][:3]==['text','number','number']
        assert len(spec['fields'])==(3 if source=='meta' else 6)
        assert spec['behaviors'][0]['scope']=='all' and spec['capability']=='records.local'
        assert not spec.get('workflow')
        def approve(candidate):
            before=g.request('app.catalog');assert g.request('app.key','0')=='state=review'
            app.press(13)
            for _ in candidate['behaviors']:
                assert g.request('app.key','0')=='state=behavior_review' and g.request('app.catalog')==before
                g.capture(output/'01-scope-review.ppm');app.press(13)
            assert schema(g,candidate['id'])==candidate
        approve(spec)
        originals=[]
        for i,stock in enumerate([4,0,7,1,5,2,8,3]):
            row=['Blocked' if i==7 else f'Item {i+1}',str(stock),str(i)]
            if source=='offline':row+=['true' if i==4 else 'false','ready',str(stock+i)]
            if i:app.press(110)
            for index,f in enumerate(spec['fields']):
                if f['kind']!='calculated':g.request('app.value',f'{index}:{row[index].encode().hex()}')
            app.press(13);originals.append(row)
        assert records(g,app_id)==originals
        def select():
            g.request('app.open',app_id);app.press(1003);app.tap(150,224);app.press(1014)
        def preview():app.tap(250,265);assert g.request('app.key','0')=='state=behavior_preview'
        select();before=collect(g,schema,records);rev=int(g.fields('app.catalog')['revision'])
        expect_error(lambda:g.request('app.behavior.run',f"{rev}:{spec['behaviors'][0]['id']}"),'behavior_preview_required')
        preview();assert collect(g,schema,records)==before
        g.capture(output/'02-batch-preview-first.ppm')
        for _ in range(7):app.tap(405,630)
        g.capture(output/'03-batch-preview-last.ppm')
        assert collect(g,schema,records)==before and int(g.fields('app.catalog')['revision'])==rev
        # Leaving the preview invalidates its authorization.
        app.press(27)
        expect_error(lambda:g.request('app.behavior.run',f"{rev}:{spec['behaviors'][0]['id']}"),'behavior_preview_required')
        select();preview();app.tap(240,720)
        expected=copy.deepcopy(originals)
        for row in expected:
            stock=int(row[1])
            if stock>=2:
                row[1]=str(stock-2);row[2]=str(int(row[2])+2)
                if source=='offline':row[3]='true'
        assert records(g,app_id)==expected and int(g.fields('app.catalog')['revision'])==rev+1
        app.press(1007);g.capture(output/'04-applied-table.ppm')
        report['checks'].append('Eight rows; preview/cancel changes nothing; native approval applies independent branch oracle atomically with one revision; no further inference')
        if source=='offline':
            # A filter may hide records, but all scope is explicit and includes them.
            select();app.press(1003);app.press(1004);app.press(1014)
            preview();app.tap(240,720)
            for row in expected:
                if int(row[1])>=2:row[1]=str(int(row[1])-2);row[2]=str(int(row[2])+2);row[3]='true'
            assert records(g,app_id)==expected
            select();before=collect(g,schema,records);revision=g.request('app.catalog')
            expect_error(lambda:app.tap(250,359),'behavior_condition_failed')
            assert g.fields('app.behavior.status')==dict(step='2',error='behavior_condition_failed',row='8')
            assert collect(g,schema,records)==before and g.request('app.catalog')==revision
            g.capture(output/'05-last-row-denied.ppm')
            # Reviewed changes replace program semantics, preserving all saved data.
            changed=copy.deepcopy(spec);changed['behaviors'][0]['steps']=[step('calculate','moved','$moved+10')]
            p=submit(g,'Change the whole-table action',app_id);p['spec']=changed;deliver(g,p);approve(changed)
            assert records(g,app_id)==expected;spec=changed
            select();preview();base=int(g.fields('app.catalog')['revision'])
            # Trusted UART diagnostics mutate data through the ordinary save path;
            # they must invalidate the previous UI preview.
            g.request('app.open',app_id);g.request('app.key','1003');g.request('app.key','13');g.request('app.value','0:4368616e676564');g.request('app.key','13')
            expected[0][0]='Changed'
            select()
            expect_error(lambda:g.request('app.behavior.preview',f"{base}:{spec['behaviors'][0]['id']}"),'app_stale_revision')
            preview();app.tap(240,720)
            for row in expected:row[2]=str(int(row[2])+10);row[5]=str(int(row[1])+int(row[2]))
            assert records(g,app_id)==expected
            # Different field names and text/check operations use the same executor.
            other=dict(id='reviewqueue',title='Review queue',capability='records.local',filter_key='',filter_label='',
                fields=[field('title','text'),field('approved','check','false')],behaviors=[dict(id='approve',label='Approve all',event='press',scope='all',steps=[step('set','approved','true')])])
            p=submit(g,'A different all-row tool');p['spec']=other;deliver(g,p);approve(other)
            for i in range(2):
                if i:app.press(110)
                g.request('app.value',f'0:{("Doc "+str(i)).encode().hex()}');app.press(13)
            g.request('app.open',other['id']);app.press(1003);app.press(13);app.press(1014);app.tap(250,265);app.tap(240,720)
            assert records(g,other['id'])==[['Doc 0','true'],['Doc 1','true']]
            assert records(g,app_id)==expected
            report['checks'].append('Hidden rows included; last-row failure rolls back entire table with row/step diagnostic; reviewed program evolution retains data; stale preview rejected; unrelated app uses same executor')
        if source=='meta':
            # A second real prompt changes behavior over the SAME saved records.
            # Each goal has its own three-attempt bound; no fallback fixtures.
            goal='Change Move all: for each row, if stock>=1, stock=stock-1 and moved=moved+1. Keep fields and data.'
            g.request('app.open',app_id);app.press(1002)
            for char in goal:app.press(ord(char))
            reply=dict(p.split('=',1) for p in app.press(13).split('; '))
            for thread in app.threads:thread.join(timeout=115);assert not thread.is_alive()
            folder=app.output/reply['submit']
            events=[json.loads(line) for line in (folder/'provider.stdout.jsonl').read_text().splitlines()]
            report['generation_calls']=app.provider_calls
            report['evolution_loop']=json.loads((folder/'loop.json').read_text());report['evolution_events']=events
            accepted=[e['plan'] for e in events if e.get('event')=='app_plan']
            assert len(accepted)==1 and accepted[0]['decision']=='install',report['evolution_loop']
            changed=accepted[0]['spec']
            assert changed['id']==app_id and len(changed['fields'])==3 and changed['behaviors'][0]['scope']=='all'
            assert records(g,app_id)==expected
            approve(changed);spec=changed
            assert records(g,app_id)==expected
            select();preview();g.capture(output/'05-live-evolved-preview.ppm')
            revision=int(g.fields('app.catalog')['revision']);app.tap(240,720)
            for row in expected:
                if int(row[1])>=1:row[1]=str(int(row[1])-1);row[2]=str(int(row[2])+1)
            assert records(g,app_id)==expected and int(g.fields('app.catalog')['revision'])==revision+1
            report['checks'].append('Second live prompt changes the existing whole-table program; approval retains all rows; new independent oracle matches every record without hardcoded app logic')
        final=collect(g,schema,records);report['final_state']=final
        app.close();app=None
        app=Studio(kernel,binary,output/'restored','replay',state=checkpoint)
        assert collect(app.guest,schema,records)==final and not app.threads
        app.guest.request('app.open',app_id);app.press(1007);app.guest.capture(output/'06-restored.ppm')
        report['checks'].append('Exact schemas, scopes, all rows and computed values recover after reboot without inference')
        report['verified']=True
    except BaseException as error:
        report['error']=str(error) or type(error).__name__
        raise
    finally:
        if app is not None:
            if source=='meta' and app.source=='meta':report['generation_calls']=app.provider_calls
            app.close()
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
