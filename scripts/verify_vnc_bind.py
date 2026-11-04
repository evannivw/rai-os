#!/usr/bin/env python3
"""Check legacy VNC defaults and an actual QEMU listener; no model requests."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time

from files_demo import ROOT


def query_listener(kernel, listen, output):
    """Start only our paused QEMU process and ask QMP which socket it bound."""
    env = dict(os.environ)
    env['RAI_DISPLAY'] = 'vnc'
    env.pop('MODEL_API_KEY', None)
    env.pop('RAI_VNC_LISTEN', None)
    if listen is not None:
        env['RAI_VNC_LISTEN'] = listen
    with tempfile.TemporaryDirectory(prefix='rai-vnc-', dir='/tmp') as directory:
        qmp = Path(directory) / 'qmp'
        with output.open('wb') as log:
            child = subprocess.Popen(
                ['sh', 'scripts/qemu.sh', str(kernel), '--graphics', '-S',
                 '-qmp', f'unix:{qmp},server=on,wait=off'],
                cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                stdout=log, stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 10
                while not qmp.exists():
                    assert child.poll() is None, 'QEMU exited before QMP startup'
                    assert time.monotonic() < deadline, 'QMP startup timed out'
                    time.sleep(.05)
                with socket.socket(socket.AF_UNIX) as client:
                    client.settimeout(5)
                    client.connect(str(qmp))
                    with client.makefile('rwb', buffering=0) as stream:
                        assert 'QMP' in json.loads(stream.readline())
                        result = None
                        for command in ['qmp_capabilities', 'query-vnc']:
                            stream.write((json.dumps(dict(execute=command)) + '\n').encode())
                            while True:
                                response = json.loads(stream.readline())
                                assert 'error' not in response, response
                                if 'return' in response:
                                    result = response['return']
                                    break
                        return result
            finally:
                child.terminate()
                try:
                    child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait()


def verify(kernel, output):
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    report = dict(verified=False, live_meta_calls=0, checks=checks)
    try:
        with tempfile.TemporaryDirectory(prefix='rai-qemu-args-') as directory:
            fake = Path(directory) / 'qemu-system-aarch64'
            fake.write_text('#!/usr/bin/env python3\nimport json,sys\nprint(json.dumps(sys.argv[1:]))\n')
            fake.chmod(0o700)
            env = dict(os.environ)
            env['PATH'] = directory + os.pathsep + env['PATH']
            env['RAI_DISPLAY'] = 'vnc'
            env.pop('RAI_VNC_LISTEN', None)
            env.pop('MODEL_API_KEY', None)
            command = ['sh', 'scripts/qemu.sh', str(kernel), '--graphics']
            args = json.loads(subprocess.check_output(command, cwd=ROOT, env=env, text=True))
            assert args[args.index('-vnc') + 1] == '127.0.0.1:0'
            env['RAI_VNC_LISTEN'] = '0.0.0.0:37'
            args = json.loads(subprocess.check_output(command, cwd=ROOT, env=env, text=True))
            assert args[args.index('-vnc') + 1] == '0.0.0.0:37'
            env['RAI_DISPLAY'] = 'invalid'
            result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
            assert result.returncode == 1 and 'RAI_DISPLAY must be' in result.stderr
            assert not result.stdout
            checks.append('Wrapper defaults to loopback, preserves explicit listen override, rejects invalid display backend')

        # Avoid interfering with another viewer/session already using the default port.
        with socket.socket() as probe:
            try:
                probe.bind(('127.0.0.1', 5900))
                default_available = True
            except OSError:
                default_available = False
        if default_available:
            listener = query_listener(kernel, None, output / 'qemu-default.log')
            assert listener['enabled'] and listener['host'] == '127.0.0.1'
            assert listener['service'] == '5900'
            report['default_listener'] = listener
            checks.append('Actual default QEMU VNC listener reports 127.0.0.1:5900 through QMP')
        else:
            report['default_listener_skipped'] = 'Port 5900 already in use; existing listener left untouched'
        with socket.socket() as probe:
            probe.bind(('127.0.0.1', 0))
            port = probe.getsockname()[1]
        assert port >= 5900
        listener = query_listener(kernel, f'127.0.0.1:{port - 5900}', output / 'qemu-override.log')
        assert listener['enabled'] and listener['host'] == '127.0.0.1'
        assert listener['service'] == str(port)
        report['override_listener'] = listener
        checks.append('Actual explicit loopback display selects its expected TCP port through QMP')
        report['verified'] = True
    finally:
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--kernel', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    verify(args.kernel.resolve(), args.output.resolve())
