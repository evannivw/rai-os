#!/usr/bin/env python3
"""Check the host UI against real QEMU, plus input rejection and child failure."""
from pathlib import Path
import subprocess
import re

ROOT = Path(__file__).resolve().parents[1]

def main():
    result = subprocess.run(
        ["sh", "scripts/console.sh"], cwd=ROOT,
        input="ipc.wait\nipc.check\nschedule.isolate\nschedule.run\nschedule.fault\nschedule.check\nuser.spin\ntimer.check\nuser.run\ncontext.check\ninfo\nuptime\nmmu.info\nmemory.info\nmemory.check\ntask.start\ntask.status\ntask.cancel 1\necho bridge alive\necho no\tallowed\ninfo\nexit\n",
        text=True, capture_output=True, timeout=60,
    )
    assert "rounds=2; reads_denied=4; writes_denied=4; own_accesses=12; memory_unchanged=true; base_restored=true" in result.stdout
    assert "a_value=42;" in result.stdout and "b_value=41;" in result.stdout
    assert "ipc_capacity=2; max_calls=16; sent=2; received=2; rejected=0;" in result.stdout
    assert "blocked=1; woken=1; closed_wakes=0; deadlocked=0" in result.stdout
    assert result.returncode == 0, result.stdout + result.stderr
    assert "Connection: kernel boot and UART console confirmed." in result.stdout
    assert re.search(r"\[kernel / OK\] uptime_ms=\d+; counter_hz=[1-9]\d*", result.stdout)
    assert result.stdout.count("[kernel / OK] task_id=1; state=completed; processed=0; total=0; words=0") == 2
    assert "[kernel / REJECTED] task_not_running" in result.stdout
    assert re.search(r"\[kernel / OK\] irq=30; ticks=6; rounds=2; registers=31; flags=true; stack=true; irq_masked=true; timer_disabled=true; elapsed_ms=\d+", result.stdout)
    assert re.search(r"a_state=exited; a_slices=\d+; a_code=0; a_value=14091860730339196423", result.stdout)
    assert re.search(r"a_state=faulted; a_slices=\d+; a_reason=exception", result.stdout)
    assert result.stdout.count("b_value=14607574309510313995") == 2
    assert "[kernel / OK] tasks=2; slice_ms=20; irqs=12; switches=11; trace=ABABABABABAB;" in result.stdout
    assert re.search(r"\[kernel / OK\] mode=EL0; state=timed_out; budget_ms=100; elapsed_ms=\d+; vector=9;", result.stdout)
    assert "[kernel / OK] mode=EL0; state=exited; code=0; syscalls=2; rejected=0; output=Hello from EL0" in result.stdout
    assert "[kernel / OK] round_trips=2; registers=31; flags=true; stack=true; resumed=true" in result.stdout
    assert "[kernel / OK] enabled=true; mode=identity; granule=4096; table_pages=12;" in result.stdout
    assert "[kernel / OK] page_size=4096; total_pages=256; free_pages=244; used_pages=12;" in result.stdout
    assert "[kernel / OK] checked_pages=244; exhausted=true; zeroed=true; reused=true; free_pages=244" in result.stdout
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
    print("PASS: real QEMU, info/uptime/echo/tasks/memory, rejected input, recovery, missing ELF and EOF")

if __name__ == "__main__":
    main()
