#!/usr/bin/env python3
"""Check the host UI against real QEMU, plus input rejection and child failure."""
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def main():
    result = subprocess.run(
        ["sh", "scripts/console.sh"], cwd=ROOT,
        input="info\necho puente vivo\necho no\tpermitido\ninfo\nexit\n",
        text=True, capture_output=True, timeout=60,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert "Conexion: arranque y consola UART confirmados." in result.stdout
    assert "[kernel / OK] puente vivo" in result.stdout
    assert "[entrada rechazada; no enviada]" in result.stdout
    assert result.stdout.count("[kernel / OK] RAI-OS 0.1.0;") == 3
    assert "Sesion cerrada; QEMU detenido." in result.stdout
    host = subprocess.check_output(["rustc", "-vV"], text=True)
    triple = next(line[6:] for line in host.splitlines() if line.startswith("host: "))
    binary = ROOT / "target" / triple / "debug/examples/control_console"
    failure = subprocess.run(
        [str(binary), "/nonexistent/rai-os-test-kernel"], cwd=ROOT,
        input="", text=True, capture_output=True, timeout=15,
    )
    assert failure.returncode != 0
    assert "Conexion cerrada:" in failure.stderr
    assert "Conexion: arranque y consola UART confirmados." not in failure.stdout
    eof = subprocess.run(
        [str(binary), "target/aarch64-unknown-none-softfloat/debug/rai-os"],
        cwd=ROOT, input="", text=True, capture_output=True, timeout=15,
    )
    assert eof.returncode == 0, eof.stdout + eof.stderr
    assert "Sesion cerrada; QEMU detenido." in eof.stdout
    output = ROOT / "target/verification/host-console.log"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(result.stdout)
    print(result.stdout)
    print("PASS: real QEMU, info/echo, rejected input, recovery, missing ELF and EOF")

if __name__ == "__main__":
    main()
