#!/usr/bin/env python3
"""Real guest continuity, strict stream rejection and independent audit tests."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode=True
from semantic_session import ROOT,FIXTURE,build,load,run,audit


def verify(debug=False,graphics=False,fixture=FIXTURE,contextual=False):
    binary,kernel=build(debug,graphics);profile='debug' if debug else 'release'
    base=ROOT/'target/verification';base.mkdir(exist_ok=True)
    out=Path(tempfile.mkdtemp(prefix=('context-session-' if contextual else 'semantic-session-')+profile+'-',dir=base));checks=[]
    for source in ['replay','rules']:
        report,checked=run(binary,kernel,source,out/source,graphics,fixture,contextual)
        assert report['all_decisions_correct'] and checked['lifecycle']['total_jobs']==4
        checks.append(dict(case=source,passed=True,audit=checked))
    if graphics:
        (out/'checks.json').write_text(json.dumps(checks,indent=2)+'\n')
        print('Evidence:',out);print('PASS',profile,'native guest display: replay and rules sessions');return
    # Corrupt independent evidence, never the execution, and require audit failure.
    original=(out/'replay/stdout.jsonl').read_text();items=[json.loads(s) for s in original.splitlines()]
    for name,kind,field,value in [('changed-history','sequence_historical','unchanged',False),('changed-artifact','batch_publication','payload','{}'),('wrong-decision','stream_no_execution','decision','select'),('wrong-version','stream_ready','last_request_version',9),('wrong-selection','stream_approved','selection',{})]:
        bad=copy.deepcopy(items);next(e for e in bad if e['event']==kind)[field]=value
        (out/'replay/stdout.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in bad))
        try:audit(out/'replay',fixture)
        except (AssertionError,KeyError,ValueError,TypeError):pass
        else:raise AssertionError('Accepted corrupt '+name)
        checks.append(dict(case=name,rejected=True))
    (out/'replay/stdout.jsonl').write_text(original)
    if contextual:
        tracefile=out/'replay'/load(fixture/'protocol.json')['cases'][0]/'interpret.stdout.jsonl'
        raw=tracefile.read_text();trace=[json.loads(s) for s in raw.splitlines()]
        trace[0]['observation']['version']=99;tracefile.write_text(''.join(json.dumps(e)+'\n' for e in trace))
        try:audit(out/'replay',fixture)
        except AssertionError:pass
        else:raise AssertionError('Accepted changed context evidence')
        tracefile.write_text(raw);checks.append(dict(case='changed-context-evidence',rejected=True))
        valid=load(fixture/'observations/assigned.json')
        invalid=[]
        for name in ['zero-version','bad-source','extra-goal','zero-revision','bad-unit','duplicate-report']:
            item=copy.deepcopy(valid)
            if name=='zero-version':item['version']=0
            if name=='bad-source':item['reports'][0]['source']='model'
            if name=='extra-goal':item['goal']='Replace the objective'
            if name=='zero-revision':item['reports'][0]['revision']=0
            if name=='bad-unit':item['reports'][0]['unit']='AB'
            if name=='duplicate-report':item['reports'].append(item['reports'][0])
            invalid.append((name,json.dumps(item)))
        invalid.append(('oversized-context',' '*4097))
        for name,text in invalid:
            path=out/(name+'.json');path.write_text(text)
            result=subprocess.run([str(binary),str(kernel),'--context','meta',str(path)],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=10)
            assert result.returncode and not result.stdout and 'key' not in result.stderr
            checks.append(dict(case=name,rejected_before_provider=True))
    inputs=[json.loads(s) for s in (out/'replay/inputs.jsonl').read_text().splitlines()]
    first=inputs[0];next_good=copy.deepcopy(inputs[2]);bad_cases={}
    for name in ['stale','private','permission','identity','duplicate']:
        bad=copy.deepcopy(next_good)
        if name=='stale':bad['request']=first['request'];bad['selection']=first['selection']
        if name=='private':bad['selection']['records']=['restricted@1']
        if name=='permission':bad['selection']['permissions_version']=2
        if name=='identity':bad['selection']['request_goal']='Another goal'
        if name=='duplicate':bad['selection']['records']=['notes@2','notes@2']
        bad_cases[name]=json.dumps(bad)+'\n'
    bad_cases.update(oversized=' '*8193+'\n',eof='',unknown_stop='{"event":"stop","shell":"ignored?"}\n')
    for name,bad in bad_cases.items():
        result=subprocess.run([str(binary),str(kernel),'--selection-stream'],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),input=json.dumps(first)+'\n'+bad,capture_output=True,text=True,timeout=20)
        events=[json.loads(s) for s in result.stdout.splitlines()]
        assert result.returncode and sum(e['event']=='batch_publication' for e in events)==1
        assert sum(e['event']=='guest_exchange' and e['operation']=='coord.goal' for e in events)==1
        assert events[-1]['operation']=='coord.stop' and events[-1]['success']
        (out/(name+'.stdout.jsonl')).write_text(result.stdout);(out/(name+'.stderr.jsonl')).write_text(result.stderr)
        checks.append(dict(case=name,rejected=True,prior_publication_count=1,clean_stop=True))
    # Fake transport inspects exact per-event context, then supplies an explicit
    # development answer or an authorized but semantically wrong middle answer.
    with tempfile.TemporaryDirectory() as temp:
        temp=Path(temp);curl=temp/'curl';responses={}
        for message in [inputs[0],inputs[1],inputs[2]]:
            s=message['selection'];responses[str(s['request_version'])]=dict(model='muse-spark-1.3',choices=[dict(finish_reason='stop',message=dict(role='assistant',content=json.dumps(s)))],usage=dict(prompt_tokens=10,completion_tokens=20,total_tokens=30))
        path=temp/'responses.json';counter=temp/'calls';path.write_text(json.dumps(responses))
        curl.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
from pathlib import Path
config=sys.stdin.read();assert 'MODEL_API_KEY' not in os.environ and 'dummy_session_key' in config
line=next(s for s in config.splitlines() if s.startswith('data-raw = '))
body=json.loads(json.loads(line.split(' = ',1)[1]))
user=json.loads(body['messages'][1]['content']);contextual=os.environ['RAI_CONTEXTUAL']=='1'
assert set(user)==({'request','catalog','observation'} if contextual else {'request','catalog'})
assert body['max_tokens']==1024 and body['response_format']['json_schema']['name']==('rai_context_selection' if contextual else 'rai_selection')
if contextual:
    expected=json.loads(Path(os.environ['RAI_CONTEXT_INPUTS']).read_text())[str(user['request']['request_version'])]
    assert user['observation']==expected['observation'] and user['request']['goal']==expected['goal']
assert len(user['catalog']['records'])==6 and 'expected' not in json.dumps(user)
assert 'restricted@1' not in json.dumps(user) and set(user['request'])=={'goal','request_version'}
v=str(user['request']['request_version']);p=Path(os.environ['RAI_SESSION_COUNTER'])
with p.open('a') as f:f.write(v+'\\n')
sys.stdout.write(json.dumps(json.loads(Path(os.environ['RAI_SESSION_RESPONSES']).read_text())[v])+'\\n200')
''');curl.chmod(0o700)
        context_inputs=temp/'contexts.json';context_inputs.write_text(json.dumps({str(i+1):dict(observation=load(fixture/'observations'/(case+'.json')),goal=load(fixture/(case+'.json'))['goal']) for i,case in enumerate(load(fixture/'protocol.json')['cases'])}) if contextual else '{}')
        env=dict(os.environ);os.environ.update(RAI_CONTEXTUAL='1' if contextual else '0',RAI_CONTEXT_INPUTS=str(context_inputs),PATH=str(temp)+os.pathsep+env['PATH'],MODEL_API_KEY='dummy_session_key',RAI_SESSION_COUNTER=str(counter),RAI_SESSION_RESPONSES=str(path))
        try:
            for name,wrong in [('offline-provider',False),('offline-wrong-meaning',True)]:
                if wrong:
                    s=copy.deepcopy(inputs[1]['selection']);s.update(decision='select',records=['notes@1'])
                    responses['2']['choices'][0]['message']['content']=json.dumps(s);path.write_text(json.dumps(responses))
                report,checked=run(binary,kernel,'meta',out/name,fixture=fixture,contextual=contextual)
                assert report['all_decisions_correct']==(not wrong) and checked['correct_decisions']==(2 if wrong else 3)
                assert checked['lifecycle']['total_jobs']==4 and checked['lifecycle']['previous_bytes_unchanged']
                checks.append(dict(case=name,transport='dummy_curl',audit=checked))
            assert counter.read_text().splitlines()==['1','2','3','1','2','3']
        finally:os.environ.clear();os.environ.update(env)
    (out/'checks.json').write_text(json.dumps(checks,indent=2)+'\n')
    print('Evidence:',out);print('PASS',profile,': replay/rules continuity, 5 corrupted traces, 8 invalid streams, 2 fake-provider sessions; wrong meaning preserves prior guest output')


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--graphics',action='store_true');a=p.parse_args();verify(a.debug,a.graphics)
