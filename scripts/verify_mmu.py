#!/usr/bin/env python3
"""Verify active EL1 mappings and five hardware-rejected accesses in real QEMU."""
import argparse
from contextlib import contextmanager
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
OUTPUT = ROOT / 'target/verification'
CASES = ['write-text', 'write-rodata', 'write-table', 'execute-data', 'unmapped']


@contextmanager
def guest(kernel, name):
    child = subprocess.Popen(['sh', 'scripts/qemu.sh', str(kernel)], cwd=ROOT,
                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    poll = selectors.DefaultSelector()
    poll.register(child.stdout, selectors.EVENT_READ)
    pending, transcript = bytearray(), bytearray()

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

    try:
        yield child, poll, line, pending, transcript
    finally:
        child.terminate()
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
        poll.close()
        (OUTPUT / f'mmu-{name}.log').write_bytes(transcript)


def build(profile, case=None):
    directory = ROOT / ('target' if case is None else f'target/mmu-self-test/{case}')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if case is not None: command += ['--features', f'mmu-test-{case}']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    bounds = [symbols[name][0] for name in ['__text_start', '__text_end', '__rodata_end', '__page_pool_start', '__page_pool_end']]
    assert all(value % 4096 == 0 for value in bounds)
    assert bounds == sorted(bounds) and len(set(bounds)) == len(bounds)
    assert bounds[0] == 0x40080000 and bounds[-1] <= 0x40200000
    assert symbols['__data_start'][0] == symbols['__rodata_end'][0]
    assert symbols['__exception_stack_top'][0] <= bounds[-2]
    if case is None:
        assert 'mmu_test_fault' not in symbols and 'exception_test_trigger' not in symbols
    return kernel, symbols


def normal(profile):
    kernel, symbols = build(profile)
    root = symbols['__page_pool_start'][0]
    info = (f'enabled=true; mode=identity; granule=4096; table_pages=5; root=0x{root:016x}; '
            'tcr=0x0000000080993019; mair=0x0000000000004400; wxn=true; caches=false')
    memory = (f'page_size=4096; total_pages=256; free_pages=251; used_pages=5; pool_start=0x{root:016x}; '
              f'pool_end=0x{symbols["__page_pool_end"][0]:016x}')
    with guest(kernel, f'normal-{profile}') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        requests = [
            ('mmu.info', 'ok', info),
            ('memory.info', 'ok', memory),
            ('memory.check', 'ok', 'checked_pages=251; exhausted=true; zeroed=true; reused=true; free_pages=251'),
            ('mmu.info', 'ok', info),
            ('mmu.info\t', 'err', 'bad_args'),
            ('mmu.info\textra', 'err', 'bad_args'),
            ('echo\tprotected kernel alive', 'ok', 'protected kernel alive'),
        ]
        for index, (command, status, payload) in enumerate(requests, 1):
            child.stdin.write(f'@rai1\t{index}\t{command}\n'.encode())
            child.stdin.flush()
            actual = line()
            assert actual == f'@rai1\t{index}\t{status}\t{payload}', actual
        child.stdin.write(b'mmu.info\n')
        child.stdin.flush()
        assert line() == 'mmu.info'
        assert line() == info
        print(f'PASS ({profile}): enabled MMU register readback, identity root, 5 owned table pages, memory check preserves mappings, arguments and shell.')
        print(transcript.decode('ascii'), end='')


def fault(profile, case):
    kernel, symbols = build(profile, case)
    targets = {'write-text': symbols['__text_start'][0], 'write-rodata': symbols['__text_end'][0],
               'write-table': symbols['__page_pool_start'][0], 'unmapped': 0x40000000}
    target = symbols['mmu_execute_target'][0] if case == 'execute-data' else targets[case]
    pc = target if case == 'execute-data' else symbols['mmu_fault_instruction'][0]
    with guest(kernel, f'{case}-{profile}') as (child, poll, line, pending, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == f'SELF-TEST: MMU {case}'
        report = line()
        match = re.fullmatch(r'EXCEPTION: vector=(\d+); esr=0x([0-9a-f]{16}); ec=0x([0-9a-f]{2}); '
                             r'elr=0x([0-9a-f]{16}); spsr=0x([0-9a-f]{16}); far_raw=0x([0-9a-f]{16}); '
                             r'entry_sp=0x([0-9a-f]{16}); handler_sp=0x([0-9a-f]{16})', report)
        assert match, report
        esr, ec, elr, spsr, far, entry_sp, handler_sp = [int(value, 16) for value in match.groups()[1:]]
        assert int(match[1]) == 4 and ec == (0x21 if case == 'execute-data' else 0x25), report
        assert (esr >> 26) & 63 == ec and esr & (1 << 25), report
        assert esr & 63 == (7 if case == 'unmapped' else 15), report  # L3 translation / permission.
        assert esr & (1 << 10) == 0, 'FAR must be valid for this injected abort'
        if case != 'execute-data': assert bool(esr & (1 << 6)) == case.startswith('write-'), report
        assert elr == pc and far == target, (report, hex(pc), hex(target))
        assert spsr & 15 == 5 and spsr & 0x3c0 == 0x3c0, report
        assert symbols['__stack_bottom'][0] <= entry_sp <= symbols['__stack_top'][0]
        assert symbols['__exception_stack_bottom'][0] <= handler_sp < symbols['__exception_stack_top'][0]
        assert entry_sp % 16 == handler_sp % 16 == 0
        assert line() == 'HALTED: exception is fatal; restart QEMU'
        child.stdin.write(b'info\n')
        child.stdin.flush()
        assert not pending and not poll.select(.25), 'Fatal exception resumed or emitted extra output'
        assert child.poll() is None
        print(f'PASS ({profile}): {case} rejected by hardware; exact class/status, fault PC/address, diagnostic stack and halt.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--case', choices=['all', *CASES], default='all')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    unit = OUTPUT / 'mmu-table-tests'
    subprocess.run(['rustc', '--edition=2024', '--test', 'src/mmu/tables.rs', '-o', str(unit)], cwd=ROOT, check=True)
    subprocess.run([str(unit)], check=True)
    normal(profile)
    for case in CASES if args.case == 'all' else [args.case]: fault(profile, case)


if __name__ == '__main__':
    main()
