#!/usr/bin/env python3
"""Fixed-case semantic demonstration: independent intent gate BEFORE real guest work."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from datetime import datetime, timezone
sys.dont_write_bytecode=True
from verify_mmu import ROOT
from check_bundle import check, unique
from check_record_index import check as check_index
from verify_bundle import bundle_fields
FIXTURE=ROOT/'fixtures/semantic/v1'


def load(path): return json.loads(path.read_text(),object_pairs_hook=unique)


def intent_gate(selection,request,expected):
    """Independent exact-answer oracle for these development cases only."""
    assert set(selection)=={'request_goal','request_version','catalog_version','permissions_version','decision','records','reason'}
    assert selection['request_goal']==request['goal']
    assert type(selection['request_version']) is int and selection['request_version']==request['request_version']
    for name in ['catalog_version','permissions_version']: assert type(selection[name]) is int and selection[name]==1
    assert selection['decision']==expected['decision'],'Wrong intent decision'
    records=selection['records'];assert isinstance(records,list) and all(isinstance(k,str) for k in records)
    assert len(records)==len(set(records)) and sorted(records)==expected['records'],'Wrong exact membership'
    assert isinstance(selection['reason'],str) and 0<len(selection['reason'].encode())<=240
    assert not any(ord(c)<32 or 127<=ord(c)<=159 for c in selection['reason'])
    return records


def audit_execution(items,request,selection,expected):
    assert items[0]['event']=='semantic_execution'
    assert items[0]['request']==request and items[0]['selection']==selection
    intent_gate(selection,request,expected)
    keys=expected['records'];serial=0;issued=0;chosen=None;cache={};candidate=None;checked=False;committed=False;published=False;finished=False;stopped=False
    for e in items[1:]:
        kind=e['event']
        if kind=='guest_boot_confirmed':continue
        if kind=='guest_exchange':
            serial+=1;assert e['request_id']==serial and e['success'],e
            op=e['operation'];arg=e['argument'];payload=e['payload']
            if op=='coord.start':assert serial==1 and arg=='index'
            elif op=='coord.goal':
                issued+=1;assert issued==1 and arg==','.join(keys);s=bundle_fields(payload)
                assert s['goal_version']=='1' and s['required']==arg and s['missing']==arg and s['completed']=='0'
            elif op=='coord.bundle.status':
                s=bundle_fields(payload);assert s['goal_version']=='1' and s['required']==','.join(keys)
                assert s['missing']==(','.join(k for k in keys if k not in cache) or '-') and int(s['completed'])==len(cache)
            elif op=='coord.select':
                version,key=arg.split(':');assert version=='1' and key in keys and key not in cache
                issued+=1;chosen=(issued,key);s=bundle_fields(payload)
                assert s['record']==key and int(s['ticket'])==issued and s['state']=='awaiting_proposal'
            elif op=='coord.propose':
                assert chosen and arg==f'{chosen[0]}:{chosen[1]}'
                s=bundle_fields(payload);assert s['state']=='completed' and s['event']=='executed' and int(s['completed'])==len(cache)+1
            elif op=='coord.result':
                assert chosen;check_index(payload,chosen[1]);cache[chosen[1]]=payload
            elif op=='coord.publish.prepare':
                assert sorted(cache)==keys;check(payload,keys,1);issued+=1;candidate=(issued,payload)
            elif op=='coord.publish.status':
                s=bundle_fields(payload);assert candidate and int(s['candidate_ticket'])==candidate[0] and s['state']=='validating'
            elif op=='coord.publish.commit':
                assert candidate and checked and arg==str(candidate[0]);s=bundle_fields(payload)
                assert s['state']=='current' and s['published_version']=='1';committed=True
            elif op=='coord.publish.read':assert committed and payload==candidate[1]
            elif op=='coord.stop':assert bundle_fields(payload)['state']=='stopped';stopped=True
            else:raise AssertionError('Unexpected operation '+op)
        elif kind=='guest_index':
            assert chosen and e['record']==chosen[1] and e['ticket']==chosen[0] and e['payload']==cache[chosen[1]];chosen=None
        elif kind=='independent_check':
            assert candidate and e['candidate_ticket']==candidate[0] and e['goal_version']==1 and e['passed'];checked=True
        elif kind=='batch_publication':assert committed and e['payload']==candidate[1];check(e['payload'],keys,1);published=True
        elif kind=='semantic_finished':assert published and e['snapshot']['required']==keys and not e['snapshot']['missing'];finished=True
        elif kind=='semantic_no_execution':
            assert expected['decision'] in ['clarify','decline'] and e['decision']==expected['decision'];assert len(items)==2;return
        else:raise AssertionError('Unexpected event '+kind)
    assert finished and stopped and len(cache)==len(keys)


def build(debug=False,graphics=False):
    profile='debug' if debug else 'release';flags=[] if debug else ['--release']
    directory='target/display' if graphics else 'target'
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags,*(['--features','guest-display','--target-dir',directory] if graphics else [])],cwd=ROOT,check=True)
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    return ROOT/f'target/host-meta/{host}/{profile}/rai-meta',ROOT/f'{directory}/aarch64-unknown-none-softfloat/{profile}/rai-os'


def run_case(binary,kernel,case,source,out,graphics=False):
    out.mkdir(exist_ok=False)
    request=load(FIXTURE/(case+'.json'));expected=load(FIXTURE/'expected.json')[case]
    env=dict(os.environ)
    if source=='replay':env['MODEL_API_KEY']=''
    args=[str(binary),str(kernel),'--interpret',str(FIXTURE/(case+'.json'))]
    if source=='replay':
        supplied=dict(request_goal=request['goal'],request_version=request['request_version'],catalog_version=1,permissions_version=1,decision=expected['decision'],records=expected['records'],reason='Oracle-supplied offline selection; not model inference.')
        (out/'supplied.json').write_text(json.dumps(supplied)+'\n')
        args[2]='--interpret-replay';args.append(str(out/'supplied.json'))
    started=time.monotonic();r=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,text=True,timeout=40)
    (out/'interpret.stdout.jsonl').write_text(r.stdout);(out/'interpret.stderr.jsonl').write_text(r.stderr)
    report=dict(case=case,source=source,interpret_exit=r.returncode,accepted=False,executed=False,pass_case=False)
    try:
        items=[json.loads(s,object_pairs_hook=unique) for s in r.stdout.splitlines()]
        reports=[e for e in items if e['event']=='provider_diagnostic']
        if reports:report['provider']=reports[0]['diagnostic']
        assert r.returncode==0,'Interpretation failed'
        assert len(reports)==(1 if source=='meta' else 0)
        selections=[e for e in items if e['event']=='semantic_selection'];assert len(selections)==1
        selection=selections[0]['selection']
        intent_gate(selection,request,expected);report['accepted']=True
        path=out/'selection.json';path.write_text(json.dumps(selection,indent=2)+'\n')
        # No second model request: the independently accepted requirement goes
        # directly to deterministic admission/execution through the existing batch path.
        e=subprocess.run([str(binary),str(kernel),*(['--graphics'] if graphics else []),'--execute-selection',str(FIXTURE/(case+'.json')),str(path)],cwd=ROOT,env=dict(env,MODEL_API_KEY=''),capture_output=True,text=True,timeout=30)
        (out/'execute.stdout.jsonl').write_text(e.stdout);(out/'execute.stderr.jsonl').write_text(e.stderr)
        report['execute_exit']=e.returncode
        report['execution_attempted']=selection['decision']=='select'
        assert e.returncode==0,'Execution failed'
        execution=[json.loads(s,object_pairs_hook=unique) for s in e.stdout.splitlines()]
        audit_execution(execution,request,selection,expected)
        report['executed']=selection['decision']=='select';report['decision']=selection['decision'];report['records']=selection['records'];report['pass_case']=True
    except (AssertionError,ValueError,KeyError,TypeError) as error: report['error']=str(error) or type(error).__name__
    report['end_to_end_ms']=round((time.monotonic()-started)*1000)
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['replay','meta'],default='replay');p.add_argument('--case',choices=['all',*load(FIXTURE/'expected.json')],default='historical');p.add_argument('--output',type=Path,required=True);p.add_argument('--debug',action='store_true');p.add_argument('--graphics',action='store_true');a=p.parse_args()
    output=a.output.resolve();output.mkdir(exist_ok=False)
    binary,kernel=build(a.debug,a.graphics)
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    cases=list(load(FIXTURE/'expected.json')) if a.case=='all' else [a.case]
    manifest=dict(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()),observed_utc=datetime.now(timezone.utc).isoformat(),source=a.source,cases=cases,binary_sha256=sha(binary),kernel_sha256=sha(kernel),fixture_hashes={p.name:sha(p) for p in sorted(FIXTURE.glob('*.json'))},model='muse-spark-1.3',max_tokens=1024,reasoning_effort='minimal',response_schema='rai_selection',maximum_calls=len(cases) if a.source=='meta' else 0,retries=0,fallback=False,graphics=a.graphics)
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    reports=[]
    for case in cases:
        report=run_case(binary,kernel,case,a.source,output/case,a.graphics);reports.append(report);print(json.dumps(report),flush=True)
    (output/'summary.json').write_text(json.dumps(reports,indent=2)+'\n')
    if not all(r['pass_case'] for r in reports):raise SystemExit(1)


if __name__=='__main__':main()
