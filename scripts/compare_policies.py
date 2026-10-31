#!/usr/bin/env python3
"""Sequential development pilot; real guest outputs and explicit simulated work credits."""
import argparse
import datetime
import hashlib
import json
import platform
import os
import signal
from pathlib import Path
import statistics
import subprocess
import sys
sys.dont_write_bytecode=True
from verify_pilot import ROOT, build, audit

SCENARIOS=['steady','priority-change','revision-quota']


def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()


def report(output,manifest,rows):
    text=['# RAI-OS policy pilot results','',
          f"Source commit: `{manifest['commit']}`. Started: {manifest['started_utc']}.",
          'Development scenarios; this small sample is not a held-out or statistically powered evaluation.',
          'Real QEMU indexes and wall times; credits are host-simulated accounting, not battery or CPU measurements.','',
          '| Scenario | Policy | Passed | On-time score range | Median completion ms (passed only) | Credits range | Decisions | Reported model tokens |',
          '| --- | --- | --- | --- | --- | --- | --- | --- |']
    for name in SCENARIOS:
        for policy in manifest['policies']:
            group=[r for r in rows if r['scenario']==name and r['policy']==policy]
            valid=[r for r in group if not r.get('audit_error')]
            done=[r['completion_ms'] for r in valid if r['passed']]
            span=lambda key: f"{min(r[key] for r in valid)}–{max(r[key] for r in valid)}" if valid else 'unverified'
            text.append(f"| {name} | {policy} | {len(done)}/{len(group)} | {span('weighted_on_time')} | {statistics.median(done) if done else '—'} | {span('spent_credits')} | {sum(r['decisions'] for r in valid)} | {sum(r['reported_tokens'] for r in valid)} |")
    text += ['', 'All trials, including failed and unauditable ones, remain in results.json and raw JSONL/stderr files.',
             'Completion includes boot, decisions/inference, UART and independent artifact checking. Build time and the later evidence audit are outside each trial. Provider draining/cleanup is in total_elapsed_ms.',
             'The native display is disabled for every timed trial. No simultaneous QEMU competitors.']
    if 'meta' in manifest['policies']:
        wins=ties=losses=0;unverified=0
        for rep in range(manifest['repetitions']):
            for name in SCENARIOS:
                group=[r for r in rows if r['scenario']==name and r['repetition']==rep]
                if any(r.get('audit_error') for r in group):unverified+=1;continue
                rank=lambda r: (r['passed'],r['weighted_on_time'],-r['completion_ms'],-r['spent_credits']) if r['passed'] else (False,0,0,0)
                ai=rank(next(r for r in group if r['policy']=='meta'))
                fixed=max(rank(r) for r in group if r['policy'] in ['edf','value'])
                wins+=ai>fixed;ties+=ai==fixed;losses+=ai<fixed
        text += ['',f'Meta versus the stronger fixed result for each paired scenario/repetition: {wins} wins, {ties} ties, {losses} losses, {unverified} unauditable pairs.',
                 'Ranking: valid final completion first, then weighted on-time score, then completion time, then credits. This is not a comparison with Android or evidence of general OS superiority.']
    text += ['', '## Per-trial outcomes','', '| Trial | Passed | Reason | Completion ms | Score | Credits | Stale choices |', '| --- | --- | --- | --- | --- | --- | --- |']
    for r in rows:text.append(f"| {r['trial']} | {r['passed']} | {r.get('audit_error') or r.get('reason') or 'verified'} | {r.get('completion_ms') or '—'} | {r.get('weighted_on_time',0)} | {r.get('spent_credits','—')} | {r.get('stale','—')} |")
    (output/'REPORT.md').write_text('\n'.join(text)+'\n')


def check_saved(output):
    manifest=json.loads((output/'manifest.json').read_text());saved=json.loads((output/'results.json').read_text())
    for entry in manifest['scenarios']:
        assert digest(ROOT/entry['path'])==entry['sha256'],'Scenario changed since this measurement'
    for row in saved:
        if row.get('audit_error'):raise AssertionError('Recorded unauditable trial '+row['trial'])
        scenario=json.loads((ROOT/f'fixtures/policy_pilot/v1/{row["scenario"]}.json').read_text())
        fresh=audit([json.loads(s) for s in (output/(row['trial']+'.jsonl')).read_text().splitlines()],scenario,row['policy'])
        assert row==dict(fresh,trial=row['trial'],repetition=row['repetition']),row['trial']
    assert len(saved)==len(SCENARIOS)*len(manifest['policies'])*manifest['repetitions']
    assert len({r['trial'] for r in saved})==len(saved)
    print(f'PASS independent re-audit: {len(saved)} retained trials')


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--live',action='store_true',help='Include actual paid Meta requests using the local credential')
    p.add_argument('--output',type=Path);p.add_argument('--repetitions',type=int,default=2,choices=[1,2,3]);p.add_argument('--audit',type=Path)
    args=p.parse_args()
    if args.audit:check_saved(args.audit);return
    if not args.output:p.error('--output must name a new directory')
    if subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip():p.error('Commit the frozen protocol/code before a measurement; the checkout must be clean')
    binary,kernel=build(True)
    output=args.output.resolve();output.mkdir(parents=True,exist_ok=False)
    policies=['edf','value','meta'] if args.live else ['edf','value']
    manifest=dict(format='rai-policy-pilot-v1',started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),host=platform.platform(),qemu=subprocess.check_output(['qemu-system-aarch64','--version'],text=True).splitlines()[0],policies=policies,repetitions=args.repetitions,kernel_sha256=digest(kernel),bridge_sha256=digest(binary),provider_source_sha256=digest(ROOT/'host/meta/src/provider.rs'),scenarios=[dict(path=f'fixtures/policy_pilot/v1/{name}.json',sha256=digest(ROOT/f'fixtures/policy_pilot/v1/{name}.json')) for name in SCENARIOS],trials=[])
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    rows=[]
    for rep in range(args.repetitions):
        for n,name in enumerate(SCENARIOS):
            shift=(rep+n)%len(policies);order=policies[shift:]+policies[:shift]
            path=ROOT/f'fixtures/policy_pilot/v1/{name}.json';scenario=json.loads(path.read_text())
            for policy in order:
                trial=f'{rep+1}-{name}-{policy}';command=[str(binary),str(kernel),'--pilot',policy,str(path)]
                assert not subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip(),'Checkout changed during comparison'
                assert digest(kernel)==manifest['kernel_sha256'] and digest(binary)==manifest['bridge_sha256'],'Binary changed during comparison'
                record=dict(trial=trial,command=command);manifest['trials'].append(record)
                try:
                    child=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=True)
                    try:
                        stdout,stderr=child.communicate(timeout=90)
                    except subprocess.TimeoutExpired:
                        # The trial owns this fresh process group, including QEMU/curl.
                        # Kill the group, not only its Rust parent, and retain output.
                        try: os.killpg(child.pid,signal.SIGKILL)
                        except ProcessLookupError: pass
                        stdout,stderr=child.communicate()
                        (output/(trial+'.jsonl')).write_text(stdout)
                        (output/(trial+'.stderr')).write_text(stderr)
                        raise AssertionError('trial_timeout')
                    except BaseException:
                        try: os.killpg(child.pid,signal.SIGKILL)
                        except ProcessLookupError: pass
                        stdout,stderr=child.communicate()
                        (output/(trial+'.jsonl')).write_text(stdout)
                        (output/(trial+'.stderr')).write_text(stderr)
                        raise
                    (output/(trial+'.jsonl')).write_text(stdout);(output/(trial+'.stderr')).write_text(stderr)
                    record['exit_code']=child.returncode
                    assert child.returncode==0,'bridge process failed'
                    items=[json.loads(s) for s in stdout.splitlines()]
                    row=audit(items,scenario,policy)
                    for i in items:
                        if i['event']=='pilot_verified_bundle':
                            (output/f'{trial}-goal-{i["snapshot"]["goal_version"]}.json').write_text(i['payload']+'\n')
                except (AssertionError,ValueError,KeyError,TypeError) as error:
                    row=dict(scenario=name,policy=policy,passed=False,audit_error=type(error).__name__+': '+str(error)[:240])
                row.update(trial=trial,repetition=rep);rows.append(row)
                (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
                (output/'results.json').write_text(json.dumps(rows,indent=2)+'\n')
                print(f'{trial}: passed={row["passed"]}; completion_ms={row.get("completion_ms")}; score={row.get("weighted_on_time")}; credits={row.get("spent_credits")}; error={row.get("audit_error") or row.get("reason")}',flush=True)
    report(output,manifest,rows)
    print('Report:',output/'REPORT.md')
    if any(r.get('audit_error') for r in rows):raise SystemExit('Retained one or more unauditable trials; do not claim a valid comparison')
    check_saved(output)


if __name__=='__main__':main()
