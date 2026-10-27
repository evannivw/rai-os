#!/usr/bin/env python3
"""Run panel domain and real-QEMU worker checks from the repository root."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rai-os'], cwd=ROOT, check=True)
    compiler = subprocess.check_output(['rustc', '-vV'], text=True)
    host = next(line[6:] for line in compiler.splitlines() if line.startswith('host: '))
    build = subprocess.run([
        'cargo', 'test', '--locked', '--manifest-path', 'host/panel/Cargo.toml',
        '--target', host, '--target-dir', 'target/host-panel', '--no-run', '--message-format=json',
    ], cwd=ROOT, text=True, capture_output=True, check=True)
    binaries = []
    for line in build.stdout.splitlines():
        item = json.loads(line)
        if item.get('reason') == 'compiler-artifact' and item.get('profile', {}).get('test') and item.get('executable'):
            binaries.append(item['executable'])
    if len(binaries) != 1:
        raise RuntimeError(f'Expected one panel test executable, got {binaries!r}')
    result = subprocess.run([binaries[0], '--include-ignored', '--test-threads=1'],
                            cwd=ROOT, text=True, capture_output=True, timeout=60)
    output = ROOT / 'target/verification/panel-worker.log'
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(result.stdout + result.stderr)
    print(result.stdout)
    if result.returncode:
        raise RuntimeError(result.stderr)
    print('PASS: panel validation, live guest clock, echo/rejection, task progress/cancel/result, priority/FIFO/pause/interruption, stop/restart, close and missing ELF.')
    print('This checks the worker; verify graphical controls separately in the native window.')


if __name__ == '__main__':
    main()
