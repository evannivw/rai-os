#!/usr/bin/env python3
"""Private local setup and explicit bounded Meta connection checks."""
import getpass
import json
import os
from pathlib import Path
import subprocess
import sys
import warnings

# Only trusted codes become user-facing text. Never display arbitrary API bodies.
def message(code):
    messages={
        'cannot_open_local_file':'No Meta key found. Run configure or set MODEL_API_KEY on this computer.',
        'missing_MODEL_API_KEY':'No Meta key found. Add MODEL_API_KEY to your private .env.local.',
        'invalid_api_key_format':'The key format is invalid. Check your private setting; nothing was sent.',
        'duplicate_api_key_setting':'More than one key setting was found. Keep one MODEL_API_KEY entry.',
        'invalid_local_configuration':'The private key file has invalid syntax. Use MODEL_API_KEY=value.',
        'meta_http_401':'Meta rejected authentication. Check or replace your own API key.',
        'meta_http_403':'Meta denied access. Check this account and model permissions.',
        'meta_http_429':'Meta usage limit reached. Check your quota or billing and try later.',
        'meta_http_402':'Meta requires available credit. Check billing for your own account.',
        'meta_timeout':'Meta timed out. Check your connection and try again later.',
        'meta_transport_failed':'Could not reach Meta securely. Check your network and system certificates.',
        'cannot_start_curl':'HTTPS client missing. Install curl or use the Docker demo.',
        'meta_http_500':'Meta service error. Try again later.',
        'meta_http_502':'Meta service error. Try again later.',
        'meta_http_503':'Meta service unavailable. Try again later.',
    }
    return messages.get(code,'Meta check failed. No key is displayed. Inspect the redacted connection result or try later.')

def check(binary, live=False, env=None, cwd=None):
    command=[str(binary),'--meta-check' if live else '--meta-config-check']
    result=subprocess.run(command,capture_output=True,text=True,timeout=40,env=env,cwd=cwd)
    try:
        events=[json.loads(line) for line in result.stdout.splitlines()]
        event=next(e for e in events if isinstance(e,dict) and e.get('event')=='meta_connection')
    except (ValueError,StopIteration):
        raise ValueError('Connection check did not return a readable status') from None
    if result.returncode or event.get('status') not in ('ready','configured'):
        raise ValueError(message(event.get('code')))
    if live and event['status']!='ready':raise ValueError('Meta connection was not verified')
    return event

def configure(binary, path=Path('.env.local'), reader=None):
    path=Path(path)
    if path.exists() or path.is_symlink():raise ValueError('A private key file already exists. Edit it locally; it will not be overwritten.')
    if 'MODEL_API_KEY' in os.environ:raise ValueError('MODEL_API_KEY is set in your environment and takes priority. Update or unset it before creating a local file.')
    if reader is None:
        if not sys.stdin.isatty():raise ValueError('Use an interactive terminal for hidden key entry, or edit .env.local privately.')
        reader=getpass.getpass
    # Refuse getpass's echo fallback; never print or accept a visibly echoed key.
    with warnings.catch_warnings():
        warnings.simplefilter('error',getpass.GetPassWarning)
        try:key=reader('Paste your own Meta API key (hidden): ')
        except getpass.GetPassWarning:raise ValueError('Hidden entry is unavailable. Edit .env.local privately.') from None
    env=dict(os.environ);env['MODEL_API_KEY']=key
    check(binary,env=env) # Rust validates format; no network call.
    flags=os.O_WRONLY|os.O_CREAT|os.O_EXCL|getattr(os,'O_NOFOLLOW',0)
    fd=os.open(path,flags,0o600)
    with os.fdopen(fd,'w') as f:
        f.write('MODEL_API_KEY='+key+'\n');f.flush();os.fsync(f.fileno())
    print('Saved privately to .env.local. Format checked; Meta has not been contacted.')
    print('Next: run check-meta to make one small request and verify access.')
