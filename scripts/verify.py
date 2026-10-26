#!/usr/bin/env python3
"""Boot a real QEMU process, fail on timeout, retain its UART transcript."""
import os
import argparse
from pathlib import Path
import selectors
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
KERNEL = ROOT / "target/aarch64-unknown-none-softfloat/debug/rai-os"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    build = ["cargo", "build", "--locked"]
    if args.release:
        build.append("--release")
    subprocess.run(build, cwd=ROOT, check=True)
    kernel = str(KERNEL).replace("/debug/", "/release/") if args.release else str(KERNEL)
    transcript = bytearray()
    process = subprocess.Popen(
        ["sh", "scripts/qemu.sh", kernel], cwd=ROOT,
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)

    def expect(marker, start=0):
        deadline = time.monotonic() + 10
        while marker not in transcript[start:]:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise AssertionError(f"Timeout waiting for {marker!r}")
            for key, _ in selector.select(remaining):
                data = os.read(key.fileobj.fileno(), 4096)
                if not data:
                    raise AssertionError(f"QEMU exited: {process.poll()}")
                transcript.extend(data)

    try:
        expect(b"RAI-OS: boot OK | AArch64 EL1 | UART PL011\r\n")
        expect(b"Console ready: help | info | echo <text>\r\n")
        print("PASS: real AArch64 boot and UART output")
        checks = 0

        def exchange(request, expected):
            nonlocal checks
            start = len(transcript)
            process.stdin.write(request)
            process.stdin.flush()
            expect(expected, start)
            actual = bytes(transcript[start:])
            if actual != expected:
                raise AssertionError(f"Unexpected response: {actual!r}, wanted {expected!r}")
            checks += 1

        info = b"RAI-OS 0.1.0; aarch64; QEMU virt-9.2; EL1; single CPU; no process isolation; AI not connected"
        exchange(b"help\n", b"help\r\nhelp | info | echo <text>\r\n")
        exchange(b"info\r\n", b"info\r\n" + info + b"\r\n")
        exchange(b"echo hello Rust\r", b"echo hello Rust\r\nhello Rust\r\n")
        exchange(b"echo\n", b"echo\r\n\r\n")
        exchange(b"oops\n", b"oops\r\nerror: unknown_command\r\n")
        exchange(b"echx\x7fo edited\n", b"echx\x08 \x08o edited\r\nedited\r\n")
        exchange(b"@rai1\t1\thelp\n", b"@rai1\t1\tok\thelp | info | echo <text>\r\n")
        exchange(b"@rai1\t2\tinfo\r\n", b"@rai1\t2\tok\t" + info + b"\r\n")
        exchange(b"@rai1\t3\techo\tbridge ready\n", b"@rai1\t3\tok\tbridge ready\r\n")
        exchange(b"@rai1\t4\techo\t\n", b"@rai1\t4\tok\t\r\n")
        for request, expected in [
            (b"@rai2\t5\tinfo", b"5\terr\tbad_version"),
            (b"@rai1\t0\tinfo", b"0\terr\tbad_id"),
            (b"@rai1\t-1\tinfo", b"0\terr\tbad_id"),
            (b"@rai1\t4294967296\tinfo", b"0\terr\tbad_id"),
            (b"@rai1\t\tinfo", b"0\terr\tbad_id"),
            (b"@rai1\t6", b"6\terr\tbad_frame"),
            (b"@rai1\t7\techo", b"7\terr\tbad_args"),
            (b"@rai1\t8\tinfo\textra", b"8\terr\tbad_args"),
            (b"@rai1\t9\techo\ta\tb", b"9\terr\tbad_args"),
            (b"@rai1\t10\texec", b"10\terr\tunknown_operation"),
            (b"@rai1\t11\techo\t\xff", b"0\terr\tinvalid_byte"),
            (b"@rai1\t12\techo\t\x00info", b"0\terr\tinvalid_byte"),
            (b"@rai1\t13\techo\t\x1b[31m", b"0\terr\tinvalid_byte"),
        ]:
            exchange(request + b"\n", b"@rai1\t" + expected + b"\r\n")
        prefix = b"@rai1\t14\techo\t"
        payload = b"x" * (256 - len(prefix))
        exchange(prefix + payload + b"\n", b"@rai1\t14\tok\t" + payload + b"\r\n")
        exchange(prefix + payload + b"xinfo\n", b"@rai1\t0\terr\tline_too_long\r\n")
        exchange(b"@rai1\t15\techo\trecovered\n", b"@rai1\t15\tok\trecovered\r\n")
        exchange(b"@rai1\t16\techo\ta\r\n@rai1\t17\techo\tb\n",
                 b"@rai1\t16\tok\ta\r\n@rai1\t17\tok\tb\r\n")
        # A frame may arrive in several host writes; only newline dispatches it.
        start = len(transcript)
        process.stdin.write(b"@rai1\t18\techo\tfrag")
        process.stdin.flush()
        if selector.select(0.1):
            raise AssertionError("Kernel responded before the frame terminator")
        process.stdin.write(b"mented\n")
        process.stdin.flush()
        expect(b"@rai1\t18\tok\tfragmented\r\n", start)
        checks += 1
        print(f"PASS: {checks} console/protocol checks, including invalid input and recovery")
    finally:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        selector.close()
        output = ROOT / "target/verification"
        output.mkdir(parents=True, exist_ok=True)
        (output / ("uart-release.log" if args.release else "uart.log")).write_bytes(transcript)
        print(transcript.decode("ascii", errors="replace"))


if __name__ == "__main__":
    main()
