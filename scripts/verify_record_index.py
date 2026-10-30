#!/usr/bin/env python3
"""Real QEMU indexing, independent acceptance, partial failure and timeout recovery."""
import argparse
import json
import subprocess
import sys
sys.dont_write_bytecode = True
from check_record_index import check, FIXTURE
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_user_timeout import HELLO


def rejected(payload, key):
    try:
        check(payload, key)
    except ValueError:
        return
    raise AssertionError('Independent checker accepted corrupted output')


def verify(profile, case):
    directory = ROOT / ('target' if case is None else f'target/index-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'index-test-{case}']
    subprocess.run(command, check=True, cwd=ROOT)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    assert symbols['__user_text_start'][0] <= symbols['record_index'][0] < symbols['__user_text_end'][0]
    catalog = json.loads((FIXTURE / 'catalog.json').read_text())['records']
    denied = next(r for r in catalog if not r['read_allowed'])
    assert (FIXTURE / denied['path']).read_bytes() not in kernel.read_bytes()
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='index') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        sequence = 0
        def request(operation, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{operation}\n'.encode()); child.stdin.flush()
            parts = line().split('\t', 3)
            assert parts[:3] == ['@rai1', str(sequence), status], parts
            return parts[3]
        before = request('mmu.info')
        for _ in range(2):
            for record in catalog:
                key = record['key']
                if not record['read_allowed']:
                    assert request(f'record.index\t{key}', 'err') == 'record_denied'
                    continue
                if case:
                    expected = 'index_failed' if case == 'partial' else 'index_timed_out'
                    assert request(f'record.index\t{key}', 'err') == expected
                else:
                    payload = request(f'record.index\t{key}')
                    counts = check(payload, key)
                    (OUTPUT / f'{key}-{profile}.index').write_text(payload + '\n')
                    for bad in [payload + ',extra:1', payload + '; state=completed', payload[:-1],
                                payload.replace('bytes=', 'bytes=0'), payload.replace('completed', 'faulted'),
                                payload.replace(f'record={key}', 'record=restricted@1')]:
                        rejected(bad, key)
                    rejected(payload, 'restricted@1')
                    if key == 'notes@1':
                        assert counts['pump'] == 2
                        for bad in [payload.replace('pump:2', 'pump:1'), payload.replace('pump:2,', ''),
                                    payload.replace('pump:2', 'Pump:2'), payload.replace('10:1,7:1', '7:1,10:1'),
                                    payload.replace('pump:2', 'pump:2,pump:2')]:
                            rejected(bad, key)
                        rejected(payload, 'notes@2')
            assert request('user.run') == HELLO
            assert request('mmu.info') == before
        for owner in ['worker', 'coord']:
            request(f'{owner}.start')
            assert request('record.index\tnotes@1', 'err') == 'user_memory_busy'
            request(f'{owner}.stop')
        for op, error in [('record.index','bad_args'), ('record.index\t','bad_args'),
                          ('record.index\tnotes@99','unknown_record'),
                          ('record.index\tnotes@1\textra','bad_args'),
                          ('record.index\tnotes@1 extra','bad_args')]:
            assert request(op, 'err') == error
        if case is None:
            child.stdin.write(b'record.index notes@1\n'); child.stdin.flush()
            assert line() == 'record.index notes@1'
            check(line(), 'notes@1')
            check(request('record.index\tnotes@2'), 'notes@2')
        assert request('timer.check').startswith('irq=30; ticks=6;')
        assert request('memory.check') == 'checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244'
        assert request('echo\tafter index') == 'after index'
        print(f'PASS ({profile}/{case or "normal"}): {sequence} requests, exact artifact or contained failure, memory ownership and recovery')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--release', action='store_true'); args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for case in [None, 'partial', 'spin']: verify('release' if args.release else 'debug', case)


if __name__ == '__main__': main()
