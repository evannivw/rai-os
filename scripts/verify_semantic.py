#!/usr/bin/env python3
"""Offline semantic gates, safe provider context and independently audited real QEMU work."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode=True
from semantic_demo import ROOT,FIXTURE,load,build,run_case,intent_gate,audit_execution


def verify(debug,graphics):
    binary,kernel=build(debug,graphics);profile='debug' if debug else 'release'
    output=ROOT/f'target/verification/semantic-{profile}{"-graphics" if graphics else ""}'
    output.mkdir(parents=True,exist_ok=True)
    output=Path(tempfile.mkdtemp(prefix="run-",dir=output))
    expected=load(FIXTURE/'expected.json');reports=[]
    cases=['historical'] if graphics else list(expected)
    for case in cases:
        report=run_case(binary,kernel,case,'replay',output/case,graphics);assert report['pass_case'],report;reports.append(report)
    case='historical';request=load(FIXTURE/(case+'.json'));good=load(output/case/'selection.json')
    # The oracle rejects authorized-but-wrong meaning, missing dependencies,
    # wrong decisions and substituted revisions, independently of Rust checks.
    for update in [dict(records=['brief@1','checklist@1','notes@2']),dict(records=['notes@1']),dict(decision='clarify',records=[]),dict(request_goal='different')]:
        bad=good|update
        try:intent_gate(bad,request,expected[case])
        except AssertionError:pass
        else:raise AssertionError('Oracle accepted wrong meaning')
    items=[json.loads(s) for s in (output/case/'execute.stdout.jsonl').read_text().splitlines()]
    for event,field,value in [('batch_publication','payload','{}'),('independent_check','passed',False),('semantic_execution','selection',good|dict(records=['empty@1']))]:
        bad=copy.deepcopy(items);next(e for e in bad if e['event']==event)[field]=value
        try:audit_execution(bad,request,good,expected[case])
        except (AssertionError,ValueError,KeyError):pass
        else:raise AssertionError('Audit accepted corrupt '+event)
    if not graphics:
        with tempfile.TemporaryDirectory(prefix='rai-semantic-test-') as temp:
            temp=Path(temp);selection=temp/'selection.json';logs=[]
            for update in [dict(request_version=2),dict(catalog_version=2),dict(permissions_version=2),dict(request_goal='different'),dict(records=['restricted@1']),dict(records=['notes@1','notes@2']),dict(records=[]),dict(decision='shell'),dict(decision='clarify'),dict(reason='bad\nreason')]:
                selection.write_text(json.dumps(good|update))
                for mode in ['--interpret-replay','--execute-selection']:
                    r=subprocess.run([str(binary),str(kernel),mode,str(FIXTURE/(case+'.json')),str(selection)],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=15)
                    assert r.returncode and 'guest_boot_confirmed' not in r.stdout,(mode,update,r.stdout)
                    logs.append(dict(mode=mode,update=update,stdout=r.stdout,stderr=r.stderr))
            (output/'invalid.json').write_text(json.dumps(logs,indent=2)+'\n')
            # Strict fake HTTPS transport: inspect EXACT context; no expected
            # decision/membership, test case ID or private document enters it.
            curl=temp/'curl';response=temp/'response.json';counter=temp/'count'
            curl.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
from pathlib import Path
config=sys.stdin.read();assert 'MODEL_API_KEY' not in os.environ
assert 'dummy_semantic_fixture_key' in config
line=next(s for s in config.splitlines() if s.startswith('data-raw = '))
body=json.loads(json.loads(line.split(' = ',1)[1]))
assert body['max_tokens']==1024 and body['response_format']['json_schema']['name']=='rai_selection'
user=json.loads(body['messages'][1]['content']);assert set(user)=={'request','catalog'}
assert set(user['request'])=={'request_version','goal'}
rows=user['catalog']['records'];assert len(rows)==6
assert all(r['key']!='restricted@1' and len(r['content'].encode())==r['bytes'] for r in rows)
assert 'expected' not in json.dumps(user) and 'rationale' not in json.dumps(user)
p=Path(os.environ['RAI_COUNTER']);p.write_text(str(int(p.read_text())+1) if p.exists() else '1')
sys.stdout.write(Path(os.environ['RAI_RESPONSE']).read_text()+'\\n200')
''');curl.chmod(0o700)
            env=dict(os.environ);os.environ.update(PATH=str(temp)+os.pathsep+env['PATH'],MODEL_API_KEY='dummy_semantic_fixture_key',RAI_COUNTER=str(counter),RAI_RESPONSE=str(response))
            try:
                for name,proposal,finish,passed in [('valid',good,'stop',True),('wrong-meaning',good|dict(records=['empty@1']),'stop',False),('stale',good|dict(permissions_version=2),'stop',False),('truncated',good,'length',False)]:
                    response.write_text(json.dumps(dict(model='muse-spark-1.3',choices=[dict(finish_reason=finish,message=dict(role='assistant',content=json.dumps(proposal)))],usage=dict(prompt_tokens=10,completion_tokens=20,total_tokens=30))))
                    report=run_case(binary,kernel,case,'meta',output/('offline-provider-'+name));report['source']='offline_curl_fixture'
                    assert report['pass_case']==passed,report
                    if not passed:assert not (output/('offline-provider-'+name)/'execute.stdout.jsonl').exists()
                    reports.append(report)
                assert counter.read_text()=='4'
            finally:os.environ.clear();os.environ.update(env)
    (output/'summary.json').write_text(json.dumps(reports,indent=2)+'\n')
    print(f'Evidence: {output}')
    print(f'PASS {profile} graphics={graphics}: {len(cases)} replay cases, independent intent/trace corruption checks'+('' if graphics else ', 20 pre-boot rejections and four offline provider cases'))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--debug',action='store_true');p.add_argument('--graphics',action='store_true');a=p.parse_args();verify(a.debug,a.graphics)
