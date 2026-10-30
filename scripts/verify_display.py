#!/usr/bin/env python3
"""Real guest pixels, observed index transitions and display memory protection."""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import selectors
import socket
import struct
import subprocess
import sys
import tempfile
import time
import zlib
sys.dont_write_bytecode = True
from verify_mmu import ROOT, OUTPUT
from verify_exceptions import elf_symbols
from check_record_index import check

FAULTS = ['display-read', 'display-write', 'display-execute', 'fw-cfg']


def png(path, pixels):
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    rows = b''.join(b'\0' + pixels[y*1440:(y+1)*1440] for y in range(800))
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>2I5B', 480,800,8,2,0,0,0)) + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND',b''))


def build(profile, fault=None):
    directory = ROOT / ('target/display' if fault is None else f'target/display-fault/{fault}')
    features = 'guest-display' if fault is None else f'user-test-{fault}'
    command = ['cargo','build','--locked','--bin','rai-os','--features',features,'--target-dir',str(directory)]
    if profile == 'release': command.append('--release')
    subprocess.run(command, cwd=ROOT, check=True)
    kernel = directory / f'aarch64-unknown-none-softfloat/{profile}/rai-os'
    symbols = elf_symbols(kernel)
    assert symbols['__display_start'][0] == 0x41000000
    assert symbols['__display_end'][0] == 0x41200000
    assert symbols['__page_pool_end'][0] <= 0x40200000
    return kernel


@contextmanager
def guest(kernel, name, mode='--capture', extra=(), banner='Display ready: 480x800 guest framebuffer'):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='rai-display-') as directory:
        sockpath = Path(directory) / 'qmp'
        command = ['sh','scripts/qemu.sh',str(kernel)]
        if mode: command += [mode]
        command += ['-qmp',f'unix:{sockpath},server=on,wait=off',*extra]
        child = subprocess.Popen(command,cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
        poll = selectors.DefaultSelector(); poll.register(child.stdout,selectors.EVENT_READ)
        pending = bytearray(); transcript = bytearray(); connection = None
        def line():
            deadline = time.monotonic() + 15
            while b'\n' not in pending:
                assert time.monotonic() < deadline, transcript.decode(errors='replace')
                for event,_ in poll.select(.1):
                    data = os.read(event.fd,4096)
                    assert data, transcript.decode(errors='replace')
                    pending.extend(data);transcript.extend(data)
            end = pending.index(b'\n')+1; result = bytes(pending[:end]);del pending[:end]
            return result.decode('ascii').rstrip('\r\n')
        sequence = 0
        def request(op, status='ok'):
            nonlocal sequence
            sequence += 1
            child.stdin.write(f'@rai1\t{sequence}\t{op}\n'.encode());child.stdin.flush()
            fields = line().split('\t',3)
            assert fields[:3] == ['@rai1',str(sequence),status], fields
            return fields[3]
        try:
            boot = line()
            if extra and 'warning: externally provided fw_cfg item names' in boot:
                boot = line()  # Expected QEMU diagnostic for deliberately malformed firmware input.
            assert boot == 'RAI-OS: boot OK | AArch64 EL1 | UART PL011', boot
            assert line() == banner
            assert line() == 'Console ready: type help for commands'
            connection = socket.socket(socket.AF_UNIX);connection.settimeout(10);connection.connect(str(sockpath))
            stream = connection.makefile('rwb',buffering=0)
            assert 'QMP' in json.loads(stream.readline())
            def qmp(command, arguments=None):
                stream.write((json.dumps({'execute':command,'arguments':arguments or {}})+'\n').encode())
                while True:
                    result = json.loads(stream.readline())
                    assert 'error' not in result, result
                    if 'return' in result: return result['return']
            qmp('qmp_capabilities')
            def capture(label):
                path = OUTPUT / f'display-{name}-{label}.ppm'
                qmp('screendump', {'filename':str(path)})
                data = path.read_bytes()
                magic, dimensions, maximum, pixels = data.split(b'\n',3)
                assert (magic,dimensions,maximum) == (b'P6',b'480 800',b'255')
                assert len(pixels) == 480*800*3
                assert pixels[:3] == bytes.fromhex('f6f7f2')
                assert pixels[(390*480+244)*3:(390*480+244)*3+3] == bytes.fromhex('203b34')
                png(path.with_suffix('.png'),pixels)
                return pixels
            yield request, capture
        finally:
            if connection: connection.close()
            child.terminate()
            try: child.wait(timeout=3)
            except subprocess.TimeoutExpired: child.kill();child.wait()
            poll.close()
            (OUTPUT / f'display-{name}.log').write_bytes(transcript)


def normal(profile, cocoa=False):
    kernel = build(profile)
    with guest(kernel,profile,mode='--graphics' if cocoa else '--capture') as (request,capture):
        boot = capture('boot')
        before = request('mmu.info'); assert 'table_pages=12;' in before
        frames = {}
        for key in ['notes@1','notes@2','brief@1','checklist@1','appendix@1','empty@1']:
            check(request(f'record.index\t{key}'),key)
            frame = capture(key.replace('@','-'))
            assert frame[:538*1440] == boot[:538*1440], 'Unrelated identity area changed'
            assert frame[538*1440:] != boot[538*1440:], 'Result did not appear'
            frames[key] = frame
        assert frames['notes@1'] != frames['notes@2']
        check(request('record.index\tnotes@1'),'notes@1')
        assert capture('repeat') == frames['notes@1'], 'Same observed result rendered differently'
        request('coord.start');request('coord.observe\tnotes@1');request('coord.propose\t1:notes@1')
        assert request('record.index\tnotes@1','err') == 'user_memory_busy'
        busy = capture('busy');assert busy != frames['notes@1']
        request('coord.stop')
        for op, error in [('record.index\trestricted@1','record_denied'), ('record.index\tnotes@1\textra','bad_args')]:
            assert request(op,'err') == error
            frame = capture(error);assert frame != busy and frame != frames['notes@1']
        check(request('record.index\tnotes@2'),'notes@2')
        assert capture('recovered') == frames['notes@2']
        assert 'state=exited; code=0;' in request('user.run')
        assert request('timer.check').startswith('irq=30; ticks=6;')
        assert request('mmu.info') == before
        assert 'free_pages=244' in request('memory.check')
        assert request('echo\tscreen survives') == 'screen survives'
        assert capture('unchanged') == frames['notes@2'], 'Unrelated command changed the screen'
    print(f'PASS {profile}: actual 480x800 pixels, six checked indexes, revision/error/recovery views, coordinator coexistence and stable idle frame')
    if cocoa: return
    for name,banner,data in [('missing','Display unavailable: ramfb not configured',None),
                             ('bad-config','display_bad_config',b'bad'),
                             ('dma-error','display_dma_error',b'\0'*28)]:
        with tempfile.TemporaryDirectory(prefix='rai-fwcfg-') as directory:
            extra=[]
            if data is not None:
                path=Path(directory)/'config';path.write_bytes(data)
                extra=['-fw_cfg',f'name=etc/ramfb,file={path}']
            with guest(kernel,f'{profile}-{name}',mode='',extra=extra,banner=banner) as (request,_):
                check(request('record.index\tnotes@1'),'notes@1')
                assert request('echo\tfallback') == 'fallback'
        print(f'PASS {profile}: {name} display fallback retains working console')


def fault(profile, case):
    kernel = build(profile,case)
    with guest(kernel,f'{profile}-{case}') as (request,capture):
        before = capture('boot')
        for _ in range(2):
            payload = request('user.run')
            fields = dict(part.split('=',1) for part in payload.split('; '))
            expected = 0x8200000e if case=='display-execute' else (0x9200004e if case=='display-write' else 0x9200000e)
            # Display RAM is a level-2 block; fw_cfg remains a level-3 page.
            if case=='fw-cfg': expected = 0x9200000f
            assert fields['state']=='faulted' and int(fields['esr'],16)==expected, payload
            assert int(fields['far_raw'],16)==(0x09020000 if case=='fw-cfg' else 0x41000000),payload
            assert capture('protected')==before, 'Unprivileged attempt modified display'
            assert request('echo\tcontained')=='contained'
        check(request('record.index\tnotes@1'),'notes@1')
        assert capture('recovery') != before
    print(f'PASS {profile}: hardware contained {case}, pixels intact, index execution recovered')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--release',action='store_true');parser.add_argument('--cocoa',action='store_true');args=parser.parse_args()
    profile='release' if args.release else 'debug'
    if not args.cocoa:
        OUTPUT.mkdir(parents=True,exist_ok=True)
        unit=OUTPUT/'display-table-tests'
        subprocess.run(['rustc','--edition=2024','--test','--cfg','feature="guest-display"','src/mmu/tables.rs','-o',str(unit)],cwd=ROOT,check=True)
        subprocess.run([str(unit)],check=True)
        summary_unit=OUTPUT/'display-summary-tests'
        subprocess.run(['rustc','--edition=2024','--test','src/display/summary.rs','-o',str(summary_unit)],cwd=ROOT,check=True)
        subprocess.run([str(summary_unit)],check=True)
    normal(profile,args.cocoa)
    if not args.cocoa:
        for case in FAULTS: fault(profile,case)


if __name__=='__main__': main()
