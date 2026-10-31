#!/usr/bin/env python3
"""Real-QEMU exact sets, revision invalidation, publication gating and EL0 goal attestation."""
import argparse
import copy
import itertools
import json
import subprocess
import sys
sys.dont_write_bytecode=True
from verify_mmu import ROOT,OUTPUT,guest
from verify_guest_coordinator import fields
from verify_coord_index import audit as audit_job
from check_bundle import check
from check_record_index import FIXTURE
from verify_user_timeout import HELLO

GOALS=[['brief@1','empty@1','notes@1'],['brief@1','checklist@1','empty@1','notes@1'],['brief@1','checklist@1','empty@1','notes@2']]


def bundle_fields(text): return dict(p.split('=',1) for p in text.split('; '))


def build(profile, feature=None):
    directory=ROOT/('target' if feature is None else f'target/bundle-self-test/{feature}')
    args=['cargo','build','--locked','--bin','rai-os','--target-dir',str(directory)]
    if profile=='release': args.append('--release')
    if feature: args += ['--features',feature]
    subprocess.run(args,cwd=ROOT,check=True)
    return directory/f'aarch64-unknown-none-softfloat/{profile}/rai-os'


def rejected(payload,keys,generation):
    try: check(payload,keys,generation)
    except (ValueError,TypeError,KeyError): return
    raise AssertionError('Accepted corrupted/incomplete/stale set')


def verify(profile,feature=None):
    with guest(build(profile,feature),f'{feature or "normal"}-{profile}',prefix='bundle') as (child,_,line,_,_):
        assert line()=='RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line()=='Console ready: type help for commands'
        serial=0
        def request(op,status='ok'):
            nonlocal serial
            serial+=1;child.stdin.write(f'@rai1\t{serial}\t{op}\n'.encode());child.stdin.flush()
            parts=line().split('\t',3);assert parts[:3]==['@rai1',str(serial),status],parts
            return parts[3]
        before=request('mmu.info')
        assert request('coord.bundle','err')=='coord_not_indexing'
        request('coord.start')
        assert request('coord.goal\tbrief@1','err')=='coord_not_indexing'
        request('coord.stop');request('coord.start\tindex')
        assert request('coord.bundle','err')=='coord_no_required_set'
        if feature=='coord-test-bad-goal':
            request('coord.observe\tnotes@1');request('coord.propose\t1:notes@1')
            assert request('coord.goal\tnotes@1','err')=='coord_bad_goal'
            report=fields(request('coord.status'))
            assert report['state']=='failed' and report['cached_records']=='0' and report['completed']=='1'
            assert request('coord.bundle','err')=='coord_not_running'
        else:
            completed=0;cached={};publications=[];previous_version=0
            expected=json.loads((FIXTURE/'expected.json').read_text())['goals']
            for n,keys in enumerate(GOALS):
                goal=bundle_fields(request('coord.goal\t'+','.join(reversed(keys))))
                version=int(goal['goal_version']);assert version>previous_version
                assert goal['required']==','.join(keys)
                assert goal['state']=='building' and int(goal['ready_records'])==len(set(keys)&cached.keys())
                assert goal['missing']==','.join(k for k in keys if k not in cached)
                assert int(goal['completed'])==completed
                report=fields(request('coord.status'))
                assert report['state']=='idle' and report['event']=='goal_changed'
                assert report['a_calls']=='3' and report['b_calls']=='0'
                assert [p for p in report['trace'].split(',') if p!='AP']==['AG','AO','AH']
                assert request('coord.bundle','err')=='coord_bundle_incomplete'
                assert request('coord.result','err')=='coord_result_unavailable'
                if n:
                    rejected(publications[-1],keys,version)
                    assert request('coord.propose\t1:brief@1','err')=='coord_no_goal'
                for key in keys:
                    if key in cached: continue
                    job=fields(request('coord.observe\t'+key))
                    ticket=job['ticket']
                    if n==2:
                        assert request('coord.propose\t4:notes@1','err')=='coord_not_required'
                    if feature=='index-stream-test-truncated' and key=='notes@2':
                        assert request(f'coord.propose\t{ticket}:{key}','err')=='coord_deadlocked'
                        break
                    result=fields(request(f'coord.propose\t{ticket}:{key}'))
                    completed+=1;payload=request('coord.result');cached[key]=payload
                    audit_job(result,payload,key,completed,len(cached),0)
                    request('timer.check');request('context.check')
                if feature and n==2: break
                assert completed==[3,4,5][n]
                status=bundle_fields(request('coord.bundle.status'))
                assert status['state']=='complete' and status['ready_records']==str(len(keys))
                payload=request('coord.bundle');index=check(payload,keys,version)
                assert index==expected[n]['index']
                obj=json.loads(payload)
                if n:
                    old={r['record']:r for r in json.loads(publications[-1])['records']}
                    for row in obj['records']:
                        if row['record'] in old: assert row==old[row['record']]
                publications.append(payload);previous_version=version
                (OUTPUT/f'bundle-goal-{n+1}-{profile}.json').write_text(payload+'\n')
                unchanged=request('coord.status');assert request('coord.bundle')==payload;assert request('coord.status')==unchanged
                for mutate in [lambda o:o['records'].pop(),lambda o:o['records'].append(copy.deepcopy(o['records'][0])),
                               lambda o:o.update(goal_version=version-1),lambda o:o['required'].reverse(),
                               lambda o:o['records'][0].update(source_hex='00'),lambda o:o['records'][0].update(index='wrong:1')]:
                    corrupt=copy.deepcopy(obj);mutate(corrupt);rejected(json.dumps(corrupt),keys,version)
                rejected(payload.replace('"format":','"goal_version":1,"format":',1),keys,version)
            if feature:
                failed=fields(request('coord.status'))
                assert failed['state']=='failed' and failed['completed']=='4' and failed['cached_records']=='0'
                assert request('coord.bundle','err')=='coord_not_running'
            else:
                # Invalid sets are rejected before changing the current requirement or goal version.
                status=request('coord.bundle.status');artifact=request('coord.bundle')
                for keys,error in [('', 'bad_args'),('notes@1,notes@1','duplicate_required_record'),('notes@1,notes@2','conflicting_revisions'),('restricted@1','record_denied'),('brief@9','unknown_record'),('brief@1,empty@1,notes@1,checklist@1,appendix@1','required_set_limit')]:
                    assert request('coord.goal\t'+keys,'err')==error
                    assert request('coord.bundle.status')==status and request('coord.bundle')==artifact
                for op in ['coord.goal\tbrief@1\textra','coord.bundle\textra','coord.bundle.status\t']:
                    assert request(op,'err')=='bad_args'
                assert request('coord.observe\tappendix@1','err')=='coord_not_required'
                assert fields(request('coord.status'))['b_calls']=='0'
                assert request('coord.bundle')==artifact
                # A new goal invalidates a pending proposal even if that record remains required.
                goal=bundle_fields(request('coord.goal\tappendix@1,empty@1'))
                old=fields(request('coord.observe\tappendix@1'))['ticket']
                request('coord.goal\tappendix@1')
                assert request(f'coord.propose\t{old}:appendix@1','err')=='coord_no_goal'
                job=fields(request('coord.observe\tappendix@1'))
                assert request(f'coord.propose\t{old}:appendix@1','err')=='coord_stale_proposal'
                request(f"coord.propose\t{job['ticket']}:appendix@1")
                # All 45 valid sets (1..4 records, one revision per document) reuse existing work.
                all_keys=['appendix@1','brief@1','checklist@1','empty@1','notes@1','notes@2'];sets=0
                for size in range(1,5):
                    for keys in itertools.combinations(all_keys,size):
                        if 'notes@1' in keys and 'notes@2' in keys: continue
                        goal=bundle_fields(request('coord.goal\t'+','.join(keys)));sets+=1
                        assert goal['state']=='complete' and goal['completed']=='6'
                        assert fields(request('coord.status'))['b_calls']=='0'
                        check(request('coord.bundle'),keys,int(goal['goal_version']))
                assert sets==45
                # Exact rollback is explicit, complete and uses no new job.
                goal=bundle_fields(request('coord.goal\t'+','.join(GOALS[0])))
                check(request('coord.bundle'),GOALS[0],int(goal['goal_version']))
                previous_version=int(goal['goal_version'])
                request('coord.stop');request('coord.start\tindex')
                goal=bundle_fields(request('coord.goal\t'+','.join(GOALS[0])))
                assert goal['ready_records']=='0' and int(goal['goal_version'])>previous_version
                assert request('coord.bundle','err')=='coord_bundle_incomplete'
                child.stdin.write(b'coord.bundle.status\n');child.stdin.flush()
                assert line()=='coord.bundle.status' and bundle_fields(line())==goal
        assert request('mmu.info')==before
        request('coord.stop');assert request('user.run')==HELLO
        assert 'free_pages=244' in request('memory.check')
        print(f'PASS bundle {profile}/{feature or "normal"}: {serial} requests; exact sources/indexes and publication/goal safety')


def graphical(profile):
    from verify_display import build,guest as display_guest
    with display_guest(build(profile),f'bundle-{profile}') as (request,capture):
        request('coord.start\tindex');request('coord.goal\t'+','.join(GOALS[0]));building=capture('building')
        for key in GOALS[0]:
            ticket=fields(request('coord.observe\t'+key))['ticket'];request(f'coord.propose\t{ticket}:{key}')
        first=capture('first-complete');assert first!=building
        middle=bundle_fields(request('coord.goal\t'+','.join(GOALS[1])))
        ticket=fields(request('coord.observe\tchecklist@1'))['ticket']
        request(f'coord.propose\t{ticket}:checklist@1')
        check(request('coord.bundle'),GOALS[1],int(middle['goal_version']))
        capture('second-complete')
        goal=bundle_fields(request('coord.goal\t'+','.join(GOALS[2])))
        changed=capture('revision-pending');assert changed!=first
        assert request('coord.bundle','err')=='coord_bundle_incomplete'
        assert capture('incomplete')!=first
        request('coord.bundle.status');assert capture('pending-restored')==changed
        for key in ['notes@2']:
            ticket=fields(request('coord.observe\t'+key))['ticket'];request(f'coord.propose\t{ticket}:{key}')
        check(request('coord.bundle'),GOALS[2],int(goal['goal_version']))
        final=capture('final-complete');assert final!=first and final!=changed
        request('coord.status');assert capture('stable')==final
        request('coord.stop');assert capture('stopped')!=final
    print(f'PASS bundle {profile}: native incomplete/complete/revision/stable/stop views')


def main():
    p=argparse.ArgumentParser();p.add_argument('--release',action='store_true');args=p.parse_args()
    profile='release' if args.release else 'debug';OUTPUT.mkdir(parents=True,exist_ok=True)
    for feature in [None,'coord-test-bad-goal','index-stream-test-truncated']: verify(profile,feature)
    graphical(profile)


if __name__=='__main__': main()
