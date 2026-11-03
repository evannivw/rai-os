#!/usr/bin/env python3
"""Interpret successive controlled events while ONE real RAI-OS guest stays alive."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import threading
import time
sys.dont_write_bytecode = True
from semantic_demo import ROOT, load, build, intent_gate
from semantic_sequence import audit as audit_sequence
from check_bundle import check as check_bundle
from check_record_index import check as check_index
FIXTURE = ROOT/'fixtures/semantic_session/v1'


def audit(folder, fixture=FIXTURE):
    """Verify decisions first, then project the actual guest trace into the existing
    two-publication auditor. No generated artifact/guest operation is substituted.
    The completion projection is checked against the real stream completion/count.
    """
    events=[json.loads(s) for s in (folder/'stdout.jsonl').read_text().splitlines()]
    inputs=[json.loads(s) for s in (folder/'inputs.jsonl').read_text().splitlines()]
    report=load(folder/'report.json');protocol=load(fixture/'protocol.json');cases=protocol['cases']
    contextual=protocol.get('event_kind')=='context_updates'
    if contextual:
        assert {load(fixture/(c+'.json'))['goal'] for c in cases}=={protocol['goal']}
        assert report['event_source']=='controlled_context_updates'
    assert len(inputs)==len(cases)+1 and inputs[-1]=={'event':'stop'}
    assert events[0]['event']=='selection_stream_started'
    assert report['source'] in ['rules','meta','replay']
    assert sum(e['event']=='guest_boot_confirmed' for e in events)==1
    assert sum(e['event']=='stream_finished' for e in events)==1
    cursor=0;ready=0;pending=None;approved=[];no_work=[];rejected=[];guest=[];last_pub=None;finished=False
    for event in events[1:]:
        kind=event['event']
        if kind=='stream_ready':
            assert pending is None and event['events_received']==cursor and event['last_request_version']==cursor
            ready+=1
        elif kind in ['stream_approved','stream_rejected']:
            assert ready==cursor+1 and pending is None and cursor<len(cases)
            name=cases[cursor];request=load(fixture/(name+'.json'));expected=load(fixture/'expected.json')[name]
            assert event['request']==request and inputs[cursor]['request']==request
            trace=[json.loads(s) for s in (folder/name/'interpret.stdout.jsonl').read_text().splitlines()]
            assert trace[0]['source']==report['source'] and trace[0]['request']==request
            if contextual:
                observation=load(fixture/'observations'/(name+'.json'))
                assert trace[0]['observation']==observation and observation['version']==request['request_version']
                assert trace[0]['trigger']=='controlled_context_update' and trace[0]['user_goal_changed'] is False
            selections=[e['selection'] for e in trace if e['event']=='semantic_selection']
            outcome=load(folder/name/'report.json');correct=False
            diagnostics=[e['diagnostic'] for e in trace if e['event']=='provider_diagnostic']
            assert len(diagnostics)==(1 if report['source']=='meta' else 0)
            if diagnostics:assert outcome['provider']==diagnostics[0]
            assert outcome['source']==report['source']
            if len(selections)==1:
                try:intent_gate(selections[0],request,expected);correct=True
                except (AssertionError,KeyError,TypeError):pass
            assert outcome['accepted']==correct
            if kind=='stream_rejected':
                assert not correct and inputs[cursor]['event']=='rejected';rejected.append(name)
            else:
                assert correct and inputs[cursor]['event']=='approved'
                assert event['selection']==inputs[cursor]['selection']==selections[0]
                pending=event['selection'];approved.append(name)
            cursor+=1
        elif kind=='stream_no_execution':
            assert pending and pending['decision'] in ['clarify','decline'] and not pending['records']
            assert event['request_version']==cursor and event['decision']==pending['decision']
            assert event['reason']==pending['reason'];no_work.append(cursor);pending=None
        elif kind=='sequence_goal':
            assert pending and pending['decision']=='select'
            assert sorted(pending['records'])==event['snapshot']['required']
            guest.append(event)
        elif kind=='batch_publication':
            assert pending and sorted(pending['records'])==event['snapshot']['required']
            check_bundle(event['payload'],event['snapshot']['required'],event['snapshot']['goal_version'])
            last_pub=event;guest.append(event);pending=None
        elif kind=='stream_finished':
            assert not pending and cursor==len(cases) and ready==cursor+1
            assert event['publications']==sum(e['event']=='batch_publication' for e in guest);finished=True
        else:
            if kind=='guest_index':check_index(event['payload'],event['record'])
            if kind=='guest_exchange' and event['operation']=='coord.goal':assert pending and pending['decision']=='select'
            if kind=='guest_exchange' and event['operation'] in ['coord.select','coord.propose','coord.result','coord.publish.prepare','coord.publish.commit']:
                assert pending and pending['decision']=='select'
            guest.append(event)
    assert finished and report['exit_code']==0
    # A failed decision remains an observed failure, not a fabricated successful
    # lifecycle. Full guest lifecycle proof requires both exact selected outputs.
    lifecycle=None
    publication_cases=protocol.get('publication_cases',['historical','latest'])
    nonexecuting_cases=protocol.get('nonexecuting_cases',['ambiguous'])
    if all(name in approved for name in publication_cases):
        assert no_work==[cases.index(name)+1 for name in nonexecuting_cases if name in approved] and last_pub
        projection=[dict(event='semantic_sequence_started',source='recorded_approved_selections',live_inference=False)]
        for event in guest:
            if event['event']=='guest_exchange' and event['operation']=='coord.stop':
                projection.append(dict(event='sequence_finished',snapshot=last_pub['snapshot'],live_inference=False))
            projection.append(event)
        lifecycle=audit_sequence(projection)
        lifecycle.pop('live_inference')
    assert report['correct_decisions']==len(approved) and report['all_decisions_correct']==(approved==cases)
    return dict(correct_decisions=len(approved),events=len(cases),all_decisions_correct=approved==cases,lifecycle=lifecycle,rejected=rejected)


def interpret(binary,kernel,source,name,output,fixture,contextual=False):
    output.mkdir();request=load(fixture/(name+'.json'));expected=load(fixture/'expected.json')[name]
    command=[str(binary),str(kernel),'--resolve' if source=='rules' else '--interpret',str(fixture/(name+'.json'))]
    if contextual:command=[str(binary),str(kernel),'--context',source,str(fixture/'observations'/(name+'.json'))]
    env=dict(os.environ)
    if source!='meta':env['MODEL_API_KEY']=''
    if source=='replay':
        plan=dict(request_goal=request['goal'],request_version=request['request_version'],catalog_version=1,permissions_version=1,decision=expected['decision'],records=expected['records'],reason='Oracle-supplied offline selection; not inference.')
        supplied=output/'supplied.json';supplied.write_text(json.dumps(plan)+'\n')
        if not contextual:command[2]='--interpret-replay'
        command.append(str(supplied))
    start=time.monotonic()
    result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=40)
    (output/'interpret.stdout.jsonl').write_text(result.stdout);(output/'interpret.stderr.jsonl').write_text(result.stderr)
    report=dict(source=source,accepted=False,exit_code=result.returncode,interpret_ms=round((time.monotonic()-start)*1000))
    message=dict(event='rejected',request=request)
    try:
        events=[json.loads(s) for s in result.stdout.splitlines()]
        diagnostic=[e['diagnostic'] for e in events if e['event']=='provider_diagnostic']
        if diagnostic:report['provider']=diagnostic[0]
        assert result.returncode==0,'Interpreter failed'
        selections=[e['selection'] for e in events if e['event']=='semantic_selection'];assert len(selections)==1
        selection=selections[0];intent_gate(selection,request,expected)
        report['accepted']=True;message=dict(event='approved',request=request,selection=selection)
    except (AssertionError,KeyError,TypeError,ValueError) as error:report['error']=str(error) or type(error).__name__
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');return message,report


def run(binary,kernel,source,output,graphics=False,fixture=FIXTURE,contextual=False):
    output.mkdir();cases=load(fixture/'protocol.json')['cases'];messages=queue.Queue();results=[];sent=0
    start=time.monotonic()
    with (output/'stdout.jsonl').open('w') as log,(output/'stderr.jsonl').open('w') as err,(output/'inputs.jsonl').open('w') as inputs:
        process=subprocess.Popen([str(binary),str(kernel),*(['--graphics'] if graphics else []),'--selection-stream'],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
        def read():
            for line in process.stdout:messages.put(line)
            messages.put(None)
        reader=threading.Thread(target=read,daemon=True);reader.start()
        try:
            while True:
                line=messages.get(timeout=45)
                if line is None:break
                log.write(line);log.flush();event=json.loads(line)
                if event['event']=='stream_ready':
                    assert event['events_received']==sent
                    if sent<len(cases):
                        name=cases[sent];message,result=interpret(binary,kernel,source,name,output/name,fixture,contextual);results.append(result);sent+=1
                    else:message={'event':'stop'}
                    text=json.dumps(message)+'\n';inputs.write(text);inputs.flush();process.stdin.write(text);process.stdin.flush()
            code=process.wait(timeout=10)
        finally:
            if process.poll() is None:process.kill();process.wait()
            reader.join(timeout=2);process.stdin.close();process.stdout.close()
    report=dict(source=source,event_source='controlled_context_updates' if contextual else 'controlled_request_updates',events_delivered=sent,exit_code=code,correct_decisions=sum(r['accepted'] for r in results),all_decisions_correct=len(results)==len(cases) and all(r['accepted'] for r in results),end_to_end_ms=round((time.monotonic()-start)*1000),graphics=graphics,presentation_pause_ms=1500 if graphics else 0)
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    checked=audit(output,fixture);(output/'audit.json').write_text(json.dumps(checked,indent=2)+'\n')
    return report,checked


def summarize(output,fixture=FIXTURE):
    manifest=load(output/'manifest.json')
    assert manifest['protocol']==load(fixture/'protocol.json')
    assert manifest['fixture_hashes']=={str(p.relative_to(fixture)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(fixture.rglob('*.json'))}
    assert manifest['policies'] in [['rules','meta'],['rules'],['meta'],['replay']]
    summary={}
    for source in manifest['policies']:
        folder=output/source;checked=audit(folder,fixture);report=load(folder/'report.json')
        calls=[load(folder/case/'report.json') for case in manifest['protocol']['cases']]
        usage=[r.get('provider',{}).get('usage') for r in calls]
        summary[source]=dict(audit=checked,end_to_end_ms=report['end_to_end_ms'],interpret_ms=[r['interpret_ms'] for r in calls],provider_calls=len(calls) if source=='meta' else 0,known_total_tokens=sum(u['total_tokens'] for u in usage if u),calls_with_usage=sum(u is not None for u in usage))
    return summary


def main(fixture=FIXTURE,contextual=False):
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['replay','rules','meta','compare'],default='replay');p.add_argument('--output',type=Path,required=True);p.add_argument('--debug',action='store_true');p.add_argument('--graphics',action='store_true');p.add_argument('--audit-only',action='store_true');a=p.parse_args()
    output=a.output.resolve()
    if a.audit_only:
        print(json.dumps(summarize(output,fixture),indent=2));return
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip())
    if a.source in ['meta','compare']:assert not dirty,'Commit frozen source before live calls'
    assert not output.exists();binary,kernel=build(a.debug,a.graphics);output.mkdir()
    sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
    policies=['rules','meta'] if a.source=='compare' else [a.source]
    manifest=dict(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=dirty,observed_utc=datetime.now(timezone.utc).isoformat(),policies=policies,protocol=load(fixture/'protocol.json'),fixture_hashes={str(p.relative_to(fixture)):sha(p) for p in sorted(fixture.rglob('*.json'))},binary_sha256=sha(binary),kernel_sha256=sha(kernel),model='muse-spark-1.3',max_tokens=1024,reasoning_effort='minimal',schema='rai_context_selection' if contextual else 'rai_selection',maximum_meta_calls=3 if 'meta' in policies else 0,retries=0,fallback=False,graphics=a.graphics)
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    for source in policies:
        report,checked=run(binary,kernel,source,output/source,a.graphics,fixture,contextual);print(json.dumps(dict(report=report,audit=checked)),flush=True)
    summary=summarize(output,fixture);(output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')


if __name__=='__main__':main()
