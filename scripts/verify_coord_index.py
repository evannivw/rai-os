#!/usr/bin/env python3
"""Real retained EL0 indexes: exact content, IPC replay, cache provenance and faults."""
import argparse
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT, guest
from verify_guest_coordinator import fields, CATALOG
from verify_user_timeout import HELLO
from check_record_index import check

FAULTS = {
    'index-stream-test-order': 'coord_index_order',
    'index-stream-test-length': 'coord_index_length',
    'index-stream-test-truncated': 'coord_deadlocked',
    'index-stream-test-overflow': 'coord_index_limit',
    'index-stream-test-after-end': 'coord_index_order',
    'index-stream-test-format': 'coord_index_format',
    'coord-index-test-forged-reuse': 'coord_bad_report',
    'worker-test-fault': 'coord_task_fault',
    'worker-test-spin': 'coord_slice_limit',
    'worker-test-flood': 'coord_call_limit',
    'worker-test-bad-reply': 'coord_bad_syscall',
    'worker-test-bad-sequence': 'coord_bad_syscall',
}


def audit(report, payload, key, completed, cached, hits, reuse=False):
    counts = check(payload, key)  # Independently reads the source fixture and verifies its hash.
    output = payload.split('; index=', 1)[1]
    assert len(report) == 18 and report['operation'] == 'record.index', report
    assert report['state'] == 'completed' and report['record'] == key
    assert report['event'] == ('reused' if reuse else 'executed')
    assert int(report['bytes']) == int(payload.split('; bytes=', 1)[1].split(';')[0])
    assert int(report['tokens']) == sum(counts.values())
    assert int(report['output_bytes']) == len(output)
    assert int(report['completed']) == completed and int(report['cached_records']) == cached
    assert int(report['cache_hits']) == hits
    chunks = (len(output) + 6) // 7
    expected = [2, 0] if reuse else [chunks + 5, chunks + 3]
    assert [int(report[f'{t}_calls']) for t in 'ab'] == expected
    states = ['ready', 'ipc']; queues = [0, 0]; cursor = 0
    calls = [0, 0]; slices = [0, 0]; sent = [0, 0]; received = [0, 0]
    published = False; previous = None
    for item in report['trace'].split(','):
        assert len(item) == 2 and item[0] in 'AB'
        task = 'AB'.index(item[0]); event = item[1]
        if event == 'W':
            assert states[task] == 'ipc' and queues[task] > 0 and previous == (task ^ 1, 'S')
            queues[task] -= 1; received[task] += 1; states[task] = 'ready'
            continue
        ready = [i for i in (cursor, cursor ^ 1) if states[i] == 'ready']
        assert ready and ready[0] == task
        if event == 'P': slices[task] += 1
        else:
            calls[task] += 1
            if event == 'S':
                sent[task] += 1; queues[task ^ 1] += 1
                assert queues[task ^ 1] <= 2
            elif event == 'R':
                assert queues[task] > 0
                queues[task] -= 1; received[task] += 1
            elif event == 'D':
                assert queues[task] == 0; states[task] = 'ipc'
            elif event == 'O':
                assert task == 0 and not published
                assert received == ([0, 0] if reuse else [chunks + 2, 1])
                published = True
            else:
                assert event == 'H' and task == 0 and published
                states[task] = 'host'
        previous = (task, event); cursor = task ^ 1
    assert states == ['host', 'ipc'] and queues == [0, 0]
    assert sent == ([0, 0] if reuse else [1, chunks + 2])
    assert calls == expected and slices == [int(report[f'{t}_slices']) for t in 'ab']
    assert all(n < 6 for n in slices)


def verify(profile, feature=None):
    directory = ROOT / ('target' if feature is None else f'target/coord-index-self-test/{feature}')
    cmd = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': cmd.append('--release')
    if feature: cmd += ['--features', feature]
    subprocess.run(cmd, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    with guest(kernel, f'{feature or "normal"}-{profile}', prefix='coord-index') as (child, _, line, _, _):
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
        before = request('mmu.info')
        assert request('coord.result', 'err') == 'coord_not_indexing'
        request('worker.start')
        assert request('coord.start\tindex', 'err') == 'user_memory_busy'
        request('coord.stop')
        assert 'state=waiting' in request('worker.status')
        request('worker.stop')
        request('coord.start\tindex')
        assert request('coord.result', 'err') == 'coord_result_unavailable'
        if feature:
            completed = 0
            if feature.startswith('index-stream-test-'):
                request('coord.observe\tnotes@1'); request('coord.propose\t1:notes@1')
                check(request('coord.result'), 'notes@1'); completed = 1
                goal = fields(request('coord.observe\tnotes@2'))
                op = f"coord.propose\t{goal['ticket']}:notes@2"
            elif feature == 'coord-index-test-forged-reuse':
                op = 'coord.observe\tempty@1'
            else:
                request('coord.observe\tnotes@1'); op = 'coord.propose\t1:notes@1'
            assert request(op, 'err') == FAULTS[feature]
            failed = fields(request('coord.status'))
            assert failed['state'] == 'failed' and failed['event'] == FAULTS[feature]
            assert int(failed['completed']) == completed and failed['cached_records'] == failed['output_bytes'] == '0'
            assert request('coord.result', 'err') == 'coord_result_unavailable'
            assert request('coord.observe\tnotes@1', 'err') == 'coord_not_running'
            assert request('user.run') == HELLO
            check(request('record.index\tnotes@1'), 'notes@1')
        else:
            assert request('coord.propose\t1:notes@1', 'err') == 'coord_no_goal'
            for op in ['record.index\tnotes@1', 'user.run', 'worker.start']:
                assert request(op, 'err') == 'user_memory_busy'
            for op in ['coord.start\tunknown', 'coord.start\tindex\textra', 'coord.result\textra']:
                assert request(op, 'err') == 'bad_args'
            assert request('coord.observe\trestricted@1', 'err') == 'record_denied'
            artifacts = {}; completed = 0; hits = 0
            keys = ['notes@1', 'notes@2', 'notes@1', 'brief@1', 'checklist@1', 'empty@1', 'appendix@1']
            keys += [r['key'] for r in CATALOG if r['read_allowed']] * 2
            for key in keys:
                goal = fields(request('coord.observe\t'+key)); ticket = goal['ticket']
                if key in artifacts:
                    hits += 1; result = goal
                    payload = request('coord.result'); assert payload == artifacts[key]
                    audit(result, payload, key, completed, len(artifacts), hits, True)
                else:
                    assert goal['state'] == 'awaiting_proposal'
                    assert request('coord.result', 'err') == 'coord_result_unavailable'
                    duplicate = fields(request('coord.observe\t'+key))
                    assert duplicate['event'] == 'duplicate' and duplicate['ticket'] == ticket
                    assert request(f'coord.propose\t4294967295:{key}', 'err') == 'coord_stale_proposal'
                    wrong = 'empty@1' if key != 'empty@1' else 'notes@1'
                    assert request(f'coord.propose\t{ticket}:{wrong}', 'err') == 'coord_wrong_record'
                    request('timer.check'); request('context.check')
                    result = fields(request(f'coord.propose\t{ticket}:{key}'))
                    completed += 1
                    payload = request('coord.result'); artifacts[key] = payload
                    audit(result, payload, key, completed, len(artifacts), hits)
                for field, value in [('completed', '999'), ('trace', 'AO,AH,BS'), ('record', 'restricted@1'), ('output_bytes', '999')]:
                    bad = dict(result); bad[field] = value
                    try: audit(bad, payload, key, completed, len(artifacts), hits, result['event'] == 'reused')
                    except (AssertionError, ValueError): pass
                    else: raise AssertionError('Accepted fabricated '+field)
                assert request(f'coord.propose\t{ticket}:{key}', 'err') == 'coord_already_completed'
                duplicate = fields(request('coord.observe\t'+key))
                assert duplicate['event'] == 'duplicate' and duplicate['ticket'] == ticket
                assert int(duplicate['cache_hits']) == hits and duplicate['b_calls'] == '0'
                assert request('coord.result') == artifacts[key]
            assert completed == 6 and hits == 13
            try: check(artifacts['notes@1'], 'notes@2')
            except ValueError: pass
            else: raise AssertionError('Accepted stale revision')
            request('coord.stop'); request('coord.start\tindex')
            goal = fields(request('coord.observe\t'+key))
            assert goal['state'] == 'awaiting_proposal' and goal['cached_records'] == '0'
            assert int(goal['ticket']) > int(ticket)
            assert request(f'coord.propose\t{ticket}:{key}', 'err') == 'coord_stale_proposal'
            result = fields(request(f"coord.propose\t{goal['ticket']}:{key}"))
            audit(result, request('coord.result'), key, 1, 1, 0)
            child.stdin.write(b'coord.result\n'); child.stdin.flush()
            assert line() == 'coord.result'; check(line(), key)
            request('coord.stop'); request('coord.start')
            assert request('coord.result', 'err') == 'coord_not_indexing'
        request('coord.stop')
        assert request('mmu.info') == before and request('user.run') == HELLO
        assert 'free_pages=244' in request('memory.check')
        print(f'PASS retained index {profile}/{feature or "normal"}: {serial} requests')


def graphical(profile):
    from verify_display import build, guest as display_guest
    with display_guest(build(profile), f'coord-index-{profile}') as (request, capture):
        boot = capture('boot')
        request('coord.start\tindex'); idle = capture('idle'); assert idle != boot
        request('coord.observe\tnotes@1'); pending = capture('pending'); assert pending != idle
        request('coord.propose\t1:notes@1'); check(request('coord.result'), 'notes@1')
        created = capture('created'); assert created != pending
        request('coord.observe\tnotes@2'); request('coord.propose\t2:notes@2')
        assert capture('revision') != created
        report = fields(request('coord.observe\tnotes@1'))
        assert report['event'] == 'reused' and report['completed'] == '2' and report['b_calls'] == '0'
        check(request('coord.result'), 'notes@1'); reused = capture('reused'); assert reused != created
        request('coord.observe\tnotes@1'); assert capture('duplicate') == reused
        request('coord.status'); assert capture('status') == reused
        assert request('coord.propose\tbad', 'err') == 'bad_args'
        assert capture('error') != reused
        request('coord.status'); assert capture('restored') == reused
        request('coord.stop'); assert capture('stopped') == boot
    print(f'PASS retained index {profile}: native pending/created/reused/error/stop pixels')


def graphical_failure(profile):
    from verify_display import guest as display_guest
    directory = ROOT / 'target/coord-index-display-fault'
    cmd = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--features',
           'guest-display,index-stream-test-truncated', '--target-dir', str(directory)]
    if profile == 'release': cmd.append('--release')
    subprocess.run(cmd, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    with display_guest(kernel, f'coord-index-failure-{profile}') as (request, capture):
        request('coord.start\tindex'); request('coord.observe\tnotes@1')
        request('coord.propose\t1:notes@1'); created = capture('created')
        request('coord.observe\tnotes@2')
        assert request('coord.propose\t2:notes@2', 'err') == 'coord_deadlocked'
        failed = capture('failed'); assert failed != created
        assert fields(request('coord.status'))['state'] == 'failed'
        assert capture('failed-status') == failed
        assert request('coord.result', 'err') == 'coord_result_unavailable'
        check(request('record.index\tnotes@1'), 'notes@1')
        assert capture('recovered') == created
    print(f'PASS retained index {profile}: failed stream removes visible result and releases worker memory')


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--release', action='store_true'); args = parser.parse_args()
    profile = 'release' if args.release else 'debug'; OUTPUT.mkdir(parents=True, exist_ok=True)
    unit = OUTPUT / 'index-stream-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/index_stream.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    verify(profile)
    for feature in FAULTS: verify(profile, feature)
    graphical(profile)
    graphical_failure(profile)


if __name__ == '__main__': main()
