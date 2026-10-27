#!/usr/bin/env python3
"""Verify pure task transitions and real guest progress, cancellation, and framing."""
import argparse
import os
from pathlib import Path
import re
import selectors
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    output = ROOT / 'target/verification'
    output.mkdir(parents=True, exist_ok=True)
    unit = output / 'task-domain-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/task.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    subprocess.run(['cargo', 'build', '--locked'] + (['--release'] if args.release else []), cwd=ROOT, check=True)
    profile = 'release' if args.release else 'debug'
    child = subprocess.Popen(['sh', 'scripts/qemu.sh', f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'], cwd=ROOT,
                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    poll = selectors.DefaultSelector()
    poll.register(child.stdout, selectors.EVENT_READ)
    pending = bytearray()
    transcript = bytearray()
    sequence = 0

    def line():
        deadline = time.monotonic() + 10
        while b'\n' not in pending:
            remaining = deadline-time.monotonic()
            if remaining <= 0:
                raise AssertionError('QEMU response timeout')
            for key, _ in poll.select(remaining):
                data = os.read(key.fileobj.fileno(), 4096)
                assert data, 'QEMU exited'
                pending.extend(data)
                transcript.extend(data)
        end = pending.index(b'\n')+1
        result = bytes(pending[:end])
        del pending[:end]
        return result.decode('ascii').rstrip('\r\n')

    def send(op, arg=None, error=None):
        nonlocal sequence
        sequence += 1
        request = f'@rai1\t{sequence}\t{op}' + ('' if arg is None else '\t'+arg)
        child.stdin.write((request+'\n').encode('ascii'))
        child.stdin.flush()
        fields = line().split('\t')
        assert fields[:3] == ['@rai1', str(sequence), 'err' if error else 'ok'], fields
        assert len(fields) == 4, fields
        if error: assert fields[3] == error, fields
        return fields[3]

    def snapshot(payload):
        match = re.fullmatch(r'task_id=(\d+); state=(idle|running|completed|cancelled); processed=(\d+); total=(\d+); words=(\d+)', payload)
        assert match, payload
        task_id, state, processed, total, words = match.groups()
        return int(task_id), state, int(processed), int(total), int(words)

    try:
        assert line().startswith('RAI-OS: boot OK')
        assert line() == 'Console ready: type help for commands'
        assert snapshot(send('task.status')) == (0, 'idle', 0, 0, 0)
        assert snapshot(send('task.start', '')) == (1, 'completed', 0, 0, 0)
        text = 'a ' * 96
        assert snapshot(send('task.start', text)) == (2, 'running', 0, 192, 0)
        send('task.start', 'replacement', error='task_busy')
        send('task.cancel', '1', error='stale_task')
        assert send('echo', 'responsive') == 'responsive'
        assert send('uptime').startswith('uptime_ms=')
        time.sleep(.25)
        progress = snapshot(send('task.status'))
        assert progress[0:2] == (2, 'running') and 0 < progress[2] < 192, progress
        cancelled = snapshot(send('task.cancel', '2'))
        assert cancelled[1] == 'cancelled'
        time.sleep(.25)
        assert snapshot(send('task.status')) == cancelled
        send('task.cancel', '2', error='task_not_running')
        send('task.start', 'x'*193, error='task_too_large')
        send('task.start', error='bad_args')
        for invalid in ['0', '-1', '4294967296', 'abc', '2\textra']:
            send('task.cancel', invalid, error='bad_args')
        send('task.status', '', error='bad_args')
        assert snapshot(send('task.status')) == cancelled
        text = 'abcdefghijk 12, abc-def! ' * 7
        started = snapshot(send('task.start', text))
        assert started[0:2] == (3, 'running')
        # No complete commands arrive for three seconds. The guest must keep
        # processing independently of the host, even with a partial UART frame.
        sequence += 1
        child.stdin.write(f'@rai1\t{sequence}\ttask.st'.encode())
        child.stdin.flush()
        assert not poll.select(3.0), 'Unexpected response before line completion'
        child.stdin.write(b'atus\n')
        child.stdin.flush()
        fields = line().split('\t')
        assert fields[:3] == ['@rai1', str(sequence), 'ok'], fields
        result = snapshot(fields[3])
        expected_words = len(re.findall('[A-Za-z0-9]+', text))
        assert result == (3, 'completed', len(text), len(text), expected_words), result
        send('task.cancel', '3', error='task_not_running')
        assert snapshot(send('task.start', '')) == (4, 'completed', 0, 0, 0)
        send('task.cancel', '3', error='stale_task')
        child.stdin.write(b'task.start ab-c\n')
        child.stdin.flush()
        assert line() == 'task.start ab-c'
        assert snapshot(line()) == (5, 'running', 0, 4, 0)
        time.sleep(.15)
        child.stdin.write(b'task.status\n')
        child.stdin.flush()
        assert line() == 'task.status'
        assert snapshot(line()) == (5, 'completed', 4, 4, 2)
        child.stdin.write(b'task.cancel 5\n')
        child.stdin.flush()
        assert line() == 'task.cancel 5'
        assert line() == 'error: task_not_running'
        print(f'PASS: {sequence} structured and 3 shell real-QEMU task exchanges; progress during incomplete input, cancellation, exact result, bounds and recovery.')
    finally:
        child.terminate()
        try: child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
        poll.close()
        (output/f'tasks-{profile}.log').write_bytes(transcript)


if __name__ == '__main__':
    main()
