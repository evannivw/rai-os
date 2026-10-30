#!/usr/bin/env python3
"""Check real EL0 results against fixture bytes with an independent host tokenizer."""
import argparse
import hashlib
import json
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_isolation import check_report as check_isolation
from verify_wait import check_report as check_wait
from verify_user_timeout import HELLO

FIXTURE = ROOT / 'fixtures/review_pack/v1'
CATALOG = json.loads((FIXTURE / 'catalog.json').read_text())['records']
CASES = [None, 'bad-request', 'wrong-kind', 'bad-reply', 'worker-fault']


def check(reply, record):
    source = (FIXTURE / record['path']).read_bytes()
    assert len(source) == record['byte_length']
    assert hashlib.sha256(source).hexdigest() == record['sha256']
    # Separate implementation: regex on trusted exact bytes, no guest code import.
    tokens = len(re.findall(rb'[A-Za-z0-9]+', source))
    pairs = [p.split('=', 1) for p in reply.split('; ')]
    fields = dict(pairs)
    assert len(pairs) == len(fields), reply
    assert fields['record'] == record['key'] and fields['worker'] == 'EL0'
    assert int(fields['bytes']) == len(source) and int(fields['tokens']) == tokens, reply
    raw_id = next(i + 1 for i, item in enumerate(CATALOG) if item['key'] == record['key'])
    packed = (raw_id << 32) | (len(source) << 16) | tokens
    for key, value in [('tasks', 2), ('slice_ms', 20), ('max_slices', 6),
                       ('ipc_capacity', 2), ('max_calls', 16), ('sent', 2),
                       ('received', 2), ('rejected', 0), ('empty', 0),
                       ('a_calls', 2), ('b_calls', 2), ('closed_wakes', 0), ('deadlocked', 0)]:
        assert int(fields[key]) == value, (key, reply)
    for name in 'ab':
        assert fields[f'{name}_state'] == 'exited' and fields[f'{name}_code'] == '0'
        assert int(fields[f'{name}_value']) == packed
    # Replay the round-robin batch, permitting real timer preemptions. Pending A
    # must not dispatch between its receive and the worker's send/wakeup.
    states = ['ready', 'ready']; cursor = 0; slices = [0, 0]
    queues = [0, 0]; sent = [0, 0]; received = [0, 0]
    blocked = 0; woken = 0; last = None
    for entry in fields['trace'].split(','):
        assert len(entry) == 2 and entry[0] in 'AB', reply
        task = 'AB'.index(entry[0]); event = entry[1]
        if event == 'W':
            assert states[task] == 'waiting' and last == ((task ^ 1), 'S')
            assert queues[task] == 1
            queues[task] -= 1; received[task] += 1; woken += 1
            states[task] = 'ready'
        else:
            ready = [i for i in (cursor, cursor ^ 1) if states[i] == 'ready']
            assert ready and task == ready[0], reply
            if event == 'P': slices[task] += 1
            elif event == 'D':
                assert queues[task] == 0
                states[task] = 'waiting'; blocked += 1
            elif event == 'S':
                # The worker must receive the request before replying.
                if task == 1: assert received[1] == 1
                queues[task ^ 1] += 1; sent[task] += 1
            elif event == 'R':
                assert queues[task] == 1
                queues[task] -= 1; received[task] += 1
            else:
                assert event == 'E' and sent[task] == received[task] == 1
                states[task] = 'exited'
            cursor = task ^ 1
            last = (task, event)
    assert sent == received == [1, 1] and queues == [0, 0]
    assert int(fields['blocked']) == blocked == woken == int(fields['woken'])
    assert states == ['exited', 'exited']
    for i, name in enumerate('ab'):
        assert int(fields[f'{name}_slices']) == slices[i] < 6
    return fields


def verify(profile, case):
    directory = ROOT / ('target' if case is None else f'target/service-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'service-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    for name in ['record_client', 'record_worker']:
        assert symbols['__user_text_start'][0] <= symbols[name][0] < symbols['__user_text_end'][0]
    denied = next(r for r in CATALOG if not r['read_allowed'])
    assert (FIXTURE / denied['path']).read_bytes() not in kernel.read_bytes()
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='service') as (child, _, line, _, transcript):
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
            for record in CATALOG:
                key = record['key']
                if not record['read_allowed']:
                    assert request(f'record.count\t{key}', 'err') == 'record_denied'
                elif case:
                    expected = {'bad-request': 'service_request_rejected', 'wrong-kind': 'service_request_rejected',
                                'bad-reply': 'service_reply_rejected', 'worker-fault': 'service_failed'}[case]
                    assert request(f'record.count\t{key}', 'err') == expected
                else:
                    result = request(f'record.count\t{key}')
                    check(result, record)
                    # Prove this checker catches a false count, identity, byte count,
                    # or success state rather than trusting worker=EL0 / exit zero.
                    fields = dict(p.split('=', 1) for p in result.split('; '))
                    for field, wrong in [('tokens', str(int(fields['tokens']) + 1)), ('record', 'unknown@1'),
                                         ('bytes', str(int(fields['bytes']) + 1)), ('a_state', 'faulted')]:
                        mutated = '; '.join(f'{k}={wrong if k == field else v}' for k, v in fields.items())
                        try: check(mutated, record)
                        except AssertionError: pass
                        else: raise AssertionError('Verifier accepted corrupted ' + field)
            assert request('mmu.info') == before
            assert request('user.run') == HELLO
            check_wait(request('ipc.wait'), symbols)
        for operation, error in [
            ('record.count', 'bad_args'), ('record.count\t', 'bad_args'),
            ('record.count\tnotes@1\textra', 'bad_args'), ('record.count\tnotes@1 extra', 'bad_args'),
            ('record.count\tnotes@99', 'unknown_record'), ('record.count\t0', 'unknown_record'),
            ('record.count\t18446744073709551615', 'unknown_record'),
            ('record.delete\tnotes@1', 'unknown_operation')]:
            assert request(operation, 'err') == error
        if case is None:
            child.stdin.write(b'record.count notes@1\n'); child.stdin.flush()
            assert line() == 'record.count notes@1'
            check(line(), next(r for r in CATALOG if r['key'] == 'notes@1'))
        check_isolation(request('schedule.isolate'), symbols)
        assert request('memory.check') == 'checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244'
        assert request('mmu.info') == before
        assert request('echo\tafter service') == 'after service'
        print(f'PASS ({profile}/{case or "normal"}): 12 bounded service invocations, permission/input/error checks and kernel continuity.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--release', action='store_true'); args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for case in CASES: verify('release' if args.release else 'debug', case)


if __name__ == '__main__': main()
