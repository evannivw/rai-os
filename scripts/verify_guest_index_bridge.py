#!/usr/bin/env python3
"""Meta index transport: real QEMU with offline decisions and independent content/IPC audit."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from verify_guest_bridge import ROOT, audit


def verify(release, graphics=False):
    profile = 'release' if release else 'debug'; flags = ['--release'] if release else []
    host = next(line[6:] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
    build = ['cargo','build','--locked','--bin','rai-os',*flags]
    directory = 'target/display' if graphics else 'target'
    if graphics: build += ['--features','guest-display','--target-dir',directory]
    subprocess.run(build,cwd=ROOT,check=True)
    subprocess.run(['cargo','build','--locked','--manifest-path','host/meta/Cargo.toml','--target',host,'--target-dir','target/host-meta',*flags],cwd=ROOT,check=True)
    binary = ROOT/f'target/host-meta/{host}/{profile}/rai-meta'
    kernel = ROOT/f'{directory}/aarch64-unknown-none-softfloat/{profile}/rai-os'
    (ROOT/'target/verification').mkdir(parents=True,exist_ok=True)
    def proposal(version,key,delay=10,operation='record.index'):
        return dict(delay_ms=delay,proposal=dict(state_version=version,operation=operation,record=key,reason='Offline index fixture; not model inference.'))
    cases = [
        ('reuse',[(0,'notes@1'),(400,'notes@1'),(1000,'notes@2'),(1800,'notes@1'),(2200,'notes@1')], [proposal(1,'notes@1'),proposal(3,'notes@2')],(['notes@1','notes@2'],0,2)),
        ('stale',[(0,'notes@1'),(150,'notes@2')],[proposal(1,'notes@1',400),proposal(2,'notes@2')],(['notes@2'],1,2)),
        ('reuse-during-inference',[(0,'notes@1'),(500,'notes@2'),(650,'notes@1')],[proposal(1,'notes@1'),proposal(2,'notes@2',400)],(['notes@1'],1,2)),
        ('obsolete-decline',[(0,'notes@1'),(150,'notes@2')],[proposal(1,'',400,'decline'),proposal(2,'notes@2')],(['notes@2'],0,2)),
        ('decline',[(0,'notes@1')],[proposal(1,'',10,'decline')],([],0,1)),
        ('empty',[(0,'empty@1'),(500,'appendix@1'),(1000,'empty@1')],[proposal(1,'empty@1'),proposal(2,'appendix@1')],(['empty@1','appendix@1'],0,2)),
    ]
    if graphics: cases = cases[:1]
    with tempfile.TemporaryDirectory(prefix='rai-meta-index-') as temp:
        temp=Path(temp); events=temp/'events.json'; replies=temp/'replies.json'
        env=dict(os.environ,MODEL_API_KEY='')
        def run(kernel_path=kernel, mode='--guest-index-replay'):
            return subprocess.run([str(binary),str(kernel_path),*(['--graphics'] if graphics else []),mode,str(events),str(replies)],cwd=ROOT,env=env,text=True,capture_output=True,timeout=25)
        for name,schedule,decisions,expected in cases:
            scenario=dict(events=[dict(at_ms=t,record=k) for t,k in schedule])
            events.write_text(json.dumps(scenario));replies.write_text(json.dumps(decisions))
            result=run();assert result.returncode==0,(name,result.stderr,result.stdout)
            items=[json.loads(line) for line in result.stdout.splitlines()]
            assert audit(items,scenario,'replay',index=True)==expected
            assert items[0]['graphics'] == graphics
            (ROOT/f'target/verification/guest-index-{name}-{profile}{"-graphics" if graphics else ""}.jsonl').write_text(result.stdout)
            if name=='reuse':
                for event,field,value in [('decision_requested','snapshot',{}),('guest_bridge_finished','model_calls',3),('guest_index','ticket',99),('guest_index','payload','invented')]:
                    bad=copy.deepcopy(items);next(i for i in bad if i['event']==event)[field]=value
                    try: audit(bad,scenario,'replay',index=True)
                    except (AssertionError,KeyError,ValueError): pass
                    else: raise AssertionError('Accepted forged '+field)
                for old,new in [('pump:2','pump:1'),('notes@1','notes@2')]:
                    bad=copy.deepcopy(items)
                    entry=next(i for i in bad if i['event']=='guest_exchange' and i['operation']=='coord.result')
                    entry['payload']=entry['payload'].replace(old,new)
                    try: audit(bad,scenario,'replay',index=True)
                    except (AssertionError,KeyError,ValueError): pass
                    else: raise AssertionError('Accepted corrupt artifact')
                bad=[i for i in items if i['event']!='guest_index']
                try: audit(bad,scenario,'replay',index=True)
                except AssertionError: pass
                else: raise AssertionError('Accepted missing artifact evidence')
        if graphics:
            result=run(ROOT/f'target/aarch64-unknown-none-softfloat/{profile}/rai-os')
            assert result.returncode and json.loads(result.stderr)['error']=='guest_display_not_ready',result.stderr
            assert 'guest_boot_confirmed' not in result.stdout
            assert 'decision_requested' not in result.stdout
            print(f'PASS {profile}: real Cocoa display, confirmed guest framebuffer, Meta-index replay, missing-feature rejection and cleanup')
            return
        scenario=dict(events=[dict(at_ms=0,record='notes@1')]);events.write_text(json.dumps(scenario))
        for decision,error,forwarded in [
            (proposal(0,'notes@1'),'stale_proposal',False),
            (proposal(1,'restricted@1'),'record_not_allowed',False),
            (proposal(1,'notes@1\ncoord.stop'),'record_not_allowed',False),
            (proposal(1,'notes@1',operation='record.count'),'operation_not_allowed',False),
            (proposal(1,'notes@1',operation='shell'),'operation_not_allowed',False),
            (proposal(1,'notes@2'),'coord_wrong_record',True),
        ]:
            replies.write_text(json.dumps([decision]));result=run()
            assert result.returncode and json.loads(result.stderr)['error']==error,result.stderr
            items=[json.loads(line) for line in result.stdout.splitlines()]
            assert any(i['event']=='proposal_forwarded' for i in items)==forwarded
            assert not any(i['event']=='guest_index' for i in items)
            assert items[-1]['operation']=='coord.stop' and items[-1]['success']
        # A model proposing an index cannot widen the old count-only bridge.
        replies.write_text(json.dumps([proposal(1,'notes@1')]))
        result=run(mode='--guest-replay')
        assert result.returncode and json.loads(result.stderr)['error']=='operation_not_allowed'
        # Four distinct jobs exhaust the budget; a fifth is never requested.
        keys=['notes@1','notes@2','brief@1','checklist@1','appendix@1']
        events.write_text(json.dumps(dict(events=[dict(at_ms=i*500,record=k) for i,k in enumerate(keys)])))
        replies.write_text(json.dumps([proposal(i+1,k) for i,k in enumerate(keys[:4])]))
        result=run();assert result.returncode and json.loads(result.stderr)['error']=='model_call_budget_exhausted'
        items=[json.loads(line) for line in result.stdout.splitlines()]
        assert sum(i['event']=='decision_requested' for i in items)==4
        assert sum(i['event']=='guest_index' for i in items)==4
        assert items[-1]['operation']=='coord.stop' and items[-1]['success']
        # Guest stream failure is not an artifact; cleanup remains explicit.
        directory=ROOT/'target/coord-index-self-test/index-stream-test-truncated'
        subprocess.run(['cargo','build','--locked','--bin','rai-os','--features','index-stream-test-truncated','--target-dir',str(directory),*flags],cwd=ROOT,check=True)
        events.write_text(json.dumps(dict(events=[dict(at_ms=0,record='notes@2')])));replies.write_text(json.dumps([proposal(1,'notes@2')]))
        result=run(directory/f'aarch64-unknown-none-softfloat/{profile}/rai-os')
        assert result.returncode and json.loads(result.stderr)['error']=='guest_session_failed', result.stderr
        items=[json.loads(line) for line in result.stdout.splitlines()]
        assert not any(i['event']=='guest_index' for i in items)
        assert items[-1]['operation']=='coord.stop' and items[-1]['success']
    print(f'PASS {profile}: exact indexes, reuse without inference, stale/declined/invalid proposals, budgets, failure cleanup and independent artifact/IPC audit')


def main():
    p=argparse.ArgumentParser();p.add_argument('--release',action='store_true');p.add_argument('--graphics',action='store_true')
    p.add_argument('--check-live',type=Path);p.add_argument('--scenario',type=Path,default=ROOT/'fixtures/guest_index/v1/observations.json')
    args=p.parse_args()
    if args.check_live:
        result=audit([json.loads(s) for s in args.check_live.read_text().splitlines()],json.loads(args.scenario.read_text()),'meta',index=True)
        print('PASS live index log audit; executed records/stale/model calls:',result)
        print('Saved logs are not cryptographic proof of provider access.')
    else: verify(args.release,args.graphics)


if __name__=='__main__': main()
