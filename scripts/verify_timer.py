#!/usr/bin/env python3
"""Verify actual GICv2/CNTP IRQs, preserved context, shutdown and unknown IRQs."""
import argparse
import re
import struct
import subprocess
import sys
sys.dont_write_bytecode = True
from verify_exceptions import elf_symbols
from verify_mmu import ROOT, OUTPUT, guest

RESULT = r'irq=30; ticks=6; rounds=2; registers=31; flags=true; stack=true; irq_masked=true; timer_disabled=true; elapsed_ms=(\d+)'


def instruction(kernel, address):
    data = kernel.read_bytes()
    offset = struct.unpack_from('<Q', data, 32)[0]
    size, count = struct.unpack_from('<HH', data, 54)
    assert size == 56
    for i in range(count):
        kind, _, file_offset, start, _, file_size, _, _ = struct.unpack_from('<IIQQQQQQ', data, offset + size * i)
        if kind == 1 and start <= address and address + 4 <= start + file_size:
            return struct.unpack_from('<I', data, file_offset + address - start)[0]
    raise AssertionError('Instruction not present in loaded ELF segment')


def build(profile, negative=False):
    directory = ROOT / ('target/timer-self-test/unexpected-irq' if negative else 'target')
    command = ['cargo', 'build', '--locked', '--bin', 'rai-os', '--target-dir', str(directory)]
    if profile == 'release': command.append('--release')
    if negative: command += ['--features', 'timer-test-unexpected-irq']
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    loop = symbols['timer_probe_loop'][0]
    assert instruction(kernel, loop) == 0x14000000  # B to itself; no polling or SVC.
    assert symbols['timer_probe_resume'][0] == loop + 4
    assert symbols['exception_vectors'][1] == 2048
    assert symbols['exception_vectors'][0] % 2048 == 0
    if not negative: assert b'SELF-TEST: unexpected SGI 1' not in kernel.read_bytes()
    return kernel, symbols


def normal(profile):
    kernel, _ = build(profile)
    with guest(kernel, f'normal-{profile}', prefix='timer') as (child, _, line, _, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'Console ready: type help for commands'
        sequence = 0
        def request(command, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{command}\n'.encode())
            child.stdin.flush()
            actual = line().split('\t', 3)
            assert actual[:3] == ['@rai1', str(sequence), status], actual
            return actual[3]
        before = request('mmu.info')
        assert 'enabled=true;' in before and 'table_pages=6;' in before
        for _ in range(3):
            reply = request('timer.check')
            match = re.fullmatch(RESULT, reply)
            assert match and int(match[1]) >= 60, reply
            assert request('context.check') == 'round_trips=2; registers=31; flags=true; stack=true; resumed=true'
            assert request('user.run') == 'mode=EL0; state=exited; code=0; syscalls=2; rejected=0; output=Hello from EL0'
            assert request('echo\tafter IRQs') == 'after IRQs'
        assert request('timer.check\t', 'err') == 'bad_args'
        assert request('timer.check\textra', 'err') == 'bad_args'
        assert request('mmu.info') == before
        assert request('memory.check') == 'checked_pages=250; exhausted=true; zeroed=true; reused=true; free_pages=250'
        child.stdin.write(b'timer.check\n')
        child.stdin.flush()
        assert line() == 'timer.check'
        match = re.fullmatch(RESULT, line())
        assert match and int(match[1]) >= 60
        assert request('echo\tfinal cleanup') == 'final cleanup'
        print(f'PASS ({profile}): 24 real timer IRQs in four checks, all GPRs/NZCV/SP, actual shutdown/masks, repeated start, invalid arguments and kernel/EL0/memory continuity.')
        print(transcript.decode('ascii'), end='')


def unexpected(profile):
    kernel, symbols = build(profile, negative=True)
    with guest(kernel, f'unexpected-{profile}', prefix='timer') as (child, poll, line, pending, transcript):
        assert line() == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011'
        assert line() == 'SELF-TEST: unexpected SGI 1'
        assert line() == 'IRQ: unexpected intid=1; vector=5'
        report = line()
        match = re.fullmatch(r'EXCEPTION: vector=5; esr=0x[0-9a-f]{16}; ec=0x[0-9a-f]{2}; '
                             r'elr=0x([0-9a-f]{16}); spsr=0x([0-9a-f]{16}); far_raw=0x[0-9a-f]{16}; '
                             r'entry_sp=0x([0-9a-f]{16}); handler_sp=0x([0-9a-f]{16})', report)
        assert match, report  # ESR/FAR have no synchronous meaning for an IRQ.
        pc, spsr, entry_sp, handler_sp = [int(v, 16) for v in match.groups()]
        assert pc == symbols['timer_probe_loop'][0] and spsr == 0xa0000345, report
        assert symbols['__stack_bottom'][0] <= entry_sp <= symbols['__stack_top'][0]
        assert symbols['__exception_stack_bottom'][0] <= handler_sp < symbols['__exception_stack_top'][0]
        assert entry_sp % 16 == handler_sp % 16 == 0
        assert line() == 'HALTED: exception is fatal; restart QEMU'
        child.stdin.write(b'info\n')
        child.stdin.flush()
        assert not pending and not poll.select(.25) and child.poll() is None
        print(f'PASS ({profile}): actual unexpected SGI 1 reached vector 5 and remained fatal; exact PC/state/stacks and halt verified.')
        print(transcript.decode('ascii'), end='')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    normal(profile)
    unexpected(profile)


if __name__ == '__main__':
    main()
