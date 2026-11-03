#!/usr/bin/env python3
"""Real guest reboot recovery and host checkpoint failure checks. No inference."""
import argparse
import copy
import json
from pathlib import Path
import sys
from unittest.mock import patch
sys.dont_write_bytecode=True
from app_replay import SPEC,CREATE,EVOLVE
from apps import Studio,records,schema,build,host_build
from app_storage import Checkpoint,collect,canonical
from verify_apps import wait,text

def verify(kernel,binary,out):
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json';checks=[]
    app=Studio(kernel,binary,out/'first','replay',state=state);g=app.guest
    try:
        for c in CREATE:app.press(ord(c))
        app.press(13);wait(app);assert not state.exists(),'Preview must not create checkpoint'
        app.press(13);assert state.exists()
        # The imported value path still validates the complete record on save.
        g.request('app.value','8:61',error='app_bad_field_index');g.request('app.value','0:zz',error='app_bad_hex')
        text(g,'Pump A');g.request('app.key','9');text(g,'Seal');app.press(13)
        first=state.read_bytes();assert records(g,'inspection')==[['Pump A','Seal','false']]
        app.press(110);text(g,'Pump B');app.press(13);saved=collect(g,schema,records)
        assert state.read_bytes()!=first;app.press(27);g.capture(out/'saved-home.ppm')
        try:Checkpoint(state,binary,kernel)
        except ValueError:pass
        else:raise AssertionError('Concurrent writer lock accepted')
        checks.append('Install and save checkpoint only committed state; second session cannot write the same device')
    finally:app.close()
    app=Studio(kernel,binary,out/'second','replay',state=state);g=app.guest
    try:
        assert collect(g,schema,records)==saved and not app.threads;g.capture(out/'restored-home.ppm')
        app.tap(132,270);app.press(1002)
        for c in EVOLVE:app.press(ord(c))
        app.press(13);wait(app);app.press(13)
        assert records(g,'inspection')==[['Pump A','Seal','false','1'],['Pump B','','false','1']]
        expected=collect(g,schema,records);previous=state.read_bytes()
        # A failed replacement leaves the old bytes intact and does not falsely
        # acknowledge persistence. The live guest can still be used and retried.
        app.press(1003);app.press(13)
        with patch('app_storage.os.replace',side_effect=OSError('injected replace failure')):
            g.request('app.value','1',error='bad_args')
            g.request('app.value','1:5265706169726564');app.press(13)
            assert app.storage_error is not None and state.read_bytes()==previous;g.capture(out/'save-failed.ppm')
        assert not list(out.glob('.rai-checkpoint-*'))
        app.press(27);assert app.storage_error is None and state.read_bytes()!=previous
        expected=collect(g,schema,records);g.capture(out/'saved-again.ppm')
        checks.append('Fresh QEMU recovered exact apps/rows without Meta; compatible update persisted; interrupted replacement preserved last good state and retry recovered')
        g.process.kill();g.process.wait(timeout=5)
    finally:app.close()
    app=Studio(kernel,binary,out/'third','replay',state=state)
    try:
        assert collect(app.guest,schema,records)==expected and not app.threads
        app.tap(132,270);app.press(1003);app.press(13);app.guest.capture(out/'restored-record.ppm')
    finally:app.close()
    good=state.read_bytes();envelope=json.loads(good)
    # Corruption is refused before a guest starts; never silently reset user data.
    state.write_bytes(good.replace(b'Repaired',b'Corrupt!'))
    try:Studio(kernel,binary,out/'corrupt','replay',state=state)
    except ValueError:pass
    else:raise AssertionError('Corrupt state accepted')
    assert state.read_bytes()!=good
    # Even a matching checksum cannot authorize structurally invalid app content.
    import hashlib
    envelope['state']['apps'][0]['spec']['capability']='network';envelope['sha256']=hashlib.sha256(canonical(envelope['state'])).hexdigest();bad=canonical(envelope);state.write_bytes(bad)
    try:Studio(kernel,binary,out/'invalid','replay',state=state)
    except ValueError:pass
    else:raise AssertionError('Invalid capabilities restored')
    assert state.read_bytes()==bad;state.write_bytes(good)
    checks.append('Abrupt guest termination retained committed state; checksum corruption and valid-checksum unsupported capability rejected without overwriting saved data')
    report=dict(live_meta_calls=0,guest_reboots=2,storage='atomic host checkpoint; no guest disk driver',checks=checks)
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve())
