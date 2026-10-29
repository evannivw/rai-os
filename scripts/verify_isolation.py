#!/usr/bin/env python3
"""Exercise private stack mappings, warmed translations, and a broken-permission control."""
import argparse
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_lifecycle import check_report as check_lifecycle
from verify_schedule import check_report as check_schedule
from verify_user_timeout import HELLO


def check_report(reply, symbols):
    pairs = [part.split('=', 1) for part in reply.split('; ')]
    assert all(len(p) == 2 for p in pairs), reply
    fields = dict(pairs)
    assert len(fields) == len(pairs) == 18, reply
    for key, value in [('rounds', '2'), ('reads_denied', '4'), ('writes_denied', '4'),
                       ('own_accesses', '12'), ('memory_unchanged', 'true'), ('base_restored', 'true')]:
        assert fields[key] == value, reply
    bottom = symbols['__user_stack_bottom'][0]
    for id, name in enumerate(['ar', 'aw', 'br', 'bw']):
        write = id % 2 == 1
        assert int(fields[f'{name}_esr'], 16) == (0x9200004f if write else 0x9200000f)
        assert int(fields[f'{name}_pc'], 16) == symbols[f'isolation_{"write" if write else "read"}_fault'][0]
        assert int(fields[f'{name}_far'], 16) == bottom + (4096 if id < 2 else 0)


def verify(profile, broken=False):
    directory = ROOT / ('target/isolation-self-test/open-peer' if broken else 'target')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if broken: command += ['--features', 'isolation-test-open-peer']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    for name in ['isolation_own', 'isolation_read', 'isolation_write']:
        assert symbols['__user_text_start'][0] <= symbols[name][0] < symbols['__user_text_end'][0]
    with guest(kernel, f'{"open-peer" if broken else "normal"}-{profile}', prefix='isolation') as (child, _, line, _, transcript):
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
        assert 'table_pages=12;' in before
        for _ in range(3):
            if broken:
                assert request('schedule.isolate', 'err') == 'isolation_expected_fault'
            else:
                check_report(request('schedule.isolate'), symbols)
            assert request('mmu.info') == before
            # Check restoration after diagnostic success AND an early error path.
            check_lifecycle(request('schedule.run'), symbols)
            check_lifecycle(request('schedule.fault'), symbols, fault=True)
            check_schedule(request('schedule.check'))
            assert request('user.run') == HELLO
            assert request('memory.check') == 'checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244'
            assert request('echo\tprivate stacks checked') == 'private stacks checked'
        if not broken:
            for suffix in ['\t', '\textra']:
                assert request('schedule.isolate' + suffix, 'err') == 'bad_args'
            child.stdin.write(b'schedule.isolate extra\n')
            child.stdin.flush()
            assert line() == 'schedule.isolate extra'
            assert line() == 'error: unknown_command'
            child.stdin.write(b'schedule.isolate\n')
            child.stdin.flush()
            assert line() == 'schedule.isolate'
            check_report(line(), symbols)
        assert request('mmu.info') == before
        assert request('echo\tafter isolation') == 'after isolation'
        print(f'PASS ({profile}/{"open-peer" if broken else "normal"}): private-stack diagnostic or expected broken-map rejection; exact fault PC/address/syndrome, repeated runs, restored base root and subsequent task/user/memory operation.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    tests = OUTPUT / 'isolation-table-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/mmu/tables.rs', '-o', str(tests)], cwd=ROOT, check=True)
    subprocess.run([str(tests)], check=True)
    verify(profile)
    verify(profile, broken=True)


if __name__ == '__main__':
    main()
