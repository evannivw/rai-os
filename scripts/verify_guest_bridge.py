#!/usr/bin/env python3
"""Independently audit the guest-owned Meta path and test it offline with real QEMU."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from verify_service import CATALOG
from verify_guest_coordinator import audit_execution, fields
from verify_coord_index import audit as audit_index
from check_record_index import check as check_index
ROOT = Path(__file__).resolve().parents[1]


def snapshot(report):
    result = {key: report[key] for key in ['state', 'event', 'record', 'trace']}
    if result['record'] == 'none': result['record'] = None
    for key in ['ticket','bytes','tokens','completed','a_calls','b_calls','a_slices','b_slices']:
        result[key] = int(report[key])
    if report.get('operation') == 'record.index':
        result['index'] = {k:int(report[k]) for k in ['cached_records','cache_hits','output_bytes']}
    return result


def audit(items, scenario, source, index=False):
    first = items[0]
    assert first['event'] == 'guest_bridge_started' and first['source'] == source
    assert first['state_owner'] == 'guest_EL0' and first['event_source'] == 'scripted_observations'
    assert first['max_model_calls'] == 4 and first['model'] == ('muse-spark-1.3' if source == 'meta' else None)
    cache = {}; hits = 0; artifact_due = False; fetched = None
    if index: assert first['operation'] == 'record.index'
    state = 'stopped'; key = None; ticket = 0; completed = 0; issued = 0; count = (0,0)
    due = None; cursor = 0; boots = 0; request_id = 0; calls = 0; stale = 0; executed = []
    pending = None; answered = False; forwarded = None; report = None; final_snapshot = None; declined = False
    expected_event = 'none'; stopped = False
    for item in items[1:]:
        event = item['event']
        if artifact_due:
            assert event == 'guest_index' or (event == 'guest_exchange' and item['operation'] == 'coord.result')
        if event == 'guest_boot_confirmed':
            boots += 1; assert boots == 1
        elif event == 'observation_due':
            assert boots == 1 and due is None
            expected = scenario['events'][cursor]
            assert item['index'] == cursor and item['record'] == expected['record']
            assert item['scheduled_ms'] == expected['at_ms'] <= item['schedule_elapsed_ms']
            due = expected['record']; cursor += 1
        elif event == 'guest_exchange':
            assert boots == 1 and not stopped
            request_id += 1; assert item['request_id'] == request_id
            op = item['operation']; argument = item['argument']
            if op == 'coord.result':
                assert index and artifact_due and fetched is None and argument is None and item['success']
                fetched = item['payload']
                audit_index(previous_report, fetched, key, completed, len(cache) + (expected_event == 'executed'), hits, expected_event == 'reused')
                if expected_event == 'reused': assert fetched == cache[key]
                else:
                    assert key not in cache
                    cache[key] = fetched
                continue
            if op == 'coord.propose':
                assert forwarded is not None and pending is not None and answered
                assert argument == f"{forwarded['state_version']}:{forwarded['record']}"
                proposed_ticket = forwarded['state_version']; proposed_key = forwarded['record']
                if proposed_ticket != ticket: error = 'coord_stale_proposal'; stale += 1
                elif proposed_key != key: error = 'coord_wrong_record'
                elif state == 'completed': error = 'coord_already_completed'
                else: error = None
                if error:
                    assert not item['success'] and item['payload'] == error
                    expected_event = error.removeprefix('coord_')
                    pending = None; forwarded = None; answered = False
                    continue
                assert item['success'] and state == 'awaiting_proposal'
                completed += 1; state = 'completed'; expected_event = 'executed'
                record = next(r for r in CATALOG if r['key'] == key)
                report = fields(item['payload'])
                if index: artifact_due = True
                else: audit_execution(report, record, completed)
                count = (int(report['bytes']), int(report['tokens'])); executed.append(key)
                pending = None; forwarded = None; answered = False
            else:
                assert item['success'], item
                report = fields(item['payload'])
                if op == 'coord.start':
                    assert request_id == 1 and state == 'stopped' and argument == ('index' if index else None)
                    state = 'idle'; expected_event = 'started'
                    assert report['a_calls'] == '2' and report['b_calls'] == '1'
                elif op == 'coord.observe':
                    assert due == argument and due is not None
                    issued += 1
                    if key == due: expected_event = 'duplicate'
                    else:
                        key = due; ticket = issued; state = 'awaiting_proposal'; count = (0,0); expected_event = 'observed'
                        if index and key in cache:
                            state = 'completed'; expected_event = 'reused'; hits += 1; artifact_due = True
                            count = (next(r['byte_length'] for r in CATALOG if r['key'] == key), sum(check_index(cache[key], key).values()))
                    due = None
                    assert report['a_calls'] == '2' and report['b_calls'] == '0' and report['b_slices'] == '0'
                elif op == 'coord.status':
                    assert argument is None
                    assert report['a_calls'] == '2' and report['b_calls'] == '0' and report['b_slices'] == '0'
                    assert all(e[0] == 'A' for e in report['trace'].split(','))
                else:
                    assert op == 'coord.stop' and argument is None and pending is None
                    final_snapshot = snapshot(previous_report)
                    state = 'stopped'; key = None; ticket = 0; completed = 0; count = (0,0); expected_event = 'none'; stopped = True
            assert report['state'] == state and report['event'] == expected_event
            assert report['record'] == (key or 'none') and int(report['ticket']) == ticket
            assert int(report['completed']) == completed and (int(report['bytes']), int(report['tokens'])) == count
            if index and not stopped:
                assert report['operation'] == 'record.index'
                assert int(report['cache_hits']) == hits
                assert int(report['cached_records']) == len(cache) + (expected_event == 'executed' and artifact_due)
                if state != 'completed': assert report['output_bytes'] == '0'
            previous_report = report
        elif event == 'decision_requested':
            assert state == 'awaiting_proposal' and pending is None and not stopped
            calls += 1; assert item['call_id'] == calls <= 4
            metadata = next(r for r in CATALOG if r['key'] == key)
            context = dict(state_version=ticket, authorized_records=[[key, metadata['byte_length']]],
                           completed_results=[], required_record=key)
            if index: context['operation'] = 'record.index'
            assert item['snapshot'] == context
            pending = dict(ticket=ticket, key=key); answered = False
        elif event == 'decision_response':
            assert pending and not answered and item['call_id'] == calls and item['source'] == source
            assert item['snapshot_ticket'] == pending['ticket'] and item['latency_ms'] >= 0
            if source == 'replay': assert item['usage'] is None
            answered = True
        elif event == 'proposal_forwarded':
            assert pending and answered and forwarded is None
            forwarded = item['proposal']
            assert item['call_id'] == calls and forwarded['state_version'] == pending['ticket']
            assert forwarded['operation'] == ('record.index' if index else 'record.count')
            assert any(r['key'] == forwarded['record'] and r['read_allowed'] for r in CATALOG)
        elif event == 'guest_index':
            assert index and artifact_due and fetched is not None
            assert item == dict(event='guest_index', ticket=ticket, record=key, origin=expected_event, payload=fetched)
            artifact_due = False; fetched = None
        elif event == 'proposal_declined':
            assert pending and answered and item['call_id'] == calls
            assert item['proposal']['state_version'] == pending['ticket']
            assert item['proposal']['operation'] == 'decline' and item['proposal']['record'] == ''
            assert item['obsolete'] == (ticket != pending['ticket'])
            declined = not item['obsolete']; pending = None; answered = False
        else:
            assert event == 'guest_bridge_finished' and item is items[-1]
            assert stopped and pending is None and not artifact_due and item['snapshot'] == final_snapshot
            assert item['model_calls'] == calls and item['remaining_events'] == len(scenario['events'])-cursor
            if declined: assert item['status'] == 'declined' and final_snapshot['state'] == 'awaiting_proposal'
            else:
                assert item['status'] == 'observed_current_goal' and final_snapshot['state'] == 'completed'
                assert cursor == len(scenario['events'])
    assert items[-1]['event'] == 'guest_bridge_finished' and boots == 1 and stopped
    return executed, stale, calls


def verify(release):
    profile = 'release' if release else 'debug'; flags = ['--release'] if release else []
    host = next(line[6:] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags],cwd=ROOT,check=True)
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    binary = ROOT/f'target/host-meta/{host}/{profile}/rai-meta'
    kernel = ROOT/f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'
    def proposal(version,key,delay=10,operation='record.count'):
        return dict(delay_ms=delay,proposal=dict(state_version=version,operation=operation,record=key,reason='Offline test; not a model decision.'))
    cases = [
        ('revisions',[(0,'notes@1'),(400,'notes@1'),(1000,'notes@2'),(1400,'notes@2')], [proposal(1,'notes@1'),proposal(3,'notes@2')],(['notes@1','notes@2'],0,2)),
        ('stale',[(0,'notes@1'),(150,'notes@2')],[proposal(1,'notes@1',400),proposal(2,'notes@2')],(['notes@2'],1,2)),
        ('obsolete-decline',[(0,'notes@1'),(150,'notes@2')],[proposal(1,'',400,'decline'),proposal(2,'notes@2')],(['notes@2'],0,2)),
        ('decline',[(0,'notes@1')],[proposal(1,'',10,'decline')],([],0,1)),
        ('empty',[(0,'empty@1')],[proposal(1,'empty@1')],(['empty@1'],0,1)),
        ('same-record-again',[(0,'notes@1'),(500,'notes@2'),(1000,'notes@1')],[proposal(1,'notes@1'),proposal(2,'notes@2'),proposal(3,'notes@1')],(['notes@1','notes@2','notes@1'],0,3)),
    ]
    with tempfile.TemporaryDirectory(prefix='rai-guest-bridge-') as temp:
        temp=Path(temp); events=temp/'events.json'; replies=temp/'replies.json'
        env=dict(os.environ,MODEL_API_KEY='') # Replay cannot load credentials successfully.
        def run(kernel_path=kernel):
            return subprocess.run([str(binary),str(kernel_path),'--guest-replay',str(events),str(replies)],cwd=ROOT,env=env,text=True,capture_output=True,timeout=20)
        for name,schedule,decisions,expected in cases:
            scenario=dict(events=[dict(at_ms=t,record=k) for t,k in schedule])
            events.write_text(json.dumps(scenario));replies.write_text(json.dumps(decisions))
            result=run();assert result.returncode==0,result.stderr
            items=[json.loads(line) for line in result.stdout.splitlines()]
            assert audit(items,scenario,'replay')==expected
            (ROOT/f'target/verification/guest-bridge-{name}-{profile}.jsonl').write_text(result.stdout)
            if name=='revisions':
                for event,field,value in [('decision_requested','snapshot',{}),('guest_bridge_finished','snapshot',{}),('guest_bridge_finished','model_calls',0)]:
                    bad=copy.deepcopy(items);next(i for i in bad if i['event']==event)[field]=value
                    try: audit(bad,scenario,'replay')
                    except (AssertionError,KeyError): pass
                    else: raise AssertionError('Accepted fabricated '+field)
                bad=copy.deepcopy(items)
                item=next(i for i in bad if i['event']=='guest_exchange' and i['operation']=='coord.propose')
                item['payload']=item['payload'].replace('tokens=7','tokens=8')
                try: audit(bad,scenario,'replay')
                except AssertionError: pass
                else: raise AssertionError('Accepted fabricated count')
        events.write_text(json.dumps(dict(events=[dict(at_ms=0,record='notes@1')])))
        for name,decision,error,forwarded in [
            ('version',proposal(0,'notes@1'),'stale_proposal',False),
            ('denied',proposal(1,'restricted@1'),'record_not_allowed',False),
            ('injection',proposal(1,'notes@1\ncoord.stop'),'record_not_allowed',False),
            ('unsupported',proposal(1,'notes@1',operation='shell'),'operation_not_allowed',False),
            ('wrong-key',proposal(1,'notes@2'),'coord_wrong_record',True),
        ]:
            replies.write_text(json.dumps([decision]));result=run()
            assert result.returncode and json.loads(result.stderr)['error']==error,(name,result.stderr)
            items=[json.loads(line) for line in result.stdout.splitlines()]
            assert any(i['event']=='proposal_forwarded' for i in items)==forwarded
            assert not any(i['event']=='guest_exchange' and i['operation']=='coord.propose' and i['success'] for i in items)
            assert items[-1]['event']=='guest_exchange' and items[-1]['operation']=='coord.stop' and items[-1]['success']
        # Every call is explicit and bounded; no fifth provider call or hidden replay retry.
        events.write_text(json.dumps(dict(events=[dict(at_ms=i*500,record=key) for i,key in enumerate(['notes@1','notes@2','notes@1','notes@2','notes@1'])])))
        replies.write_text(json.dumps([proposal(i+1,key) for i,key in enumerate(['notes@1','notes@2','notes@1','notes@2'])]))
        result=run();assert result.returncode and json.loads(result.stderr)['error']=='model_call_budget_exhausted'
        items=[json.loads(line) for line in result.stdout.splitlines()]
        assert sum(i['event']=='decision_requested' for i in items)==4
        assert sum(i['event']=='guest_exchange' and i['operation']=='coord.propose' and i['success'] for i in items)==4
        assert items[-1]['operation']=='coord.stop' and items[-1]['success']
        # Invalid replay/scenario bounds fail before QEMU or credentials, even with a bad kernel path.
        for text,decisions,error in [
            ('{"events":[]}',[proposal(1,'notes@1')],'scenario_requires_1_to_8_events'),
            ('{"events":[{"at_ms":0,"record":"restricted@1"}]}',[proposal(1,'notes@1')],'record_not_allowed'),
            ('{"events":[{"at_ms":0,"record":"notes@1"}]}',[proposal(1,'notes@1',2001)],'invalid_replay_limits'),
        ]:
            events.write_text(text);replies.write_text(json.dumps(decisions));result=run('/nonexistent/should-not-boot')
            assert result.returncode and json.loads(result.stderr)['error']==error
            assert 'guest_boot_confirmed' not in result.stdout
    print(f'PASS ({profile}): guest-owned bridge, independent counts/IPC audit, stale rejection in EL0, duplicates, decline, invalid proposals and cleanup; no provider calls.')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--release',action='store_true')
    parser.add_argument('--check-live',type=Path)
    parser.add_argument('--scenario',type=Path,default=ROOT/'fixtures/guest_bridge/v1/observations.json')
    args=parser.parse_args()
    if args.check_live:
        result=audit([json.loads(line) for line in args.check_live.read_text().splitlines()],json.loads(args.scenario.read_text()),'meta')
        print('PASS live guest bridge audit; records/stale rejections/model calls:',result)
        print('Saved logs alone are not cryptographic proof of provider access.')
    else: verify(args.release)

if __name__=='__main__': main()
