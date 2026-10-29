#!/usr/bin/env python3
"""Replay real blocking-receive traces: blocked tasks must never be dispatched."""
import argparse
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT,OUTPUT,guest
from verify_exceptions import elf_symbols
from verify_ipc import check_report as check_ipc
from verify_isolation import check_report as check_isolation
from verify_schedule import check_report as check_schedule
from verify_user_timeout import HELLO
CASES=[None,'both','peer-exit','peer-fault','peer-spin','call-limit','queued','invalid-send']


def check_report(reply,symbols,case=None):
    pairs=[p.split('=',1) for p in reply.split('; ')]
    assert all(len(p)==2 for p in pairs),reply
    fields=dict(pairs);assert len(fields)==len(pairs)
    for key,value in [('tasks','2'),('slice_ms','20'),('max_slices','6'),('ipc_capacity','2'),('max_calls','16')]:
        assert fields[key]==value
    states=['ready','ready'];calls=[0,0];slices=[0,0];cursor=0
    observed=dict(blocked=0,woken=0,closed_wakes=0,deadlocked=0)
    final_calls=[int(fields[f'{name}_calls']) for name in 'ab']
    trace=fields['trace'].split(',');assert 2<=len(trace)<=92
    handled=0
    last_dispatch=None
    for entry in trace:
        assert len(entry)==2 and entry[0] in 'AB',entry
        id='AB'.index(entry[0]);event=entry[1]
        if event in 'WC':
            assert states[id]=='waiting',trace
            if event=='W':assert last_dispatch==((id^1),'S'),trace
            else:assert states[id^1] in ['exited','faulted','limit_reached'],trace
            states[id]='ready'
            observed['woken' if event=='W' else 'closed_wakes']+=1
            handled+=1
            continue
        if event=='X':
            assert 'ready' not in states and states[id]=='waiting',trace
            assert states[id^1] in ['waiting','deadlocked'],trace
            states[id]='deadlocked';observed['deadlocked']+=1
            continue
        ready=[n for n in (cursor,cursor^1) if states[n]=='ready']
        assert ready and id==ready[0],(trace,states)
        if event in 'SRD':
            calls[id]+=1;assert calls[id]<16
            if event=='D':states[id]='waiting';observed['blocked']+=1
            else:handled+=1
        elif event=='P':
            slices[id]+=1;assert slices[id]<6
        elif event=='L':
            if final_calls[id]==16:
                calls[id]+=1;assert calls[id]==16
            else:
                slices[id]+=1;assert slices[id]==6
            states[id]='limit_reached'
        else:
            assert event in 'EF'
            states[id]='exited' if event=='E' else 'faulted'
        cursor=id^1
        last_dispatch=(id,event)
    assert all(s in ['exited','faulted','limit_reached','deadlocked'] for s in states)
    assert calls==final_calls
    for key,value in observed.items():assert int(fields[key])==value,reply
    totals={key:int(fields[key]) for key in ['sent','received','rejected','empty']}
    assert sum(totals.values())==handled
    assert totals['received']<=totals['sent']
    expected_states=['exited','exited'];values=[41,42]
    if case=='both': expected_states=['deadlocked','deadlocked'];values=[None,None]
    elif case in ['peer-exit','peer-fault','peer-spin','call-limit']:
        expected_states=['exited', {'peer-exit':'exited','peer-fault':'faulted'}.get(case,'limit_reached')]
        values=[5,99 if case=='peer-exit' else None]
    elif case=='queued':values=[42,41]
    assert states==expected_states,reply
    for id,name in enumerate('ab'):
        assert fields[f'{name}_state']==states[id]
        assert int(fields[f'{name}_slices'])==slices[id]
        if states[id]=='exited':
            assert fields[f'{name}_code']=='0' and int(fields[f'{name}_value'])==values[id],reply
    if case=='both':
        assert calls==[1,1] and observed==dict(blocked=2,woken=0,closed_wakes=0,deadlocked=2)
        assert totals==dict(sent=0,received=0,rejected=0,empty=0)
    elif case in ['peer-exit','peer-fault','peer-spin','call-limit']:
        assert calls==[1,16 if case=='call-limit' else 0]
        assert observed==dict(blocked=1,woken=0,closed_wakes=1,deadlocked=0)
        assert totals==dict(sent=0,received=0,rejected=1,empty=15 if case=='call-limit' else 0)
        assert slices[0]==0 # blocked A got no CPU turns while B ran or spun
        if case=='peer-spin':assert slices[1]==6
    else:
        assert calls==[2,3 if case=='invalid-send' else 2]
        assert observed==dict(blocked=1,woken=1,closed_wakes=0,deadlocked=0)
        assert totals==dict(sent=2,received=2,rejected=1 if case=='invalid-send' else 0,empty=0)
    if case=='peer-fault':
        assert fields['b_reason']=='exception'
        assert int(fields['b_esr'],16)==0x9200000f
        assert int(fields['b_pc'],16)==symbols['lifecycle_fault_instruction'][0]
        assert int(fields['b_far_raw'],16)==0x40080000


def verify(profile,case):
    directory=ROOT/('target' if case is None else f'target/wait-self-test/{case}')
    command=['cargo','build','--locked','--bin','rai-os','--target-dir',str(directory)]
    if profile=='release':command.append('--release')
    if case:command+=['--features',f'wait-test-{case}']
    subprocess.run(command,cwd=ROOT,check=True)
    kernel=directory/f'aarch64-unknown-none-softfloat/{profile}/rai-os';symbols=elf_symbols(kernel)
    for name in ['wait_program_a','wait_program_b']:
        assert symbols['__user_text_start'][0]<=symbols[name][0]<symbols['__user_text_end'][0]
    with guest(kernel,f'{case or "normal"}-{profile}',prefix='wait') as (child,_,line,_,transcript):
        assert line()=='RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line()=='Console ready: type help for commands'
        sequence=0
        def request(command,status='ok'):
            nonlocal sequence
            sequence+=1
            child.stdin.write(f'@rai1\t{sequence}\t{command}\n'.encode());child.stdin.flush()
            parts=line().split('\t',3);assert parts[:3]==['@rai1',str(sequence),status],parts
            return parts[3]
        before=request('mmu.info')
        for _ in range(3):
            check_report(request('ipc.wait'),symbols,case)
            check_ipc(request('ipc.check'),symbols)
            assert request('user.run')==HELLO
            assert request('mmu.info')==before
        check_schedule(request('schedule.check'))
        check_isolation(request('schedule.isolate'),symbols)
        assert request('memory.check')=='checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244'
        for extra in ['\t','\textra']:assert request('ipc.wait'+extra,'err')=='bad_args'
        child.stdin.write(b'ipc.wait extra\n');child.stdin.flush()
        assert line()=='ipc.wait extra' and line()=='error: unknown_command'
        if case is None:
            child.stdin.write(b'ipc.wait\n');child.stdin.flush();assert line()=='ipc.wait'
            check_report(line(),symbols)
        assert request('mmu.info')==before
        assert request('echo\tafter wake')=='after wake'
        print(f'PASS ({profile}/{case or "normal"}): replayed blocking/wakeup/closure/deadlock transitions, exact task results, no waiter dispatch, fresh repeated runs and IPC/MMU/isolation/kernel continuity.')
        print(transcript.decode('ascii'),end='')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--release',action='store_true');args=parser.parse_args()
    OUTPUT.mkdir(parents=True,exist_ok=True)
    for source,name in [('src/ipc.rs','wait-mailboxes'),('src/schedule/lifecycle.rs','wait-lifecycle')]:
        binary=OUTPUT/f'{name}-tests'
        subprocess.run(['rustc','--edition=2024','--test',source,'-o',str(binary)],cwd=ROOT,check=True)
        subprocess.run([str(binary)],check=True)
    for case in CASES:verify('release' if args.release else 'debug',case)


if __name__=='__main__':main()
