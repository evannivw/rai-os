#!/usr/bin/env python3
"""Verify real EL0 message exchange, bounded failure paths, and continued isolation."""
import argparse
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_lifecycle import check_report as check_lifecycle
from verify_schedule import check_report as check_schedule
from verify_isolation import check_report as check_isolation
from verify_user_timeout import HELLO
CASES = [None, 'rejections', 'flood', 'peer-exit', 'peer-fault']


def check_report(reply, symbols, case=None):
    pairs = [p.split('=', 1) for p in reply.split('; ')]
    assert all(len(p) == 2 for p in pairs), reply
    fields = dict(pairs)
    assert len(fields) == len(pairs), reply
    for key, value in [('tasks','2'), ('slice_ms','20'), ('max_slices','6'), ('ipc_capacity','2'), ('max_calls','16')]:
        assert fields[key] == value, reply
    states, slices, calls, cursor = ['ready']*2, [0,0], [0,0], 0
    final_calls = [int(fields[f'{name}_calls']) for name in 'ab']
    trace = fields['trace'].split(',')
    assert 2 <= len(trace) <= 46
    serviced = 0
    for item in trace:
        ready = [id for id in (cursor, cursor ^ 1) if states[id] == 'ready']
        assert ready and len(item) == 2 and item[0] == 'AB'[ready[0]], (trace, states)
        id, event = ready[0], item[1]
        if event in 'SR':
            calls[id] += 1
            serviced += 1
            assert calls[id] < 16
        elif event == 'P':
            slices[id] += 1
            assert slices[id] < 6
        elif event == 'L':
            if final_calls[id] == 16:
                calls[id] += 1
                assert calls[id] == 16
            else:
                slices[id] += 1
                assert slices[id] == 6
            states[id] = 'limit_reached'
        else:
            assert event in 'EF'
            states[id] = 'exited' if event == 'E' else 'faulted'
        cursor = id ^ 1
    assert 'ready' not in states and calls == final_calls
    totals = {key: int(fields[key]) for key in ['sent','received','rejected','empty']}
    assert sum(totals.values()) == serviced
    assert totals['received'] <= totals['sent']
    expected_states = {'rejections': ['exited','limit_reached'], 'flood': ['limit_reached','exited'],
                       'peer-fault': ['exited','faulted']}.get(case, ['exited','exited'])
    assert states == expected_states, reply
    expected_values = {'rejections': [10,None], 'flood': [None,99], 'peer-exit': [5,99],
                       'peer-fault': [5,None]}.get(case, [42,41])
    for id,name in enumerate('ab'):
        assert fields[f'{name}_state'] == states[id]
        assert int(fields[f'{name}_slices']) == slices[id]
        if states[id] == 'exited':
            assert fields[f'{name}_code'] == '0'
            assert int(fields[f'{name}_value']) == expected_values[id]
    if case is None:
        assert totals['sent'] == totals['received'] == 2 and totals['rejected'] == 0
    elif case == 'rejections':
        assert calls == [10,16]
        assert totals == dict(sent=2,received=0,rejected=21,empty=2)
    elif case == 'flood':
        assert calls == [16,0]
        assert totals == dict(sent=0,received=0,rejected=0,empty=15)
    else:
        assert calls == [2,0]
        assert totals == dict(sent=0,received=0,rejected=1,empty=1)
    if case == 'peer-fault':
        assert fields['b_reason'] == 'exception'
        assert int(fields['b_esr'],16) == 0x9200000f
        assert int(fields['b_far_raw'],16) == 0x40080000
        assert int(fields['b_pc'],16) == symbols['lifecycle_fault_instruction'][0]


def verify(profile, case):
    directory = ROOT / ('target' if case is None else f'target/ipc-self-test/{case}')
    command = ['cargo','build','--locked','--bin','rai-os','--target-dir',str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features',f'ipc-test-{case}']
    subprocess.run(command,cwd=ROOT,check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    for name in ['ipc_program_a','ipc_program_b']:
        assert symbols['__user_text_start'][0] <= symbols[name][0] < symbols['__user_text_end'][0]
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='ipc') as (child,_,line,_,transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        sequence = 0
        def request(command, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{command}\n'.encode());child.stdin.flush()
            parts = line().split('\t',3)
            assert parts[:3] == ['@rai1',str(sequence),status],parts
            return parts[3]
        before = request('mmu.info')
        for _ in range(3):
            check_report(request('ipc.check'),symbols,case)
            assert request('mmu.info') == before
            check_lifecycle(request('schedule.run'),symbols)
            check_lifecycle(request('schedule.fault'),symbols,fault=True)
            assert request('user.run') == HELLO
        check_schedule(request('schedule.check'))
        check_isolation(request('schedule.isolate'),symbols)
        assert request('memory.check') == 'checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244'
        for extra in ['\t','\textra']:
            assert request('ipc.check'+extra,'err') == 'bad_args'
        child.stdin.write(b'ipc.check extra\n');child.stdin.flush()
        assert line() == 'ipc.check extra' and line() == 'error: unknown_command'
        if case is None:
            child.stdin.write(b'ipc.check\n');child.stdin.flush()
            assert line() == 'ipc.check'
            check_report(line(),symbols)
        assert request('mmu.info') == before
        assert request('echo\tafter IPC') == 'after IPC'
        print(f'PASS ({profile}/{case or "normal"}): real EL0 IPC, expected results/errors, replayed scheduling/call bounds, fresh queues, preserved task isolation and kernel/user continuity.')
        print(transcript.decode('ascii'),end='')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--release',action='store_true');args=parser.parse_args()
    OUTPUT.mkdir(parents=True,exist_ok=True)
    for source,name in [('src/ipc.rs','mailboxes'),('src/schedule/lifecycle.rs','ipc-lifecycle')]:
        binary=OUTPUT/f'{name}-tests'
        subprocess.run(['rustc','--edition=2024','--test',source,'-o',str(binary)],cwd=ROOT,check=True)
        subprocess.run([str(binary)],check=True)
    for case in CASES:verify('release' if args.release else 'debug',case)


if __name__ == '__main__':main()
