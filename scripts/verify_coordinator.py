#!/usr/bin/env python3
"""Replay coordinator evidence independently; real-QEMU offline integration tests."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from verify_service import CATALOG, check as check_service
ROOT = Path(__file__).resolve().parents[1]


def audit(items, scenario, source):
    first = items[0]
    assert first['event'] == 'coordinator_started' and first['source'] == source
    assert first['event_source'] == 'scripted_release_fixture' and first['max_model_calls'] == 4
    assert first['model'] == ('muse-spark-1.3' if source == 'meta' else None)
    version = 0; latest = None; released = []; completed = {}; index = 0; calls = 0
    pending = None; responded = False; dispatched = None; executed = []; boots = 0; stale = 0
    def snapshot():
        return dict(state_version=version, required_record=latest,
                    authorized_records=[[key, next(r['byte_length'] for r in CATALOG if r['key'] == key)] for key in released],
                    completed_results=list(completed.values()))
    for item in items[1:]:
        event = item['event']
        if event == 'release_applied':
            expected = scenario['events'][index]
            assert item['index'] == index and item['record'] == expected['record']
            assert item['scheduled_ms'] == expected['at_ms'] <= item['elapsed_ms']
            key = expected['record']
            changed = latest is None or int(key[-1]) > int(latest[-1])
            if changed:
                latest = key; released.append(key); version += 1
            assert item['changed'] == changed and item['state_version'] == version
            index += 1
        elif event == 'decision_requested':
            assert pending is None and dispatched is None and latest not in completed
            assert item['snapshot'] == snapshot(), item
            calls += 1; assert item['call_id'] == calls <= 4
            pending = dict(version=version, record=latest); responded = False
        elif event == 'decision_response':
            assert pending and not responded and item['call_id'] == calls
            assert item['snapshot_version'] == pending['version'] and item['source'] == source
            if source == 'replay': assert item['usage'] is None
            responded = True
        elif event == 'stale_proposal_discarded':
            assert pending and responded and pending['version'] != version
            assert item['snapshot_version'] == pending['version'] and item['state_version'] == version
            pending = None; stale += 1
        elif event == 'guest_boot_confirmed':
            boots += 1; assert boots == 1
        elif event == 'operation_dispatched':
            assert pending and responded and dispatched is None and boots == 1
            proposal = item['proposal']
            assert proposal['state_version'] == pending['version'] == version
            assert proposal['operation'] == 'record.count' and proposal['record'] == latest
            assert latest not in completed and latest not in executed
            assert item['request_id'] == len(executed) + 1
            executed.append(latest); dispatched = (item['request_id'], latest); pending = None
        elif event == 'observation_recorded':
            assert dispatched and item['request_id'] == dispatched[0]
            key = dispatched[1]
            record = next(r for r in CATALOG if r['key'] == key)
            fields = check_service(item['payload'], record)
            observed = dict(record=key, bytes=int(fields['bytes']), tokens=int(fields['tokens']))
            assert item['observed'] == observed and key not in completed
            completed[key] = observed; version += 1; dispatched = None
            assert item['state_version'] == version
            assert item['pending_record'] == (None if latest in completed else latest)
        elif event == 'coordinator_finished':
            assert item is items[-1]
            assert item['status'] == 'observed_current_revision'
            assert index == len(scenario['events']) and pending is None and dispatched is None
            assert latest in completed and item['snapshot'] == snapshot()
            assert item['model_calls'] == calls
        else: raise AssertionError('Unexpected event: ' + event)
    assert items[-1]['event'] == 'coordinator_finished'
    return executed, stale, calls


def verify(release):
    profile = 'release' if release else 'debug'
    host = next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    flags = ['--release'] if release else []
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags],cwd=ROOT,check=True)
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    binary = ROOT / f'target/host-meta/{host}/{profile}/rai-meta'
    kernel = ROOT / f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'
    def proposal(version,key,delay=10):
        return dict(delay_ms=delay,proposal=dict(state_version=version,operation='record.count',record=key,reason='Offline coordinator fixture; not a model decision.'))
    cases = [
        ('revisions',[(0,'notes@1'),(300,'notes@1'),(1000,'notes@2'),(1400,'notes@2')], [proposal(1,'notes@1'),proposal(3,'notes@2')], ['notes@1','notes@2'],0),
        ('stale',[(0,'notes@1'),(150,'notes@2')],[proposal(1,'notes@1',400),proposal(2,'notes@2')],['notes@2'],1),
    ]
    with tempfile.TemporaryDirectory(prefix='rai-coordinator-') as temp:
        temp = Path(temp); event_file=temp/'events.json'; decisions=temp/'decisions.json'
        # An empty environment key would fail any credential load. Replay must not
        # read the user's real file or contact the provider.
        env=dict(os.environ,MODEL_API_KEY='')
        for name, schedule, replies, expected_keys, expected_stale in cases:
            scenario=dict(events=[dict(at_ms=t,record=k) for t,k in schedule])
            event_file.write_text(json.dumps(scenario)); decisions.write_text(json.dumps(replies))
            result=subprocess.run([str(binary),str(kernel),'--coordinate-replay',str(event_file),str(decisions)],cwd=ROOT,env=env,text=True,capture_output=True,timeout=20)
            assert result.returncode==0,result.stderr
            items=[json.loads(line) for line in result.stdout.splitlines()]
            assert audit(items,scenario,'replay')==(expected_keys,expected_stale,2)
            (ROOT/f'target/verification/coordinator-{name}-{profile}.jsonl').write_text(result.stdout)
            # The auditor must reject altered history, a false count and a fake
            # final state. All mutations are synthetic; originals stay untouched.
            for event,field,value in [('observation_recorded','state_version',999),('decision_requested','snapshot',{}),('coordinator_finished','model_calls',0)]:
                bad=json.loads(json.dumps(items))
                next(item for item in bad if item['event']==event)[field]=value
                try: audit(bad,scenario,'replay')
                except (AssertionError,KeyError): pass
                else: raise AssertionError('Auditor accepted altered '+event)
        event_file.write_text(json.dumps(dict(events=[dict(at_ms=0,record='notes@1')])))
        for name,reply,error in [('future',proposal(1,'notes@2'),'record_not_pending'),('version',proposal(0,'notes@1'),'stale_proposal')]:
            decisions.write_text(json.dumps([reply]))
            result=subprocess.run([str(binary),'/nonexistent/should-not-start','--coordinate-replay',str(event_file),str(decisions)],cwd=ROOT,env=env,text=True,capture_output=True,timeout=5)
            assert result.returncode and json.loads(result.stderr)['error']==error
            assert 'guest_boot_confirmed' not in result.stdout
        decline=proposal(1,'');decline['proposal']['operation']='decline'
        decisions.write_text(json.dumps([decline]))
        result=subprocess.run([str(binary),'/nonexistent/should-not-start','--coordinate-replay',str(event_file),str(decisions)],cwd=ROOT,env=env,text=True,capture_output=True,timeout=5)
        assert result.returncode==0 and json.loads(result.stdout.splitlines()[-1])['status']=='declined'
        assert 'guest_boot_confirmed' not in result.stdout
    print(f'PASS ({profile}): revision/duplicate loop, delayed stale response, independent result/history audit, bad proposals and decline without execution. Offline replay only.')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--release',action='store_true')
    parser.add_argument('--check-live',type=Path);parser.add_argument('--scenario',type=Path,default=ROOT/'fixtures/coordinator/v1/revisions.json')
    args=parser.parse_args()
    if args.check_live:
        items=[json.loads(line) for line in args.check_live.read_text().splitlines()]
        outcome=audit(items,json.loads(args.scenario.read_text()),'meta')
        print('PASS: live capture independently audited; records/stale discards/proposals:',outcome)
        print('A saved log alone is not cryptographic proof of provider access.')
    else:verify(args.release)


if __name__=='__main__':main()
