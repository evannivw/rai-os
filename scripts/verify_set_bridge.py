#!/usr/bin/env python3
"""Whole-set Meta choices, guest freshness gates and independent exact artifact audit."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_bundle import build, bundle_fields
from verify_guest_coordinator import fields
from verify_coord_index import audit as audit_job
from check_bundle import check

META = {'brief@1':56,'notes@1':30,'notes@2':44,'checklist@1':41,'empty@1':0,'appendix@1':108}


def audit(items, scenario, source):
    assert items[0]['event']=='set_bridge_started' and items[0]['source']==source
    assert items[0]['event_source']=='scripted_required_sets' and items[0]['max_model_calls']==8
    assert items[0]['model']==('muse-spark-1.3' if source=='meta' else None)
    cache={}; issued=0; current=None; cursor=0; calls=0; serial=0; chosen=None; report=None
    due=None; pending=None; answered=False; forwarded=None; artifact=None; bundle=None
    jobs=[]; stale=0; publications=[]; finished=False; stopped=False
    def expected():
        return dict(goal_version=current['goal_version'], required=current['required'], missing=[k for k in current['required'] if k not in cache], completed=len(cache))
    def state(payload):
        s=bundle_fields(payload);e=expected()
        assert set(s)=={'goal_version','state','required','ready_records','total_records','completed','missing'}
        assert int(s['goal_version'])==e['goal_version'] and s['required']==','.join(e['required'])
        assert s['missing']==(','.join(e['missing']) or '-') and int(s['completed'])==len(cache)
        assert int(s['total_records'])==len(e['required']) and int(s['ready_records'])==len(e['required'])-len(e['missing'])
        assert s['state']==('building' if e['missing'] else 'complete')
    for item in items[1:]:
        event=item['event']
        if event=='guest_boot_confirmed': continue
        if event=='set_due':
            assert due is None and cursor<len(scenario['events'])
            e=scenario['events'][cursor]; assert item['index']==cursor and item['required']==sorted(e['required'])
            assert item['scheduled_ms']==e['at_ms'] and item['schedule_elapsed_ms']>=e['at_ms']
            due=item['required'];cursor+=1
        elif event=='decision_requested':
            assert pending is None and current and expected()['missing']
            calls+=1; assert item['call_id']==calls and calls<=8
            c=item['snapshot']; e=expected()
            assert c==dict(operation='record.index',state_version=e['goal_version'],authorized_records=[[k,META[k]] for k in e['missing']],completed_results=[],required_record=None,index_set=e)
            pending=c; answered=False
        elif event=='decision_response':
            assert pending and not answered and item['call_id']==calls and item['source']==source
            assert item['snapshot_goal']==pending['state_version'];answered=True
            if source=='meta': assert item['usage'] is None or item['usage']['total_tokens']>=item['usage']['completion_tokens']
        elif event=='proposal_declined':
            p=item['proposal'];assert pending and answered and item['call_id']==calls
            assert p['state_version']==pending['state_version'] and p['operation']=='decline' and p['record']==''
            assert item['obsolete'] and current['goal_version']!=pending['state_version'];pending=None
        elif event=='proposal_forwarded':
            p=item['proposal'];assert pending and answered and forwarded is None and item['call_id']==calls
            assert p['state_version']==pending['state_version'] and p['operation']=='record.index' and p['record'] in pending['index_set']['missing']
            assert 0<len(p['reason'].encode())<=240
            forwarded=p
        elif event=='guest_exchange':
            serial+=1;assert item['request_id']==serial
            op=item['operation'];arg=item['argument'];payload=item['payload'];ok=item['success']
            if op=='coord.start':
                assert ok and arg=='index' and serial==1 and fields(payload)['state']=='idle'
            elif op=='coord.goal':
                assert ok and due and arg==','.join(due)
                issued+=1;current=dict(goal_version=issued,required=due);due=None;state(payload)
            elif op=='coord.bundle.status': assert ok;state(payload)
            elif op=='coord.select':
                assert forwarded and pending and arg==f"{forwarded['state_version']}:{forwarded['record']}"
                if forwarded['state_version']!=current['goal_version']:
                    assert not ok and payload=='coord_stale_goal';stale+=1;forwarded=None;pending=None
                else:
                    assert ok and forwarded['record'] in expected()['missing']
                    issued+=1;chosen=(issued,forwarded['record']);r=fields(payload)
                    assert r['state']=='awaiting_proposal' and r['event']=='observed' and int(r['ticket'])==issued and r['record']==chosen[1]
                    assert int(r['completed'])==len(cache) and int(r['b_calls'])==0 and int(r['a_calls'])==2
                    forwarded=None;pending=None
            elif op=='coord.propose':
                assert ok and chosen and arg==f'{chosen[0]}:{chosen[1]}' and chosen[1] not in cache
                report=fields(payload);assert int(report['ticket'])==chosen[0]
            elif op=='coord.result':
                assert ok and chosen and report and artifact is None
                key=chosen[1];audit_job(report,payload,key,len(cache)+1,len(cache)+1,0)
                artifact=payload;cache[key]=payload;jobs.append(key)
            elif op=='coord.bundle':
                assert ok and current and not expected()['missing'] and bundle is None
                check(payload,current['required'],current['goal_version']);bundle=payload
                # Records retain exactly the worker's prior index, never a host merge.
                for row in json.loads(payload)['records']:
                    assert row['index']==cache[row['record']].split('; index=',1)[1]
            elif op=='coord.stop': assert ok and fields(payload)['state']=='stopped';stopped=True
            else: raise AssertionError('Unexpected operation '+op)
        elif event=='guest_index':
            assert chosen and artifact and item['ticket']==chosen[0] and item['record']==chosen[1]
            assert item['origin']=='executed' and item['payload']==artifact
            chosen=None;report=None;artifact=None
        elif event=='guest_bundle':
            assert bundle and item['payload']==bundle and item['snapshot']==expected()
            publications.append(current['goal_version']);bundle=None
        elif event=='set_bridge_finished':
            assert item['status']=='observed_current_set' and item['model_calls']==calls and item['snapshot']==expected()
            assert not expected()['missing'] and cursor==len(scenario['events']) and publications[-1]==current['goal_version'];finished=True
        else: raise AssertionError('Unexpected event '+event)
    assert finished and stopped and not any([pending,forwarded,chosen,artifact,bundle,due])
    return jobs,stale,calls,publications


def gate(profile):
    with guest(build(profile),'gate-'+profile,prefix='set') as (child,_,line,_,_):
        assert 'boot OK' in line();assert 'Console ready' in line();serial=0
        def request(op,ok=True):
            nonlocal serial
            serial+=1;child.stdin.write(f'@rai1\t{serial}\t{op}\n'.encode());child.stdin.flush()
            p=line().split('\t',3);assert p[:3]==['@rai1',str(serial),'ok' if ok else 'err'],p
            return p[3]
        assert request('coord.select\t1:notes@1',False)=='coord_not_running'
        request('coord.start');assert request('coord.select\t1:notes@1',False)=='coord_not_indexing'
        request('coord.stop');request('coord.start\tindex')
        assert request('coord.select\t1:notes@1',False)=='coord_no_required_set'
        request('coord.goal\tbrief@1,notes@1');old=request('coord.status')
        for value,error in [('0:notes@1','bad_args'),('4294967296:notes@1','bad_args'),('1:restricted@1','record_denied'),('1:appendix@1','coord_not_required'),('2:notes@1','coord_stale_goal'),('1:notes@1\textra','bad_args')]:
            assert request('coord.select\t'+value,False)==error
            assert request('coord.status')==old
        selected=fields(request('coord.select\t1:notes@1'));assert selected['ticket']=='2'
        request('coord.goal\tbrief@1,notes@2');old=request('coord.status')
        assert request('coord.select\t1:brief@1',False)=='coord_stale_goal';assert request('coord.status')==old
        assert request('coord.propose\t2:notes@1',False)=='coord_not_required'
        assert request('coord.bundle',False)=='coord_bundle_incomplete'
        selected=fields(request('coord.select\t3:notes@2'));assert selected['ticket']=='4'
        request('coord.propose\t4:notes@2');old=request('coord.status')
        assert request('coord.select\t3:notes@2',False)=='coord_already_completed';assert request('coord.status')==old
        # Same-record selection also becomes stale if requirements change before execution.
        selected=fields(request('coord.select\t3:brief@1'));ticket=selected['ticket']
        request('coord.goal\tbrief@1,notes@2');assert request('coord.propose\t'+ticket+':brief@1',False)=='coord_no_goal'
        child.stdin.write(b'coord.select 6:brief@1\n');child.stdin.flush()
        assert line()=='coord.select 6:brief@1' and fields(line())['ticket']=='7'
        request('coord.propose\t7:brief@1')
        check(request('coord.bundle'),['brief@1','notes@2'],6)
        request('coord.stop');request('coord.start\tindex');request('coord.goal\tbrief@1')
        assert request('coord.select\t6:brief@1',False)=='coord_stale_goal'
        request('coord.stop');assert 'free_pages=244' in request('memory.check')
    print(f'PASS {profile}: generation/membership/cache gate, unchanged rejected state, intervening goal, restart and cleanup ({serial} requests)')


def verify(release,graphics):
    profile='release' if release else 'debug';flags=['--release'] if release else []
    OUTPUT.mkdir(parents=True,exist_ok=True)
    if not graphics: gate(profile)
    directory='target/display' if graphics else 'target'
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags,*(['--features','guest-display','--target-dir',directory] if graphics else [])],cwd=ROOT,check=True)
    host=next(line[6:] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    binary=ROOT/f'target/host-meta/{host}/{profile}/rai-meta';kernel=ROOT/f'{directory}/aarch64-unknown-none-softfloat/{profile}/rai-os'
    def proposal(v,key,delay=10,op='record.index'):
        return dict(delay_ms=delay,proposal=dict(state_version=v,operation=op,record=key,reason='Offline set fixture; not model inference.'))
    first=['brief@1','empty@1','notes@1'];second=['brief@1','checklist@1','empty@1','notes@1'];third=['brief@1','checklist@1','empty@1','notes@2']
    cases=[
        ('revision',[(0,first),(5000,second),(10000,third)],[proposal(1,'notes@1'),proposal(1,'brief@1'),proposal(1,'empty@1'),proposal(5,'checklist@1'),proposal(7,'notes@2')],(['notes@1','brief@1','empty@1','checklist@1','notes@2'],0,5,[1,5,7])),
        ('stale-shared-key',[(0,['brief@1','notes@1']),(700,['brief@1','notes@2'])],[proposal(1,'brief@1',1800),proposal(2,'notes@2'),proposal(2,'brief@1')],(['notes@2','brief@1'],1,3,[2])),
        ('obsolete-decline',[(0,['notes@1']),(700,['notes@2'])],[proposal(1,'',1800,'decline'),proposal(2,'notes@2')],(['notes@2'],0,2,[2])),
        ('cached-goal',[(0,['notes@1']),(1500,['notes@1']),(3000,['notes@2']),(3800,['notes@1'])],[proposal(1,'notes@1'),proposal(4,'notes@2',1800)],(['notes@1'],1,2,[1,3,5])),
        ('simultaneous',[(0,['notes@1']),(0,['empty@1','appendix@1'])],[proposal(2,'appendix@1'),proposal(2,'empty@1')],(['appendix@1','empty@1'],0,2,[2])),
    ]
    if graphics:
        name,_,decisions,expected=cases[0]
        cases=[(name,[(0,first),(12000,second),(22000,third)],decisions,expected)]
    with tempfile.TemporaryDirectory(prefix='rai-set-') as temp:
        temp=Path(temp);events=temp/'events.json';replies=temp/'replay.json'
        def run():
            return subprocess.run([str(binary),str(kernel),*(['--graphics'] if graphics else []),'--guest-set-replay',str(events),str(replies)],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=45)
        for name,schedule,decisions,expected in cases:
            scenario=dict(events=[dict(at_ms=t,required=keys) for t,keys in schedule]);events.write_text(json.dumps(scenario));replies.write_text(json.dumps(decisions))
            result=run();assert result.returncode==0,(name,result.stderr,result.stdout)
            (OUTPUT/f'set-{name}-{profile}{"-graphics" if graphics else ""}.jsonl').write_text(result.stdout)
            items=[json.loads(s) for s in result.stdout.splitlines()]
            actual=audit(items,scenario,'replay')
            if graphics:
                assert actual[:3]==expected[:3] and actual[3][-1]==expected[3][-1],(actual,expected)
            else: assert actual==expected,(actual,expected)
            if name=='revision':
                for event,field,value in [('decision_requested','snapshot',{}),('set_bridge_finished','model_calls',0),('guest_bundle','payload','{}'),('guest_index','payload','invented')]:
                    bad=copy.deepcopy(items);next(i for i in bad if i['event']==event)[field]=value
                    try: audit(bad,scenario,'replay')
                    except (AssertionError,ValueError,KeyError): pass
                    else: raise AssertionError('Accepted corrupted '+field)
        if not graphics:
            events.write_text(json.dumps(dict(events=[dict(at_ms=0,required=['notes@1'])])))
            for p,error in [(proposal(0,'notes@1'),'stale_proposal'),(proposal(1,'restricted@1'),'record_not_allowed'),(proposal(1,'brief@1'),'proposal_not_in_requested_missing_set'),(proposal(1,'notes@1',op='record.count'),'operation_not_allowed'),(proposal(1,'',op='decline'),'model_declined_current_set')]:
                replies.write_text(json.dumps([p]));result=run();assert result.returncode and json.loads(result.stderr)['error']==error,(error,result.stderr)
                items=[json.loads(s) for s in result.stdout.splitlines()]
                assert not any(i.get('operation') in ['coord.select','coord.propose'] for i in items)
                assert items[-1]['operation']=='coord.stop' and items[-1]['success']
            # A completed record cannot be selected again from a later model snapshot.
            events.write_text(json.dumps(dict(events=[dict(at_ms=0,required=['brief@1','notes@1'])])))
            replies.write_text(json.dumps([proposal(1,'notes@1'),proposal(1,'notes@1')]))
            result=run();assert result.returncode and json.loads(result.stderr)['error']=='proposal_not_in_requested_missing_set'
            assert sum(json.loads(s).get('operation')=='coord.propose' for s in result.stdout.splitlines())==1
            # Four late answers plus four real jobs consume this mode's eight-call budget.
            initial=['brief@1','checklist@1','empty@1','notes@1']
            schedule=[(0,initial),(500,initial),(1900,initial),(3300,initial),(4700,initial),(8000,third)]
            events.write_text(json.dumps(dict(events=[dict(at_ms=t,required=k) for t,k in schedule])))
            replies.write_text(json.dumps([proposal(v,'brief@1',1400) for v in range(1,5)]+[proposal(5,k) for k in initial]))
            result=run();assert result.returncode and json.loads(result.stderr)['error']=='model_call_budget_exhausted',result.stderr
            assert json.loads(result.stdout.splitlines()[-1])['operation']=='coord.stop'
            # Bad scenarios fail before QEMU or any credential access.
            events.write_text('{"events":[{"at_ms":0,"required":["notes@1","notes@2"]}]}')
            result=run();assert result.returncode and not result.stdout
            # Incomplete source stream must fail closed and stop the retained pair.
            bad=build(profile,'index-stream-test-truncated');goodkernel=kernel;kernel=bad
            events.write_text(json.dumps(dict(events=[dict(at_ms=0,required=['notes@2'])])))
            replies.write_text(json.dumps([proposal(1,'notes@2')]));result=run();kernel=goodkernel
            assert result.returncode and not any(json.loads(s)['event']=='guest_bundle' for s in result.stdout.splitlines())
            assert json.loads(result.stdout.splitlines()[-1])['operation']=='coord.stop'
    print(f'PASS {profile}{" graphics" if graphics else ""}: set planning, exact artifacts, selective revisions, stale/invalid choices and cleanup')


def main():
    p=argparse.ArgumentParser();p.add_argument('--release',action='store_true');p.add_argument('--graphics',action='store_true')
    p.add_argument('--check-live',type=Path);p.add_argument('--scenario',type=Path,default=ROOT/'fixtures/guest_set/v1/goals.json')
    args=p.parse_args()
    if args.check_live:
        print('PASS live whole-set audit:',audit([json.loads(s) for s in args.check_live.read_text().splitlines()],json.loads(args.scenario.read_text()),'meta'))
        print('Logs are not cryptographic proof of provider access; no efficiency comparison is implied.')
    else: verify(args.release,args.graphics)


if __name__=='__main__':main()
