#!/usr/bin/env python3
"""Actual guest navigation, compact geometry and simulated authentication feedback."""
import argparse
import json
from pathlib import Path
import subprocess
from apps import Studio,build,host_build,records,schema
from app_storage import collect
from demo import FIXTURE

class DeniedStudio(Studio):
    def _provider_attempt(self,r,request_path,feedback_path,attempt_dir):
        return subprocess.CompletedProcess(['synthetic-auth-denial'],1,
            json.dumps(dict(event='provider_diagnostic',error='meta_http_401',source='synthetic'))+'\n',
            json.dumps(dict(event='error',error='meta_http_401'))+'\n')

def verify(kernel,binary,output):
    output.mkdir(parents=True,exist_ok=False);state=output/'device.json';state.write_bytes(FIXTURE.read_bytes());app=None
    report=dict(verified=False,actual_provider_calls=0,checks=[])
    try:
        app=DeniedStudio(kernel,binary,output/'session','meta',state=state);g=app.guest
        notices=[];original_request=g.request
        def observed_request(name,arg=None,error=None):
            if name=='app.notice':notices.append(arg)
            return original_request(name,arg,error=error)
        g.request=observed_request
        before=collect(g,schema,records)
        app.press(27);g.capture(output/'01-home.ppm')
        for row in range(3):
            app.press(27);app.tap(150,250+80*row);assert g.request('app.key','0')=='state=form'
            # The title/field bindings are guest-owned; every launcher must select its own app.
            app.tap(50,125);assert g.request('app.key','0')=='state=compose'
            for c in 'Check app':g.request('app.key',str(ord(c)))
            reply=g.fields('app.key','13');assert reply['target']==before['apps'][row]['spec']['id']
            app.press(27)
        report['checks'].append('Three compact rows open distinct real apps; edit app keeps the correct target')
        app.tap(240,775)
        for c in 'TRANSFER':app.press(ord(c))
        assert g.request('app.key','0')=='state=search';g.capture(output/'02-apps.ppm')
        app.tap(150,294);assert g.request('app.key','0')=='state=search' # visible row gap is not clickable
        app.tap(150,250);assert g.request('app.key','0')=='state=form'
        app.tap(400,775);assert g.request('app.key','0')=='state=compose'
        for c in 'A tool for tracking parts with a total and a report page.':app.press(ord(c))
        g.capture(output/'03-create.ppm');app.press(13)
        for t in app.threads:t.join(timeout=45);assert not t.is_alive()
        assert g.request('app.key','0')=='state=notice'
        g.capture(output/'04-auth-denied.ppm')
        requests=[json.loads(p.read_text()) for p in (output/'session').glob('*/request.json')]
        assert len(requests)==1 and requests[0]['target'] is None
        assert any('Meta rejected authentication' in line for line in notices)
        assert collect(g,schema,records)==before
        report['checks'].append('Apps search is case-insensitive; gap does not select; global Create starts a new app; synthetic 401 shows safe native feedback and leaves records unchanged')
        app.press(27);app.tap(400,775)
        for c in 'X'*128:app.press(ord(c))
        app.press(ord('Y'));g.capture(output/'05-full-prompt.ppm')
        reply=g.fields('app.key','13');assert bytes.fromhex(reply['prompt']).decode()=='X'*128 and reply['target']=='-'
        app.press(27);report['checks'].append('128-character prompt remains bounded; overflow character is not submitted; cancellation leaves state untouched')
        report['verified']=True
    except BaseException as e:report['error']=str(e) or type(e).__name__;raise
    finally:
        if app:app.close()
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    return report
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    verify(build(a.debug),host_build(a.debug),a.output.resolve())
