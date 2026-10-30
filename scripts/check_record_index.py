#!/usr/bin/env python3
"""Independent exact per-record index checker; no kernel implementation imported."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

FIXTURE = Path(__file__).resolve().parents[1] / 'fixtures/review_pack/v1'
MAX_ARTIFACT_BYTES = 512


def check(payload, key):
    """Trust only the requested key and immutable fixture, not artifact metadata."""
    catalog = json.loads((FIXTURE / 'catalog.json').read_text())['records']
    record = next((r for r in catalog if r['key'] == key and r['read_allowed']), None)
    if record is None:
        raise ValueError('Requested record is unknown or denied')
    if len(payload) > MAX_ARTIFACT_BYTES:
        raise ValueError('Index artifact exceeds its byte limit')
    source = (FIXTURE / record['path']).read_bytes()
    if len(source) != record['byte_length'] or hashlib.sha256(source).hexdigest() != record['sha256']:
        raise ValueError('Fixture bytes differ from the catalog')
    # Separate algorithm: regex, lowercase, multiset, then sorted serialization.
    counts = Counter(token.decode('ascii').lower() for token in re.findall(rb'[A-Za-z0-9]+', source))
    index = ','.join(f'{token}:{count}' for token, count in sorted(counts.items())) or '-'
    expected = f'record={key}; bytes={len(source)}; worker=EL0; state=completed; index={index}'
    if payload != expected:
        raise ValueError('Index differs from the exact requested revision, token frequencies or canonical format')
    return counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact', type=Path)
    parser.add_argument('--record', required=True, help='Expected exact revision, independent of the artifact')
    parser.add_argument('--find', help='Optional complete ASCII token lookup after successful verification')
    args = parser.parse_args()
    try:
        with args.artifact.open('rb') as file:
            data = file.read(MAX_ARTIFACT_BYTES + 2)
        if not data.endswith(b'\n') or len(data) > MAX_ARTIFACT_BYTES + 1:
            raise ValueError('Expected one bounded, newline-terminated artifact')
        counts = check(data[:-1].decode('ascii'), args.record)
        if args.find is not None and re.fullmatch(r'[A-Za-z0-9]+', args.find) is None:
            raise ValueError('Search requires one complete ASCII alphanumeric token')
    except (OSError, ValueError) as error:
        parser.exit(1, f'REJECTED: {error}\n')
    print(f'VERIFIED: {args.record}; distinct_tokens={len(counts)}; total_tokens={sum(counts.values())}')
    if args.find is not None:
        print(f'{args.find.lower()}: {counts.get(args.find.lower(), 0)} occurrence(s)')


if __name__ == '__main__':
    main()
