#!/usr/bin/env python3
"""Capture native publication states from real QEMU and verify their observed authority."""
import argparse
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_display import build, guest
from verify_mmu import ROOT, OUTPUT
from verify_guest_coordinator import fields
from verify_bundle import bundle_fields
from check_bundle import check


def verify(profile, fault=False):
    if fault:
        directory=ROOT/'target/publication-display-fault'
        command=['cargo','build','--locked','--bin','rai-os','--features',
                 'guest-display,index-stream-test-truncated','--target-dir',str(directory)]
        if profile=='release': command.append('--release')
        subprocess.run(command,cwd=ROOT,check=True)
        kernel=directory/f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    else: kernel=build(profile)
    name=f'publication-view-{profile}'+('-fault' if fault else '')
    with guest(kernel,name) as (request,capture):
        snapshots={}
        def snap(label, state, availability):
            status=bundle_fields(request('coord.publish.status'))
            assert status['state']==state and status['availability']==availability,status
            frame=capture(label)
            if snapshots:
                assert frame[:538*1440]==next(iter(snapshots.values()))[:538*1440], 'Identity area changed'
            snapshots[label]=frame
            return frame
        def job(key):
            token=fields(request('coord.observe\t'+key))['ticket']
            return request(f'coord.propose\t{token}:{key}')
        def prepare(keys, version):
            payload=request('coord.publish.prepare')
            token=bundle_fields(request('coord.publish.status'))['candidate_ticket']
            check(payload,keys,version)
            return payload,token
        request('coord.publish.status');empty=snap('empty','empty','none')
        request('coord.start\tindex')
        keys=['brief@1','empty@1','notes@1']
        goal=bundle_fields(request('coord.goal\t'+','.join(keys)));version=int(goal['goal_version'])
        preparing=snap('preparing','building','none');assert preparing!=empty
        for key in keys: job(key)
        unverified=snap('indexed-unverified','building','none');assert unverified!=preparing
        assert request('coord.publish.read','err')=='publication_unavailable'
        payload,token=prepare(keys,version)
        verifying=snap('verifying-first','validating','none');assert verifying!=unverified
        request('coord.publish.commit\t'+token)
        current=snap('current-first','current','current');assert current!=verifying
        assert request('coord.publish.read')==payload
        assert capture('read-current')==current
        # Invalid parsed and execution requests preserve state; notices cannot create failure/success.
        assert request('coord.publish.commit\t0','err')=='bad_args'
        assert capture('invalid-notice')!=current
        assert snap('current-restored','current','current')==current
        keys=['brief@1','empty@1','notes@2']
        goal=bundle_fields(request('coord.goal\t'+','.join(keys)));version=int(goal['goal_version'])
        assert goal['ready_records']=='2' and goal['missing']=='notes@2'
        updating=snap('updating','building','historical');assert updating!=current
        assert request('coord.publish.read')==payload and capture('read-previous')==updating
        if fault:
            token=fields(request('coord.observe\tnotes@2'))['ticket']
            assert request(f'coord.propose\t{token}:notes@2','err')=='coord_deadlocked'
            failure=capture('worker-failure-notice')
            blocked=snap('worker-blocked','blocked','historical');assert blocked!=updating and failure!=updating
            assert bundle_fields(request('coord.publish.status'))['reason']=='coord_deadlocked'
            assert request('coord.publish.read')==payload
            assert capture('worker-failure-read')==blocked
        else:
            job('notes@2')
            candidate,token=prepare(keys,version)
            snap('verifying-update','validating','historical')
            # Explicitly scripted host rejection, not an undisclosed random defect.
            request('coord.publish.reject\t'+token)
            blocked=snap('blocked','blocked','historical')
            assert blocked!=updating and request('coord.publish.read')==payload
            assert capture('read-blocked')==blocked
            candidate,fresh=prepare(keys,version)
            baseline=snap('retry','validating','historical')
            assert request('coord.publish.commit\t'+token,'err')=='publication_stale_candidate'
            late=capture('late-notice');assert late!=baseline
            assert snap('retry-restored','validating','historical')==baseline
            assert request('coord.publish.read')==payload
            check(candidate,keys,version)
            request('coord.publish.commit\t'+fresh)
            final=snap('current-updated','current','current');assert final!=current and final!=blocked
            assert request('coord.publish.read')==candidate and capture('read-updated')==final
            assert fields(request('coord.status'))['completed']=='4'
            assert capture('status-stable')==final
            # Already indexed is not equivalent to current after a new goal generation.
            request('coord.goal\t'+','.join(keys))
            cached=snap('cached-new-goal','building','historical');assert cached!=final
        request('coord.stop')
        assert snap('stopped','empty','none')==empty
        print(f'PASS {name}: native states, exact retained bytes, notices, progress and stop')


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--release',action='store_true')
    args=parser.parse_args();profile='release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True,exist_ok=True)
    verify(profile);verify(profile,True)


if __name__=='__main__': main()
