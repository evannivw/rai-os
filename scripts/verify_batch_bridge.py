#!/usr/bin/env python3
"""Offline bounded batch tests in real QEMU; independent exact artifact/trace checks."""
import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT
from check_bundle import check
from check_record_index import check as check_index
from verify_bundle import bundle_fields, build


def audit(items, scenario, source):
    """Reconstruct authority, work and publication from wire observations, not summaries."""
    assert items[0]['event']=='batch_started' and items[0]['source']==source
    assert items[0]['max_operations']==4 and items[0]['max_model_calls']==8
    assert items[0]['event_source']=='scripted_required_sets'
    assert items[0]['model']==('muse-spark-1.3' if source=='meta' else None)
    serial=issued=cursor=calls=0
    current=None;due=None;pending=None;plan=[];plan_version=None;selected=None;stepped=None
    candidate=None;checked=False;published=None;read=None;completed=None;finished=False;stopped=False
    cache={};jobs=[];publications=[];discards=[];diagnostics=[]
    def snapshot():
        return dict(goal_version=current[0],required=current[1],missing=[k for k in current[1] if k not in cache],completed=len(cache))
    def set_status(text):
        s=bundle_fields(text);e=snapshot()
        assert int(s['goal_version'])==e['goal_version'] and s['required']==','.join(e['required'])
        assert s['missing']==(','.join(e['missing']) or '-') and int(s['completed'])==len(cache)
        assert int(s['ready_records'])==len(e['required'])-len(e['missing'])
    for e in items[1:]:
        kind=e['event']
        if kind=='guest_boot_confirmed': continue
        if kind=='provider_diagnostic':
            assert source=='meta' and pending and e['state_version']==pending['goal_version'];diagnostics.append(e)
        elif kind=='set_due':
            want=scenario['events'][cursor];assert e['index']==cursor and e['required']==sorted(want['required'])
            assert e['scheduled_ms']==want['at_ms'] and e['schedule_elapsed_ms']>=want['at_ms'];cursor+=1;due=e['required']
        elif kind=='batch_requested':
            assert pending is None and not plan and snapshot()['missing'];calls+=1
            assert calls<=8 and e['call_id']==calls and e['snapshot']['index_set']==snapshot()
            assert e['snapshot']['state_version']==current[0]
            assert [r[0] for r in e['snapshot']['authorized_records']]==snapshot()['missing'];pending=snapshot()
        elif kind=='batch_response':
            assert pending and e['source']==source and e['call_id']==calls and e['snapshot_goal']==pending['goal_version']
        elif kind=='batch_validated':
            p=e['plan'];assert pending and e['call_id']==calls and p['state_version']==current[0]==pending['goal_version']
            assert p['decision']=='execute' and 1<=len(p['operations'])<=4
            assert all(s['operation']=='record.index' for s in p['operations'])
            plan=[s['record'] for s in p['operations']]
            assert len(set(plan))==len(plan) and sorted(plan)==pending['missing']
            plan_version=p['state_version'];pending=None
        elif kind=='batch_discarded':
            assert e['state_version']!=current[0];discards.append(e['stage'])
            if e['stage']=='response': assert pending and pending['goal_version']==e['state_version'];pending=None
            elif e['stage']=='execution': assert plan_version==e['state_version'] and len(plan)==e['remaining'];plan=[]
            elif e['stage']=='publication': assert checked;checked=False
            else: raise AssertionError('Unknown discard')
        elif kind=='batch_step':
            assert plan and plan_version==e['state_version']==current[0]
            stepped=plan.pop(0);assert e['record']==stepped and e['remaining']==len(plan)
        elif kind=='guest_exchange':
            serial+=1;assert e['request_id']==serial
            op=e['operation'];arg=e['argument'];payload=e['payload'];assert e['success'],e
            if op=='coord.start': assert serial==1 and arg=='index'
            elif op=='coord.goal':
                assert due and arg==','.join(due);issued+=1;current=(issued,due);due=None;candidate=None;set_status(payload)
            elif op=='coord.bundle.status': set_status(payload)
            elif op=='coord.select':
                assert stepped in snapshot()['missing'] and arg==f'{current[0]}:{stepped}' and plan_version==current[0]
                issued+=1;selected=(issued,stepped);s=bundle_fields(payload)
                assert int(s['ticket'])==issued and s['state']=='awaiting_proposal' and s['record']==stepped
                assert int(s['completed'])==len(cache) and int(s['b_calls'])==0
            elif op=='coord.propose':
                assert selected and arg==f'{selected[0]}:{selected[1]}';s=bundle_fields(payload)
                assert s['state']=='completed' and s['event']=='executed' and int(s['completed'])==len(cache)+1
                completed=selected[1]
            elif op=='coord.result':
                assert selected and completed==selected[1] and completed not in cache
                check_index(payload,completed);cache[completed]=payload;jobs.append(completed)
            elif op=='coord.publish.prepare':
                assert not snapshot()['missing'];check(payload,current[1],current[0]);issued+=1
                candidate=(issued,payload,current[0]);checked=False
            elif op=='coord.publish.status':
                s=bundle_fields(payload);assert candidate and int(s['candidate_ticket'])==candidate[0]
                assert int(s['target_version'])==current[0] and s['state']=='validating'
                assert int(s['published_version'])==(published[2] if published else 0)
            elif op=='coord.publish.commit':
                assert candidate and checked and arg==str(candidate[0]) and candidate[2]==current[0]
                s=bundle_fields(payload);assert s['state']=='current' and s['availability']=='current' and int(s['published_version'])==current[0]
                published=candidate;candidate=None;checked=False
            elif op=='coord.publish.read': assert published and payload==published[1];read=payload
            elif op=='coord.stop': assert bundle_fields(payload)['state']=='stopped';stopped=True
            else: raise AssertionError('Unexpected operation '+op)
        elif kind=='guest_index':
            assert selected and e['ticket']==selected[0] and e['record']==selected[1]
            assert e['payload']==cache[selected[1]] and e['origin']=='executed';selected=None;stepped=None;completed=None
        elif kind=='independent_check':
            assert candidate and e['candidate_ticket']==candidate[0] and e['goal_version']==candidate[2] and e['passed'];checked=True
        elif kind=='batch_publication':
            assert published and read==published[1]==e['payload'] and e['snapshot']==snapshot()
            check(e['payload'],current[1],current[0]);publications.append(current[0]);read=None
        elif kind=='batch_finished':
            assert not snapshot()['missing'] and e['snapshot']==snapshot() and e['model_calls']==calls
            assert e['status']=='verified_current_publication' and publications[-1]==current[0] and cursor==len(scenario['events']);finished=True
        else: raise AssertionError('Unexpected event '+kind)
    assert finished and stopped and not plan and pending is None and candidate is None
    if source=='meta': assert len(diagnostics)==calls and all(d['outcome']=='decoded' for d in diagnostics)
    else: assert not diagnostics
    return dict(jobs=jobs,calls=calls,publications=publications,discards=discards)


def proposal(v,keys,delay=10):
    return dict(delay_ms=delay,proposal=dict(state_version=v,decision='execute',operations=[dict(operation='record.index',record=k) for k in keys],reason='Offline batch fixture; not model inference.'))


def verify(release,graphics=False):
    profile='release' if release else 'debug';flags=['--release'] if release else []
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    directory='target/display' if graphics else 'target'
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags,*(['--features','guest-display','--target-dir',directory] if graphics else [])],cwd=ROOT,check=True)
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    binary=ROOT/f'target/host-meta/{host}/{profile}/rai-meta';kernel=ROOT/f'{directory}/aarch64-unknown-none-softfloat/{profile}/rai-os'
    records=[];first=['brief@1','empty@1','notes@1'];second=['brief@1','empty@1','notes@2']
    with tempfile.TemporaryDirectory(prefix='rai-batch-') as temp:
        temp=Path(temp);events=temp/'events.json';plans=temp/'plans.json'
        env=dict(os.environ,MODEL_API_KEY='')
        def run(label,schedule,answers,expected=None,error=None,extra_env=None,live=False):
            scenario=dict(events=[dict(at_ms=t,required=keys) for t,keys in schedule])
            events.write_text(json.dumps(scenario));plans.write_text(json.dumps(answers))
            print('CASE '+label,flush=True)
            result=subprocess.run([str(binary),str(kernel),*(['--graphics'] if graphics else []),*(['--guest-batch',str(events)] if live else ['--guest-batch-replay',str(events),str(plans)])],cwd=ROOT,env=env|(extra_env or {}),capture_output=True,text=True,timeout=45)
            pidfile=temp/'qemu.pid'
            if pidfile.exists():
                try: os.kill(int(pidfile.read_text()),signal.SIGTERM)
                except ProcessLookupError: pass
                pidfile.unlink()
            items=[json.loads(s) for s in result.stdout.splitlines()]
            records.append(dict(case=label,transport='offline_curl_fixture' if live else 'replay',scenario=scenario,stdout=items,stderr=result.stderr,exit_code=result.returncode))
            if error:
                assert result.returncode and json.loads(result.stderr)['error']==error,(label,result.stdout,result.stderr)
                assert items[-1]['operation']=='coord.stop' and items[-1]['success']
            else:
                assert result.returncode==0,(label,result.stdout,result.stderr)
                actual=audit(items,scenario,'meta' if live else 'replay')
                assert actual==expected,(label,actual,expected)
            return items
        items=run('four-record-plan',[(0,first+['checklist@1'])],[proposal(1,first+['checklist@1'])],dict(jobs=first+['checklist@1'],calls=1,publications=[1],discards=[]))
        for event,field,value in [('batch_validated','plan',{}),('batch_finished','model_calls',0),('batch_publication','payload','{}'),('independent_check','passed',False)]:
            corrupt=copy.deepcopy(items);next(e for e in corrupt if e['event']==event)[field]=value
            try: audit(corrupt,dict(events=[dict(at_ms=0,required=first+['checklist@1'])]),'replay')
            except (AssertionError,KeyError,ValueError): pass
            else: raise AssertionError('Audit accepted corrupt '+event)
        if graphics:
            run('native-revision',[(0,first),(12000,second)],[proposal(1,first),proposal(6,['notes@2'])],dict(jobs=first+['notes@2'],calls=2,publications=[1,6],discards=[]))
        else:
            run('revision-and-cached-rollback',[(0,first),(2000,second),(4000,first)],[proposal(1,first),proposal(6,['notes@2'])],dict(jobs=first+['notes@2'],calls=2,publications=[1,6,9],discards=[]))
            run('stale-response-coalesced',[(0,first),(300,second),(400,['empty@1'])],[proposal(1,first,1000),proposal(3,['empty@1'])],dict(jobs=['empty@1'],calls=2,publications=[3],discards=['response']))
            bad=[]
            for field,value,error in [('record','restricted@1','record_not_allowed'),('record','brief@1','duplicate_batch_record'),('operation','shell','operation_not_allowed'),('record','appendix@1','batch_record_not_missing')]:
                p=proposal(1,['brief@1','notes@1']);p['proposal']['operations'][1][field]=value;bad.append((field+'-'+value,p,error))
            p=proposal(0,['brief@1','notes@1']);bad.append(('version',p,'stale_batch'))
            p=proposal(1,['brief@1']);bad.append(('incomplete',p,'incomplete_batch'))
            p=proposal(1,[]);bad.append(('empty',p,'batch_requires_1_to_4_operations'))
            p=proposal(1,[]);p['proposal']['decision']='decline';bad.append(('declined',p,'model_declined_current_batch'))
            p=proposal(1,['brief@1','notes@1']);p['proposal']['reason']='bad\nreason';bad.append(('reason',p,'invalid_batch_reason'))
            p=proposal(1,['brief@1']*5);bad.append(('too-many',p,'batch_requires_1_to_4_operations'))
            for label,p,error in bad:
                out=run('invalid-'+label,[(0,['brief@1','notes@1'])],[p],error=error)
                assert not any(e.get('operation') in ['coord.select','coord.propose','coord.publish.commit'] for e in out)
            # Delay REAL QEMU's first index reply; a scheduled change then lands
            # between jobs. No fake guest result is introduced.
            wrapper=temp/'qemu-system-aarch64';real_qemu=shutil.which('qemu-system-aarch64')
            wrapper.write_text('#!'+sys.executable+'\n'+'''import os,subprocess,sys,threading,time,signal
child=subprocess.Popen([os.environ['RAI_REAL_QEMU'],*sys.argv[1:]],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
from pathlib import Path
Path(os.environ['RAI_QEMU_PID']).write_text(str(child.pid))
def stop(*args):
    child.terminate();child.wait();raise SystemExit(0)
signal.signal(signal.SIGTERM,stop)
def forward():
    for line in sys.stdin.buffer:
        child.stdin.write(line);child.stdin.flush()
    child.stdin.close()
threading.Thread(target=forward,daemon=True).start()
delayed=False
try:
    for line in child.stdout:
        if b'; index=' in line and not delayed:
            delayed=True;time.sleep(1.2)
        sys.stdout.buffer.write(line);sys.stdout.buffer.flush()
finally:
    child.terminate();child.wait()
''');wrapper.chmod(0o700)
            wrapper_env=dict(PATH=str(temp)+os.pathsep+env['PATH'],RAI_REAL_QEMU=real_qemu,RAI_QEMU_PID=str(temp/'qemu.pid'))
            run('change-between-jobs',[(0,first),(400,second)],[proposal(1,first),proposal(3,['empty@1','notes@2'])],dict(jobs=['brief@1','empty@1','notes@2'],calls=2,publications=[3],discards=['execution']),extra_env=wrapper_env)
            wrapper.unlink()
            # Delay the REAL independent checker to test freshness after validation.
            checker=temp/'python3';real_python=shutil.which('python3')
            checker.write_text('#!'+sys.executable+'\n'+'''import os,sys,time
assert 'MODEL_API_KEY' not in os.environ
time.sleep(1.2)
os.execv(os.environ['RAI_REAL_PYTHON'],[os.environ['RAI_REAL_PYTHON'],*sys.argv[1:]])
''');checker.chmod(0o700)
            run('change-during-check',[(0,['empty@1']),(400,['notes@2'])],[proposal(1,['empty@1']),proposal(4,['notes@2'])],dict(jobs=['empty@1','notes@2'],calls=2,publications=[4],discards=['publication']),extra_env=dict(PATH=str(temp)+os.pathsep+env['PATH'],RAI_REAL_PYTHON=real_python))
            checker.write_text('#!'+sys.executable+'\nraise SystemExit(1)\n')
            out=run('checker-failure',[(0,['empty@1'])],[proposal(1,['empty@1'])],error='independent_bundle_check_failed',extra_env=dict(PATH=str(temp)+os.pathsep+env['PATH']))
            assert any(e.get('operation')=='coord.publish.reject' for e in out) and not any(e.get('operation')=='coord.publish.commit' for e in out)
            checker.unlink()
            # Request construction + strict envelope decode, no network or real key.
            curl=temp/'curl';counter=temp/'calls';fixture=temp/'response.json'
            curl.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
from pathlib import Path
text=sys.stdin.read();assert 'MODEL_API_KEY' not in os.environ
assert 'dummy_batch_fixture' in text
line=next(s for s in text.splitlines() if s.startswith('data-raw = '))
body=json.loads(json.loads(line.split(' = ',1)[1]))
assert body['max_tokens']==1024 and body['reasoning_effort']=='minimal'
assert body['response_format']['json_schema']['name']=='rai_batch'
assert body['response_format']['json_schema']['schema']['properties']['operations']['maxItems']==4
p=Path(os.environ['RAI_CALL_COUNT']);p.write_text(str(int(p.read_text())+1) if p.exists() else '1')
sys.stdout.write(Path(os.environ['RAI_RESPONSE']).read_text()+'\\n200')
''');curl.chmod(0o700)
            curl_env=dict(PATH=str(temp)+os.pathsep+env['PATH'],MODEL_API_KEY='dummy_batch_fixture',RAI_CALL_COUNT=str(counter),RAI_RESPONSE=str(fixture))
            envelope=dict(model='muse-spark-1.3',choices=[dict(finish_reason='stop',message=dict(role='assistant',content=json.dumps(proposal(1,first)['proposal'])))],usage=dict(prompt_tokens=10,completion_tokens=20,total_tokens=30))
            fixture.write_text(json.dumps(envelope))
            run('offline-provider-success',[(0,first)],[],dict(jobs=first,calls=1,publications=[1],discards=[]),extra_env=curl_env,live=True)
            for label,content,finish,error in [('invalid-json','{}','stop','invalid_batch_json'),('truncated',None,'length','meta_finish_length')]:
                envelope['choices'][0]['message']['content']=content;envelope['choices'][0]['finish_reason']=finish;fixture.write_text(json.dumps(envelope))
                out=run('offline-provider-'+label,[(0,first)],[],error=error,extra_env=curl_env,live=True)
                assert not any(e.get('operation')=='coord.select' for e in out)
                reports=[e for e in out if e['event']=='provider_diagnostic'];assert len(reports)==1 and reports[0]['error']==error
            assert counter.read_text()=='3','Unexpected provider retry'
            assert 'dummy_batch_fixture' not in json.dumps(records)
            # Actual worker stream failure cannot publish any candidate.
            kernel=build(profile,'index-stream-test-truncated')
            out=run('truncated-worker',[(0,['notes@2'])],[proposal(1,['notes@2'])],error='guest_rejected_control_operation')
            assert not any(e.get('operation')=='coord.publish.commit' for e in out)
    OUTPUT.mkdir(parents=True,exist_ok=True)
    (OUTPUT/f'batch-{profile}{"-graphics" if graphics else ""}.json').write_text(json.dumps(records,indent=2)+'\n')
    print(f'PASS {profile}{" graphics" if graphics else ""}: {len(records)} offline real-QEMU batch cases; independent artifact and trace audit')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--release',action='store_true');p.add_argument('--graphics',action='store_true');p.add_argument('--check-live',type=Path);p.add_argument('--scenario',type=Path,default=ROOT/'fixtures/guest_batch/v1/goals.json');a=p.parse_args()
    if a.check_live:
        print(audit([json.loads(s) for s in a.check_live.read_text().splitlines()],json.loads(a.scenario.read_text()),'meta'))
    else: verify(a.release,a.graphics)
