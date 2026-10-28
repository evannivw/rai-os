#!/usr/bin/env python3
"""Verify one real EL0 Rust program, bounded syscalls and fault containment."""
import argparse
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_exceptions import elf_symbols
from verify_mmu import ROOT, OUTPUT, guest

CASES = ['invalid-calls', 'read-kernel', 'write-code', 'uart', 'execute-stack', 'privileged']
CONTEXT = 'round_trips=2; registers=31; flags=true; stack=true; resumed=true'


def build(profile, case=None):
    directory = ROOT / ('target' if case is None else f'target/user-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'user-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    address = lambda name: symbols[name][0]
    bounds = [address(name) for name in ['__user_text_start', '__user_text_end', '__user_rodata_end', '__user_stack_bottom', '__user_stack_top', '__page_pool_start']]
    assert bounds == sorted(set(bounds)) and all(n % 4096 == 0 for n in bounds)
    assert bounds[0] >= address('__exception_stack_top')
    assert bounds[3] == bounds[2] + 4096 and bounds[4] - bounds[3] == 8192
    assert bounds[5] >= bounds[4] + 4096
    assert bounds[0] <= address('user_program') < bounds[1]
    assert symbols['exception_vectors'][1] == 2048
    if case is None: assert 'user_fault_instruction' not in symbols
    return kernel, symbols


def verify(profile, case=None):
    kernel, symbols = build(profile, case)
    name = case or 'normal'
    with guest(kernel, f'{name}-{profile}', prefix='user') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        sequence = 0
        def request(command, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{command}\n'.encode())
            child.stdin.flush()
            actual = line().split('\t', 3)
            assert actual[:3] == ['@rai1', str(sequence), status], actual
            return actual[3]
        before = request('mmu.info')
        assert before.startswith('enabled=true;')
        for _ in range(2):
            result = request('user.run')
            if case in (None, 'invalid-calls'):
                calls, rejected = (10, 8) if case else (2, 0)
                assert result == f'mode=EL0; state=exited; code=0; syscalls={calls}; rejected={rejected}; output=Hello from EL0', result
            else:
                match = re.fullmatch(r'mode=EL0; state=faulted; esr=0x([0-9a-f]{16}); pc=0x([0-9a-f]{16}); far_raw=0x([0-9a-f]{16}); spsr=0x([0-9a-f]{16}); sp=0x([0-9a-f]{16}); syscalls=0', result)
                assert match, result
                esr, pc, far, spsr, sp = [int(v,16) for v in match.groups()]
                assert spsr & ~0xf0000000 == 0x3c0, result
                assert symbols['__user_stack_bottom'][0] <= sp <= symbols['__user_stack_top'][0] and sp % 16 == 0
                if case == 'privileged':
                    assert esr == 0x02000000, result  # Undefined privileged access at EL0.
                else:
                    expected = 0x8200000f if case == 'execute-stack' else 0x9200000f
                    if case == 'write-code': expected |= 1 << 6
                    assert esr == expected, result
                    expected_far = {'read-kernel': 0x40080000, 'uart': 0x09000000,
                                    'write-code': symbols['user_program'][0], 'execute-stack': sp}[case]
                    assert far == expected_far, result
                expected_pc = sp if case == 'execute-stack' else symbols['user_fault_instruction'][0]
                assert pc == expected_pc, result
            assert request('context.check') == CONTEXT
            assert request('echo\tkernel survived') == 'kernel survived'
        assert request('mmu.info') == before
        assert request('memory.check') == 'checked_pages=251; exhausted=true; zeroed=true; reused=true; free_pages=251'
        assert request('user.run\t', 'err') == 'bad_args'
        assert request('user.run\textra', 'err') == 'bad_args'
        if case is None:
            child.stdin.write(b'user.run\n')
            child.stdin.flush()
            assert line() == 'user.run'
            assert line() == 'mode=EL0; state=exited; code=0; syscalls=2; rejected=0; output=Hello from EL0'
        print(f'PASS ({profile} / {name}): real EL0 execution, exact result/fault, repeat invocation, kernel context/echo continuity, mappings and page ownership, rejected arguments.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    unit = OUTPUT / 'user-policy-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/user/policy.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    verify(profile)
    for case in CASES: verify(profile, case)


if __name__ == '__main__':
    main()
