#!/usr/bin/env python3
"""Offline setup boundary checks with a synthetic key; never contact Meta."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
from unittest.mock import patch
from meta_setup import check,configure,message
from workspace import host_build

DUMMY='LLM_synthetic_only_no_access_0000'
def verify(binary,output):
    output.mkdir(parents=True,exist_ok=False);checks=[]
    with tempfile.TemporaryDirectory(prefix='rai-setup-') as folder:
        cwd=Path(folder);env=dict(os.environ);env.pop('MODEL_API_KEY',None)
        def fails(fn,fragment):
            try:fn()
            except ValueError as e:
                assert fragment in str(e) and DUMMY not in str(e)
            else:raise AssertionError('Expected rejection')
        fails(lambda:check(binary,env=env,cwd=cwd),'No Meta key')
        (cwd/'.env.local').write_text('MODEL_API_KEY='+DUMMY+'\n')
        assert check(binary,env=env,cwd=cwd)['status']=='configured'
        bad=dict(env,MODEL_API_KEY='bad')
        fails(lambda:check(binary,env=bad,cwd=cwd),'format is invalid')
        checks.append('Missing key and invalid environment override rejected; valid format is configured, not authenticated')
        with patch.dict(os.environ,env,clear=True):
            private=cwd/'private.env';capture=io.StringIO()
            with contextlib.redirect_stdout(capture):configure(binary,private,reader=lambda _:DUMMY)
            assert DUMMY not in capture.getvalue() and private.read_text()=='MODEL_API_KEY='+DUMMY+'\n'
            assert private.stat().st_mode & 0o777==0o600
            fails(lambda:configure(binary,private,reader=lambda _:DUMMY),'already exists')
            invalid=cwd/'invalid.env';fails(lambda:configure(binary,invalid,reader=lambda _:'$(do not run)'),'format is invalid');assert not invalid.exists()
            link=cwd/'link.env';link.symlink_to(private)
            fails(lambda:configure(binary,link,reader=lambda _:DUMMY),'already exists')
        checks.append('Hidden-entry adapter validates before exclusive 0600 write; existing files/symlinks and malformed keys preserved/rejected without disclosure')
        for code,expected in [('meta_http_401','authentication'),('meta_http_403','permissions'),('meta_http_429','quota'),('meta_timeout','timed out'),('meta_transport_failed','securely')]:
            event=dict(event='meta_connection',status='error',code=code)
            result=subprocess.CompletedProcess([],1,json.dumps(event),DUMMY)
            with patch('meta_setup.subprocess.run',return_value=result):fails(lambda:check(binary,live=True),expected)
            assert len(message(code))<=160
        with patch('meta_setup.subprocess.run',return_value=subprocess.CompletedProcess([],0,json.dumps(dict(event='meta_connection',status='configured')),'')):
            fails(lambda:check(binary,live=True),'not verified')
        checks.append('Synthetic 401/403/429/timeout/transport responses produce distinct safe messages; configured cannot masquerade as connected')
    result=dict(verified=True,provider_calls=0,checks=checks)
    (output/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':
    import argparse
    p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);a=p.parse_args();verify(host_build(),a.output.resolve())
