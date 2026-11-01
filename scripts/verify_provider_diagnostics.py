#!/usr/bin/env python3
"""Offline curl fixtures exercise safe diagnostics and real-QEMU admission/cleanup."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode=True
from verify_mmu import ROOT, OUTPUT
from check_bundle import check
from verify_meta import check_capture

SENTINEL='PRIVATE_RESPONSE_MUST_NOT_BE_LOGGED'
DUMMY='dummy_diagnostic_test_key_not_real'


def base(operation='record.count'):
    proposal=dict(state_version=1,operation=operation,record='empty@1',reason='Explicit offline provider fixture.')
    return dict(model='muse-spark-1.3', choices=[dict(finish_reason='stop', message=dict(role='assistant', content=json.dumps(proposal)))],
                usage=dict(prompt_tokens=10,completion_tokens=20,total_tokens=30))


def verify(release):
    profile='release' if release else 'debug';flags=['--release'] if release else []
    host=next(s[6:] for s in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if s.startswith('host: '))
    subprocess.run(['cargo','build','--locked','--bin','rai-os',*flags],cwd=ROOT,check=True)
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    binary=ROOT/f'target/host-meta/{host}/{profile}/rai-meta'
    kernel=ROOT/f'target/aarch64-unknown-none-softfloat/{profile}/rai-os'
    records=[]
    with tempfile.TemporaryDirectory(prefix='rai-provider-report-') as temp:
        temp=Path(temp);counter=temp/'calls'; response=temp/'response.json';scenario=temp/'scenario.json'
        scenario.write_text(json.dumps({'events':[{'at_ms':0,'required':['empty@1']}]}))
        curl=temp/'curl'
        curl.write_text('#!'+sys.executable+'\n'+'''import json,os,sys,time
from pathlib import Path
config=sys.stdin.read()
assert "MODEL_API_KEY" not in os.environ
assert "dummy_diagnostic_test_key_not_real" in config
assert sys.argv[1]=="-q" and "--location" not in sys.argv and "--retry" not in sys.argv
p=Path(os.environ['RAI_TEST_CALLS']);p.write_text(str(int(p.read_text())+1) if p.exists() else '1')
time.sleep(.01)
f=json.loads(Path(os.environ['RAI_TEST_RESPONSE']).read_text())
sys.stderr.write('PRIVATE_RESPONSE_MUST_NOT_BE_LOGGED')
sys.stdout.write(f['body']+'\\n'+f['http'])
raise SystemExit(f['exit'])
''');curl.chmod(0o700)
        env=dict(os.environ,PATH=str(temp)+os.pathsep+os.environ['PATH'],MODEL_API_KEY=DUMMY,
                 RAI_TEST_CALLS=str(counter),RAI_TEST_RESPONSE=str(response))
        def run(label,body,error=None,http='200',exit_code=0,asynchronous=False,decoded=False):
            response.write_text(json.dumps(dict(body=body if isinstance(body,str) else json.dumps(body),http=http,exit=exit_code)))
            before=int(counter.read_text()) if counter.exists() else 0
            args=[str(binary),str(kernel)]
            args += ['--guest-set',str(scenario)] if asynchronous else ['Count the empty record.']
            result=subprocess.run(args,cwd=ROOT,env=env,text=True,capture_output=True,timeout=25)
            assert SENTINEL not in result.stdout+result.stderr and DUMMY not in result.stdout+result.stderr
            assert int(counter.read_text())==before+1,'Unexpected retry'
            out=[json.loads(l) for l in result.stdout.splitlines() if l]
            err=[json.loads(l) for l in result.stderr.splitlines() if l]
            reports=[e for e in out if e['event']=='provider_diagnostic'];assert len(reports)==1,out
            report=reports[0];d=report['diagnostic']
            assert report['source']=='meta' and report['state_version']==1 and d['latency_ms']>=10
            assert report['outcome']==('decoded' if not error or decoded else 'rejected')
            if error:
                assert result.returncode!=0 and err[-1]['error']==error,(label,result.stdout,result.stderr)
                assert report['error']==(None if decoded else error)
                assert not any(e['event'] in ['guest_result','guest_index','proposal_forwarded'] for e in out)
                if asynchronous:
                    exchanges=[e for e in out if e['event']=='guest_exchange']
                    assert exchanges[-1]['operation']=='coord.stop' and exchanges[-1]['success']
                    assert not any(e['operation'] in ['coord.select','coord.propose'] for e in exchanges)
                else: assert not any(e['event']=='guest_boot_confirmed' for e in out)
            else:
                assert result.returncode==0,result.stderr
                if asynchronous:
                    bundles=[e for e in out if e['event']=='guest_bundle'];assert len(bundles)==1
                    check(bundles[0]['payload'],['empty@1'],1)
                else: check_capture(out,'meta')
            records.append(dict(case=label,transport='offline_curl_fixture',stdout=out,stderr=err))
            return d
        run('valid-count',base())
        for finish,code in [('length','meta_finish_length'),('content_filter','meta_finish_content_filter'),
                            ('tool_calls','meta_finish_tool_calls'),('function_call','meta_finish_tool_calls'),
                            (SENTINEL,'meta_finish_other')]:
            v=base();v['choices'][0]['finish_reason']=finish
            d=run('finish-'+code,v,code);assert d['usage']['total_tokens']==30
        for field,code in [('refusal','meta_refusal_present'),('tool_calls','meta_tool_calls_present')]:
            v=base();v['choices'][0]['message'][field]=SENTINEL
            assert run(field,v,code)['usage']['total_tokens']==30
        v=base();v['choices'][0]['message']['role']=SENTINEL
        assert run('wrong-role',v,'meta_unexpected_role')['role']=='other'
        v=base();v['model']=SENTINEL
        assert run('wrong-model',v,'unexpected_meta_model')['model_matches'] is False
        v=base();v['choices']=[];run('no-choice',v,'unexpected_meta_choices')
        v=base();v['choices'].append(copy.deepcopy(v['choices'][0]));run('multiple-choices',v,'unexpected_meta_choices')
        v=base();v['choices'][0]['message']['content']=None;run('missing-content',v,'missing_meta_content')
        v=base();v['choices'][0]['message']['content']=SENTINEL
        assert run('invalid-proposal',v,'invalid_proposal_json')['stage']=='proposal'
        assert run('invalid-envelope',SENTINEL,'invalid_meta_response')['usage'] is None
        run('duplicate-envelope',json.dumps(base()).replace('{','{"model":"muse-spark-1.3",',1),'invalid_meta_response')
        v=base();v['usage']=None;assert run('unknown-usage',v)['usage'] is None
        for status in ['401','429','500']:
            d=run('http-'+status,SENTINEL,'meta_http_'+status,http=status)
            assert d['http_status']==int(status) and d['usage'] is None and d['stage']=='http'
        for code,error in [(28,'meta_timeout'),(7,'meta_transport_failed')]:
            d=run('curl-'+str(code),SENTINEL,error,exit_code=code)
            assert d['transport_exit_code']==code and d['http_status'] is None and d['stage']=='transport'
        v=base();v['choices'][0]['message']['content']=json.dumps(dict(state_version=1,operation='shell',record='',reason='Unsupported fixture.'))
        run('decoded-is-not-authorized',v,'operation_not_allowed',decoded=True)
        run('async-success',base('record.index'),asynchronous=True)
        v=base('record.index');v['choices'][0]['finish_reason']='length'
        assert run('async-rejection',v,'meta_finish_length',asynchronous=True)['usage']['total_tokens']==30
    OUTPUT.mkdir(parents=True,exist_ok=True)
    (OUTPUT/f'provider-diagnostics-{profile}.json').write_text(json.dumps(records,indent=2)+'\n')
    print(f'PASS {profile}: {len(records)} OFFLINE provider cases; safe diagnostics, no retries, real-QEMU admission/cleanup')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--release',action='store_true')
    verify(parser.parse_args().release)
