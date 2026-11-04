#!/usr/bin/env python3
"""Import real bytes into RAI-OS RAM, then copy/move/read them through guest commands."""
# Protocol replies and exact restore checks currently rely on assertions. Refuse
# optimized mode before any guest, provider request or saved-state operation.
if not __debug__:
    raise SystemExit(
        'RAI-OS host tools require Python assertions. Run without -O, -OO or PYTHONOPTIMIZE.'
    )

import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import tempfile
import threading
import sys
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]


def build(debug=False):
    subprocess.run(['cargo','build','--locked','--bin','rai-os','--features','guest-display','--target-dir','target/display',*([] if debug else ['--release'])],cwd=ROOT,check=True)
    return ROOT/f'target/display/aarch64-unknown-none-softfloat/{"debug" if debug else "release"}/rai-os'


class Guest:
    def __init__(self,kernel,graphics=False,vnc=False,vnc_password=None,expect_display=True):
        self.temp=tempfile.TemporaryDirectory(prefix='rai-files-');self.socket=Path(self.temp.name)/'qmp'
        self.log=[];self.lines=queue.Queue();self.serial=0
        env=dict(os.environ);env.pop('MODEL_API_KEY',None);env.pop('RAI_VNC_PASSWORD',None)
        self.vnc_socket=Path(self.temp.name)/'vnc'
        extra=['-vnc',f'unix:{self.vnc_socket}'+(',password=on' if vnc_password else '')] if vnc else []
        self.process=subprocess.Popen(['sh','scripts/qemu.sh',str(kernel),'--graphics' if graphics else '--capture','-qmp',f'unix:{self.socket},server=on,wait=off',*extra],cwd=ROOT,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
        def read():
            for line in self.process.stdout:self.lines.put(line.decode('utf-8',errors='replace').rstrip('\r\n'))
            self.lines.put(None)
        self.reader=threading.Thread(target=read,daemon=True);self.reader.start()
        try:
            assert self.line()=='RAI-OS: boot OK | AArch64 EL1 | UART PL011'
            if expect_display:assert self.line()=='Display ready: 480x800 guest framebuffer'
            assert self.line()=='Console ready: type help for commands'
            if vnc_password:self.qmp(dict(execute='set_password',arguments=dict(protocol='vnc',password=vnc_password)))
        except BaseException:self.close();raise
    def line(self):
        value=self.lines.get(timeout=20)
        assert value is not None,'Guest exited'
        self.log.append(value);return value
    def request(self,name,arg=None,error=None):
        self.serial+=1;frame=f'@rai1\t{self.serial}\t{name}'+('' if arg is None else '\t'+arg)+'\n'
        assert len(frame.encode())<=257
        self.process.stdin.write(frame.encode());self.process.stdin.flush()
        fields=self.line().split('\t',3);assert fields[:3]==['@rai1',str(self.serial),'err' if error else 'ok'],fields
        if error:assert fields[3]==error,fields
        return fields[3]
    def fields(self,name,arg=None):return dict(p.split('=',1) for p in self.request(name,arg).split('; '))
    def read(self,path):
        row=self.fields('fs.read',path);data=bytes.fromhex(row['data']);assert len(data)==int(row['bytes']) and row['path']==path;return data
    def upload(self,path,data):
        assert len(data)<=512
        generation=self.fields('fs.list')['generation']
        ticket=self.fields('fs.begin',f'{generation}:{len(data)}:{path}')['ticket']
        for offset in range(0,len(data),64):self.request('fs.append',f'{ticket}:{offset}:{data[offset:offset+64].hex()}')
        self.request('fs.commit',ticket);assert self.read(path)==data
    def qmp(self,command):
        with socket.socket(socket.AF_UNIX) as connection:
            connection.settimeout(10);connection.connect(str(self.socket));stream=connection.makefile('rwb',buffering=0)
            assert 'QMP' in json.loads(stream.readline())
            for command in [dict(execute='qmp_capabilities'),command]:
                stream.write((json.dumps(command)+'\n').encode())
                while True:
                    response=json.loads(stream.readline());assert 'error' not in response,response
                    if 'return' in response:break
    def capture(self,path):
        self.qmp(dict(execute='screendump',arguments=dict(filename=str(path.resolve()))))
        data=path.read_bytes();magic,size,maximum,pixels=data.split(b'\n',3)
        assert (magic,size,maximum)==(b'P6',b'480 800',b'255') and len(pixels)==480*800*3
        from verify_display import png
        png(path.with_suffix('.png'),pixels)
    def close(self):
        if self.process.poll() is None:
            self.process.terminate()
            try:self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:self.process.kill();self.process.wait()
        self.reader.join(timeout=2);self.process.stdin.close();self.process.stdout.close();self.temp.cleanup()


def run(kernel,output,verify=False,graphics=False,hold=False,import_file=None):
    brief=b'Aurora workspace\nApproved handoff: collect source notes, preserve originals, and show pending work.\n'
    if import_file:
        with import_file.open('rb') as source:brief=source.read(513)
    assert len(brief)<=512,'This first guest store accepts at most 512 bytes per file'
    output.mkdir(parents=True,exist_ok=False);guest=Guest(kernel,graphics);checks=[]
    payload=bytes(range(256))*2
    try:
        assert guest.fields('fs.list')['files']=='-'
        guest.upload('/inbox/brief.txt',brief);guest.upload('/inbox/sample.bin',payload)
        generation=guest.fields('fs.list')['generation'];guest.request('fs.copy',f'{generation}:/inbox/brief.txt:/work/brief.txt')
        generation=guest.fields('fs.list')['generation'];guest.request('fs.move',f'{generation}:/work/brief.txt:/archive/brief.txt')
        assert guest.read('/inbox/brief.txt')==guest.read('/archive/brief.txt')==brief
        guest.request('fs.read','/work/brief.txt',error='fs_not_found')
        guest.request('fs.list');guest.capture(output/'files.ppm')
        checks.append('512-byte binary roundtrip; source preserved; copy and move in guest')
        if verify:
            before=guest.request('fs.list');generation=guest.fields('fs.list')['generation']
            for name,arg,error in [('fs.move',f'{int(generation)-1}:/inbox/brief.txt:/work/brief.txt','fs_stale_generation'),('fs.copy',f'{generation}:/inbox/brief.txt:/archive/brief.txt','fs_exists'),('fs.begin',f'{generation}:1:/../escape','fs_bad_path'),('fs.begin',f'{generation}:513:/large','fs_file_too_large')]:
                guest.request(name,arg,error=error);assert guest.request('fs.list')==before
            ticket=guest.fields('fs.begin',f'{generation}:2:/inbox/partial')['ticket']
            guest.request('fs.commit',ticket,error='fs_incomplete');guest.request('fs.read','/inbox/partial',error='fs_not_found')
            guest.request('fs.append',f'{ticket}:0:41zz',error='fs_bad_hex')
            guest.request('fs.append',f'{ticket}:1:41',error='fs_bad_offset')
            guest.request('fs.append',f'{int(ticket)+1}:0:41',error='fs_stale_ticket')
            guest.request('fs.append',f'{ticket}:0:414243',error='fs_upload_overflow')
            guest.request('fs.append',f'{ticket}:0:4142')
            guest.request('fs.move',f'{generation}:/archive/brief.txt:/work/brief.txt')
            guest.request('fs.commit',ticket,error='fs_stale_generation');guest.request('fs.abort',ticket)
            guest.request('fs.read','/inbox/partial',error='fs_not_found')
            assert guest.read('/work/brief.txt')==brief and guest.read('/inbox/sample.bin')==payload
            for i in range(5):guest.upload(f'/empty{i}',b'')
            before=guest.request('fs.list');generation=guest.fields('fs.list')['generation']
            guest.request('fs.begin',f'{generation}:0:/extra',error='fs_full')
            guest.request('fs.copy',f'{generation}:/inbox/brief.txt:/extra',error='fs_full');assert guest.request('fs.list')==before
            checks.append('rejected stale/collision/traversal/oversize/partial/malformed/offset/ticket/overflow/capacity operations; failed uploads remain invisible')
        if hold:input('Guest files are visible. Press Enter to close QEMU.\n')
    finally:
        (output/'uart.log').write_text('\n'.join(guest.log)+'\n');guest.close()
    if verify:
        again=Guest(kernel)
        try:assert again.fields('fs.list')['files']=='-';checks.append('fresh boot has no files: RAM-only lifetime confirmed')
        finally:(output/'reboot.log').write_text('\n'.join(again.log)+'\n');again.close()
    report=dict(execution='guest_EL1',storage='volatile_ram',ai_planning=False,checks=checks,source_sha256=hashlib.sha256(brief).hexdigest(),binary_sha256=hashlib.sha256(payload).hexdigest(),capacity=8,max_file_bytes=512,commands=guest.serial)
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--kernel',type=Path);p.add_argument('--output',type=Path,required=True);p.add_argument('--debug',action='store_true');p.add_argument('--verify',action='store_true');p.add_argument('--graphics',action='store_true');p.add_argument('--hold',action='store_true');p.add_argument('--import-file',type=Path);a=p.parse_args()
    print(json.dumps(run(a.kernel or build(a.debug),a.output.resolve(),a.verify,a.graphics,a.hold,a.import_file),indent=2))
