#!/usr/bin/env python3
"""Offline bridge tests with real QEMU; optional audit of a separately captured live run."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from verify_service import CATALOG, check as check_service
ROOT = Path(__file__).resolve().parents[1]


def events(text):
    return [json.loads(line) for line in text.splitlines() if line]


def check_capture(items, source):
    assert items[0]['event'] == 'proposal_source' and items[0]['source'] == source
    assert items[-1]['event'] == 'finished'
    proposals = [item['proposal'] for item in items if item['event'] == 'validated_proposal']
    assert len(proposals) == 1
    proposal = proposals[0]
    assert proposal['state_version'] == 1
    if source == 'meta':
        responses = [item for item in items if item['event'] == 'model_response']
        assert len(responses) == 1 and responses[0]['model'] == 'muse-spark-1.3'
        assert responses[0]['latency_ms'] >= 0
    else:
        assert not any(item['event'] == 'model_response' for item in items)
        assert items[0]['model'] is None
    results = [item for item in items if item['event'] == 'guest_result']
    boots = [item for item in items if item['event'] == 'guest_boot_confirmed']
    if proposal['operation'] == 'decline':
        assert proposal['record'] == '' and not results and not boots
        assert items[-1]['status'] == 'declined'
    else:
        assert proposal['operation'] == 'record.count' and len(results) == len(boots) == 1
        assert items[-1]['status'] == 'observed'
        record = next(r for r in CATALOG if r['key'] == proposal['record'] and r['read_allowed'])
        fields = check_service(results[0]['payload'], record)
        assert results[0]['observed'] == dict(record=record['key'], bytes=int(fields['bytes']), tokens=int(fields['tokens']))
        assert results[0]['request_id'] == 1


def verify(release):
    profile = 'release' if release else 'debug'
    compiler = subprocess.check_output(['rustc', '-vV'], text=True)
    host = next(line[6:] for line in compiler.splitlines() if line.startswith('host: '))
    flags = ['--release'] if release else []
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rai-os', *flags], cwd=ROOT, check=True)
    subprocess.run(['cargo', 'build', '--locked', '--manifest-path', 'host/meta/Cargo.toml',
                    '--target', host, '--target-dir', 'target/host-meta', *flags], cwd=ROOT, check=True)
    binary = ROOT / f'target/host-meta/{host}/{profile}/rai-meta'
    kernel = ROOT / f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'
    transcript = []
    with tempfile.TemporaryDirectory(prefix='rai-meta-test-') as temp:
        temp = Path(temp)
        qemu = shutil.which('qemu-system-aarch64'); assert qemu
        # Only dummy credentials in this test. Prove replay never invokes curl and
        # that the QEMU child does not inherit a host MODEL_API_KEY environment.
        wrapper = temp / 'qemu-system-aarch64'
        wrapper.write_text('#!' + sys.executable + '\nimport os\nfrom pathlib import Path\n'
                           + f'Path({str(temp / "child-env")!r}).write_text("leaked" if "MODEL_API_KEY" in os.environ else "removed")\n'
                           + f'os.execv({qemu!r}, [{qemu!r}, *os.sys.argv[1:]])\n')
        wrapper.chmod(0o700)
        fake_curl = temp / 'curl'
        fake_curl.write_text('#!' + sys.executable + '\nfrom pathlib import Path\n'
                            + f'Path({str(temp / "network-called")!r}).touch()\nraise SystemExit(99)\n')
        fake_curl.chmod(0o700)
        env = dict(os.environ, MODEL_API_KEY='dummy_bridge_test_key_not_real', PATH=str(temp) + os.pathsep + os.environ['PATH'])
        path = temp / 'proposal.json'
        def run(proposal, expected=None, elf=kernel):
            path.write_text(proposal if isinstance(proposal, str) else json.dumps(proposal))
            result = subprocess.run([str(binary), str(elf), '--replay', str(path)], cwd=ROOT,
                                    env=env, text=True, capture_output=True, timeout=20)
            items = events(result.stdout)
            if expected:
                assert result.returncode != 0
                assert events(result.stderr)[-1]['error'] == expected, result.stderr
                assert not any(item['event'] in ['guest_boot_confirmed', 'guest_result'] for item in items)
            else:
                assert result.returncode == 0, result.stderr
                check_capture(items, 'replay')
            transcript.extend(result.stdout.splitlines())
            transcript.extend(result.stderr.splitlines())
            assert not (temp / 'network-called').exists(), 'Offline test contacted provider'
            return items
        base = dict(state_version=1, operation='record.count', record='notes@2', reason='Explicit offline replay; not a model decision.')
        for record in CATALOG:
            proposal = dict(base, record=record['key'])
            run(proposal, None if record['read_allowed'] else 'record_not_allowed')
        assert (temp / 'child-env').read_text() == 'removed'
        for changes, error in [({'state_version': 0}, 'stale_proposal'), ({'operation': 'shell'}, 'operation_not_allowed'),
                               ({'record': 'notes@99'}, 'record_not_allowed'), ({'record': 'notes@2\tinfo'}, 'record_not_allowed'),
                               ({'reason': '\x1b[2J'}, 'invalid_proposal_reason')]:
            run(dict(base, **changes), error, elf='/nonexistent/should-not-start')
        for raw in [json.dumps(dict(base, shell='unexpected')), json.dumps(base).replace('{', '{"state_version":1,')]:
            run(raw, 'invalid_proposal_json', elf='/nonexistent/should-not-start')
        run('x' * 2049, 'local_file_too_large', elf='/nonexistent/should-not-start')
        run(dict(base, operation='decline', record=''), elf='/nonexistent/should-not-start')
        # Empty environment setting must fail locally, not fall back to the real file.
        result = subprocess.run([str(binary), str(kernel), 'Count the latest notes.'], cwd=ROOT,
                                env=dict(env, MODEL_API_KEY=''), text=True, capture_output=True, timeout=5)
        assert result.returncode and events(result.stderr)[0]['error'] == 'invalid_api_key_format'
        assert not (temp / 'network-called').exists()
    output = ROOT / f'target/verification/meta-offline-{profile}.jsonl'
    output.write_text('\n'.join(transcript) + '\n')
    print(f'PASS ({profile}): six real-QEMU records independently checked; denied/stale/injected proposals blocked; decline without boot; no provider calls; credentials absent from QEMU environment.')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--check-live', type=Path)
    args = parser.parse_args()
    if args.check_live:
        check_capture(events(args.check_live.read_text()), 'meta')
        print('PASS: captured Meta run is consistent; guest counts independently checked when present. A log alone is not cryptographic proof of a network call.')
    else:
        verify(args.release)


if __name__ == '__main__': main()
