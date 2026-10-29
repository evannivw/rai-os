#!/usr/bin/env python3
"""Check actual EL0 IRQ termination, absolute syscall budgets and cleanup in QEMU."""
import argparse
import re
import subprocess
import sys
import time
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_user import build, CONTEXT
from verify_timer import instruction, RESULT as TIMER_RESULT

TIMEOUT = (r'mode=EL0; state=timed_out; budget_ms=100; elapsed_ms=(\d+); vector=([89]); '
           r'pc=0x([0-9a-f]{16}); spsr=0x([0-9a-f]{16}); sp=0x([0-9a-f]{16}); syscalls=(\d+)')
HELLO = 'mode=EL0; state=exited; code=0; syscalls=2; rejected=0; output=Hello from EL0'


def check_timeout(reply, symbols, kind):
    match = re.fullmatch(TIMEOUT, reply)
    assert match, reply
    elapsed, vector = int(match[1]), int(match[2])
    pc, state, sp = [int(match[i], 16) for i in (3, 4, 5)]
    calls = int(match[6])
    assert 100 <= elapsed < 2000, reply  # Broad failure bound, not a latency benchmark.
    assert state & ~0xf0000000 == 0x340, reply
    assert symbols['__user_stack_bottom'][0] <= sp <= symbols['__user_stack_top'][0]
    assert sp % 16 == 0
    assert symbols['__user_text_start'][0] <= pc < symbols['__user_text_end'][0]
    if kind == 'spin':
        assert vector == 9 and calls == 0 and pc == symbols['user_spin'][0], reply
        assert sp == symbols['__user_stack_top'][0], reply
    elif kind == 'expired-syscall':
        assert vector == 8 and calls == 0, reply
    else:
        assert calls > 0, reply
        assert symbols['user_syscall_loop'][0] <= pc < symbols['user_syscall_loop_end'][0], reply


def verify(profile, case=None):
    kernel, symbols = build(profile, case)
    assert instruction(kernel, symbols['user_spin'][0]) == 0x14000000
    assert symbols['user_spin'][1] == 4  # One instruction; no voluntary return/poll/SVC.
    if case is None:
        assert 'user_syscall_loop' not in symbols
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='user-timeout') as (child, poll, line, pending, transcript):
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
            check_timeout(request('user.spin' if case is None else 'user.run'), symbols, case or 'spin')
            assert request('context.check') == CONTEXT
            assert re.fullmatch(TIMER_RESULT, request('timer.check'))
            assert request('echo\tafter timeout') == 'after timeout'
            if case is None:
                assert request('user.run') == HELLO
        # A stopped deadline must not surface later or contaminate the next owner.
        time.sleep(.15)
        assert not pending and not poll.select(0), 'Unexpected late output'
        assert request('echo\tafter old deadline') == 'after old deadline'
        assert request('mmu.info') == before
        assert request('memory.check') == 'checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244'
        for command in ['user.spin\t', 'user.spin\textra']:
            assert request(command, 'err') == 'bad_args'
        child.stdin.write(b'user.spin extra\n')
        child.stdin.flush()
        assert line() == 'user.spin extra'
        assert line() == 'error: unknown_command'
        child.stdin.write(b'user.spin\n')
        child.stdin.flush()
        assert line() == 'user.spin'
        check_timeout(line(), symbols, 'spin')
        assert request('echo\tconsole remains usable') == 'console remains usable'
        print(f'PASS ({profile}/{case or "normal"}): actual timeout, saved EL0 PC/state/SP, fixed deadline, repeated invocation, timer/context/memory/console continuity and invalid arguments.')
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
    for case in [None, 'syscall-loop', 'expired-syscall']:
        verify(profile, case)


if __name__ == '__main__':
    main()
