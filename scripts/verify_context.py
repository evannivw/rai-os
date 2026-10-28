#!/usr/bin/env python3
"""Check repeated EL1 context round trips and fail-closed unknown SVCs in QEMU."""
import argparse
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_exceptions import elf_symbols
from verify_mmu import ROOT, OUTPUT, guest

RESULT = 'round_trips=2; registers=31; flags=true; stack=true; resumed=true'


def build(profile, case=None):
    directory = ROOT / ('target' if case is None else f'target/context-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'context-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    assert symbols['context_probe_resume'][0] == symbols['context_probe_svc'][0] + 4
    assert symbols['exception_vectors'][0] % 2048 == 0
    assert symbols['exception_vectors'][1] == 2048
    if case is None: assert 'context_negative_trigger' not in symbols
    return kernel, symbols


def normal(profile):
    kernel, _ = build(profile)
    with guest(kernel, f'normal-{profile}', prefix='context') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        requests = [('context.check', 'ok', RESULT)] * 16
        requests += [('context.check\t', 'err', 'bad_args'),
                     ('context.check\textra', 'err', 'bad_args'),
                     ('memory.check', 'ok', 'checked_pages=251; exhausted=true; zeroed=true; reused=true; free_pages=251'),
                     ('context.check', 'ok', RESULT),
                     ('echo\tcontext remains alive', 'ok', 'context remains alive')]
        for index, (command, status, payload) in enumerate(requests, 1):
            child.stdin.write(f'@rai1\t{index}\t{command}\n'.encode())
            child.stdin.flush()
            actual = line()
            assert actual == f'@rai1\t{index}\t{status}\t{payload}', actual
        for command, expected in [('context.check extra', 'error: unknown_command'),
                                  ('context.check', RESULT)]:
            child.stdin.write((command + '\n').encode())
            child.stdin.flush()
            assert line() == command
            assert line() == expected
        print(f'PASS ({profile}): 36 SVC returns; all 31 GPRs, both NZCV patterns, DAIF/SP, repeated guard reset, invalid arguments and memory/echo continuity.')
        print(transcript.decode('ascii'), end='')


def rejected(profile, case):
    kernel, symbols = build(profile, case)
    with guest(kernel, f'{case}-{profile}', prefix='context') as (child, poll, line, pending, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == f'SELF-TEST: context {case} after valid returns'
        report = line()
        match = re.fullmatch(r'EXCEPTION: vector=(\d+); esr=0x([0-9a-f]{16}); ec=0x([0-9a-f]{2}); '
                             r'elr=0x([0-9a-f]{16}); spsr=0x([0-9a-f]{16}); far_raw=0x([0-9a-f]{16}); '
                             r'entry_sp=0x([0-9a-f]{16}); handler_sp=0x([0-9a-f]{16})', report)
        assert match, report
        esr, ec, elr, spsr, _far, entry_sp, handler_sp = [int(x,16) for x in match.groups()[1:]]
        assert int(match[1]) == 4 and ec == 0x15
        assert esr == (0x56005243 if case == 'unknown-svc' else 0x56005242)
        assert elr == symbols['context_negative_resume'][0]
        assert elr != symbols['context_probe_resume'][0]
        assert spsr & 0x3cf == 0x3c5
        assert symbols['__stack_bottom'][0] <= entry_sp <= symbols['__stack_top'][0]
        assert symbols['__exception_stack_bottom'][0] <= handler_sp < symbols['__exception_stack_top'][0]
        assert entry_sp % 16 == handler_sp % 16 == 0
        assert line() == 'HALTED: exception is fatal; restart QEMU'
        child.stdin.write(b'info\n')
        child.stdin.flush()
        assert not pending and not poll.select(.25)
        assert child.poll() is None
        print(f'PASS ({profile}): {case} remains fatal after successful returns; exact syndrome, next PC, stack and halt checked.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    unit = OUTPUT / 'context-policy-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/context/policy.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    normal(profile)
    for case in ['unknown-svc', 'wrong-site']: rejected(profile, case)


if __name__ == '__main__':
    main()
