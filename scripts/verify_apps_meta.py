#!/usr/bin/env python3
"""Two-call live development smoke: generate, use, evolve. No retries or fallback."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,CREATE,EVOLVE,records,schema,build,host_build
from verify_apps import text

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    assert not subprocess.check_output(['git','status','--porcelain'],text=True).strip(),'Freeze source before live calls'
    kernel=build();binary=host_build();out=a.output.resolve();app=Studio(kernel,binary,out,'meta');g=app.guest
    report=dict(source='meta',maximum_calls=2,retries=0,development_fixture=True,ai_advantage_measured=False,created=False,evolved=False,source_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),bridge_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
    def plan(prompt):
        for c in prompt:app.press(ord(c))
        reply=app.press(13);rid=dict(p.split('=',1) for p in reply.split('; '))['submit']
        with app.lock:g.capture(out/f'thinking-{rid}.ppm')
        for t in app.threads:t.join(timeout=45);assert not t.is_alive()
        events=[json.loads(line) for line in (out/rid/'provider.stdout.jsonl').read_text().splitlines()]
        plans=[e['plan'] for e in events if e.get('event')=='app_plan'];assert len(plans)==1,'No accepted live plan'
        p=plans[0];assert p['decision']=='install' and g.request('app.key','0')=='state=review';return p
    try:
        p=plan(CREATE);s=p['spec'];report['create_plan']=p
        assert g.fields('app.catalog')['apps']=='-'
        # Independent semantic expectations for the disclosed request, before installation.
        assert len(s['fields'])==3 and [f['kind'] for f in s['fields']]==['text','text','check']
        assert 'equipment' in (s['fields'][0]['key']+' '+s['fields'][0]['label']).lower()
        assert 'note' in (s['fields'][1]['key']+' '+s['fields'][1]['label']).lower()
        assert s['fields'][2]['default']=='false' and s['filter_key']==s['fields'][2]['key']
        assert s['capability']=='records.local';g.capture(out/'create-review.ppm');app.press(13)
        app_id=s['id'];assert schema(g,app_id)==s and records(g,app_id)==[];report['created']=True
        text(g,'Pump A');g.request('app.key','9');text(g,'Check seal');g.capture(out/'form.ppm');g.request('app.key','13')
        g.request('app.key','110');text(g,'Pump B');g.request('app.key','9');text(g,'Ready');g.request('app.key','9');g.request('app.key','32');g.request('app.key','13');g.capture(out/'records.ppm')
        before=records(g,app_id);assert before==[['Pump A','Check seal','false'],['Pump B','Ready','true']]
        g.request('app.key','1004');g.capture(out/'pending.ppm');app.press(1002);p=plan(EVOLVE);report['update_plan']=p;new=p['spec']
        assert records(g,app_id)==before and schema(g,app_id)==s
        assert new['id']==app_id and new['fields'][:3]==s['fields'] and len(new['fields'])==4
        assert new['filter_key']==s['filter_key'] and new['filter_label']==s['filter_label']
        f=new['fields'][3];assert f['kind']=='number' and f['required'] and f['default']=='1' and 'priority' in (f['key']+' '+f['label']).lower()
        g.capture(out/'update-review.ppm');app.press(13);assert records(g,app_id)==[row+['1'] for row in before]
        g.request('app.key','1003');g.request('app.key','13');g.capture(out/'evolved-record.ppm');report['evolved']=True;report['final_records']=records(g,app_id)
    except (AssertionError,OSError,ValueError) as e:
        report['error']=str(e) or type(e).__name__;raise
    finally:
        (out/'live-report.json').write_text(json.dumps(report,indent=2)+'\n');app.close();print(json.dumps(report,indent=2))
if __name__=='__main__':main()
