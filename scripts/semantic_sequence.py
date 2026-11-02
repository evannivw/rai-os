#!/usr/bin/env python3
"""Replay two independently checked recorded Meta decisions in one actual guest session."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from semantic_demo import ROOT,FIXTURE,load,build,intent_gate
from check_bundle import check
from check_record_index import check as check_index
from verify_bundle import bundle_fields
DEFAULT=ROOT/'docs/evidence/2026-11-01-semantic/live'


def audit(items):
    assert items[0]['source']=='recorded_approved_selections' and not items[0]['live_inference']
    goals=[];cache={};serial=0;issued=0;current=None;selected=None;candidate=None;checked=False;last=None;publications=[];historical=False;stopped=False;finished=False
    requireds=[['brief@1','checklist@1','notes@1'],['brief@1','checklist@1','notes@2']]
    for e in items[1:]:
        kind=e['event']
        if kind=='guest_boot_confirmed':continue
        if kind=='guest_exchange':
            serial+=1;assert e['request_id']==serial and e['success'];op=e['operation'];p=e['payload'];arg=e['argument']
            if op=='coord.start':assert serial==1
            elif op=='coord.goal':
                issued+=1;s=bundle_fields(p);current=(issued,requireds[len(goals)]);assert arg==','.join(current[1]) and int(s['goal_version'])==issued
            elif op=='coord.bundle.status':
                s=bundle_fields(p);assert int(s['goal_version'])==current[0] and int(s['completed'])==len(cache)
                assert s['missing']==(','.join(k for k in current[1] if k not in cache) or '-')
            elif op=='coord.select':
                v,key=arg.split(':');assert int(v)==current[0] and key in current[1] and key not in cache
                issued+=1;selected=(issued,key);assert int(bundle_fields(p)['ticket'])==issued
            elif op=='coord.propose':assert selected and arg==f'{selected[0]}:{selected[1]}' and bundle_fields(p)['event']=='executed'
            elif op=='coord.result':assert selected;check_index(p,selected[1]);cache[selected[1]]=p
            elif op=='coord.publish.prepare':
                check(p,current[1],current[0]);issued+=1;candidate=(issued,p);checked=False
            elif op=='coord.publish.status':
                s=bundle_fields(p)
                if s['state']=='validating':assert candidate and int(s['candidate_ticket'])==candidate[0]
                else:assert last and s['availability']=='historical' and int(s['published_version'])==last[0]
            elif op=='coord.publish.commit':
                assert candidate and checked and arg==str(candidate[0]) and bundle_fields(p)['state']=='current';last=(current[0],candidate[1]);candidate=None
            elif op=='coord.publish.read':assert last and p==last[1]
            elif op=='coord.stop':assert bundle_fields(p)['state']=='stopped';stopped=True
            else:raise AssertionError(op)
        elif kind=='sequence_goal':
            assert e['index']==len(goals) and e['snapshot']['required']==current[1] and e['snapshot']['goal_version']==current[0]
            if goals:assert e['snapshot']['missing']==['notes@2'] and e['snapshot']['completed']==3
            goals.append(current[0])
        elif kind=='sequence_historical':assert last and e['published_version']==last[0] and e['target_version']==current[0] and e['unchanged'];historical=True
        elif kind=='guest_index':assert selected and e['payload']==cache[selected[1]];selected=None
        elif kind=='independent_check':assert candidate and e['candidate_ticket']==candidate[0] and e['passed'];checked=True
        elif kind=='batch_publication':assert last and e['payload']==last[1];publications.append(last[0])
        elif kind=='sequence_finished':assert e['snapshot']['completed']==4 and not e['snapshot']['missing'] and not e['live_inference'];finished=True
        else:raise AssertionError(kind)
    assert finished and stopped and historical and goals==publications and len(goals)==2 and len(cache)==4
    return dict(goals=goals,publications=publications,total_jobs=4,reused_on_update=2,new_jobs_on_update=1,previous_bytes_unchanged=True,live_inference=False)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--debug',action='store_true');p.add_argument('--graphics',action='store_true');a=p.parse_args();output=a.output.resolve();output.mkdir(exist_ok=False)
    approvals=[]
    for name in ['historical','latest']:
        r=load(FIXTURE/(name+'.json'));s=load(DEFAULT/name/'selection.json');intent_gate(s,r,load(FIXTURE/'expected.json')[name]);approvals.append(dict(request=r,selection=s))
    path=output/'approved.json';path.write_text(json.dumps(approvals,indent=2)+'\n')
    binary,kernel=build(a.debug,a.graphics)
    command=[str(binary),str(kernel),*(['--graphics'] if a.graphics else []),'--sequence-replay',str(path)]
    result=subprocess.run(command,cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=40)
    (output/'stdout.jsonl').write_text(result.stdout);(output/'stderr.jsonl').write_text(result.stderr)
    assert result.returncode==0,result.stderr
    report=audit([json.loads(s) for s in result.stdout.splitlines()]);(output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))


if __name__=='__main__':main()
