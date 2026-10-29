#!/usr/bin/env python3
"""Check page bookkeeping and actual RAM allocation/zeroing in QEMU."""
import argparse
import os
from pathlib import Path
import re
import selectors
import subprocess
import sys
import time

sys.dont_write_bytecode = True
from verify_exceptions import elf_symbols

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    output = ROOT / 'target/verification'
    output.mkdir(parents=True, exist_ok=True)
    unit = output / 'frame-domain-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/memory/frames.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rai-os'] + (['--release'] if args.release else []), cwd=ROOT, check=True)
    kernel = ROOT / f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    start, end = (symbols[name][0] for name in ['__page_pool_start', '__page_pool_end'])
    assert 0x40080000 <= symbols['__kernel_reserved_end'][0] <= start
    assert symbols['__stack_top'][0] <= start and symbols['__exception_stack_top'][0] <= start
    assert end <= 0x48000000 and end - start == 1048576
    assert start % 4096 == end % 4096 == 0
    expected = f'page_size=4096; total_pages=256; free_pages=250; used_pages=6; pool_start=0x{start:016x}; pool_end=0x{end:016x}'
    check_result = 'checked_pages=250; exhausted=true; zeroed=true; reused=true; free_pages=250'
    child = subprocess.Popen(['sh', 'scripts/qemu.sh', str(kernel)], cwd=ROOT,
                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    poll = selectors.DefaultSelector()
    poll.register(child.stdout, selectors.EVENT_READ)
    pending, transcript = bytearray(), bytearray()
    count = 0

    def line():
        deadline = time.monotonic() + 10
        while b'\n' not in pending:
            remaining = deadline - time.monotonic()
            assert remaining > 0, f'QEMU timeout: {transcript!r}'
            for key, _ in poll.select(remaining):
                data = os.read(key.fileobj.fileno(), 4096)
                assert data, f'QEMU exited: {child.poll()}'
                pending.extend(data)
                transcript.extend(data)
        end = pending.index(b'\n') + 1
        result = bytes(pending[:end])
        del pending[:end]
        return result.decode('ascii').rstrip('\r\n')

    def send(operation, expected, argument=None, error=False):
        nonlocal count
        count += 1
        request = f'@rai1\t{count}\t{operation}' + ('' if argument is None else '\t' + argument) + '\n'
        child.stdin.write(request.encode('ascii'))
        child.stdin.flush()
        status = 'err' if error else 'ok'
        actual = line()
        assert actual == f'@rai1\t{count}\t{status}\t{expected}', actual

    try:
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        send('memory.info', expected)
        for _ in range(2):
            send('memory.check', check_result)
            send('memory.info', expected)
        for op in ['memory.info', 'memory.check']:
            for argument in ['', 'unexpected']:
                send(op, 'bad_args', argument, error=True)
        send('memory.free', 'unknown_operation', '0x40080000', error=True)
        send('memory.info', expected)
        for command, result in [('memory.info', expected), ('memory.check', check_result), ('memory.check extra', 'error: unknown_command')]:
            child.stdin.write((command + '\n').encode('ascii'))
            child.stdin.flush()
            assert line() == command
            assert line() == result
        send('task.start', 'task_id=1; state=running; processed=0; total=7; words=0', 'one two')
        time.sleep(.2)
        send('task.status', 'task_id=1; state=completed; processed=7; total=7; words=2')
        send('echo', 'memory remains usable', 'memory remains usable')
        send('memory.info', expected)
        print(f'PASS ({profile}): {count} structured + 3 shell exchanges; ELF reservations, 250 available physical pages, exhaustion, zeroing/reuse, invalid requests, task/echo recovery.')
        print(transcript.decode('ascii'), end='')
    finally:
        child.terminate()
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
        poll.close()
        (output / f'memory-{profile}.log').write_bytes(transcript)


if __name__ == '__main__':
    main()
