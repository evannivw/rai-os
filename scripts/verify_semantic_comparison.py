#!/usr/bin/env python3
"""Offline resolver, comparison audit and same-session playback verification."""
import sys,os,json,tempfile,copy,subprocess
from pathlib import Path
sys.dont_write_bytecode=True;sys.path.insert(0,'scripts')
from semantic_demo import build,run_case
from semantic_comparison import FIXTURE,audit_trial,schedule_for,summarize
from semantic_sequence import audit
from verify_mmu import ROOT
import argparse
p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');args=p.parse_args()
profile='debug' if args.debug else 'release';binary,kernel=build(args.debug);base=ROOT/'target/verification';base.mkdir(exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='comparison-'+profile+'-',dir=base))
sequence=out/'sequence'
subprocess.run([sys.executable,'-B','scripts/semantic_sequence.py',*(['--debug'] if args.debug else []),'--output',str(sequence)],cwd=ROOT,check=True)
expected=json.loads((FIXTURE/'expected.json').read_text())
for name in expected:
 report=run_case(binary,kernel,name,'rules',out/name,fixture=FIXTURE)
 assert report['pass_case'],report
 assert audit_trial(out/name,name,'rules')==report
# Audit must reject corrupted success claims and changed selections.
p=out/'explicit/report.json';orig=p.read_text();v=json.loads(orig);v['accepted']=False;p.write_text(json.dumps(v))
try:audit_trial(out/'explicit','explicit','rules')
except AssertionError:pass
else:raise AssertionError('Accepted corrupt report')
p.write_text(orig)
trace=[json.loads(s) for s in (sequence/'stdout.jsonl').read_text().splitlines()]
for event,field,value in [('sequence_historical','unchanged',False),('batch_publication','payload','{}'),('sequence_finished','snapshot',{})]:
 bad=copy.deepcopy(trace);next(e for e in bad if e['event']==event)[field]=value
 try:audit(bad)
 except (AssertionError,KeyError,ValueError):pass
 else:raise AssertionError('Accepted corrupt sequence')
# Whole sequence validation occurs before boot, including a bad second entry.
entries=json.loads((sequence/'approved.json').read_text());entries[1]['selection']['permissions_version']=2
bad=out/'invalid-sequence.json';bad.write_text(json.dumps(entries))
r=subprocess.run([str(binary),str(kernel),'--sequence-replay',str(bad)],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=15)
assert r.returncode and not r.stdout
protocol=json.loads((FIXTURE/'protocol.json').read_text());schedule=schedule_for(protocol)
assert len(schedule)==48 and sum(e['policy']=='meta' for e in schedule)==24
for i in range(0,48,2):assert {e['policy'] for e in schedule[i:i+2]}=={'meta','rules'}
(out/'manifest.json').write_text(json.dumps(dict(protocol=protocol,cases=protocol['cases'],schedule=schedule[:-1])))
try:summarize(out)
except AssertionError:pass
else:raise AssertionError('Accepted omitted comparison trial')
print('Evidence:',out)
print('PASS',profile,': 12 deterministic cases with real QEMU, corrupt comparison/sequence rejection, bad suffix rejected before boot')
