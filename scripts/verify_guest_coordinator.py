#!/usr/bin/env python3
"""Audit guest-resident coordination, real EL0 IPC and stale-proposal rejection."""
import argparse
import re
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_exceptions import elf_symbols
from verify_service import CATALOG, FIXTURE
from verify_user_timeout import HELLO

CASES = [None, 'coord-test-forged-completion', 'coord-test-fault', 'coord-test-spin', 'coord-test-flood',
         'coord-test-bad-report', 'coord-test-worker-gateway', 'worker-test-fault',
         'worker-test-spin', 'worker-test-flood', 'worker-test-bad-reply', 'worker-test-bad-sequence']


def fields(reply):
    pairs = [p.split('=', 1) for p in reply.split('; ')]
    result = dict(pairs)
    assert len(pairs) == len(result), reply
    assert result['coordinator'] == result['worker'] == 'EL0'
    return result


def audit_execution(report, record, count):
    source = (FIXTURE / record['path']).read_bytes()
    assert report['state'] == 'completed' and report['event'] == 'executed'
    assert report['record'] == record['key']
    assert int(report['bytes']) == len(source)
    assert int(report['tokens']) == len(re.findall(rb'[A-Za-z0-9]+', source))
    assert int(report['completed']) == count
    assert report['a_calls'] == '5' and report['b_calls'] == '3'
    # Independent trace replay: both tasks must actually dispatch, blocking tasks
    # are skipped, messages are queued/consumed, and both end waiting again.
    states = ['ready', 'ipc']; queues = [0, 0]; cursor = 0
    calls = [0, 0]; slices = [0, 0]; sent = [0, 0]; received = [0, 0]
    published = False; previous = None
    for item in report['trace'].split(','):
        assert len(item) == 2 and item[0] in 'AB', report
        task = 'AB'.index(item[0]); event = item[1]
        if event == 'W':
            assert states[task] == 'ipc' and queues[task] > 0
            assert previous == (task ^ 1, 'S'), report
            queues[task] -= 1; received[task] += 1; states[task] = 'ready'
            continue
        ready = [i for i in (cursor, cursor ^ 1) if states[i] == 'ready']
        assert ready and ready[0] == task, report
        if event == 'P': slices[task] += 1
        else:
            calls[task] += 1
            if event == 'S':
                if task == 1: assert received[1] == 1
                sent[task] += 1; queues[task ^ 1] += 1
                assert queues[task ^ 1] <= 2
            elif event == 'R':
                assert queues[task] > 0
                queues[task] -= 1; received[task] += 1
            elif event == 'D':
                assert queues[task] == 0; states[task] = 'ipc'
            elif event == 'O':
                assert task == 0 and not published and received == [2, 1]
                published = True
            else:
                assert event == 'H' and task == 0 and published
                states[task] = 'host'
        previous = (task, event); cursor = task ^ 1
    assert states == ['host', 'ipc'] and queues == [0, 0]
    assert sent == [1, 2] and received == [2, 1] and calls == [5, 3]
    assert slices == [int(report['a_slices']), int(report['b_slices'])]
    assert all(n < 6 for n in slices)


def verify(profile, feature):
    directory = ROOT / ('target' if feature is None else f'target/guest-coordinator-self-test/{feature}')
    cmd = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': cmd.append('--release')
    if feature: cmd += ['--features', feature]
    subprocess.run(cmd, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    for name in ['guest_coordinator', 'persistent_worker']:
        assert symbols['__user_text_start'][0] <= symbols[name][0] < symbols['__user_text_end'][0]
    with guest(kernel, f'{feature or "normal"}-{profile}', prefix='guest-coordinator') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        serial = 0
        def request(op, status='ok'):
            nonlocal serial
            serial += 1
            child.stdin.write(f'@rai1\t{serial}\t{op}\n'.encode()); child.stdin.flush()
            parts = line().split('\t', 3)
            assert parts[:3] == ['@rai1', str(serial), status], parts
            return parts[3]
        before = request('mmu.info'); memory = request('memory.info')
        assert fields(request('coord.status'))['state'] == 'stopped'
        assert request('coord.observe\tnotes@1', 'err') == 'coord_not_running'
        assert request('coord.propose\t1:notes@1', 'err') == 'coord_not_running'
        # The two alternative persistent sessions cannot overwrite each other's RAM.
        request('worker.start')
        assert request('coord.start', 'err') == 'user_memory_busy'
        request('coord.stop')
        assert fields(request('coord.status'))['state'] == 'stopped'
        assert 'state=waiting' in request('worker.status')
        request('worker.stop')
        initial = fields(request('coord.start'))
        assert initial['state'] == 'idle' and initial['completed'] == '0'
        assert initial['a_calls'] == '2' and initial['b_calls'] == '1'
        assert request('coord.start', 'err') == 'coord_not_stopped'
        for op in ['worker.start', 'user.run', 'user.spin', 'schedule.check', 'schedule.isolate',
                   'schedule.run', 'schedule.fault', 'ipc.check', 'ipc.wait', 'record.count\tnotes@1']:
            assert request(op, 'err') == 'user_memory_busy'
        request('worker.stop')  # Stopping an inactive owner must not clear this pair's RAM.
        assert fields(request('coord.status')) == initial
        for op in ['coord.start\t', 'coord.status\textra', 'coord.stop\t', 'coord.observe',
                   'coord.propose', 'coord.propose\t', 'coord.propose\t0:notes@1',
                   'coord.propose\t-1:notes@1', 'coord.propose\t+1:notes@1',
                   'coord.propose\t4294967296:notes@1', 'coord.observe\tnotes@1\textra',
                   'coord.propose\t1:notes@1\textra']:
            assert request(op, 'err') == 'bad_args'
        assert request('coord.observe\trestricted@1', 'err') == 'record_denied'
        assert request('coord.propose\t1:restricted@1', 'err') == 'record_denied'
        assert request('coord.observe\tunknown@1', 'err') == 'unknown_record'
        assert fields(request('coord.status')) == initial
        if feature:
            first_input_fault = feature.startswith('coord-test-') and feature != 'coord-test-worker-gateway'
            expected = {'coord-test-forged-completion': 'coord_bad_report', 'coord-test-fault': 'coord_task_fault', 'coord-test-spin': 'coord_slice_limit',
                        'coord-test-flood': 'coord_call_limit', 'coord-test-bad-report': 'coord_bad_report',
                        'coord-test-worker-gateway': 'coord_bad_syscall', 'worker-test-fault': 'coord_task_fault',
                        'worker-test-spin': 'coord_slice_limit', 'worker-test-flood': 'coord_call_limit',
                        'worker-test-bad-reply': 'coord_bad_syscall', 'worker-test-bad-sequence': 'coord_bad_syscall'}[feature]
            if first_input_fault:
                assert request('coord.observe\tnotes@1', 'err') == expected
            else:
                assert fields(request('coord.observe\tnotes@1'))['state'] == 'awaiting_proposal'
                assert request('coord.propose\t1:notes@1', 'err') == expected
            failed = fields(request('coord.status'))
            assert failed['state'] == 'failed' and failed['event'] == expected and failed['completed'] == '0'
            task = 'a' if first_input_fault else 'b'
            if feature.endswith('-spin'): assert failed[f'{task}_slices'] == '6'
            if feature.endswith('-flood'): assert failed[f'{task}_calls'] == '16'
            assert request('coord.start', 'err') == 'coord_not_stopped'
            assert request('coord.propose\t1:notes@1', 'err') == 'coord_not_running'
            assert request('user.run') == HELLO
            assert fields(request('coord.status')) == failed
        else:
            assert request('coord.propose\t1:notes@1', 'err') == 'coord_no_goal'
            first = fields(request('coord.observe\tnotes@1'))
            assert first['ticket'] == '1' and first['state'] == 'awaiting_proposal'
            duplicate = fields(request('coord.observe\tnotes@1'))
            assert duplicate['ticket'] == '1' and duplicate['event'] == 'duplicate' and duplicate['b_calls'] == '0'
            current = fields(request('coord.observe\tnotes@2'))
            assert current['ticket'] == '3' and current['completed'] == '0'
            for proposal, error in [('1:notes@1', 'coord_stale_proposal'), ('4294967295:notes@2', 'coord_stale_proposal'), ('3:notes@1', 'coord_wrong_record')]:
                assert request('coord.propose\t'+proposal, 'err') == error
                state = fields(request('coord.status'))
                assert state['ticket'] == '3' and state['record'] == 'notes@2'
                assert state['completed'] == '0' and state['b_calls'] == '0' and state['a_calls'] == '2'
                assert all(e[0] == 'A' for e in state['trace'].split(','))
            record = next(r for r in CATALOG if r['key'] == 'notes@2')
            observed = fields(request('coord.propose\t3:notes@2'))
            audit_execution(observed, record, 1)
            # A fabricated success, trace, or count must fail the independent audit.
            for field, wrong in [('tokens', '999'), ('trace', 'AO,AH'), ('completed', '0'), ('record', 'notes@1')]:
                modified = dict(observed); modified[field] = wrong
                try: audit_execution(modified, record, 1)
                except AssertionError: pass
                else: raise AssertionError(f'Accepted false {field}')
            assert request('coord.propose\t3:notes@2', 'err') == 'coord_already_completed'
            duplicate = fields(request('coord.observe\tnotes@2'))
            assert duplicate['event'] == 'duplicate' and duplicate['state'] == 'completed' and duplicate['completed'] == '1'
            completed = 1
            for _ in range(3):
                for record in CATALOG:
                    if not record['read_allowed']: continue
                    goal = fields(request('coord.observe\t'+record['key']))
                    assert goal['state'] == 'awaiting_proposal' and goal['b_calls'] == '0'
                    request('timer.check'); request('context.check')
                    result = fields(request(f"coord.propose\t{goal['ticket']}:{record['key']}"))
                    completed += 1; audit_execution(result, record, completed)
            old_ticket = goal['ticket']; old_key = goal['record']
            assert request('mmu.info') == before and request('memory.info') == memory
            request('coord.stop'); request('coord.start')
            goal = fields(request('coord.observe\t'+old_key))
            assert int(goal['ticket']) > int(old_ticket)
            assert request(f'coord.propose\t{old_ticket}:{old_key}', 'err') == 'coord_stale_proposal'
            record = next(r for r in CATALOG if r['key'] == old_key)
            audit_execution(fields(request(f"coord.propose\t{goal['ticket']}:{old_key}")), record, 1)
            child.stdin.write(b'coord.status\n'); child.stdin.flush()
            assert line() == 'coord.status'
            assert fields(line())['completed'] == '1'
        assert fields(request('coord.stop'))['state'] == 'stopped'
        assert fields(request('coord.stop'))['state'] == 'stopped'
        assert request('mmu.info') == before and request('memory.info') == memory
        assert request('user.run') == HELLO
        assert 'a_state=exited' in request('schedule.run')
        assert 'memory_unchanged=true; base_restored=true' in request('schedule.isolate')
        print(f'PASS guest coordinator {feature or "normal"} ({profile}): {serial} requests; retained EL0 state, admission, IPC and recovery')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(); parser.add_argument('--release', action='store_true')
    args = parser.parse_args(); OUTPUT.mkdir(parents=True, exist_ok=True)
    for feature in CASES: verify('release' if args.release else 'debug', feature)
