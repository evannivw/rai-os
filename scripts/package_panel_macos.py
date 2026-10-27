#!/usr/bin/env python3
"""Package an already-built host panel as a local macOS application."""
import argparse
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    if sys.platform != 'darwin':
        parser.error('This optional application bundle is for macOS only.')
    compiler = subprocess.check_output(['rustc', '-vV'], text=True)
    host = next(line[6:] for line in compiler.splitlines() if line.startswith('host: '))
    profile = 'release' if args.release else 'debug'
    binary = ROOT / 'target/host-panel' / host / profile / 'rai-panel'
    bundle = ROOT / 'target/RAI-OS Device Lab.app'
    executables = bundle / 'Contents/MacOS'
    executables.mkdir(parents=True, exist_ok=True)
    # Replace atomically so macOS does not reuse a cached signature for an
    # executable whose bytes were overwritten in place.
    staged = executables / 'rai-panel.new'
    shutil.copy2(binary, staged)
    staged.replace(executables / 'rai-panel')
    with (bundle / 'Contents/Info.plist').open('wb') as file:
        plistlib.dump(dict(CFBundleName='RAI-OS Device Lab', CFBundleDisplayName='RAI-OS Device Lab',
            CFBundleIdentifier='dev.raios.device-lab', CFBundleExecutable='rai-panel',
            CFBundlePackageType='APPL', CFBundleVersion='1', CFBundleShortVersionString='0.1.0',
            NSHighResolutionCapable=True), file)
    subprocess.run(['codesign', '--force', '--sign', '-', str(bundle)], check=True)
    print(bundle)
    print('Keep this local bundle inside the repository target directory; it resolves the repository from that layout.')


if __name__ == '__main__':
    main()
