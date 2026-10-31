#!/usr/bin/env python3
"""Independent exact-source/index check for a bounded guest index set, not full R1-R6 acceptance."""
import argparse
from collections import defaultdict
import hashlib
import json
import re
import sys
sys.dont_write_bytecode=True
from pathlib import Path
from check_record_index import check as check_index, FIXTURE

MAX_BYTES = 4096


def unique(pairs):
    result = {}
    for key,value in pairs:
        if key in result: raise ValueError('Duplicate JSON field')
        result[key]=value
    return result


def check(payload, required, generation):
    # Expected goal identity and requirements come from the caller, not the artifact.
    if not isinstance(payload,str) or len(payload.encode('ascii')) > MAX_BYTES:
        raise ValueError('Artifact size or encoding is invalid')
    required=sorted(required)
    if not 1 <= len(required) <= 4 or len(set(required)) != len(required) or len({k.split('@')[0] for k in required}) != len(required):
        raise ValueError('Expected set must contain unique document revisions')
    if type(generation) is not int or not 1 <= generation <= 4294967295:
        raise ValueError('Expected goal version is invalid')
    obj=json.loads(payload,object_pairs_hook=unique)
    if not isinstance(obj,dict) or set(obj) != {'format','goal_version','required','records'}:
        raise ValueError('Unexpected artifact fields')
    if obj['format'] != 'rai-index-set-v1' or type(obj['goal_version']) is not int or obj['goal_version'] != generation or obj['required'] != required:
        raise ValueError('Artifact does not match the requested goal')
    rows=obj['records']
    if not isinstance(rows,list) or len(rows) != len(required):
        raise ValueError('Missing or extra record')
    catalog={r['key']:r for r in json.loads((FIXTURE/'catalog.json').read_text())['records']}
    postings=defaultdict(dict)
    for row,key in zip(rows,required):
        if not isinstance(row,dict) or set(row) != {'record','bytes','source_hex','index'} or row['record'] != key:
            raise ValueError('Record identity/order or fields differ')
        meta=catalog.get(key)
        if meta is None or not meta['read_allowed']: raise ValueError('Unknown or denied revision')
        source=(FIXTURE/meta['path']).read_bytes()
        if len(source) != meta['byte_length'] or hashlib.sha256(source).hexdigest() != meta['sha256']:
            raise ValueError('Fixture bytes differ from catalog')
        if type(row['bytes']) is not int or row['bytes'] != len(source) or row['source_hex'] != source.hex():
            raise ValueError('Source bytes differ from required revision')
        counts=check_index(f"record={key}; bytes={len(source)}; worker=EL0; state=completed; index={row['index']}",key)
        for token,count in counts.items(): postings[token][key]=count
    return dict(sorted(postings.items()))


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('artifact',type=Path)
    p.add_argument('--required',required=True);p.add_argument('--goal-version',type=int,required=True)
    p.add_argument('--find',help='Complete ASCII token lookup after independent verification')
    args=p.parse_args()
    try:
        with args.artifact.open('rb') as f: data=f.read(MAX_BYTES+2)
        if not data.endswith(b'\n') or len(data)>MAX_BYTES+1: raise ValueError('Expected bounded newline-terminated artifact')
        index=check(data[:-1].decode('ascii'),args.required.split(','),args.goal_version)
        if args.find is not None and re.fullmatch(r'[A-Za-z0-9]+',args.find) is None: raise ValueError('Search requires a complete ASCII alphanumeric token')
    except (OSError,ValueError,TypeError,KeyError) as error: p.exit(1,f'REJECTED: {error}\n')
    print(f'VERIFIED: goal_version={args.goal_version}; records={len(args.required.split(","))}; distinct_tokens={len(index)}')
    if args.find is not None:
        postings=index.get(args.find.lower(),{})
        print(args.find.lower()+': '+(', '.join(f'{key}={count}' for key,count in sorted(postings.items())) or 'no matches'))


if __name__=='__main__': main()
