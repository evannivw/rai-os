#!/usr/bin/env python3
"""Eight-order process demonstration. One optional frozen Meta call; no retries."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,deliver,schema,records,states,build,host_build
from verify_apps import submit

GOAL='Repair shop: device, job, quote, paid, balance=quote-paid in cents. Initially received; start, finish, deliver if balance<=0.'

def fixture():
    def field(key,kind,default='',formula=None):
        f=dict(key=key,label=key.title(),kind=kind,required=True,default=default)
        if formula:f['formula']=formula
        return f
    def action(id,label,frm,to,guard=''):return dict(id=id,label=label,**{'from':frm},to=to,guard_field=guard,guard_op='le' if guard else 'none',guard_value=0)
    return dict(id='workshop',title='Repair workshop',capability='records.local',fields=[field('device','text'),field('job','text'),field('quote','number','0'),field('paid','number','0'),field('balance','calculated',formula='$quote-$paid')],filter_key='',filter_label='',workflow=dict(initial='received',actions=[action('start','Start repair','received','repairing'),action('finish','Finish repair','repairing','ready'),action('deliver','Deliver device','ready','delivered','balance')]))

def verify(kernel,binary,out,source='offline'):
    try:
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True,stderr=subprocess.DEVNULL).strip();dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except (FileNotFoundError,subprocess.CalledProcessError):commit=None;dirty=None
    if source=='meta':assert commit and not dirty,'Freeze sources before the one live request'
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json';exchange=out/'exchange'
    report=dict(source=source,maximum_live_calls=1 if source=='meta' else 0,retries=0,source_commit=commit,source_worktree_dirty=dirty,kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),verified=False,development_example=True,checks=[])
    app=Studio(kernel,binary,out/'session','meta' if source=='meta' else 'replay',state=state,exchange=exchange);g=app.guest
    try:
        if source=='meta':
            for c in GOAL:app.press(ord(c))
            reply=app.press(13);rid=dict(p.split('=',1) for p in reply.split('; '))['submit']
            for t in app.threads:t.join(timeout=45);assert not t.is_alive()
            plans=[e['plan'] for e in map(json.loads,(out/'session'/rid/'provider.stdout.jsonl').read_text().splitlines()) if e.get('event')=='app_plan']
            assert len(plans)==1 and plans[0]['decision']=='install';spec=plans[0]['spec'];report['plan']=plans[0]
        else:
            p=submit(g,GOAL);p['spec']=fixture();spec=p['spec'];deliver(g,p);report['plan']=p
        assert spec['capability']=='records.local' and [f['kind'] for f in spec['fields']]==['text','text','number','number','calculated']
        flow=spec['workflow'];assert len(flow['actions'])==3
        chain=[];current=flow['initial']
        for _ in range(3):
            options=[a for a in flow['actions'] if a['from']==current];assert len(options)==1
            chain.append(options[0]);current=options[0]['to']
        assert len({flow['initial'],*(a['to'] for a in chain)})==4
        assert all(a['guard_op']=='none' for a in chain[:2])
        assert chain[2]['guard_field']==spec['fields'][4]['key'] and chain[2]['guard_op'] in ['eq','le'] and chain[2]['guard_value']==0
        assert g.fields('app.catalog')['apps']=='-';g.capture(out/'01-fields.ppm');app.press(13)
        assert g.request('app.key','0')=='state=workflow_review' and g.fields('app.catalog')['apps']=='-'
        g.capture(out/'02-process-review.ppm');app.press(13);app_id=spec['id'];assert schema(g,app_id)==spec
        rows=[['Phone A','Screen','12000','2000','10000'],['Tablet B','Battery','8000','8000','0'],['Phone C','Port','5000','1000','4000'],['Laptop D','Keyboard','15000','0','15000'],['Watch E','Battery','3000','0','3000'],['Tablet F','Screen','18000','4000','14000'],['Phone G','Speaker','6000','0','6000'],['Laptop H','Hinge','9000','3000','6000']]
        stream=io.StringIO(newline='');writer=csv.writer(stream);writer.writerow([f['key'] for f in spec['fields'][:4]]);writer.writerows([r[:4] for r in rows])
        (exchange/'inbox'/f'{app_id}.csv').write_text(stream.getvalue());app.tap(315,160);assert g.request('app.key','0')=='state=import_review';app.press(13)
        assert records(g,app_id)==rows and states(g,app_id)==[flow['initial']]*8
        report['checks'].append('Generated fields and full process reviewed before install; eight imported orders start in the initial state')
        def select(row):
            g.request('app.open',app_id);app.press(1003);app.tap(150,224+50*row);app.tap(235,160);assert g.request('app.key','0')=='state=workflow'
        def run(step,error=None,base=None):
            revision=g.fields('app.catalog')['revision'] if base is None else str(base)
            if error:g.request('app.workflow.run',f"{revision}:{chain[step]['id']}",error=error)
            else:app.tap(250,258+84*flow['actions'].index(chain[step]))
        select(0);before=g.request('app.catalog');run(2,'workflow_wrong_state');assert g.request('app.catalog')==before
        old=g.fields('app.catalog')['revision'];run(0);run(0,'app_stale_revision',old);run(1)
        before=g.request('app.catalog');run(2,'workflow_guard_failed');assert g.request('app.catalog')==before and records(g,app_id)==rows
        g.capture(out/'03-delivery-blocked.ppm')
        app.press(1003);app.tap(150,224);g.request('app.value','3:3132303030');app.press(13);rows[0][3:]=['12000','0']
        app.tap(235,160);run(2);g.capture(out/'04-delivered.ppm');assert states(g,app_id)[0]==chain[2]['to']
        before=g.request('app.catalog');run(2,'workflow_wrong_state');app.press(1003);app.tap(150,224)
        g.request('app.value','3:30',error='workflow_closed');g.request('app.key','13',error='workflow_closed');assert g.request('app.catalog')==before
        g.request('app.restore.state',f'{app_id}:0:{flow["initial"]}',error='app_restore_denied')
        g.request('app.restore.begin','1',error='app_restore_denied')
        select(1);run(1,'workflow_wrong_state');run(0);run(1);run(2)
        select(2);run(0);run(1);run(2,'workflow_guard_failed')
        select(3);run(0);app.press(1003);g.capture(out/'05-eight-orders.ppm')
        expected_states=[chain[2]['to'],chain[2]['to'],chain[1]['to'],chain[0]['to']]+[flow['initial']]*4
        assert states(g,app_id)==expected_states and records(g,app_id)==rows
        app.press(1007);app.tap(400,650);g.capture(out/'06-balances.ppm');app.tap(405,160)
        exported=next((exchange/'outbox').glob('*.csv'));assert list(csv.reader(io.StringIO(exported.read_text(),newline='')))==[[f['key'] for f in spec['fields']]]+rows
        assert len(app.threads)==(1 if source=='meta' else 0)
        report['checks'].append('Out-of-order, stale, unpaid delivery, duplicate completion, terminal edits and runtime restore bypass rejected without mutation')
        report['final_rows']=rows;report['final_states']=expected_states
    except BaseException as e:report['error']=str(e) or type(e).__name__;raise
    finally:
        (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        app.close()
    restored=Studio(kernel,binary,out/'restored','replay',state=state,exchange=exchange)
    try:
        assert schema(restored.guest,app_id)==spec and records(restored.guest,app_id)==rows and states(restored.guest,app_id)==expected_states and not restored.threads
        restored.guest.request('app.open',app_id);restored.press(1003);restored.guest.capture(out/'07-restored-orders.ppm')
        restored.guest.request('app.restore.state',f'{app_id}:0:{flow["initial"]}',error='app_restore_denied')
        report['checks'].append('CSV field data exported exactly; fresh QEMU restores all process states and data without inference; restoration gate closes')
        report['verified']=True
    finally:restored.close();(out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
