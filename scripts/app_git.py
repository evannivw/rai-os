#!/usr/bin/env python3
"""One model-selected Git operation, gated by the current native app request."""
import json
import os
import subprocess
from apps import MAX_SESSION_ATTEMPTS, ROOT, deliver


def plan(app, request):
    folder=app.output/str(request['request_id'])
    trace=dict(source=app.source,state='starting',maximum_provider_calls=1)
    message='Git request failed. No restore installed. Inspect the session log.'
    try:
        folder.mkdir();path=folder/'request.json';path.write_text(json.dumps(request,indent=2)+'\n')
        with app.lock:
            if not app._planning_active(request):trace['state']='cancelled_or_stale';return
            if app.git_store is None:raise ValueError('Start with --git-store PATH to enable app definition history.')
            if request['target'] is None:raise ValueError('Open an installed app, then use edit app or F2 to request Git tools.')
            if app.provider_calls>=MAX_SESSION_ATTEMPTS:raise ValueError('Session provider limit reached.')
            app.guest.request('app.progress',f"{request['request_id']}:{request['base_revision']}:1")
            app.provider_calls+=1
        result=app._git_plan_attempt(request,path,folder)
        (folder/'provider.stdout.jsonl').write_text(result.stdout)
        (folder/'provider.stderr.jsonl').write_text(result.stderr)
        if result.returncode:raise ValueError('Git planning failed; see retained diagnostics.')
        events=[json.loads(line) for line in result.stdout.splitlines()]
        pending=[event['pending'] for event in events if isinstance(event,dict) and event.get('event')=='app_git_plan']
        if len(pending)!=1:raise ValueError('Invalid Git planner result.')
        pending_path=folder/'pending.json';pending_path.write_text(json.dumps(pending[0],indent=2)+'\n')
        with app.lock:
            if not app._planning_active(request):trace['state']='cancelled_or_stale';return
            # This is the side-effect boundary. UI input/cancel cannot interleave
            # between guest admission and the short, bounded local Git operation.
            env=dict(os.environ);env.pop('MODEL_API_KEY',None);env.pop('RAI_VNC_PASSWORD',None)
            result=subprocess.run([str(app.binary),str(app.kernel),'--app-git-apply',str(app.git_store),str(path),str(pending_path)],cwd=ROOT,env=env,capture_output=True,text=True,timeout=25)
            (folder/'git.stdout.jsonl').write_text(result.stdout)
            (folder/'git.stderr.jsonl').write_text(result.stderr)
            if result.returncode:
                errors=[]
                for line in result.stderr.splitlines():
                    try:
                        event=json.loads(line)
                        if isinstance(event,dict) and event.get('event')=='error':errors.append(event.get('error'))
                    except ValueError:pass
                messages={
                    'app_destructive_update':'Restore rejected: older fields are incompatible. Current data is unchanged.',
                    'workflow_change_denied':'Restore rejected: workflow definitions differ. Current data is unchanged.',
                    'app_capability_change':'Restore rejected: app permissions differ. Current data is unchanged.',
                    'stale_app_git_head':'Git history changed during planning. Submit a fresh request.',
                    'app_git_version_limit':'This app reached its 64-version Git limit.',
                    'app_git_busy':'This app history is in use by another operation.',
                }
                raise ValueError(messages.get(errors[-1] if errors else None,'Git validation or execution failed; see retained diagnostics.'))
            events=[json.loads(line) for line in result.stdout.splitlines()]
            results=[e for e in events if isinstance(e,dict) and e.get('event')=='app_git_result']
            proposals=[e['plan'] for e in events if isinstance(e,dict) and e.get('event')=='app_plan']
            if len(results)!=1 or len(proposals)!=(1 if results[0]['operation']=='restore' else 0):raise ValueError('Invalid Git execution result.')
            if proposals:deliver(app.guest,proposals[0]);trace['state']='review'
            else:app.guest.request('app.notice',f"{request['request_id']}:{results[0]['notice']}");trace['state']='completed'
            trace['operation']=results[0]['operation']
            app.guest.capture(folder/'git-result.ppm')
    except (ValueError,KeyError,TypeError,AssertionError,OSError,subprocess.TimeoutExpired) as error:
        trace['state']='failed';trace['error']=type(error).__name__
        if isinstance(error,ValueError):message=str(error)[:140]
        with app.lock:
            try:
                if app._planning_active(request):app.guest.request('app.notice',f"{request['request_id']}:{message}")
            except (AssertionError,OSError,KeyError):pass
    finally:
        trace['session_provider_attempts']=app.provider_calls
        try:(folder/'git-loop.json').write_text(json.dumps(trace,indent=2)+'\n')
        except OSError:print('Unable to save the Git session log.')
