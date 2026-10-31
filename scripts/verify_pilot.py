#!/usr/bin/env python3
"""Independent policy-pilot ledger/source/IPC audit and offline real-QEMU checks."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from check_bundle import check
from verify_mmu import ROOT, OUTPUT
from verify_guest_coordinator import fields
from verify_coord_index import audit as audit_job
from verify_set_bridge import META


def cost(key): return max(1,(META[key]+31)//32)


def audit(items, scenario, policy):
    first=items[0]
    assert first['event']=='pilot_started' and first['scenario_id']==scenario['id'] and first['policy']==policy
    assert first['model']==('muse-spark-1.3' if policy=='meta' else None)
    assert first['deadline_ms']==scenario['deadline_ms'] and first['max_decisions']==8
    assert first['credit_model']=='host_simulated_ceil_bytes_per_32_min_1'
    cursor=0;issued=0;generation=0;required=[];ready={};artifacts={};spent=0;serial=0;clock=0
    calls=0;stale=0;pending=None;response=None;selection=None;selected=None;debit=None;report=None
    raw_result=None;raw_bundle=None;publication=None;due=None;rules=None;finished=None;stopped=False
    inference=[];tokens=[];jobs=[];trace=[]
    def state():
        return dict(goal_version=generation,required=required,missing=[k for k in required if k not in ready],completed=len(ready))
    def status(payload):
        f=dict(part.split('=',1) for part in payload.split('; '));s=state()
        assert set(f)=={'goal_version','state','required','ready_records','total_records','completed','missing'}
        assert int(f['goal_version'])==generation and f['required']==','.join(required)
        assert f['missing']==(','.join(s['missing']) or '-') and int(f['completed'])==len(ready)
        assert int(f['total_records'])==len(required) and int(f['ready_records'])==len(required)-len(s['missing'])
        assert f['state']==('building' if s['missing'] else 'complete')
    def context(c,now):
        assert c['index_set']==state() and c['state_version']==generation and c['operation']=='record.index'
        assert c['required_record'] is None and c['completed_results']==[]
        assert c['authorized_records']==[[k,META[k]] for k in state()['missing'] if spent+cost(k)<=rules['credit_cap']]
        view=c['pilot'];assert clock<=view['elapsed_ms']<=now
        assert view==dict(elapsed_ms=view['elapsed_ms'],deadline_ms=scenario['deadline_ms'],credit_cap=rules['credit_cap'],spent_credits=spent,jobs=[dict(**j,credits=cost(j['record']),ready_at_ms=ready.get(j['record'])) for j in sorted(rules['jobs'],key=lambda j:j['record'])])
    def response_usage(item):
        assert item['latency_ms']>=0
        if policy=='meta':
            inference.append(item['latency_ms'])
            usage=item['usage']
            if usage is not None:
                assert usage['total_tokens']==usage['prompt_tokens']+usage['completion_tokens']
                tokens.append(usage['total_tokens'])
        else: assert item['usage'] is None
    for item in items[1:]:
        e=item['event'];at=item.get('at_ms',item.get('elapsed_ms',item.get('total_elapsed_ms',clock)))
        assert type(at) is int and at>=clock,(e,clock,at)
        if e=='guest_boot_confirmed': pass
        elif e=='pilot_event':
            assert due is None and item['index']==cursor and item['input']==scenario['events'][cursor]
            assert item['input']['at_ms']<=at
            rules=item['input'];due=sorted(j['record'] for j in rules['jobs']);cursor+=1
            trace.append(dict(at_ms=at,event='goal',detail=','.join(due)))
        elif e=='pilot_decision_requested':
            assert pending is None and at<scenario['deadline_ms']
            context(item['snapshot'],at);assert item['snapshot']['authorized_records']
            calls+=1;assert calls<=8 and item['call_id']==calls
            pending=item['snapshot'];response=None
        elif e=='pilot_decision_response':
            assert pending is not None and response is None and item['call_id']==calls
            response_usage(item);response=item['proposal']
            if policy in ['edf','value']:
                from fractions import Fraction
                allowed={k for k,_ in pending['authorized_records']};v=pending['pilot']
                available=[j for j in v['jobs'] if j['record'] in allowed]
                def rank(j):
                    common=(j['due_ms']<v['elapsed_ms'],)
                    return common+((j['due_ms'],-j['weight'],j['credits'],j['record']) if policy=='edf' else (-Fraction(j['weight'],j['credits']),j['due_ms'],j['record']))
                assert response['record']==min(available,key=rank)['record']
        elif e=='pilot_decision_drained':
            assert pending is not None and item['call_id']==calls
            if 'error' not in item: response_usage(item)
            pending=None;response=None
        elif e=='pilot_selection':
            p=item['proposal'];assert pending and response==p and selection is None and at<scenario['deadline_ms']
            assert p['state_version']==pending['state_version'] and p['operation']=='record.index'
            assert p['record'] in dict(pending['authorized_records']) and 0<len(p['reason'].encode())<=240
            selection=p;pending=None;response=None
        elif e=='pilot_declined':
            assert pending and response==item['proposal'] and response['operation']=='decline' and response['record']==''
            assert response['state_version']==pending['state_version']
            assert item['obsolete']==(pending['state_version']!=generation)
            pending=None;response=None
        elif e=='pilot_selection_superseded':
            assert selected and item['ticket']==selected[0] and selected[2]!=generation and debit is None
            selected=None;stale+=1
        elif e=='pilot_debit':
            assert selected and selected[2]==generation and at<scenario['deadline_ms'] and debit is None
            key=selected[1];assert key in state()['missing']
            cutoff=item['event_cutoff_ms'];assert clock<=cutoff<=at
            assert cursor==len(scenario['events']) or scenario['events'][cursor]['at_ms']>cutoff
            spent+=cost(key)
            assert item['record']==key and item['ticket']==selected[0] and item['goal_version']==generation
            assert item['credits']==cost(key) and item['spent']==spent and item['cap']==rules['credit_cap'] and spent<=rules['credit_cap']
            debit=item;trace.append(dict(at_ms=at,event='dispatch',detail=key,credits=spent))
        elif e=='pilot_ready':
            assert selected and raw_result and item['record']==selected[1] and selected[1] not in ready
            key=selected[1];ready[key]=at;artifacts[key]=raw_result;jobs.append(key)
            selected=None;debit=None;report=None;raw_result=None
            trace.append(dict(at_ms=at,event='ready',detail=key))
        elif e=='pilot_verified_bundle':
            assert raw_bundle==item['payload'] and item['snapshot']==state() and not state()['missing']
            check(item['payload'],required,generation);publication=(generation,at);raw_bundle=None
            trace.append(dict(at_ms=at,event='verified',detail=str(generation)))
        elif e=='pilot_blocked':
            context(item['snapshot'],at)
            assert item['reason']=='no_affordable_work' and not item['snapshot']['authorized_records']
            trace.append(dict(at_ms=at,event='blocked',detail='credit cap'))
        elif e=='guest_exchange':
            serial+=1;assert item['request_id']==serial
            op=item['operation'];arg=item['argument'];payload=item['payload'];ok=item['success']
            if op=='coord.start': assert ok and serial==1 and arg=='index' and fields(payload)['state']=='idle'
            elif op=='coord.goal':
                assert ok and due and arg==','.join(due)
                issued+=1;generation=issued;required=due;due=None;publication=None;status(payload)
            elif op=='coord.bundle.status': assert ok;status(payload)
            elif op=='coord.select':
                assert selection and arg==f"{selection['state_version']}:{selection['record']}"
                if selection['state_version']!=generation:
                    assert not ok and payload=='coord_stale_goal';stale+=1
                    trace.append(dict(at_ms=at,event='stale',detail=selection['record']))
                else:
                    assert ok;issued+=1;r=fields(payload);selected=(issued,selection['record'],generation)
                    assert r['state']=='awaiting_proposal' and r['event']=='observed' and r['ticket']==str(issued) and r['record']==selected[1] and r['b_calls']=='0' and r['completed']==str(len(ready))
                selection=None
            elif op=='coord.propose':
                assert selected and debit and arg==f'{selected[0]}:{selected[1]}'
                if ok: report=fields(payload);assert report['ticket']==str(selected[0])
                else: report=None
            elif op=='coord.result':
                assert ok and report and selected and selected[1] not in ready
                audit_job(report,payload,selected[1],len(ready)+1,len(ready)+1,0);raw_result=payload
            elif op=='coord.bundle':
                assert ok and not state()['missing']
                check(payload,required,generation)
                for row in json.loads(payload)['records']: assert row['index']==artifacts[row['record']].split('; index=',1)[1]
                raw_bundle=payload
            elif op=='coord.stop': assert ok and fields(payload)['state']=='stopped';stopped=True
            else: raise AssertionError('Unexpected pilot operation '+op)
        elif e=='guest_index':
            assert raw_result and selected and item['payload']==raw_result and item['record']==selected[1] and item['ticket']==selected[0] and item['origin']=='executed'
        elif e=='pilot_finished':
            assert finished is None and stopped and item['decisions']==calls and item['stale']==stale
            assert item['spent_credits']==spent and item['worker_jobs']==len(ready) and item['ready']==ready
            assert item['final_snapshot']==(state() if generation else None)
            if item['passed']:
                assert item['reason'] is None and cursor==len(scenario['events']) and not state()['missing']
                assert publication and publication[0]==generation and publication[1]<=scenario['deadline_ms']
                assert item['completion_ms']==publication[1] and not any([pending,selection,selected,debit,raw_result,raw_bundle])
                weight=sum(j['weight'] for j in scenario['events'][-1]['jobs'] if ready[j['record']]<=j['due_ms'])
            else:
                assert item['reason'] and item['completion_ms'] is None;weight=0
                if item['reason']=='deadline_exceeded':assert at>=scenario['deadline_ms']
            assert item['weighted_on_time']==weight and item['total_elapsed_ms']==at
            finished=item
        else: raise AssertionError('Unexpected event '+e)
        clock=at
    assert finished and stopped and due is None
    return dict(scenario=scenario['id'],policy=policy,passed=finished['passed'],reason=finished['reason'],completion_ms=finished['completion_ms'],weighted_on_time=finished['weighted_on_time'],spent_credits=spent,worker_jobs=len(ready),decisions=calls,stale=stale,inference_ms=sum(inference),reported_tokens=sum(tokens),usage_responses=len(tokens),total_elapsed_ms=finished['total_elapsed_ms'],trace=trace)


def build(release):
    flags=['--release'] if release else []
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags],cwd=ROOT,check=True)
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    profile='release' if release else 'debug'
    return ROOT/f'target/host-meta/{host}/{profile}/rai-meta',ROOT/f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'


def verify(release):
    binary,kernel=build(release);profile='release' if release else 'debug';OUTPUT.mkdir(exist_ok=True,parents=True)
    with tempfile.TemporaryDirectory(prefix='rai-pilot-') as tmp:
        tmp=Path(tmp);scenario_path=tmp/'scenario.json';replay_path=tmp/'replay.json'
        def run(scenario,policy,proposals=None):
            scenario_path.write_text(json.dumps(scenario))
            args=[str(binary),str(kernel),'--pilot',policy,str(scenario_path)]
            if proposals is not None:replay_path.write_text(json.dumps(proposals));args.append(str(replay_path))
            p=subprocess.run(args,cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=40)
            assert p.returncode==0,(p.stderr,p.stdout)
            items=[json.loads(s) for s in p.stdout.splitlines()]
            (OUTPUT/f'pilot-{scenario["id"]}-{policy}-{profile}.jsonl').write_text(p.stdout)
            result=audit(items,scenario,policy);return result,items
        def job(key,due=2000,weight=1):return dict(record=key,due_ms=due,weight=weight)
        def scenario(name,events,deadline=4000):return dict(id=name,events=events,deadline_ms=deadline)
        def event(at,jobs,cap=6):return dict(at_ms=at,jobs=jobs,credit_cap=cap)
        def proposal(v,key,delay=0,operation='record.index'):return dict(delay_ms=delay,proposal=dict(state_version=v,operation=operation,record=key,reason='Offline comparison test, not Meta inference.'))
        normal=scenario('normal',[event(0,[job('brief@1',1000,2),job('empty@1',2000,5)])])
        for policy in ['edf','value']:
            result,items=run(normal,policy);assert result['passed'] and result['weighted_on_time']==7 and result['spent_credits']==3
            keys=[i['record'] for i in items if i['event']=='pilot_ready'];assert keys==(['brief@1','empty@1'] if policy=='edf' else ['empty@1','brief@1'])
            for ev,field,value in [('pilot_finished','weighted_on_time',99),('pilot_debit','spent',0),('pilot_decision_requested','snapshot',{}),('pilot_verified_bundle','payload','{}')]:
                bad=copy.deepcopy(items);next(i for i in bad if i['event']==ev)[field]=value
                try:audit(bad,normal,policy)
                except (AssertionError,ValueError,KeyError):pass
                else:raise AssertionError('Accepted forged '+field)
        stale=scenario('stale',[event(0,[job('notes@1')]),event(700,[job('notes@2')])])
        result,_=run(stale,'replay',[proposal(1,'notes@1',1500),proposal(2,'notes@2')]);assert result['passed'] and result['stale']==1 and result['spent_credits']==2
        # Same membership, new priority metadata: old authority still becomes stale.
        same=scenario('same-membership',[event(0,[job('notes@1')]),event(700,[job('notes@1',2500,8)])])
        result,_=run(same,'replay',[proposal(1,'notes@1',1500),proposal(2,'notes@1')]);assert result['passed'] and result['stale']==1 and result['weighted_on_time']==8
        blocked=scenario('blocked',[event(0,[job('brief@1',500)],1)],800)
        result,_=run(blocked,'edf');assert not result['passed'] and result['spent_credits']==0 and result['decisions']==0
        reduced=scenario('reduced',[event(0,[job('notes@1')],2),event(700,[job('notes@2')],2)],2000)
        result,_=run(reduced,'edf');assert not result['passed'] and result['spent_credits']==1 and result['worker_jobs']==1
        grandfather=scenario('grandfather',[event(0,[job('notes@1')],2),event(700,[job('notes@1')],0)],2000)
        result,_=run(grandfather,'edf');assert result['passed'] and result['spent_credits']==1 and result['decisions']==1
        repeated=[event(0,[job('brief@1',10000),job('empty@1',10000)])]+[event(t,[job('brief@1',10000),job('empty@1',10000)]) for t in range(500,6501,1000)]
        budget=scenario('decision-limit',repeated,12000)
        result,_=run(budget,'replay',[proposal(v,'brief@1',1000) for v in range(1,8)]+[proposal(8,'brief@1')])
        assert not result['passed'] and result['reason']=='decision_budget_exhausted' and result['decisions']==8 and result['stale']==7
        rollback=scenario('rollback',[event(0,[job('notes@1',1700)]),event(700,[job('notes@2',1700)]),event(1400,[job('notes@1',1700)])],1800)
        result,_=run(rollback,'replay',[proposal(1,'notes@1'),proposal(3,'notes@2',1500)])
        assert result['passed'] and result['worker_jobs']==1 and result['completion_ms']<=1800 and result['total_elapsed_ms']>=2000
        late=scenario('late',[event(0,[job('notes@1',500)])],800)
        result,_=run(late,'replay',[proposal(1,'notes@1',1500)]);assert not result['passed'] and result['spent_credits']==0 and result['total_elapsed_ms']>=1500
        for name,p,error in [('denied',proposal(1,'restricted@1'),'record_not_allowed'),('wrong-mode',proposal(1,'notes@1',operation='record.count'),'operation_not_allowed'),('unoffered',proposal(1,'appendix@1'),'proposal_not_authorized_at_request'),('decline',proposal(1,'',operation='decline'),'policy_declined')]:
            s=scenario(name,[event(0,[job('notes@1')])]);result,_=run(s,'replay',[p]);assert not result['passed'] and result['reason']==error and result['spent_credits']==0
        # Corpus correctness includes invalid references, not only accepted runs.
        s=scenario('bad',[event(0,[job('restricted@1')])]);scenario_path.write_text(json.dumps(s))
        p=subprocess.run([str(binary),str(kernel),'--pilot','meta',str(scenario_path)],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True)
        assert p.returncode and not p.stdout and 'record_not_allowed' in p.stderr
    print(f'PASS {profile}: 14 real-QEMU pilot runs, exact policy inputs/outputs, stale authority, budgets, deadline drain, rejection and independent audit mutations')


def main():
    p=argparse.ArgumentParser();p.add_argument('--release',action='store_true');p.add_argument('--artifact',action='store_true');args=p.parse_args()
    if args.artifact:
        data=sys.stdin.buffer.read(8193)
        if len(data)>8192:raise ValueError('Oversized artifact envelope')
        obj=json.loads(data);assert set(obj)=={'payload','required','goal_version'}
        check(obj['payload'],obj['required'],obj['goal_version']);print('VERIFIED')
    else:verify(args.release)


if __name__=='__main__':main()
