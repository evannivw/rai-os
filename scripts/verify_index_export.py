#!/usr/bin/env python3
"""Exercise the user-facing exporter/checker and preserve existing output files."""
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from check_record_index import check
from verify_mmu import ROOT, OUTPUT


def main():
    host = subprocess.check_output(['rustc', '-vV'], text=True)
    triple = next(line[6:] for line in host.splitlines() if line.startswith('host: '))
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rai-os'], cwd=ROOT, check=True)
    subprocess.run(['cargo', 'build', '--locked', '--example', 'index_record', '--target', triple], cwd=ROOT, check=True)
    binary = ROOT / 'target' / triple / 'debug/examples/index_record'
    kernel = ROOT / 'target/aarch64-unknown-none-softfloat/debug/rai-os'
    OUTPUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=OUTPUT) as directory:
        artifact = Path(directory) / 'notes.index'
        def export(key, output=artifact, elf=kernel):
            return subprocess.run([str(binary), str(elf), key, str(output)], cwd=ROOT, capture_output=True, text=True, timeout=15)
        result = export('notes@1')
        assert result.returncode == 0, result.stderr
        original = artifact.read_bytes()
        check(original.decode('ascii').rstrip('\n'), 'notes@1')
        assert 'request_us=' in result.stderr and 'not independently verified yet' in result.stderr
        assert export('notes@2').returncode != 0
        assert artifact.read_bytes() == original, 'Exporter overwrote existing output'
        for key in ['restricted@1', 'unknown@1', 'notes@1\ninfo', '']:
            path = Path(directory) / 'invalid.index'
            assert export(key, path).returncode != 0
            assert not path.exists()
        missing = Path(directory) / 'missing.index'
        assert export('notes@1', missing, Path(directory) / 'missing.elf').returncode != 0
        assert not missing.exists()
        command = [sys.executable, 'scripts/check_record_index.py', str(artifact), '--record', 'notes@1']
        good = subprocess.run(command + ['--find', 'PUMP'], cwd=ROOT, capture_output=True, text=True)
        assert good.returncode == 0 and 'pump: 2 occurrence(s)' in good.stdout
        for data in [original.replace(b'pump:2', b'pump:1'), b'x' * 513 + b'\n', original + b'\n', b'\xff\n']:
            artifact.write_bytes(data)
            bad = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            assert bad.returncode != 0 and 'VERIFIED' not in bad.stdout, bad
    print('PASS: real-QEMU export, exact search, non-overwrite, denied/input/missing-ELF failure and corrupt/oversized/non-ASCII artifact rejection')


if __name__ == '__main__': main()
