#!/usr/bin/env python3
"""Exercise a persistent EL0 continuation and exclusive RAM ownership in real QEMU."""
import argparse
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_service import CATALOG, FIXTURE
from verify_user_timeout import HELLO

CASES = [None, 'fault', 'spin', 'flood', 'bad-reply', 'bad-sequence']


def fields(reply):
    pairs = [item.split('=', 1) for item in reply.split('; ')]
    result = dict(pairs)
    assert len(pairs) == len(result), reply
    return result


def verify(profile, case):
    directory = ROOT / ('target' if case is None else f'target/worker-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case: command += ['--features', f'worker-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    assert symbols['__user_text_start'][0] <= symbols['persistent_worker'][0] < symbols['__user_text_end'][0]
    with guest(kernel, f'{case or "normal"}-{profile}', prefix='worker') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        serial = 0
        def request(operation, status='ok'):
            nonlocal serial
            serial += 1
            child.stdin.write(f'@rai1\t{serial}\t{operation}\n'.encode()); child.stdin.flush()
            parts = line().split('\t', 3)
            assert parts[:3] == ['@rai1', str(serial), status], parts
            return parts[3]
        before = request('mmu.info')
        memory = request('memory.info')
        stopped = 'worker=EL0; state=stopped; completed=0; reason=none; calls=0; slices=0'
        assert request('worker.status') == stopped
        assert request('worker.stop') == stopped
        assert request('worker.count\tnotes@1', 'err') == 'worker_not_waiting'
        for command in ['worker.start\textra', 'worker.status\textra', 'worker.stop\t',
                        'worker.count', 'worker.count\t', 'worker.count\tnotes@1\textra']:
            assert request(command, 'err') == 'bad_args'
        for _ in range(2):
            started = fields(request('worker.start'))
            assert started['state'] == 'waiting' and started['completed'] == '0' and started['calls'] == '1'
            assert request('worker.start', 'err') == 'worker_not_stopped'
            assert request('worker.count\trestricted@1', 'err') == 'record_denied'
            assert request('worker.count\tunknown@1', 'err') == 'unknown_record'
            # Every other EL0 entry must reject before clearing the suspended stack.
            for op in ['user.run', 'user.spin', 'schedule.check', 'schedule.isolate',
                       'schedule.run', 'schedule.fault', 'ipc.check', 'ipc.wait', 'record.count\tnotes@1']:
                assert request(op, 'err') == 'user_memory_busy'
            assert fields(request('worker.status')) == started
            assert request('echo\tstill waiting') == 'still waiting'
            assert 'worker.count <key>' in request('help')
            assert request('mmu.info') == before
            assert request('memory.info') == memory
            assert 'free_pages=244' in request('memory.check')
            assert 'round_trips=2' in request('context.check')
            assert 'timer_disabled=true' in request('timer.check')
            if case:
                expected = {'fault': 'worker_fault', 'spin': 'worker_slice_limit',
                            'flood': 'worker_call_limit', 'bad-reply': 'worker_reply_rejected',
                            'bad-sequence': 'worker_reply_rejected'}[case]
                assert request('worker.count\tnotes@1', 'err') == expected
                failed = fields(request('worker.status'))
                assert failed['state'] == 'failed' and failed['reason'] == expected
                assert failed['completed'] == '0'
                if case == 'spin': assert failed['slices'] == '6' and failed['calls'] == '0'
                if case == 'flood': assert failed['calls'] == '16'
                assert request('worker.count\tnotes@2', 'err') == 'worker_not_waiting'
                assert request('worker.start', 'err') == 'worker_not_stopped'
                # Failure releases the lease without needing stop, and never resumes a stale frame.
                assert request('user.run') == HELLO
                assert fields(request('worker.status')) == failed
            else:
                completed = 0
                # More than sixteen requests prove budgets reset per admission, not per lifetime.
                for _ in range(4):
                    for record in CATALOG:
                        if not record['read_allowed']: continue
                        result = fields(request(f"worker.count\t{record['key']}"))
                        completed += 1
                        source = (FIXTURE / record['path']).read_bytes()
                        assert result['record'] == record['key']
                        assert result['bytes'] == str(len(source))
                        assert result['tokens'] == str(len(re.findall(rb'[A-Za-z0-9]+', source)))
                        assert result['worker'] == 'EL0' and result['state'] == 'waiting'
                        assert result['completed'] == str(completed) and result['reason'] == 'none'
                        assert result['calls'] == '3' and int(result['slices']) < 6
                        assert fields(request('worker.status'))['completed'] == str(completed)
                        if completed == 12:
                            assert 'timer_disabled=true' in request('timer.check')
                            assert 'round_trips=2' in request('context.check')
                        assert request('user.run', 'err') == 'user_memory_busy'
                        assert request('worker.count\trestricted@1', 'err') == 'record_denied'
                child.stdin.write(b'worker.count notes@2\n'); child.stdin.flush()
                assert line() == 'worker.count notes@2'
                assert fields(line())['completed'] == str(completed + 1)
            assert request('worker.stop') == stopped
            assert request('worker.stop') == stopped
            assert request('mmu.info') == before and request('memory.info') == memory
            assert request('user.run') == HELLO
            assert 'a_state=exited' in request('schedule.run')
            assert 'memory_unchanged=true; base_restored=true' in request('schedule.isolate')
            assert 'record=notes@1; bytes=30; tokens=7;' in request('record.count\tnotes@1')
        print(f'PASS worker {case or "normal"} ({profile}): {serial} protocol requests, continuation/ownership/recovery checks')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for case in CASES: verify('release' if args.release else 'debug', case)
