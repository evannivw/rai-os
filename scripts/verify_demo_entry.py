#!/usr/bin/env python3
"""Exercise the actual demo CLI: VNC startup, resume and empty live mode."""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import time

sys.dont_write_bytecode = True
from files_demo import ROOT
from demo import prepare
from workspace_vnc import exact


def verify(output):
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    def launch(action, folder, label=None):
        with socket.socket() as s:
            s.bind(('127.0.0.1', 0)); port = s.getsockname()[1]
        env = dict(os.environ)
        env.pop('RAI_VNC_PASSWORD', None)
        env.pop('MODEL_API_KEY', None)
        if action=='live':env['MODEL_API_KEY']='LLM_synthetic_format_only_0000'
        with (output / f'{label or action}.log').open('w') as log:
            process = subprocess.Popen([sys.executable, '-B', 'scripts/demo.py', action,
                                        '--output', str(folder), '--port', str(port)],
                                       cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
            connected = None
            try:
                deadline = time.monotonic() + 45
                while connected is None:
                    if process.poll() is not None:
                        raise AssertionError(f'{action} exited before VNC startup')
                    try: connected = socket.create_connection(('127.0.0.1', port), timeout=1)
                    except ConnectionRefusedError:
                        if time.monotonic() >= deadline: raise
                        time.sleep(.1)
                with connected:
                    connected.settimeout(5)
                    assert exact(connected, 12) == b'RFB 003.008\n'
                    connected.sendall(b'RFB 003.008\n')
                    assert exact(connected, 2) == b'\x01\x01'
                    connected.sendall(b'\x01'); assert exact(connected, 4) == b'\0'*4
                    connected.sendall(b'\x01'); header = exact(connected, 24)
                    assert header[:4] == b'\x01\xe0\x03\x20'  # Actual 480 x 800 framebuffer.
                    exact(connected, int.from_bytes(header[20:24], 'big'))
            finally:
                if process.poll() is None: process.send_signal(signal.SIGINT)
                try: process.wait(timeout=15)
                except subprocess.TimeoutExpired: process.kill(); process.wait(); raise
            assert process.returncode == 0
    def rejected(action, folder, marker, label):
        # Failed startup must preserve the existing device and return a useful
        # message rather than an uncaught traceback. No provider request is made.
        before = {str(p.relative_to(folder)): p.read_bytes()
                  for p in folder.rglob('*') if p.is_file()} if folder.exists() else {}
        result = subprocess.run(
            [sys.executable, '-B', 'scripts/demo.py', action, '--output', str(folder)],
            cwd=ROOT, capture_output=True, text=True, timeout=45,
        )
        (output / f'{label}.log').write_text(result.stdout + result.stderr)
        assert result.returncode == 1 and marker in result.stderr, result.stderr
        assert 'Traceback' not in result.stderr, result.stderr
        after = {str(p.relative_to(folder)): p.read_bytes()
                 for p in folder.rglob('*') if p.is_file()} if folder.exists() else {}
        assert before == after, 'Rejected startup changed saved files'

    try:
        # Check supplied paths before creating even a partial output directory.
        # Dummy binaries make this a no-guest, no-provider boundary check.
        for label, seed in [('missing', output / 'missing-seed.json'), ('directory', output)]:
            rejected_output = output / f'rejected-seed-{label}'
            try:
                prepare(rejected_output, Path('unused-bridge'), Path('unused-kernel'), showcase=seed)
            except ValueError as error:
                assert 'existing regular file' in str(error)
            else:
                raise AssertionError('Invalid supplied checkpoint path accepted')
            assert not rejected_output.exists(), 'Rejected checkpoint left a partial output directory'
        saved = output / 'saved'
        launch('saved', saved)
        original = (saved / 'device.json').read_bytes()
        assert len(json.loads(original)['state']['apps']) == 3
        launch('resume', saved)
        assert (saved / 'device.json').read_bytes() == original
        assert len(list(saved.glob('session-*'))) == 2
        (saved / 'START.txt').unlink()
        launch('resume', saved, 'resume-without-instructions')
        assert (saved / 'device.json').read_bytes() == original
        assert not (saved / 'START.txt').exists()
        assert len(list(saved.glob('session-*'))) == 3
        checks.append('Saved CLI serves actual VNC framebuffer; resume preserves checkpoint bytes and separate session logs, even when optional START.txt is missing')
        rejected('saved', saved, 'output directory already exists', 'existing-output')
        rejected('resume', output / 'missing', 'No saved demo manifest', 'missing-resume')
        malformed = output / 'malformed'
        malformed.mkdir()
        (malformed / 'demo.json').write_text('{broken')
        rejected('resume', malformed, 'manifest could not be read', 'invalid-json')
        (malformed / 'demo.json').write_text('[]')
        rejected('resume', malformed, 'Unsupported demo manifest', 'invalid-shape')
        (malformed / 'demo.json').write_text(json.dumps(dict(format='rai-demo-1', source='saved')))
        rejected('resume', malformed, 'Saved device is missing', 'missing-checkpoint')
        checks.append('Missing/non-file supplied checkpoints leave no partial output; existing output, missing resume, invalid manifest JSON/shape and missing checkpoint fail clearly without modifying saved files')
        live = output / 'live'
        launch('live', live)
        assert json.loads((live / 'device.json').read_text())['state'] == dict(version=1, apps=[])
        assert json.loads((live / 'demo.json').read_text())['source'] == 'meta'
        assert not list(output.rglob('request.json'))
        checks.append('Live CLI starts with an empty device; no model request without explicit user submission; all sessions stop cleanly')
    finally:
        (output / 'report.json').write_text(json.dumps(dict(checks=checks, verified=len(checks)==3, live_calls=0), indent=2)+'\n')
    print(json.dumps(checks, indent=2))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    verify(args.output.resolve())
