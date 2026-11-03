#!/usr/bin/env python3
"""Prompt-to-microapp bridge. QEMU owns schemas, records, UI and installation."""
import argparse
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import threading
import time
sys.dont_write_bytecode=True
from files_demo import Guest,ROOT,build
from workspace import host_build
from workspace_vnc import serve
from app_storage import Checkpoint,collect,restore
from app_exchange import Exchange

MAX_ATTEMPTS = 3
MAX_SESSION_ATTEMPTS = 8
REQUEST_TIMEOUT = 110
ATTEMPT_TIMEOUT = 40
def schema(g,app_id):
    row=g.fields('app.schema',app_id);fields=[]
    for item in row['fields'].split(','):
        key,kind,required,label,default,formula=item.split('~')
        fields.append(dict(key=key,kind=kind,required=required=='1',label=bytes.fromhex(label).decode('ascii'),default=bytes.fromhex(default).decode('ascii')))
        if formula:fields[-1]['formula']=bytes.fromhex(formula).decode('ascii')
    result=dict(id=row['id'],title=bytes.fromhex(row['title']).decode('ascii'),capability=row['capability'],fields=fields,filter_key=row['filter'],filter_label=bytes.fromhex(row['filter_label']).decode('ascii'))
    if row['initial']:
        actions=[]
        for item in row['actions'].split(','):
            key,label,frm,to,guard,op,value=item.split('~')
            actions.append(dict(id=key,label=bytes.fromhex(label).decode('ascii'),**{'from':frm},to=to,guard_field=guard,guard_op=op,guard_value=int(value)))
        result['workflow']=dict(initial=row['initial'],actions=actions)
    if row.get('behaviors'):
        buttons=[]
        for item in row['behaviors'].split(','):
            parts=item.split('~')
            key,label,event,encoded=parts[:4]
            if len(parts) not in (4,5) or (len(parts)==5 and parts[4]!='all'):raise ValueError('Invalid behavior scope')
            steps=[]
            for entry in encoded.split('/'):
                op,field,value,compare,skip=entry.split('^')
                steps.append(dict(op=op,field=field,value=bytes.fromhex(value).decode('ascii'),compare=compare,skip=int(skip)))
            button=dict(id=key,label=bytes.fromhex(label).decode('ascii'),event=event,steps=steps)
            if len(parts)==5:button['scope']='all'
            buttons.append(button)
        result['behaviors']=buttons
    if row.get('pages'):
        pages=[]
        for encoded in row['pages'].split('|'):
            title,*blocks=encoded.split('~');page=dict(title=bytes.fromhex(title).decode('ascii'),blocks=[])
            for encoded_block in blocks:
                kind,title,value,label,aggregate,scope=encoded_block.split('^')
                page['blocks'].append(dict(kind=kind,title=bytes.fromhex(title).decode('ascii'),value=bytes.fromhex(value).decode('ascii'),label_field=label,aggregate=aggregate,scope=scope))
            pages.append(page)
        result['pages']=pages
    return result

def states(g,app_id):
    data=g.fields('app.export',app_id)
    return [] if data['count']=='0' else data['states'].split(',')

def records(g,app_id):
    data=g.fields('app.export',app_id)
    rows=[] if data['count']=='0' else [[bytes.fromhex(v).decode('ascii') for v in row.split(',')] for row in data['rows'].split('|')]
    assert len(rows)==int(data['count'])
    return rows

def deliver(g,p):
    rid=p['request_id']
    if p['decision']!='install':g.request('app.notice',f"{rid}:{p['reason']}");return
    s=p['spec'];g.request('app.begin',f"{rid}:{p['base_revision']}:{s['id']}:{s['title'].encode().hex()}:{s['capability']}")
    for f in s['fields']:
        g.request('app.field',f"{rid}:{f['key']}:{f['kind']}:{int(f['required'])}:{f['label'].encode().hex()}:{f['default'].encode().hex()}")
        if f.get('formula'):g.request('app.formula',f"{rid}:{f['key']}:{f['formula'].encode().hex()}")
    if s.get('workflow'):
        flow=s['workflow'];g.request('app.flow',f"{rid}:{flow['initial']}")
        for a in flow['actions']:g.request('app.action',f"{rid}:{a['id']}:{a['label'].encode().hex()}:{a['from']}:{a['to']}:{a['guard_field']}:{a['guard_op']}:{a['guard_value']}")
    for button in s.get('behaviors',[]):
        g.request('app.button',f"{rid}:{button['id']}:{button['label'].encode().hex()}:{button['event']}")
        if button.get('scope','row') != 'row':g.request('app.button.scope',f"{rid}:{button['id']}:{button['scope']}")
        for step in button['steps']:
            g.request('app.step',f"{rid}:{button['id']}:{step['op']}:{step['field']}:{step['value'].encode().hex()}:{step['compare']}:{step['skip']}")
    for index,page in enumerate(s.get('pages',[])):
        g.request('app.page',f"{rid}:{page['title'].encode().hex()}")
        for b in page['blocks']:
            g.request('app.block',f"{rid}:{index}:{b['kind']}:{b['title'].encode().hex()}:{b['value'].encode().hex()}:{b['label_field']}:{b['aggregate']}:{b['scope']}")
    g.request('app.filter',f"{rid}:{s['filter_key']}:{s['filter_label'].encode().hex()}")
    g.request('app.ready',f"{rid}:{p['reason']}")

class Studio:
    def __init__(self,kernel,binary,output,source='meta',vnc=False,state=None,exchange=None,git_store=None):
        self.output=output;output.mkdir(parents=True,exist_ok=False)
        password=os.environ.get('RAI_VNC_PASSWORD')
        if password and (not password.isascii() or not 1<=len(password)<=8):raise ValueError('RAI_VNC_PASSWORD must be 1 to 8 ASCII characters')
        self.exchange=Exchange(exchange) if exchange else None
        self.checkpoint=Checkpoint(state,binary,kernel) if state else None
        self.saved_revision=None;self.storage_error=None
        self.lock=threading.RLock();self.threads=[];self.closed=False
        self.binary=binary;self.kernel=kernel;self.source=source
        self.git_store = Path(git_store).absolute() if git_store is not None else None
        self.repair_enabled = source == 'meta'
        self.provider_calls = 0
        self._attempt_timeout = ATTEMPT_TIMEOUT
        try:
            saved=self.checkpoint.load() if self.checkpoint else None
            self.guest=Guest(kernel,vnc=vnc,vnc_password=password if vnc else None)
            self.guest.request('app.open');self.guest.request('app.source',source)
            if saved is not None:restore(self.guest,saved,deliver,schema,records)
            self.saved_revision=self.guest.fields('app.catalog')['revision']
            self.guest.request('app.storage','restored' if saved is not None else 'RAM only')
        except BaseException:
            if hasattr(self,'guest'):self.guest.close()
            if self.checkpoint:self.checkpoint.close()
            raise
    def persist(self):
        if not self.checkpoint:return
        revision=self.guest.fields('app.catalog')['revision']
        if revision==self.saved_revision:return
        self.guest.request('app.storage','unsaved')
        try:
            self.checkpoint.save(collect(self.guest,schema,records))
            self.saved_revision=revision;self.storage_error=None;self.guest.request('app.storage','saved')
        except (OSError,ValueError,subprocess.TimeoutExpired):
            self.storage_error='Checkpoint failed; current changes remain in RAM.'
            self.guest.request('app.storage','save failed');print(self.storage_error,flush=True)
    def press(self,key):
        return self.dispatch('app.key',str(key))
    def dispatch(self,name,arg):
        with self.lock:
            if self.closed:return
            reply=self.guest.request(name,arg)
            if reply.startswith('transfer='):
                event=dict(p.split('=',1) for p in reply.split('; '))
                try:
                    if self.exchange is None:raise ValueError('No exchange folder selected. Start with --exchange PATH.')
                    self.exchange.transfer(self.guest,event,schema,records)
                except FileNotFoundError:
                    self.guest.request('app.transfer.notice',f"{event['request']}:Place {event['app']}.csv in the selected exchange inbox, then request Import again.")
                except FileExistsError:
                    self.guest.request('app.transfer.notice',f"{event['request']}:This export already exists. Existing files are never overwritten.")
                except ValueError as error:
                    self.guest.request('app.transfer.notice',f"{event['request']}:{str(error)[:160]}")
                except (OSError,AssertionError):
                    self.guest.request('app.transfer.notice',f"{event['request']}:Transfer rejected. Check file access, values, formulas and available row slots. No rows were imported.")
            if reply.startswith('submit='):
                state=dict(p.split('=',1) for p in reply.split('; '));rid=int(state['submit'])
                if any(t.is_alive() for t in self.threads) or self.provider_calls >= MAX_SESSION_ATTEMPTS:
                    self.guest.request('app.notice',f'{rid}:A request is running or this session reached its eight-attempt limit.');return reply
                catalog=self.guest.fields('app.catalog');ids=[];count=0
                for entry in catalog['apps'].split(','):
                    if entry=='-':continue
                    app_id,version,rows,title=entry.split(':');ids.append(app_id)
                    if app_id==state['target']:count=int(rows)
                request=dict(request_id=rid,base_revision=int(state['revision']),goal=bytes.fromhex(state['prompt']).decode('ascii'),target=None if state['target']=='-' else schema(self.guest,state['target']),app_ids=ids,record_count=count)
                thread=threading.Thread(target=self.plan,args=(request,),daemon=False);self.threads.append(thread);thread.start()
            self.persist()
            return reply
    def tap(self,x,y):
        with self.lock:
            if self.closed:return
            # Pointer geometry belongs to the guest. Submission has the same shape
            # as a key reply; a click on Create needs the same planning path.
            return self.dispatch('app.tap',f'{x}:{y}')
    def _planning_active(self, r):
        # Read-only check before any progress update, call, or response delivery.
        # A cancelled/new request must not receive an old attempt's message.
        if self.closed:
            return False
        pending = self.guest.fields('app.planning')
        return (pending['state'] == 'thinking'
                and pending['request'] == str(r['request_id'])
                and pending['base'] == str(r['base_revision'])
                and pending['revision'] == str(r['base_revision']))

    def _provider_attempt(self, r, request_path, feedback_path, attempt_dir):
        """One process, one possible API request; validation stays in Rust."""
        if feedback_path is not None:
            cmd = [str(self.binary), str(self.kernel), '--app-repair', str(request_path), str(feedback_path)]
        else:
            cmd = [str(self.binary), str(self.kernel), '--app-plan', str(request_path)]
            if self.source == 'replay':
                from app_replay import propose
                supplied = attempt_dir / 'supplied.json'
                supplied.write_text(json.dumps(propose(r)) + '\n')
                cmd[2] = '--app-plan-replay'
                cmd.append(str(supplied))
        env = dict(os.environ)
        env.pop('RAI_VNC_PASSWORD', None)
        return subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True,
                              text=True, timeout=self._attempt_timeout)

    def _git_plan_attempt(self, request, request_path, folder):
        if self.source != 'meta':
            raise ValueError('Git planning requires Meta; scripted choices belong to the offline verifier.')
        env=dict(os.environ);env.pop('RAI_VNC_PASSWORD',None)
        return subprocess.run([str(self.binary),str(self.kernel),'--app-git-plan',str(self.git_store),str(request_path)],
                              cwd=ROOT,env=env,capture_output=True,text=True,timeout=40)

    def plan(self, r):
        if r['goal'].lstrip().lower().startswith('git:'):
            from app_git import plan
            return plan(self,r)
        folder = self.output / str(r['request_id'])
        request_path = folder / 'request.json'
        rid = r['request_id']
        trace = dict(source=self.source, max_attempts=MAX_ATTEMPTS if self.repair_enabled else 1,
                     session_limit=MAX_SESSION_ATTEMPTS, attempts=[], state='running')
        deadline = time.monotonic() + REQUEST_TIMEOUT
        feedback_path = None
        message = 'No app installed. The proposal was unavailable or invalid. See the session log.'
        # Aggregate logs preserve older tooling, while each attempt has its own files.
        stdout_log = folder / 'provider.stdout.jsonl'
        stderr_log = folder / 'provider.stderr.jsonl'
        try:
            folder.mkdir()
            request_path.write_text(json.dumps(r, indent=2) + '\n')
            stdout_log.write_text(''); stderr_log.write_text('')
            for attempt in range(1, trace['max_attempts'] + 1):
                with self.lock:
                    if not self._planning_active(r):
                        trace['state'] = 'cancelled_or_stale'; return
                    if self.provider_calls >= MAX_SESSION_ATTEMPTS:
                        trace['state'] = 'session_limit'
                        message = 'Session attempt limit reached. No app installed.'
                        break
                    # Reserve a complete bounded call; never kill a Rust process
                    # early while its separately bounded HTTPS child is still active.
                    if deadline - time.monotonic() < ATTEMPT_TIMEOUT:
                        trace['state'] = 'deadline'
                        message = 'Planning time limit reached. No app installed.'
                        break
                    self.guest.request('app.progress', f"{rid}:{r['base_revision']}:{attempt}")
                    self.provider_calls += 1
                attempt_dir = folder / f'attempt-{attempt:02d}'
                attempt_dir.mkdir()
                entry = dict(attempt=attempt, kind='repair' if feedback_path else 'initial', state='started')
                trace['attempts'].append(entry)
                (folder / 'loop.json').write_text(json.dumps(trace, indent=2) + '\n')
                try:
                    result = self._provider_attempt(r, request_path, feedback_path, attempt_dir)
                except subprocess.TimeoutExpired as error:
                    # TimeoutExpired can carry bytes even when text=True. Preserve
                    # partial adapter output without pretending a response finished.
                    for stream, raw in [('stdout', error.stdout), ('stderr', error.stderr)]:
                        data = raw or b''
                        if isinstance(data, str): data = data.encode('utf-8')
                        (attempt_dir / f'provider.{stream}.jsonl').write_bytes(data)
                    entry['code'] = 'adapter_timeout'
                    raise
                (attempt_dir / 'provider.stdout.jsonl').write_text(result.stdout)
                (attempt_dir / 'provider.stderr.jsonl').write_text(result.stderr)
                entry['exit_code'] = result.returncode
                with stderr_log.open('a') as log:
                    # Raw per-attempt bytes remain above; aggregate stderr stays JSONL.
                    for line in result.stderr.splitlines():
                        try: event = json.loads(line)
                        except json.JSONDecodeError: event = dict(event='adapter_error', message=line)
                        if not isinstance(event, dict): raise ValueError('invalid_diagnostic_event')
                        event['attempt'] = attempt
                        log.write(json.dumps(event) + '\n')
                events = [json.loads(line) for line in result.stdout.splitlines()]
                if any(not isinstance(event, dict) for event in events):
                    raise ValueError('invalid_planner_event')
                with stdout_log.open('a') as log:
                    for event in events:
                        event['attempt'] = attempt
                        log.write(json.dumps(event) + '\n')
                with self.lock:
                    if not self._planning_active(r):
                        entry['state'] = 'discarded'; trace['state'] = 'cancelled_or_stale'; return
                plans = [event['plan'] for event in events if event.get('event') == 'app_plan']
                failures = [event for event in events if event.get('event') == 'app_validation_failed']
                if result.returncode == 0:
                    if len(plans) != 1 or failures:
                        raise ValueError('invalid_planner_result')
                    with self.lock:
                        if not self._planning_active(r):
                            entry['state'] = 'discarded'; trace['state'] = 'cancelled_or_stale'; return
                        deliver(self.guest, plans[0])
                        self.guest.capture(folder / 'proposal.ppm')
                    entry['state'] = 'accepted'; trace['state'] = 'review' if plans[0]['decision'] == 'install' else plans[0]['decision']
                    trace['accepted_attempt'] = attempt
                    return
                entry['state'] = 'rejected'
                if len(failures) != 1 or plans:
                    trace['state'] = 'provider_failed'; break
                failure = failures[0]
                code = failure['code']
                entry['code'] = code
                if not isinstance(code, str) or not code.replace('_', '').isalnum() or len(code) > 64:
                    raise ValueError('invalid_failure_code')
                message = f'Proposal rejected: {code}. No app installed.'
                if not self.repair_enabled or failure.get('repairable') is not True:
                    trace['state'] = 'not_repairable'; break
                if attempt == trace['max_attempts']:
                    trace['state'] = 'exhausted'
                    message = f'Stopped after {attempt} attempts: {code}. No app installed.'
                    break
                # Only rejected definitions and trusted validator feedback; no rows.
                feedback = dict(attempt=attempt + 1, code=code, proposal=failure['proposal'])
                feedback_path = folder / f'feedback-{attempt + 1:02d}.json'
                feedback_path.write_text(json.dumps(feedback, indent=2) + '\n')
            with self.lock:
                if self._planning_active(r):
                    self.guest.request('app.notice', f'{rid}:{message}')
        except (ValueError, KeyError, TypeError, AttributeError, AssertionError, subprocess.TimeoutExpired, OSError) as error:
            trace['state'] = 'failed'
            trace['error'] = type(error).__name__
            if trace['attempts'] and trace['attempts'][-1]['state'] == 'started':
                trace['attempts'][-1]['state'] = 'failed'
            with self.lock:
                if not self.closed:
                    try:
                        if self._planning_active(r): self.guest.request('app.notice', f'{rid}:{message}')
                    except (AssertionError, OSError, KeyError, TypeError, ValueError): pass
        finally:
            trace['session_attempts_used'] = self.provider_calls
            try: (folder / 'loop.json').write_text(json.dumps(trace, indent=2) + '\n')
            except OSError: print('Unable to save the final app planning log.', file=sys.stderr)

    def close(self):
        with self.lock:self.closed=True
        for t in self.threads:t.join(timeout=45)
        try:
            with self.lock:
                if self.guest.process.poll() is None:self.persist()
        finally:
            try:(self.output/'uart.log').write_text('\n'.join(self.guest.log)+'\n')
            finally:
                self.guest.close()
                if self.checkpoint:self.checkpoint.close()

def run(kernel,binary,output,source='meta',port=5900,bind='127.0.0.1',state=None,exchange=None,git_store=None):
    app=Studio(kernel,binary,output,source,vnc=True,state=state,exchange=exchange,git_store=git_store);keys=queue.Queue(maxsize=256)
    if app.exchange:print(f'CSV exchange folder: {app.exchange.root}',flush=True)
    def press(key):keys.put_nowait(key)
    def keyboard():
        while True:
            key=keys.get()
            if key is None:return
            try:
                if isinstance(key,tuple):app.tap(*key)
                else:app.press(key)
            except (AssertionError,OSError):print('Guest rejected input.',flush=True)
    worker=threading.Thread(target=keyboard,daemon=True);worker.start()
    try:serve(app.guest.vnc_socket,press,port,bind,tap=lambda x,y:keys.put_nowait((x,y)))
    finally:keys.put(None);worker.join(timeout=10);app.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',choices=['meta','replay'],default='meta');p.add_argument('--output',type=Path,required=True);p.add_argument('--port',type=int,default=5900);p.add_argument('--debug',action='store_true');p.add_argument('--state',type=Path,default=ROOT/'target/device.json');p.add_argument('--temporary',action='store_true');p.add_argument('--exchange',type=Path,default=ROOT/'target/exchange');p.add_argument('--git-store',type=Path);a=p.parse_args()
    try:run(build(a.debug),host_build(a.debug),a.output.resolve(),a.source,a.port,state=None if a.temporary else a.state,exchange=a.exchange,git_store=a.git_store)
    except KeyboardInterrupt:pass
