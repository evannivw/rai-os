#!/usr/bin/env python3
"""Verify IRQ-driven EL0 context resumption, distinct state and failed-run cleanup."""
import argparse
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_timer import instruction, RESULT as TIMER_RESULT
from verify_user import CONTEXT
from verify_user_timeout import HELLO, check_timeout

RESULT = (r'tasks=2; slice_ms=20; irqs=12; switches=11; trace=ABABABABABAB; '
          r'a_turns=6; b_turns=6; a_ms=(\d+); b_ms=(\d+); progress=([0-9,]+); context=true; stacks=true')


def check_report(reply):
    match = re.fullmatch(RESULT, reply)
    assert match, reply
    assert all(120 <= int(ms) < 5000 for ms in match.group(1, 2)), reply
    values = [int(v) for v in match[3].split(',')]
    assert len(values) == 12
    for id in (0, 1):
        counters = values[id::2]
        assert counters[0] > 0 and all(a < b for a, b in zip(counters, counters[1:])), reply
    assert all(v % 3 == 0 for v in values[1::2]), reply


def build(profile, case):
    directory = ROOT / ('target' if case is None else f'target/schedule-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'schedule-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    for id, add in [('a', 0x91000400), ('b', 0x91000c00)]:
        name = 'schedule_program_' + id
        start = symbols[name][0]
        assert symbols['__user_text_start'][0] <= start < symbols['__user_text_end'][0]
        if case is None:
            assert symbols[name][1] == 12
            assert [instruction(kernel, start + i * 4) for i in range(3)] == [add, 0xf90003e0, 0x17fffffe]
    assert symbols['__user_stack_top'][0] - symbols['__user_stack_bottom'][0] == 8192
    assert symbols['exception_vectors'][1] == 2048
    return kernel, symbols


def verify(profile, case=None):
    kernel, symbols = build(profile, case)
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='schedule') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        sequence = 0
        def request(command, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{command}\n'.encode())
            child.stdin.flush()
            reply = line().split('\t', 3)
            assert reply[:3] == ['@rai1', str(sequence), status], reply
            return reply[3]
        before = request('mmu.info')
        for _ in range(3 if case is None else 2):
            if case:
                expected = 'schedule_context_corrupt' if case == 'corrupt-register' else 'schedule_user_fault'
                assert request('schedule.check', 'err') == expected
            else:
                check_report(request('schedule.check'))
            assert request('user.run') == HELLO
            assert request('context.check') == CONTEXT
            assert re.fullmatch(TIMER_RESULT, request('timer.check'))
            assert request('echo\tresumption checked') == 'resumption checked'
        check_timeout(request('user.spin'), symbols, 'spin')
        assert request('mmu.info') == before
        assert request('memory.check') == 'checked_pages=250; exhausted=true; zeroed=true; reused=true; free_pages=250'
        assert request('schedule.check\t', 'err') == 'bad_args'
        assert request('schedule.check\textra', 'err') == 'bad_args'
        child.stdin.write(b'schedule.check extra\n')
        child.stdin.flush()
        assert line() == 'schedule.check extra'
        assert line() == 'error: unknown_command'
        if case is None:
            child.stdin.write(b'schedule.check\n')
            child.stdin.flush()
            assert line() == 'schedule.check'
            check_report(line())
        assert request('echo\tstill alive') == 'still alive'
        print(f'PASS ({profile}/{case or "normal"}): real EL0 IRQ switching or expected rejection; observed counters, context/stack checks, repeat runs, timer cleanup and kernel/user/memory continuity.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    unit = OUTPUT / 'timer-budget-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/timer/budget.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    for case in [None, 'corrupt-register', 'user-fault']:
        verify(profile, case)


if __name__ == '__main__':
    main()
