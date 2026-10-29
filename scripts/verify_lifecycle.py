#!/usr/bin/env python3
"""Verify task lifecycle independently from the trace, results and actual QEMU faults."""
import argparse
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_user import CONTEXT
from verify_user_timeout import HELLO
from verify_schedule import check_report as check_schedule
from verify_timer import RESULT as TIMER_RESULT
import re

CASES = [None, 'spin', 'both-spin', 'invalid-exit', 'bad-call']


def checksum(seed, iterations):
    value = seed
    for i in range(iterations):
        value = (value * 33 + i) & ((1 << 64) - 1)
    return value


def check_report(reply, symbols, case=None, fault=False):
    pairs = [part.split('=', 1) for part in reply.split('; ')]
    assert all(len(p) == 2 for p in pairs), reply
    fields = dict(pairs)
    assert len(fields) == len(pairs), reply
    assert fields['tasks'] == '2' and fields['slice_ms'] == '20' and fields['max_slices'] == '6'
    states, slices, cursor = ['ready', 'ready'], [0, 0], 0
    trace = fields['trace'].split(',')
    assert 2 <= len(trace) <= 12
    for item in trace:
        assert len(item) == 2 and item[0] in 'AB' and item[1] in 'PEFL', item
        ready = [id for id in (cursor, cursor ^ 1) if states[id] == 'ready']
        assert ready and item[0] == 'AB'[ready[0]], (trace, states)
        id, event = ready[0], item[1]
        if event in 'PL':
            slices[id] += 1
            assert slices[id] <= 6
            if event == 'L':
                assert slices[id] == 6
                states[id] = 'limit_reached'
            else:
                assert slices[id] < 6
        else:
            states[id] = 'exited' if event == 'E' else 'faulted'
        cursor = id ^ 1
    assert 'ready' not in states
    expected = ['exited', 'exited']
    if fault or case in ('invalid-exit', 'bad-call'): expected[0] = 'faulted'
    if not fault and case in ('spin', 'both-spin'): expected[0] = 'limit_reached'
    if not fault and case == 'both-spin': expected[1] = 'limit_reached'
    assert states == expected, reply
    for id, name in enumerate('ab'):
        assert fields[f'{name}_state'] == states[id] and int(fields[f'{name}_slices']) == slices[id]
        if states[id] == 'exited':
            assert fields[f'{name}_code'] == '0'
            assert int(fields[f'{name}_value']) == checksum(*[(7, 1024), (11, 4096)][id]), reply
        elif states[id] == 'limit_reached':
            assert slices[id] == 6
        else:
            assert id == 0
            esr, pc, far, status, sp = [int(fields[f'a_{key}'], 16) for key in ['esr', 'pc', 'far_raw', 'spsr', 'sp']]
            assert status & ~0xf0000000 == 0x340
            bottom = symbols['__user_stack_bottom'][0]
            assert bottom <= sp <= bottom + 4096 and sp % 16 == 0
            if fault:
                assert fields['a_reason'] == 'exception'
                assert esr == 0x9200000f and far == 0x40080000
                assert pc == symbols['lifecycle_fault_instruction'][0]
            else:
                assert fields['a_reason'] == 'bad_syscall' and esr == 0x56000000
                assert symbols['__user_text_start'][0] <= pc < symbols['__user_text_end'][0]
    return fields


def build(profile, case):
    directory = ROOT / ('target' if case is None else f'target/lifecycle-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'lifecycle-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    for name in ['lifecycle_program_a', 'lifecycle_program_b', 'lifecycle_fault']:
        assert symbols['__user_text_start'][0] <= symbols[name][0] < symbols['__user_text_end'][0]
    assert symbols['__user_stack_top'][0] - symbols['__user_stack_bottom'][0] == 8192
    return kernel, symbols


def verify(profile, case):
    kernel, symbols = build(profile, case)
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='lifecycle') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        sequence = 0
        def request(command, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{command}\n'.encode())
            child.stdin.flush()
            fields = line().split('\t', 3)
            assert fields[:3] == ['@rai1', str(sequence), status], fields
            return fields[3]
        before = request('mmu.info')
        for _ in range(2):
            check_report(request('schedule.run'), symbols, case)
            check_report(request('schedule.fault'), symbols, case, fault=True)
            assert request('user.run') == HELLO
            assert request('context.check') == CONTEXT
            assert re.fullmatch(TIMER_RESULT, request('timer.check'))
            assert request('echo\tterminal task skipped') == 'terminal task skipped'
        check_schedule(request('schedule.check'))
        assert request('mmu.info') == before
        assert request('memory.check') == 'checked_pages=250; exhausted=true; zeroed=true; reused=true; free_pages=250'
        for command in ['schedule.run', 'schedule.fault']:
            for suffix in ['\t', '\textra']:
                assert request(command + suffix, 'err') == 'bad_args'
            child.stdin.write((command + ' extra\n').encode())
            child.stdin.flush()
            assert line() == command + ' extra'
            assert line() == 'error: unknown_command'
            if case is None:
                child.stdin.write((command + '\n').encode())
                child.stdin.flush()
                assert line() == command
                check_report(line(), symbols, fault=(command == 'schedule.fault'))
        assert request('echo\tafter lifecycle') == 'after lifecycle'
        print(f'PASS ({profile}/{case or "normal"}): independently replayed lifecycle/fairness/terminal trace, exact Rust results or expected faults/limits, repeat runs and kernel/user/timer/memory continuity.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    unit = OUTPUT / 'lifecycle-domain-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/schedule/lifecycle.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    for case in CASES: verify(profile, case)


if __name__ == '__main__':
    main()
