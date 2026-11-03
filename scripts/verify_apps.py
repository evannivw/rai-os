#!/usr/bin/env python3
"""Real QEMU lifecycle, model-boundary rejection and RFB input. No live inference."""
import argparse
import copy
import json
from pathlib import Path
import socket
import struct
import sys
import threading
import time
sys.dont_write_bytecode=True
from apps import Studio,CREATE,EVOLVE,SPEC,deliver,schema,records,build,host_build
from workspace_vnc import session,exact

def wait(app):
    for t in app.threads:t.join(timeout=45);assert not t.is_alive(),'Planner did not finish'
    assert app.guest.request('app.key','0')=='state=review'

def text(g,value):
    for c in value:g.request('app.key',str(ord(c)))

def submit(g,prompt,target=None):
    g.request('app.open',target);g.request('app.key','1002');text(g,prompt)
    r=dict(p.split('=',1) for p in g.request('app.key','13').split('; '))
    return dict(request_id=int(r['submit']),base_revision=int(r['revision']),decision='install',reason='Verifier proposal.',spec=copy.deepcopy(SPEC))

def stage(g,p):
    s=p['spec'];rid=p['request_id']
    g.request('app.begin',f"{rid}:{p['base_revision']}:{s['id']}:{s['title'].encode().hex()}:{s['capability']}")
    for f in s['fields']:g.request('app.field',f"{rid}:{f['key']}:{f['kind']}:{int(f['required'])}:{f['label'].encode().hex()}:{f['default'].encode().hex()}")
    g.request('app.filter',f"{rid}:{s['filter_key']}:{s['filter_label'].encode().hex()}")

def verify(kernel,binary,output):
    app=Studio(kernel,binary,output,'replay',vnc=True);g=app.guest;checks=[]
    try:
        g.capture(output/'home.ppm')
        client,server=socket.socketpair();errors=[]
        def relay():
            try:
                with server:session(server,g.vnc_socket,app.press)
            except (EOFError,OSError):pass
            except BaseException as e:errors.append(repr(e))
        worker=threading.Thread(target=relay,daemon=True);worker.start();client.settimeout(20)
        with client:
            assert exact(client,12)==b'RFB 003.008\n';client.sendall(b'RFB 003.008\n');assert exact(client,2)==b'\x01\x01';client.sendall(b'\x01');assert exact(client,4)==b'\0'*4
            client.sendall(b'\x01');h=exact(client,24);assert struct.unpack('!HH',h[:4])==(480,800);exact(client,struct.unpack('!I',h[20:])[0])
            client.sendall(struct.pack('!BBH',2,0,1)+struct.pack('!i',0));client.sendall(struct.pack('!BBHHHH',3,0,0,0,480,800));h=exact(client,4);assert h[0]==0 and struct.unpack('!H',h[2:])[0]==1
            assert struct.unpack('!HHHHi',exact(client,12))==(0,0,480,800,0);pixels=exact(client,480*800*4);assert len(set(pixels))>20
            def key(code):client.sendall(struct.pack('!BBHI',4,1,0,code))
            for c in CREATE:key(ord(c))
            key(0xff0d);deadline=time.monotonic()+40
            while not list(output.glob('*/proposal.ppm')):
                assert time.monotonic()<deadline,'No RFB proposal';time.sleep(.05)
            with app.lock:
                assert g.fields('app.catalog')['apps']=='-';g.capture(output/'review.ppm')
            # Actual clickable primary action in the viewer.
            client.sendall(struct.pack('!BBHH',5,1,240,730));client.sendall(struct.pack('!BBHH',5,0,240,730));deadline=time.monotonic()+10
            while True:
                with app.lock:catalog=g.fields('app.catalog')
                if catalog['apps']!='-':break
                assert time.monotonic()<deadline;time.sleep(.05)
        worker.join(timeout=3);assert not worker.is_alive() and not errors,errors
        checks.append('Actual RFB pixels, typed prompt and pointer approval; no app installed before confirmation')
        assert schema(g,'inspection')==SPEC
        g.request('app.key','13',error='app_invalid_value');assert records(g,'inspection')==[]
        text(g,'Pump A');g.request('app.key','9');text(g,'Check seal');g.capture(output/'form.ppm');g.request('app.key','13')
        assert records(g,'inspection')==[['Pump A','Check seal','false']]
        g.request('app.key','110');text(g,'Pump B');g.request('app.key','9');text(g,'Ready');g.request('app.key','9');g.request('app.key','32');g.request('app.key','13');g.capture(output/'records.ppm')
        before=records(g,'inspection');assert before==[['Pump A','Check seal','false'],['Pump B','Ready','true']]
        g.request('app.key','1004');g.capture(output/'pending.ppm');assert records(g,'inspection')==before
        app.press(1002)
        for c in EVOLVE:app.press(ord(c))
        app.press(13);wait(app);assert records(g,'inspection')==before;g.capture(output/'update-review.ppm');app.press(13)
        assert records(g,'inspection')==[row+['1'] for row in before]
        g.capture(output/'evolved.ppm');checks.append('Required input rejected; two records; pending view; additive schema update preserves every original byte and initializes priority')
        # Invalid numeric data cannot partially change an existing row.
        g.request('app.key','1003');g.request('app.key','13');g.request('app.key','9');g.request('app.key','9');g.request('app.key','9');g.request('app.key','8');text(g,'bad')
        saved=records(g,'inspection');g.request('app.key','13',error='app_invalid_value');assert records(g,'inspection')==saved
        # Install a second app and prove its writes do not touch the first namespace.
        p=submit(g,'Second app');p['spec']['id']='stock';p['spec']['title']='Inventory';deliver(g,p);g.request('app.key','13');assert records(g,'stock')==[]
        text(g,'Cable');g.request('app.key','13');assert records(g,'stock')==[['Cable','','false']] and records(g,'inspection')==saved
        g.request('app.open');g.capture(output/'library.ppm');checks.append('Second app has independent records; rejected numeric edit has no side effects')
        # Malformed/unavailable capabilities, duplicate fields, old requests and destructive migrations.
        catalog=g.request('app.catalog');stock=records(g,'stock')
        p=submit(g,'Invalid update','inspection');rid=p['request_id'];rev=p['base_revision']
        g.request('app.begin',f'{rid}:{rev}:inspection:54657374:network',error='app_capability_denied')
        g.request('app.begin',f'{rid}:{rev-1}:inspection:54657374:records.local',error='app_stale_revision')
        g.request('app.begin',f'{rid}:{rev}:inspection:54657374:records.local')
        g.request('app.field',f'{rid}:bad:shell:0:426164:',error='app_unknown_kind')
        g.request('app.field',f'{rid}:bad:text:0:zz:',error='app_bad_hex')
        f=p['spec']['fields'][0];g.request('app.field',f'{rid}:equipment:text:1:45717569706d656e74:')
        g.request('app.field',f'{rid}:equipment:text:1:45717569706d656e74:',error='app_duplicate_field')
        g.request('app.ready',f'{rid}:Missing old fields',error='app_destructive_update')
        p['spec']=schema(g,'inspection');p['spec']['id']='stock';stage(g,p);g.request('app.ready',f'{rid}:Overwrite another app',error='app_destructive_update')
        g.request('app.key','27');g.request('app.ready',f'{rid}:Late reply',error='app_stale_request')
        assert g.request('app.catalog')==catalog and records(g,'inspection')==saved and records(g,'stock')==stock
        checks.append('Capability, revision, kind, hex, duplicate field, destructive migration, cross-app replacement and cancelled reply rejected unchanged')
        app.press(1002);app.press(ord('x'));app.press(13)
        for t in app.threads:t.join(timeout=45);assert not t.is_alive()
        assert g.request('app.key','0')=='state=notice';g.capture(output/'unsupported.ppm');assert g.request('app.catalog')==catalog
        checks.append('Unsupported offline prompt explicitly clarified without installed app')
        report=dict(source='replay',live_meta_calls=0,graphics='native_guest_framebuffer',input='RFB_to_UART',storage='guest_RAM',isolation='interpreter_namespaces_not_processes',checks=checks)
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:app.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve())
