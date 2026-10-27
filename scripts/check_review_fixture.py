#!/usr/bin/env python3
"""Check trusted review-pack fixture consistency, not candidate pack correctness."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

DEFAULT_FIXTURE = Path(__file__).resolve().parents[1] / 'fixtures/review_pack/v1'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def token_counts(raw):
    """Scan ASCII bytes without sharing a future guest index implementation."""
    counts = Counter()
    token = bytearray()
    for byte in raw + b'\n':
        if 65 <= byte <= 90:
            byte += 32
        if 97 <= byte <= 122 or 48 <= byte <= 57:
            token.append(byte)
        elif token:
            counts[token.decode('ascii')] += 1
            token.clear()
    return counts


def check_fixture(base):
    catalog = read_json(base / 'catalog.json')
    expected = read_json(base / 'expected.json')
    require(catalog['format_version'] == expected['format_version'] == 1, 'unsupported format')
    records, bodies = {}, {}
    for record in catalog['records']:
        key = record['key']
        require(key not in records, f'duplicate record: {key}')
        require(key == f"{record['record_id']}@{record['revision']}", f'invalid key: {key}')
        require(type(record['revision']) is int and record['revision'] > 0, 'invalid revision')
        require(record['project_id'] == 'aurora', 'unexpected project')
        require(type(record['read_allowed']) is bool, 'invalid permission')
        require(record['path'] == f'records/{key}.txt' and '/' not in key, 'invalid path')
        raw = (base / record['path']).read_bytes()
        require(raw.isascii(), f'non-ASCII source: {key}')
        require(len(raw) == record['byte_length'], f'length mismatch: {key}')
        require(hashlib.sha256(raw).hexdigest() == record['sha256'], f'digest mismatch: {key}')
        records[key], bodies[key] = record, raw

    goals = {goal['goal_version']: goal for goal in expected['goals']}
    require(len(goals) == len(expected['goals']), 'duplicate expected goal')
    for goal in goals.values():
        keys = goal['required']
        require(keys == sorted(set(keys)), 'required keys must be unique and sorted')
        index = {}
        for key in keys:
            require(key in records and records[key]['read_allowed'], f'invalid required key: {key}')
            for token, count in token_counts(bodies[key]).items():
                index.setdefault(token, {})[key] = count
        require(index == goal['index'], f"incorrect reference index for goal {goal['goal_version']}")
        require(list(goal['index']) == sorted(index), 'index tokens must be sorted')
        for postings in goal['index'].values():
            require(list(postings) == sorted(postings), 'index postings must be sorted')
        seen = set()
        for preview in goal['optional_previews']:
            key = preview['key']
            require(key in records and records[key]['read_allowed'], 'invalid optional key')
            require(key not in keys and key not in seen, 'duplicate or required preview')
            seen.add(key)
            raw = bodies[key][:64]
            require(preview['text'].encode('ascii') == raw, 'incorrect reference preview')
            require(preview['byte_length'] == len(raw), 'incorrect preview length')
            require(type(preview['weight']) is int and preview['weight'] > 0, 'invalid weight')

    paths = sorted((base / 'scenarios').glob('*.json'))
    require({path.stem for path in paths} == set(expected['checkpoints']), 'scenario set mismatch')
    event_count = 0
    for path in paths:
        scenario = read_json(path)
        require(scenario['format_version'] == 1 and scenario['scenario_id'] == path.stem, 'invalid scenario')
        require(type(scenario['deadline_ms']) is int and scenario['deadline_ms'] > 0, 'invalid deadline')
        released, unavailable, limits = set(), set(), {}
        checkpoints = []
        goal_version, last_time = 0, -1
        require(bool(scenario['events']) and scenario['events'][0]['at_ms'] == 0, 'missing initial event')
        for event in scenario['events']:
            event_count += 1
            require(set(event) <= {'at_ms', 'release', 'unavailable', 'available', 'goal', 'limits'}, 'unknown event field')
            at = event['at_ms']
            require(type(at) is int and last_time < at < scenario['deadline_ms'], 'invalid event order/time')
            last_time = at
            for key in event.get('release', []):
                require(key in records and key not in released, f'invalid release: {key}')
                released.add(key)
            for key in event.get('unavailable', []):
                require(key in released and key not in unavailable, f'invalid outage: {key}')
                unavailable.add(key)
            for key in event.get('available', []):
                require(key in unavailable, f'invalid recovery: {key}')
                unavailable.remove(key)
            if 'goal' in event:
                goal = event['goal']
                require(goal['goal_version'] == goal_version + 1, 'nonsequential goal version')
                goal_version = goal['goal_version']
                reference = goals[goal_version]
                require(goal['required'] == reference['required'], 'goal required set mismatch')
                optional = [dict(key=p['key'], weight=p['weight']) for p in reference['optional_previews']]
                require(goal['optional'] == optional, 'goal optional set mismatch')
                require(set(goal['required']) | {p['key'] for p in optional} <= released, 'goal uses unreleased record')
            require(goal_version in goals, 'missing active goal')
            limits.update(event.get('limits', {}))
            require(set(limits) == {'scratch_bytes', 'total_work_units', 'output_bytes'}, 'invalid limits')
            require(all(type(v) is int and v > 0 for v in limits.values()), 'nonpositive limit')
            require(limits['output_bytes'] == 16384, 'v1 output limit must remain fixed')
            checkpoints.append(dict(at_ms=at, goal_version=goal_version,
                required_unavailable=sorted(set(goals[goal_version]['required']) & unavailable),
                scratch_bytes=limits['scratch_bytes'], total_work_units=limits['total_work_units']))
        require(checkpoints == expected['checkpoints'][path.stem], f'checkpoint mismatch: {path.stem}')
    return len(records), len(paths), event_count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture', nargs='?', type=Path, default=DEFAULT_FIXTURE)
    args = parser.parse_args()
    try:
        records, scenarios, events = check_fixture(args.fixture)
    except (ValueError, KeyError, TypeError, OSError) as error:
        parser.exit(1, f'Fixture check failed: {error}\n')
    print(f'Fixture consistent: {records} revisions, {scenarios} scenarios, {events} event checkpoints.')
    print('This does not verify a candidate pack, an execution log, or performance.')


if __name__ == '__main__':
    main()
