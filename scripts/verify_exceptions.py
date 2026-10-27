#!/usr/bin/env python3
"""Check real EL1 exception entry with an isolated, deliberately faulting image."""
import argparse
import os
from pathlib import Path
import re
import selectors
import struct
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def elf_symbols(path):
    """Read the linked ELF64 symbol table without requiring a host ELF toolchain."""
    data = path.read_bytes()
    assert data[:6] == b'\x7fELF\x02\x01', 'Expected little-endian ELF64'
    assert struct.unpack_from('<H', data, 18)[0] == 183, 'Expected AArch64'
    offset = struct.unpack_from('<Q', data, 40)[0]
    entry_size, count = struct.unpack_from('<HH', data, 58)
    assert entry_size == 64
    sections = [struct.unpack_from('<IIQQQQIIQQ', data, offset + i * entry_size)
                for i in range(count)]
    symbols = {}
    for section in sections:
        if section[1] != 2:
            continue
        names = sections[section[6]]
        strings = data[names[4]:names[4] + names[5]]
        assert section[9] == 24
        for position in range(section[4], section[4] + section[5], section[9]):
            name, _, _, _, value, size = struct.unpack_from('<IBBHQQ', data, position)
            end = strings.index(0, name)
            symbols[strings[name:end].decode('ascii')] = (value, size)
    return symbols


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    profile = 'release' if args.release else 'debug'
    # This feature must not overwrite the normal image used by the panel.
    build_dir = ROOT / 'target/exception-self-test'
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rai-os',
                    '--features', 'exception-self-test', '--target-dir', str(build_dir)]
                   + (['--release'] if args.release else []), cwd=ROOT, check=True)
    kernel = build_dir / 'aarch64-unknown-none-softfloat' / profile / 'rai-os'
    symbols = elf_symbols(kernel)
    address = lambda name: symbols[name][0]
    assert address('exception_vectors') % 2048 == 0
    assert symbols['exception_vectors'][1] == 2048
    assert address('exception_vectors_end') - address('exception_vectors') == 2048
    assert address('__exception_stack_top') - address('__exception_stack_bottom') == 16384
    assert address('__exception_stack_bottom') >= address('__stack_top')
    transcript = bytearray()
    process = subprocess.Popen(['sh', 'scripts/qemu.sh', str(kernel)], cwd=ROOT,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    try:
        deadline = time.monotonic() + 10
        while b'HALTED: exception is fatal; restart QEMU\r\n' not in transcript:
            remaining = deadline - time.monotonic()
            assert remaining > 0, f'Exception output timeout: {transcript!r}'
            for key, _ in selector.select(remaining):
                data = os.read(key.fileobj.fileno(), 4096)
                assert data, f'QEMU exited: {process.poll()}'
                transcript.extend(data)
        pattern = (
            rb'RAI-OS: boot OK \| AArch64 EL1 \| UART PL011\r\n'
            rb'SELF-TEST: deliberate BRK #0x5241\r\n'
            rb'EXCEPTION: vector=(\d+); esr=0x([0-9a-f]{16}); ec=0x([0-9a-f]{2}); '
            rb'elr=0x([0-9a-f]{16}); spsr=0x([0-9a-f]{16}); far_raw=0x([0-9a-f]{16}); '
            rb'entry_sp=0x([0-9a-f]{16}); handler_sp=0x([0-9a-f]{16})\r\n'
            rb'HALTED: exception is fatal; restart QEMU\r\n')
        match = re.fullmatch(pattern, transcript)
        assert match, f'Unexpected diagnostics: {transcript!r}'
        vector = int(match[1])
        esr, ec, elr, spsr, _far, entry_sp, handler_sp = [int(x, 16) for x in match.groups()[1:]]
        assert vector == 4, vector  # EL1, SP_EL1, synchronous slot (+0x200).
        assert esr == 0xf2005241 and ec == 0x3c, (esr, ec)
        assert elr == address('exception_test_instruction'), hex(elr)
        assert spsr & 0xf == 5 and spsr & 0x3c0 == 0x3c0, hex(spsr)
        assert address('__stack_bottom') <= entry_sp <= address('__stack_top')
        assert address('__exception_stack_bottom') <= handler_sp < address('__exception_stack_top')
        assert entry_sp % 16 == handler_sp % 16 == 0
        # BRK does not define a fault address: deliberately do not assert FAR.
        process.stdin.write(b'info\n')
        process.stdin.flush()
        assert not selector.select(0.25), 'Fatal handler returned or emitted more output'
        assert process.poll() is None, 'Expected a halted guest, not a QEMU exit'
        print(f'PASS ({profile}): real BRK, vector 4, ESR/class, exact ELF fault address, saved EL1h state, dedicated aligned stack, fatal halt.')
        print(transcript.decode('ascii'), end='')
    finally:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        selector.close()
        output = ROOT / 'target/verification'
        output.mkdir(parents=True, exist_ok=True)
        (output / f'exceptions-{profile}.log').write_bytes(transcript)


if __name__ == '__main__':
    main()
