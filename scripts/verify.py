#!/usr/bin/env python3
"""Boot a real QEMU process, fail on timeout, retain its UART transcript."""
import os
import re
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
        expect(b"Console ready: type help for commands\r\n")
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
        exchange(b"help\n", b"help\r\nhelp | info | uptime | mmu.info | context.check | timer.check | schedule.check | schedule.isolate | ipc.check | ipc.wait | record.count <key> | record.index <key> | worker.start | worker.count <key> | worker.status | worker.stop | coord.start [index] | coord.result | coord.observe <key> | coord.propose <ticket>:<key> | coord.status | coord.stop | schedule.run | schedule.fault | user.run | user.spin | memory.info | memory.check | echo <text> | task.start <text> | task.status | task.cancel <id>\r\n")
        exchange(b"info\r\n", b"info\r\n" + info + b"\r\n")
        exchange(b"echo hello Rust\r", b"echo hello Rust\r\nhello Rust\r\n")
        exchange(b"echo\n", b"echo\r\n\r\n")
        exchange(b"oops\n", b"oops\r\nerror: unknown_command\r\n")
        exchange(b"echx\x7fo edited\n", b"echx\x08 \x08o edited\r\nedited\r\n")
        exchange(b"@rai1\t1\thelp\n", b"@rai1\t1\tok\thelp | info | uptime | mmu.info | context.check | timer.check | schedule.check | schedule.isolate | ipc.check | ipc.wait | record.count <key> | record.index <key> | worker.start | worker.count <key> | worker.status | worker.stop | coord.start [index] | coord.result | coord.observe <key> | coord.propose <ticket>:<key> | coord.status | coord.stop | schedule.run | schedule.fault | user.run | user.spin | memory.info | memory.check | echo <text> | task.start <text> | task.status | task.cancel <id>\r\n")
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
        exchange(b"@rai1\t19\tuptime\textra\n", b"@rai1\t19\terr\tbad_args\r\n")
        exchange(b"uptime extra\n", b"uptime extra\r\nerror: unknown_command\r\n")

        def read_uptime(request, pattern):
            nonlocal checks
            start = len(transcript)
            process.stdin.write(request)
            process.stdin.flush()
            # Wait for the complete response, not just the shell input echo.
            expect(b"counter_hz=", start)
            payload_start = transcript.index(b"counter_hz=", start)
            expect(b"\r\n", payload_start)
            actual = bytes(transcript[start:])
            match = re.fullmatch(pattern, actual)
            if match is None:
                raise AssertionError(f"Invalid uptime response: {actual!r}")
            milliseconds, frequency = map(int, match.groups())
            assert 0 < frequency <= 0xffffffff, frequency
            checks += 1
            return milliseconds, frequency

        before, frequency = read_uptime(
            b"@rai1\t20\tuptime\n",
            rb"@rai1\t20\tok\tuptime_ms=(\d+); counter_hz=(\d+)\r\n")
        # QEMU keeps running while UART is waiting for input. A broad tolerance
        # catches unit/frequency errors without claiming benchmark precision.
        time.sleep(0.25)
        after, next_frequency = read_uptime(
            b"uptime\n",
            rb"uptime\r\nuptime_ms=(\d+); counter_hz=(\d+)\r\n")
        assert next_frequency == frequency, (frequency, next_frequency)
        assert 100 <= after - before <= 10000, (before, after)
        for request_id in range(21, 26):
            value, next_frequency = read_uptime(
                f"@rai1\t{request_id}\tuptime\n".encode(),
                f"@rai1\t{request_id}\tok\tuptime_ms=".encode()
                + rb"(\d+); counter_hz=(\d+)\r\n")
            assert value >= after and next_frequency == frequency
            after = value
        print(f"PASS: guest counter advances; counter_hz={frequency}")
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
