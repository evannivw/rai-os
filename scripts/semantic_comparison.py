#!/usr/bin/env python3
"""Frozen development comparison; identical admission, execution and independent checks."""
import argparse
import hashlib
import json
import statistics
import subprocess
import sys
from datetime import datetime,timezone
from pathlib import Path
sys.dont_write_bytecode=True
from semantic_demo import ROOT,load,build,run_case,intent_gate,audit_execution
FIXTURE=ROOT/'fixtures/semantic_comparison/v1'


def audit_trial(folder,case,policy):
    report=load(folder/'report.json');request=load(FIXTURE/(case+'.json'));expected=load(FIXTURE/'expected.json')[case]
    assert report['case']==case and report['source']==policy
    events=[json.loads(s) for s in (folder/'interpret.stdout.jsonl').read_text().splitlines()]
    assert events[0]['source']==policy and events[0]['request']==request
    diagnostics=[e for e in events if e['event']=='provider_diagnostic']
    if policy=='rules':assert not diagnostics
    else:assert len(diagnostics)<=1
    if diagnostics:assert report['provider']==diagnostics[0]['diagnostic']
    selections=[e['selection'] for e in events if e['event']=='semantic_selection']
    accepted=False
    if len(selections)==1:
        try:intent_gate(selections[0],request,expected);accepted=True
        except (AssertionError,KeyError,TypeError):pass
    assert report['accepted']==accepted
    execution=folder/'execute.stdout.jsonl'
    if not accepted:
        assert not report['pass_case'] and not execution.exists()
    else:
        assert report['interpret_exit']==0 and load(folder/'selection.json')==selections[0]
        items=[json.loads(s) for s in execution.read_text().splitlines()]
        passed=report['execute_exit']==0
        if passed:
            try:audit_execution(items,request,selections[0],expected)
            except (AssertionError,KeyError,ValueError):passed=False
        assert report['pass_case']==passed
        if passed:assert report['executed']==(selections[0]['decision']=='select')
    return report


def schedule_for(protocol):
    schedule=[]
    for rep in range(protocol['repetitions']):
        for i,case in enumerate(protocol['cases']):
            policies=['rules','meta'] if (rep+i)%2==0 else ['meta','rules']
            for policy in policies:schedule.append(dict(case=case,policy=policy,repetition=rep+1,directory=f'{rep+1}-{case}-{policy}'))
    return schedule


def summarize(output):
    manifest=load(output/'manifest.json');rows=[]
    protocol=load(FIXTURE/'protocol.json')
    assert manifest['protocol']==protocol and manifest['cases']==protocol['cases']
    assert manifest['schedule']==schedule_for(protocol),'Missing, reordered or duplicated trials'
    hashes={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(FIXTURE.glob('*.json'))}
    assert manifest['fixture_hashes']==hashes,'Fixture changed since measurement'

    for entry in manifest['schedule']:
        report=audit_trial(output/entry['directory'],entry['case'],entry['policy']);rows.append(report|dict(repetition=entry['repetition']))
    result={'development_only':True,'policies':{},'cases':{}}
    for policy in ['rules','meta']:
        trials=[r for r in rows if r['source']==policy]
        usage=[r.get('provider',{}).get('usage') for r in trials]
        result['policies'][policy]=dict(trials=len(trials),passed=sum(r['pass_case'] for r in trials),executed=sum(r['executed'] for r in trials),known_token_total=sum(u['total_tokens'] for u in usage if u),calls_with_usage=sum(u is not None for u in usage) if policy=='meta' else 0,failures=[dict(case=r['case'],repetition=r['repetition'],error=r.get('error')) for r in trials if not r['pass_case']])
    for case in manifest['cases']:
        result['cases'][case]={}
        for policy in ['rules','meta']:
            trials=[r for r in rows if r['case']==case and r['source']==policy]
            result['cases'][case][policy]=dict(passed=sum(r['pass_case'] for r in trials),trials=len(trials),wall_ms=[r['end_to_end_ms'] for r in trials],median_ms=statistics.median(r['end_to_end_ms'] for r in trials))
    return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--audit-only',action='store_true');a=p.parse_args();output=a.output.resolve()
    if a.audit_only:print(json.dumps(summarize(output),indent=2));return
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip(),'Commit source before comparison'
    protocol=load(FIXTURE/'protocol.json');assert protocol['repetitions']==2 and protocol['maximum_meta_calls']==24
    output.mkdir(exist_ok=False);binary,kernel=build()
    schedule=schedule_for(protocol)
    assert len(schedule)==48 and sum(e['policy']=='meta' for e in schedule)==24
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    manifest=dict(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),observed_utc=datetime.now(timezone.utc).isoformat(),cases=protocol['cases'],schedule=schedule,protocol=protocol,binary_sha256=sha(binary),kernel_sha256=sha(kernel),fixture_hashes={p.name:sha(p) for p in sorted(FIXTURE.glob('*.json'))},rust=subprocess.check_output(['rustc','-V'],text=True).strip(),qemu=subprocess.check_output(['qemu-system-aarch64','--version'],text=True).splitlines()[0])
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    # Every failed outcome remains in its folder; no retry/fallback or early
    # stopping because a measured policy is doing badly.
    for entry in schedule:
        report=run_case(binary,kernel,entry['case'],entry['policy'],output/entry['directory'],fixture=FIXTURE)
        print(json.dumps(dict(case=entry['case'],policy=entry['policy'],repetition=entry['repetition'],passed=report['pass_case'],wall_ms=report['end_to_end_ms'])),flush=True)
    summary=summarize(output);(output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary['policies'],indent=2))


if __name__=='__main__':main()
