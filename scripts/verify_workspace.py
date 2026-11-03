#!/usr/bin/env python3
"""Real guest atomic review/apply, rejection and RFB keyboard tests; no live API."""
import argparse
import json
from pathlib import Path
import socket
import struct
import sys
import threading
import time
sys.dont_write_bytecode=True
from workspace import Workspace,EXAMPLE,SEED,build,host_build
from workspace_vnc import session,exact

def verify(kernel,binary,output):
    app=Workspace(kernel,binary,output,'replay',vnc=True);g=app.guest;checks=[]
    try:
        g.capture(output/'compose.ppm')
        # An actual standard RFB connection drives keys through the same relay as a viewer.
        client,server=socket.socketpair();errors=[]
        def relay():
            try:
                with server:session(server,g.vnc_socket,app.press)
            except (EOFError,OSError):pass
            except BaseException as e:errors.append(repr(e))
        thread=threading.Thread(target=relay,daemon=True);thread.start();client.settimeout(20)
        with client:
            assert exact(client,12)==b'RFB 003.008\n';client.sendall(b'RFB 003.008\n')
            assert exact(client,2)==b'\x01\x01';client.sendall(b'\x01');assert exact(client,4)==b'\0'*4
            client.sendall(b'\x01');header=exact(client,24);assert struct.unpack('!HH',header[:4])==(480,800)
            exact(client,struct.unpack('!I',header[20:])[0])
            client.sendall(struct.pack('!BBH',2,0,1)+struct.pack('!i',0))
            client.sendall(struct.pack('!BBHHHH',3,0,0,0,480,800))
            message=exact(client,4);assert message[0]==0 and struct.unpack('!H',message[2:])[0]==1
            rectangle=exact(client,12);assert struct.unpack('!HHHHi',rectangle)==(0,0,480,800,0)
            pixels=exact(client,480*800*4);assert len(set(pixels))>20
            def key(code):client.sendall(struct.pack('!BBHI',4,1,0,code))
            for char in EXAMPLE:key(ord(char))
            key(0xff0d)
            deadline=time.monotonic()+30
            while not list(output.glob('*/proposal.ppm')):
                assert time.monotonic()<deadline,'No proposal from keyboard input';time.sleep(.05)
            for t in app.threads:t.join(timeout=2)
            with app.lock:
                before=g.fields('fs.list');assert before['generation']=='5' and len(before['files'].split(','))==4
                g.request('ui.key','0');g.capture(output/'review.ppm')
            # Apply through the same clickable button a viewer displays.
            client.sendall(struct.pack('!BBHH',5,1,240,730));client.sendall(struct.pack('!BBHH',5,0,240,730))
            deadline=time.monotonic()+10
            while True:
                with app.lock:current=g.fields('fs.list')
                if current['generation']=='6':break
                assert time.monotonic()<deadline;time.sleep(.05)
            with app.lock:
                for p,data in SEED.items():assert g.read(p)==data
                assert g.read('/work/aurora.txt')==SEED['/inbox/aurora.txt']
                assert g.read('/work/field.txt')==SEED['/inbox/field.txt']
                g.request('fs.read','/work/draft.txt',error='fs_not_found')
                g.request('ui.key','9');g.capture(output/'done.ppm')
            checks.append('RFB 3.8 framebuffer, keyboard submission, real guest preview without writes, pointer confirmation; exact originals and selected copies')
        thread.join(timeout=3);assert not thread.is_alive() and not errors,errors
        # Invalid last action, cancelled result, stale catalog and replay limitations.
        def submit(text):
            g.request('ui.open')
            for c in text:g.request('ui.key',str(ord(c)))
            row=dict(p.split('=',1) for p in g.request('ui.key','13').split('; '));return row['submit']
        rid=submit('Move notes');generation=g.fields('fs.list')['generation'];before=g.request('fs.list')
        g.request('ui.plan',f'{rid}:{generation}');g.request('ui.action',f'{rid}:move:/inbox/draft.txt:/work/draft.txt')
        g.request('ui.action',f'{rid}:copy:/inbox/weekend.txt:/work/aurora.txt')
        g.request('ui.ready',f'{rid}:Invalid second destination',error='fs_exists');assert g.request('fs.list')==before
        g.request('ui.key','27');g.request('ui.ready',f'{rid}:Late response',error='ui_stale_request')
        rid=submit('Move draft');g.request('ui.plan',f'{rid}:{generation}');g.request('ui.action',f'{rid}:move:/inbox/draft.txt:/work/draft.txt');g.request('ui.ready',f'{rid}:Move draft for review.')
        g.request('fs.move',f'{generation}:/inbox/weekend.txt:/inbox/personal.txt');before=g.request('fs.list')
        g.request('ui.key','13',error='fs_stale_generation');assert g.request('fs.list')==before
        g.request('fs.read','/work/draft.txt',error='fs_not_found');g.capture(output/'stale.ppm')
        checks.append('Invalid last action changes nothing; cancelled request rejected; changed catalog blocks whole commit')
        g.request('ui.open');g.request('ui.key',str(ord('x')));submit_reply=app.press(13)
        assert submit_reply.startswith('submit=')
        for t in app.threads:t.join(timeout=40);assert not t.is_alive()
        assert g.request('ui.key','0')=='state=notice';g.capture(output/'clarify.ppm')
        checks.append('Unknown offline instruction produces an explicit replay clarification, not invented inference')
        result=dict(source='replay',live_meta_calls=0,graphics='native_guest_framebuffer',input='host_RFB_to_guest_UART',checks=checks)
        (output/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
    finally:app.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve())
