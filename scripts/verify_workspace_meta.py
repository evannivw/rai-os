#!/usr/bin/env python3
"""One live Meta file-plan smoke test on disclosed development data; no retries.
Checks the returned action set independently before confirming any guest writes.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from workspace import Workspace,EXAMPLE,SEED,build,host_build

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    assert not subprocess.check_output(['git','status','--porcelain'],text=True).strip(),'Freeze source before live calls'
    kernel=build();binary=host_build();out=a.output.resolve();app=Workspace(kernel,binary,out,'meta')
    report=dict(source='meta',maximum_calls=1,retries=0,development_fixture=True,ai_advantage_measured=False,applied=False)
    report['source_commit']=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
    try:
        app.guest.capture(out/'compose.ppm')
        for c in EXAMPLE:app.press(ord(c))
        app.press(13)
        with app.lock:app.guest.capture(out/'thinking.ppm')
        for t in app.threads:t.join(timeout=45);assert not t.is_alive()
        traces=list(out.glob('*/provider.stdout.jsonl'));assert len(traces)==1
        events=[json.loads(line) for line in traces[0].read_text().splitlines()]
        plans=[e['plan'] for e in events if e.get('event')=='file_plan']
        with app.lock:
            g=app.guest;before=g.fields('fs.list');assert before['generation']=='5'
            for path,data in SEED.items():assert g.read(path)==data
            if len(plans)!=1:raise AssertionError('No valid live plan')
            plan=plans[0];report['plan']=plan
            assert plan['decision']=='plan' and len(plan['actions'])==2
            assert {a['source'] for a in plan['actions']}=={'/inbox/aurora.txt','/inbox/field.txt'}
            assert all(a['kind']=='copy' and a['destination'].startswith('/work/') for a in plan['actions'])
            g.request('ui.key','0');g.capture(out/'review.ppm')
            assert g.request('ui.key','13')=='state=done'
            for path,data in SEED.items():assert g.read(path)==data
            for action in plan['actions']:assert g.read(action['destination'])==SEED[action['source']]
            assert g.fields('fs.list')['generation']=='6'
            g.request('ui.key','9');g.capture(out/'done.ppm');report['applied']=True
    except (AssertionError,OSError,ValueError) as e:
        report['error']=str(e) or type(e).__name__;raise
    finally:
        (out/'live-report.json').write_text(json.dumps(report,indent=2)+'\n');app.close();print(json.dumps(report,indent=2))
if __name__=='__main__':main()
