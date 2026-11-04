#!/usr/bin/env python3
"""Bounded RFB 3.3/3.8 input relay. Pixels originate only from QEMU's framebuffer.
Standard protocol: https://www.rfc-editor.org/rfc/rfc6143.html
No web frontend, authentication, physical keyboard driver or remote exposure.
"""
import socket
import queue
import struct
import threading

def exact(s,n):
    data=b''
    while len(data)<n:
        chunk=s.recv(n-len(data))
        if not chunk:raise EOFError('VNC disconnected')
        data+=chunk
    return data

def session(client,path,press,tap=None):
    with socket.socket(socket.AF_UNIX) as backend:
        backend.connect(str(path));client.settimeout(15);backend.settimeout(15)
        banner=exact(backend,12);assert banner==b'RFB 003.008\n';client.sendall(banner)
        version=exact(client,12)
        assert version in (banner,b'RFB 003.003\n'),'RFB 3.3 or 3.8 required'
        legacy=version==b'RFB 003.003\n'
        backend.sendall(banner)
        count=exact(backend,1);types=exact(backend,count[0]);security=2 if 2 in types else 1;assert security in types
        if legacy:client.sendall(struct.pack('!I',security))
        else:
            client.sendall(bytes([1,security]));assert exact(client,1)==bytes([security])
        backend.sendall(bytes([security]))
        if security==2:
            client.sendall(exact(backend,16));backend.sendall(exact(client,16))
        result=exact(backend,4)
        if not legacy or security==2:client.sendall(result)
        assert result==b'\0'*4,'VNC authentication failed'
        backend.sendall(exact(client,1));header=exact(backend,24);size=struct.unpack('!I',header[20:24])[0];assert size<=4096
        client.sendall(header+exact(backend,size));client.settimeout(None);backend.settimeout(None)
        def pixels():
            try:
                while True:
                    data=backend.recv(65536)
                    if not data:break
                    client.sendall(data)
            except OSError:pass
            finally:
                try:client.shutdown(socket.SHUT_RDWR)
                except OSError:pass
        worker=threading.Thread(target=pixels,daemon=True);worker.start();buttons=0
        try:
            while True:
                kind=exact(client,1)[0]
                if kind==0:backend.sendall(bytes([kind])+exact(client,19))
                elif kind==2:
                    header=exact(client,3);n=struct.unpack('!H',header[1:])[0];assert n<=256
                    enc=struct.unpack('!'+str(n)+'i',exact(client,n*4))
                    # Avoid extension-specific keyboard/clipboard messages we do not implement.
                    enc=[v for v in enc if v in (0,1,5,6,7,16,-223,-239)]
                    if not enc:enc=[0]
                    backend.sendall(struct.pack('!BBH',2,0,len(enc))+struct.pack('!'+str(len(enc))+'i',*enc))
                elif kind==3:backend.sendall(bytes([kind])+exact(client,9))
                elif kind==4:
                    data=exact(client,7);key=struct.unpack('!I',data[3:])[0]
                    if data[0]:
                        key={0xff0d:13,0xff08:8,0xff1b:27,0xff09:9,0xffbe:1001,0xffbf:1002,0xffc0:1003,0xffc1:1004,0xff52:1005,0xff54:1006,0xffc2:1007,0xff53:1008,0xff51:1009,0xffc3:1010,0xffc4:1011,0xffc5:1012,0xffc6:1013,0xffc7:1014,0xffc8:1015,0xffc9:1016}.get(key,key)
                        if key in (8,9,13,27,1001,1002,1003,1004,1005,1006,1007,1008,1009,1010,1011,1012,1013,1014,1015,1016) or 32<=key<=126:press(key)
                elif kind==5:
                    mask,x,y=struct.unpack('!BHH',exact(client,5))
                    if mask&1 and not buttons&1:
                        if tap is not None:tap(x,y)
                        elif 24<=x<456 and 706<=y<754:press(13)
                        elif y>=765:press(27)
                        elif 490<=y<535:press(9)
                    buttons=mask
                elif kind==6:
                    header=exact(client,7);n=struct.unpack('!I',header[3:])[0];assert n<=128
                    # Paste printable text only. Never turn pasted newlines into submission.
                    data=exact(client,n)
                    if all(32<=b<=126 for b in data):
                        for b in data:press(b)
                else:raise ValueError('Unsupported VNC client message')
        finally:
            try:backend.shutdown(socket.SHUT_RDWR)
            except OSError:pass
            worker.join(timeout=2)

def serve(path,press,port=5900,bind='127.0.0.1',tap=None):
    with socket.socket() as listener:
        listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1);listener.bind((bind,port));listener.listen(1)
        print(f'RAI-OS workspace ready: VNC localhost:{port}. Type an instruction; Enter previews, Enter confirms, Esc cancels.',flush=True)
        while True:
            client,_=listener.accept()
            with client:
                try:session(client,path,press,tap)
                except (OSError,EOFError,ValueError,AssertionError,queue.Full) as error:print(f'VNC viewer disconnected ({type(error).__name__}: {error}); guest workspace retained.',flush=True)
