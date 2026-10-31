#!/usr/bin/env python3
"""Real-QEMU publication continuity, trusted validation handshake and stale-commit rejection."""
import argparse
import json
import sys
sys.dont_write_bytecode = True
from verify_mmu import OUTPUT, guest
from verify_bundle import build, bundle_fields
from verify_guest_coordinator import fields
from verify_coord_index import audit as audit_job
from check_bundle import check
from verify_user_timeout import HELLO

FIRST = ['brief@1', 'empty@1', 'notes@1']
SECOND = ['brief@1', 'empty@1', 'notes@2']


def exercise(request, name, fault=False):
    status = lambda: bundle_fields(request('coord.publish.status'))
    read = lambda: request('coord.publish.read')
    reject = lambda op, error: expect_error(request, op, error)
    before = request('mmu.info')
    assert status() == dict(target_version='0', published_version='0', candidate_ticket='0',
                           state='empty', availability='none', reason='-')
    reject('coord.publish.read', 'publication_unavailable')
    reject('coord.publish.prepare', 'coord_not_indexing')
    reject('coord.publish.commit\t1', 'publication_no_candidate')
    request('coord.start')
    reject('coord.publish.prepare', 'coord_not_indexing')
    request('coord.stop'); request('coord.start\tindex')
    reject('coord.publish.prepare', 'coord_no_required_set')
    completed = 0

    def goal(keys):
        return bundle_fields(request('coord.goal\t' + ','.join(keys)))

    def work(key):
        nonlocal completed
        ticket = fields(request('coord.observe\t' + key))['ticket']
        result = fields(request(f'coord.propose\t{ticket}:{key}'))
        completed += 1
        audit_job(result, request('coord.result'), key, completed, completed, 0)

    def prepare(keys, generation):
        payload = request('coord.publish.prepare')
        ticket = status()['candidate_ticket']
        assert status()['state'] == 'validating' and ticket != '0'
        # Independent reference code, not guest/model claims, gates every success.
        check(payload, keys, generation)
        return payload, ticket

    first = goal(FIRST); gen1 = int(first['goal_version'])
    assert status()['state'] == 'building' and status()['availability'] == 'none'
    reject('coord.publish.prepare', 'coord_bundle_incomplete')
    for key in FIRST: work(key)
    assert request('coord.bundle')  # Completeness alone must not publish.
    reject('coord.publish.read', 'publication_unavailable')
    old, ticket = prepare(FIRST, gen1)
    reject('coord.publish.read', 'publication_unavailable')
    unchanged = status()
    reject('coord.publish.commit\t4294967295', 'publication_stale_candidate')
    assert status() == unchanged
    pub = bundle_fields(request('coord.publish.commit\t' + ticket))
    assert pub['state'] == 'current' and pub['published_version'] == str(gen1)
    assert pub['availability'] == 'current' and read() == old
    reject('coord.publish.commit\t' + ticket, 'publication_no_candidate')
    reject('coord.publish.prepare', 'publication_already_current')
    (OUTPUT / f'publication-{name}-initial.json').write_text(old + '\n')

    # Invalid requirements never invalidate a valid publication.
    for keys, error in [('notes@1,notes@2', 'conflicting_revisions'), ('restricted@1', 'record_denied')]:
        reject('coord.goal\t' + keys, error)
        assert status() == pub and read() == old
    second = goal(SECOND); gen2 = int(second['goal_version'])
    assert second['ready_records'] == '2' and second['missing'] == 'notes@2'
    assert status()['state'] == 'building' and status()['availability'] == 'historical'
    assert read() == old
    reject('coord.publish.prepare', 'coord_bundle_incomplete')
    before_stale = request('coord.status')
    reject(f'coord.select\t{gen1}:notes@1', 'coord_stale_goal')
    assert request('coord.status') == before_stale and read() == old
    if fault:
        job = fields(request('coord.observe\tnotes@2'))
        reject(f"coord.propose\t{job['ticket']}:notes@2", 'coord_deadlocked')
        failed = fields(request('coord.status'))
        assert failed['state'] == 'failed' and failed['cached_records'] == '0'
        assert status()['state'] == 'blocked' and status()['reason'] == 'coord_deadlocked'
        assert status()['availability'] == 'historical' and read() == old
        check(read(), FIRST, gen1)
        # Saved output is owned by EL1, independent of the released EL0 RAM.
        assert request('user.run') == HELLO
        assert read() == old
        reject('coord.publish.prepare', 'coord_not_running')
        reject('coord.publish.commit\t' + ticket, 'publication_no_candidate')
        (OUTPUT / f'publication-{name}-after-failure.json').write_text(read() + '\n')
    else:
        work('notes@2')
        assert fields(request('coord.status'))['completed'] == '4'
        candidate, token = prepare(SECOND, gen2)
        assert read() == old
        # An actual corrupted checker input is rejected; report the failed validation.
        changed = json.loads(candidate); changed['records'][0]['source_hex'] = '00'
        try: check(json.dumps(changed), SECOND, gen2)
        except ValueError: pass
        else: raise AssertionError('Independent checker accepted changed source bytes')
        blocked = bundle_fields(request('coord.publish.reject\t' + token))
        assert blocked['state'] == 'blocked' and blocked['reason'] == 'validation_rejected'
        assert read() == old
        reject('coord.publish.commit\t' + token, 'publication_no_candidate')
        candidate, token2 = prepare(SECOND, gen2)
        assert token2 != token
        reject('coord.publish.commit\t' + token, 'publication_stale_candidate')
        # Re-preparation also invalidates old authority within the SAME goal.
        candidate, token3 = prepare(SECOND, gen2)
        assert token3 != token2
        reject('coord.publish.reject\t' + token2, 'publication_stale_candidate')
        reject('coord.publish.commit\t' + token2, 'publication_stale_candidate')
        assert read() == old
        # Goal changes AFTER independent validation, before acknowledgment.
        revised = goal(SECOND); gen3 = int(revised['goal_version'])
        assert gen3 > gen2 and revised['completed'] == '4' and revised['missing'] == '-'
        reject('coord.publish.commit\t' + token3, 'publication_no_candidate')
        assert read() == old and status()['availability'] == 'historical'
        candidate, current_token = prepare(SECOND, gen3)
        reject('coord.publish.commit\t' + token3, 'publication_stale_candidate')
        current = bundle_fields(request('coord.publish.commit\t' + current_token))
        assert current['state'] == 'current' and current['published_version'] == str(gen3)
        assert read() == candidate
        check(read(), SECOND, gen3)
        new_rows = {r['record']: r for r in json.loads(candidate)['records']}
        for row in json.loads(old)['records']:
            if row['record'] != 'notes@1': assert new_rows[row['record']] == row
        assert fields(request('coord.status'))['completed'] == '4'
        (OUTPUT / f'publication-{name}-updated.json').write_text(read() + '\n')
        unchanged = status()
        for op in ['coord.publish.commit', 'coord.publish.reject', 'coord.publish.commit\t0',
                   'coord.publish.commit\t4294967296', 'coord.publish.reject\t-1',
                   'coord.publish.prepare\textra', 'coord.publish.read\textra',
                   'coord.publish.status\t', 'coord.publish.commit\t1\textra']:
            reject(op, 'bad_args'); assert status() == unchanged and read() == candidate
        # Four-record maximum and longest public index remain bounded and owned.
        keys = ['appendix@1', 'brief@1', 'checklist@1', 'notes@2']
        expanded = goal(keys)
        for key in expanded['missing'].split(','): work(key)
        payload, token = prepare(keys, int(expanded['goal_version']))
        request('coord.publish.commit\t' + token)
        assert read() == payload
        assert fields(request('coord.status'))['completed'] == '6'
        # Identical-content rollback needs fresh verification, no new worker jobs.
        restored = goal(FIRST)
        payload, token = prepare(FIRST, int(restored['goal_version']))
        request('coord.publish.commit\t' + token)
        assert read() == payload and fields(request('coord.status'))['completed'] == '6'
        # Stop invalidates a live candidate; tokens must not be reused after restart.
        next_goal = goal(SECOND)
        _, obsolete = prepare(SECOND, int(next_goal['goal_version']))
        request('coord.stop'); request('coord.start\tindex')
        assert status()['state'] == 'empty'
        restart = goal(['empty@1'])
        ticket = fields(request('coord.observe\tempty@1'))['ticket']
        request('coord.propose\t' + ticket + ':empty@1')
        _, fresh = prepare(['empty@1'], int(restart['goal_version']))
        assert int(fresh) > int(obsolete)
        reject('coord.publish.commit\t' + obsolete, 'publication_stale_candidate')
        request('coord.publish.commit\t' + fresh)
    request('timer.check'); request('context.check')
    assert request('mmu.info') == before
    request('coord.stop')
    assert status()['state'] == 'empty' and status()['availability'] == 'none'
    reject('coord.publish.read', 'publication_unavailable')
    assert request('user.run') == HELLO
    assert 'free_pages=244' in request('memory.check')
    print(f'PASS publication {name}: checked artifacts, retained output, freshness, cleanup')


def expect_error(request, operation, error):
    actual = request(operation, 'err')
    assert actual == error, (operation, actual, error)


def normal(profile, feature=None):
    name = f'{feature or "normal"}-{profile}'
    with guest(build(profile, feature), name, prefix='publication') as (child, _, line, _, _):
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
        exercise(request, name, fault=feature is not None)
        # Shell framing reaches the same operations and validates the ticket grammar.
        for op, expected in [('coord.publish.status', 'state=empty'),
                             ('coord.publish.commit 0', 'bad_args'),
                             ('coord.publish.commit 1', 'publication_no_candidate'),
                             ('coord.publish.reject 1', 'publication_no_candidate')]:
            child.stdin.write((op + '\n').encode()); child.stdin.flush()
            assert line() == op
            assert expected in line()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args(); profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    normal(profile)
    normal(profile, 'index-stream-test-truncated')
    from verify_display import build as graphics_build, guest as graphics_guest
    with graphics_guest(graphics_build(profile), f'publication-{profile}') as (request, _):
        exercise(request, f'graphics-{profile}')
    print('Native-display lifecycle exercised; dedicated visual checks use verify_publication_display.py.')


if __name__ == '__main__': main()
