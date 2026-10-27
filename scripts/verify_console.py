#!/usr/bin/env python3
"""Check the host UI against real QEMU, plus input rejection and child failure."""
from pathlib import Path
import subprocess
import re

ROOT = Path(__file__).resolve().parents[1]

def main():
    result = subprocess.run(
        ["sh", "scripts/console.sh"], cwd=ROOT,
        input="info\nuptime\ntask.start\ntask.status\ntask.cancel 1\necho bridge alive\necho no\tallowed\ninfo\nexit\n",
        text=True, capture_output=True, timeout=60,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert "Connection: kernel boot and UART console confirmed." in result.stdout
    assert re.search(r"\[kernel / OK\] uptime_ms=\d+; counter_hz=[1-9]\d*", result.stdout)
    assert result.stdout.count("[kernel / OK] task_id=1; state=completed; processed=0; total=0; words=0") == 2
    assert "[kernel / REJECTED] task_not_running" in result.stdout
    assert "[kernel / OK] bridge alive" in result.stdout
    assert "[input rejected; not sent]" in result.stdout
    assert result.stdout.count("[kernel / OK] RAI-OS 0.1.0;") == 3
    assert "Session closed; QEMU stopped." in result.stdout
    host = subprocess.check_output(["rustc", "-vV"], text=True)
    triple = next(line[6:] for line in host.splitlines() if line.startswith("host: "))
    binary = ROOT / "target" / triple / "debug/examples/control_console"
    failure = subprocess.run(
        [str(binary), "/nonexistent/rai-os-test-kernel"], cwd=ROOT,
        input="", text=True, capture_output=True, timeout=15,
    )
    assert failure.returncode != 0
    assert "Connection closed:" in failure.stderr
    assert "Connection: kernel boot and UART console confirmed." not in failure.stdout
    eof = subprocess.run(
        [str(binary), "target/aarch64-unknown-none-softfloat/debug/rai-os"],
        cwd=ROOT, input="", text=True, capture_output=True, timeout=15,
    )
    assert eof.returncode == 0, eof.stdout + eof.stderr
    assert "Session closed; QEMU stopped." in eof.stdout
    output = ROOT / "target/verification/host-console.log"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(result.stdout)
    print(result.stdout)
    print("PASS: real QEMU, info/uptime/echo/tasks, rejected input, recovery, missing ELF and EOF")

if __name__ == "__main__":
    main()
