#!/usr/bin/env python3
"""Real Git + QEMU verification. Scripted choices offline; capped Meta calls live."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
from apps import Studio,ROOT,build,host_build,deliver,schema,records
from app_storage import collect
from verify_apps import submit
from verify_app_behaviors import field,step


def definition(amount):
    return dict(id='versiontool',title='Versioned tool',capability='records.local',filter_key='',filter_label='',
        fields=[field('name','text'),field('count','number','0')],behaviors=[dict(id='advance',label='Advance all',event='press',scope='all',steps=[step('calculate','count',f'$count+{amount}')])])

def choice(operation,revision='',message=''):
    return dict(operation=operation,revision=revision,message=message,reason='Explicit verifier Git operation.')

class ScriptedGit(Studio):
    def __init__(self,*args,**kwargs):
        super().__init__(*args,source='replay',**kwargs);self.selection=None;self.after=None
    def _git_plan_attempt(self,request,request_path,folder):
        assert self.selection is not None,'No implicit Git tool fixture'
        path=folder/'supplied-choice.json';path.write_text(json.dumps(self.selection)+'\n')
        env=dict(os.environ);env.pop('MODEL_API_KEY',None);env.pop('RAI_VNC_PASSWORD',None)
        result=subprocess.run([str(self.binary),str(self.kernel),'--app-git-plan-replay',str(self.git_store),str(request_path),str(path)],cwd=ROOT,env=env,capture_output=True,text=True,timeout=15)
        if self.after:self.after(self,request,folder,result)
        return result

def verify(kernel,binary,output,source='offline'):
    try:
        commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True,stderr=subprocess.DEVNULL).strip()
        dirty=bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip())
    except (FileNotFoundError,subprocess.CalledProcessError):commit=dirty=None
    if source=='meta':assert commit and not dirty,'Freeze sources before live Git operations'
    output.mkdir(parents=True,exist_ok=False)
    report=dict(verified=False,source=source,source_commit=commit,source_worktree_dirty=dirty,
                maximum_live_calls=3 if source=='meta' else 0,generation_calls=0,checks=[],operations=[])
    checkpoint=output/'device.json';store=output/'history';repo=store/'versiontool.git';app=None
    def git(*args):
        return subprocess.check_output(['git','--no-pager','--git-dir',str(repo),*args],text=True).strip()
    try:
        app=Studio(kernel,binary,output/'session','meta',state=checkpoint,git_store=store) if source=='meta' else ScriptedGit(kernel,binary,output/'session',state=checkpoint,git_store=store)
        g=app.guest
        def approve(spec):
            old=g.request('app.catalog');assert g.request('app.key','0')=='state=review';app.press(13)
            for _ in spec['behaviors']:
                assert g.request('app.key','0')=='state=behavior_review' and g.request('app.catalog')==old;app.press(13)
            assert schema(g,spec['id'])==spec
        def install(spec,target=None):
            p=submit(g,'Disclosed schema for Git verification',target);p['spec']=spec;deliver(g,p);approve(spec)
        def request(goal,selected=None,expected='completed'):
            if source=='offline':app.selection=selected
            g.request('app.open','versiontool');app.press(1002)
            for char in goal:app.press(ord(char))
            reply=dict(p.split('=',1) for p in app.press(13).split('; '))
            for thread in app.threads:thread.join(timeout=45);assert not thread.is_alive()
            folder=app.output/reply['submit'];trace=json.loads((folder/'git-loop.json').read_text())
            report['generation_calls']=app.provider_calls if source=='meta' else 0
            report['operations'].append(dict(goal=goal,trace=trace))
            assert trace['state']==expected,trace
            p=folder/'git.stdout.jsonl'
            events=[json.loads(line) for line in p.read_text().splitlines()] if p.exists() else []
            return folder,events
        first=definition(1);second=definition(10);install(first)
        for i,(name,count) in enumerate([('LOCAL_ONLY_A','5'),('LOCAL_ONLY_B','9')]):
            if i:app.press(110)
            g.request('app.value',f'0:{name.encode().hex()}');g.request('app.value',f'1:{count.encode().hex()}');app.press(13)
        original=records(g,'versiontool')
        request('git: save this definition as version one',choice('save',message='Version one'))
        one=git('rev-parse','refs/heads/versions');assert len(one)==40
        assert 'LOCAL_ONLY' not in git('show',one+':app.json')
        assert git('ls-tree','--name-only',one)=='app.json'
        install(second,'versiontool');assert records(g,'versiontool')==original
        def run():
            g.request('app.open','versiontool');app.press(1003);app.press(13);app.press(1014);app.tap(250,265);app.tap(240,720)
        run();expected=[['LOCAL_ONLY_A','15'],['LOCAL_ONLY_B','19']];assert records(g,'versiontool')==expected
        request('git: save this changed definition as version two',choice('save',message='Version two'))
        two=git('rev-parse','refs/heads/versions');assert two!=one and git('rev-parse',two+'^')==one
        if source=='offline':
            _,events=request('git: list saved versions',choice('history'))
            result=next(e for e in events if e['event']=='app_git_result')
            assert [v['oid'] for v in result['detail']['history']]==[two,one]
            _,events=request('git: compare the first version with the current definition',choice('diff',one))
            difference=next(e for e in events if e['event']=='app_git_result')['detail']['diff']
            assert '$count+1' in difference and '$count+10' in difference
            assert git('rev-parse','refs/heads/versions')==two
        folder,events=request('git: restore the previous saved version, keeping current records',choice('restore',one),'review')
        proposed=next(e['plan'] for e in events if e['event']=='app_plan')
        assert proposed['spec']==first and records(g,'versiontool')==expected and schema(g,'versiontool')==second
        g.capture(output/'01-restore-review.ppm');approve(first)
        assert records(g,'versiontool')==expected and git('rev-parse','refs/heads/versions')==two
        run();expected=[['LOCAL_ONLY_A','16'],['LOCAL_ONLY_B','20']];assert records(g,'versiontool')==expected
        app.press(1007);g.capture(output/'02-restored-behavior.ppm')
        report['checks'].append('Real commits contain definitions only; restore requires native approval, preserves records and Git history, and executes the old program over current data')
        if source=='offline':
            app.after=lambda a,r,f,result:a.press(27)
            request('git: save current definition',choice('save',message='Cancelled save'),'cancelled_or_stale')
            app.after=None;assert git('rev-parse','refs/heads/versions')==two
            request('git: save the restored definition as a new version',choice('save',message='Restored program'))
            three=git('rev-parse','refs/heads/versions');assert three!=two and git('rev-parse',three+'^')==two
            # The planned tool must not run against a newer Git HEAD.
            def advance_head(a,r,folder,result):
                other=copy.deepcopy(r);other['target']['title']='Concurrent version'
                rp=folder/'concurrent-request.json';rp.write_text(json.dumps(other)+'\n')
                cp=folder/'concurrent-choice.json';cp.write_text(json.dumps(choice('save',message='Concurrent writer'))+'\n')
                env=dict(os.environ);env.pop('MODEL_API_KEY',None)
                command=[str(binary),str(kernel),'--app-git-plan-replay',str(store),str(rp),str(cp)]
                planned=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=15);assert planned.returncode==0,planned.stderr
                pending=next(json.loads(line)['pending'] for line in planned.stdout.splitlines() if json.loads(line)['event']=='app_git_plan')
                pp=folder/'concurrent-pending.json';pp.write_text(json.dumps(pending)+'\n')
                applied=subprocess.run([str(binary),str(kernel),'--app-git-apply',str(store),str(rp),str(pp)],cwd=ROOT,env=env,capture_output=True,text=True,timeout=25)
                assert applied.returncode==0,applied.stderr
            app.after=advance_head
            failed,_=request('git: list history',choice('history'),'failed');app.after=None
            assert 'stale_app_git_head' in (failed/'git.stderr.jsonl').read_text()
            assert schema(g,'versiontool')==first and records(g,'versiontool')==expected
            assert app.provider_calls==8
            # Dispatcher refuses a ninth provider invocation in this session.
            g.request('app.open','versiontool');app.press(1002)
            for ch in 'git: history':app.press(ord(ch))
            app.press(13);assert app.provider_calls==8 and g.request('app.key','0')=='state=notice'
            report['checks'].append('History/diff use real Git; canceled saves do not commit; saved rollback appends history; changed Git HEAD rejects pending action; ninth provider call denied')
        final=collect(g,schema,records);report['final_state']=final
        report['git_history']=git('log','--format=%H %s','refs/heads/versions').splitlines()
        app.close();app=None
        app=Studio(kernel,binary,output/'restored','replay',state=checkpoint,git_store=store)
        assert collect(app.guest,schema,records)==final and not app.threads
        app.guest.request('app.open','versiontool');app.press(1007);app.guest.capture(output/'03-reboot.ppm')
        report['checks'].append('Current records and restored definition recover after a fresh QEMU boot without inference')
        if source=='offline':
            app.close();app=None
            app=ScriptedGit(kernel,binary,output/'incompatible',state=checkpoint,git_store=store);g=app.guest
            grown=copy.deepcopy(first);grown['fields'].append(field('note','text','kept'));install(grown,'versiontool')
            before=collect(g,schema,records);head=git('rev-parse','refs/heads/versions')
            failure,_=request('git: restore version one',choice('restore',one),'failed')
            assert 'app_destructive_update' in (failure/'git.stderr.jsonl').read_text()
            assert collect(g,schema,records)==before and git('rev-parse','refs/heads/versions')==head
            # Cancel via a changed guest request/revision before tool application.
            def change_guest(a,r,folder,result):
                a.press(27);a.guest.request('app.open','versiontool');a.press(1003);a.press(13)
                a.guest.request('app.value','1:3330');a.press(13)
            app.after=change_guest
            request('git: save current definition',choice('save',message='Stale guest'),'cancelled_or_stale');app.after=None
            assert git('rev-parse','refs/heads/versions')==head
            report['checks'].append('A restore that would remove a field is rejected with all current data unchanged; changed guest state prevents a pending save')
        report['verified']=True
    except BaseException as error:
        report['error']=str(error) or type(error).__name__;raise
    finally:
        if app is not None:
            if source=='meta' and app.source=='meta':report['generation_calls']=app.provider_calls
            app.close()
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--debug',action='store_true');p.add_argument('--source',choices=['offline','meta'],default='offline');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    verify(build(a.debug),host_build(a.debug),a.output.resolve(),a.source)
