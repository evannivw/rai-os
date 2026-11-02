#!/usr/bin/env python3
"""Offline mission publication, permission, semantic and portable-reader fault tests."""
import argparse,copy,hashlib,json,os,shutil,subprocess,sys,tempfile
from pathlib import Path
sys.dont_write_bytecode=True
from offline_mission import ROOT,FIXTURE,build,run,replay_plan
from check_offline_pack import check_package
p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');a=p.parse_args();profile='debug' if a.debug else 'release';binary=build(a.debug)
base=ROOT/'target/verification';base.mkdir(exist_ok=True);out=Path(tempfile.mkdtemp(prefix='offline-mission-'+profile+'-',dir=base))
r=run(binary,'replay',out/'valid');assert r['published'];published=out/'valid/package/published'
def digest(folder):return {str(p.relative_to(folder)):hashlib.sha256(p.read_bytes()).hexdigest() for p in folder.rglob('*') if p.is_file()}
original=digest(published);results=[]
for name,edit,early in [
 ('private',lambda p:p['selected'].append('private-pricing'),True),
 ('traversal',lambda p:p['selected'].append('../private-pricing'),True),
 ('invented-evidence',lambda p:p['evidence'][0].update(quote='This approval was invented.'),True),
 ('stale-permission',lambda p:p.update(permissions_version=2),True),
 ('wrong-goal',lambda p:p.update(request_goal='Another mission'),True),
 ('duplicate',lambda p:p['selected'].append('approval-log'),True),
 ('newest-unapproved',lambda p:p['selected'].__setitem__(p['selected'].index('deck@2'),'deck@3'),False),
 ('wrong-interface',lambda p:p['selected'].__setitem__(p['selected'].index('spec@1'),'spec@2'),False),
 ('missing-support',lambda p:p['selected'].remove('qa@1'),False),
 ('optional',lambda p:p['selected'].append('optional-roadmap'),False),
 ('lost-action',lambda p:p['pending'].pop(),False),
]:
 plan=replay_plan();edit(plan);path=out/(name+'.json');path.write_text(json.dumps(plan));target=out/name
 result=subprocess.run([str(binary),'host-only','--mission-build',str(FIXTURE/'request.json'),str(path),str(target)],cwd=ROOT,env=dict(os.environ,MODEL_API_KEY=''),capture_output=True,text=True,timeout=20)
 assert result.returncode and not (target/'published').exists();assert not target.exists() if early else (target/'candidate').exists()
 assert digest(published)==original
 results.append(dict(case=name,stdout=result.stdout,stderr=result.stderr,rejected=True))
# Never overwrite an existing published package.
result=subprocess.run([str(binary),'host-only','--mission-build',str(FIXTURE/'request.json'),str(out/'valid/plan.json'),str(out/'valid/package')],cwd=ROOT,capture_output=True,text=True);assert result.returncode and digest(published)==original
# Independent checker rejects corrupt files, remote resources and active content.
for name in ['source','extra-private','missing-file','reader-text','script','remote-link','remote-css','symlink']:
 target=out/('corrupt-'+name);shutil.copytree(published,target)
 if name=='source':(target/'documents/spec@1.txt').write_text('changed')
 elif name=='extra-private':(target/'documents/private-pricing.txt').write_text('PRIVATE_FIXTURE_SENTINEL')
 elif name=='missing-file':(target/'documents/qa@1.txt').unlink()
 elif name=='symlink':(target/'documents/qa@1.txt').unlink();(target/'documents/qa@1.txt').symlink_to(published/'documents/qa@1.txt')
 else:
  html=(target/'index.html').read_text()
  if name=='reader-text':html=html.replace('Interface generation two','Invented interface')
  elif name=='script':html=html.replace('</body>','<script>alert(1)</script></body>')
  elif name=='remote-link':html=html.replace('href="manifest.json"','href="https://example.invalid/x"')
  elif name=='remote-css':html=html.replace('</style>',"body{background:url(https://example.invalid/x)}</style>")
  (target/'index.html').write_text(html)
 try:check_package(target)
 except (AssertionError,ValueError,KeyError):pass
 else:raise AssertionError('Accepted corruption '+name)
 results.append(dict(case=name,independent_rejection=True))
# Provider context tests, no real key/network. One good and one invalid outcome.
with tempfile.TemporaryDirectory(prefix='rai-mission-curl-') as temp:
 temp=Path(temp);response=temp/'response.json';counter=temp/'count';curl=temp/'curl'
 curl.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
from pathlib import Path
config=sys.stdin.read();assert 'MODEL_API_KEY' not in os.environ
assert 'dummy_mission_fixture_key' in config
line=next(s for s in config.splitlines() if s.startswith('data-raw = '));body=json.loads(json.loads(line.split(' = ',1)[1]))
assert body['max_tokens']==2048 and body['response_format']['json_schema']['name']=='rai_offline_mission'
user=json.loads(body['messages'][1]['content']);assert set(user)=={'request','catalog'}
rows=user['catalog']['records'];assert len(rows)==13 and all(r['id']!='private-pricing' for r in rows)
assert 'PRIVATE_FIXTURE_SENTINEL' not in json.dumps(user) and 'required_evidence' not in json.dumps(user)
p=Path(os.environ['RAI_COUNT']);p.write_text(str(int(p.read_text())+1) if p.exists() else '1')
sys.stdout.write(Path(os.environ['RAI_RESPONSE']).read_text()+'\\n200')
''');curl.chmod(0o700)
 env=dict(os.environ);os.environ.update(PATH=str(temp)+os.pathsep+env['PATH'],MODEL_API_KEY='dummy_mission_fixture_key',RAI_COUNT=str(counter),RAI_RESPONSE=str(response))
 try:
  for name,plan in [('valid',replay_plan()),('wrong-meaning',replay_plan())]:
   if name=='wrong-meaning':plan['selected'][plan['selected'].index('deck@2')]='deck@3'
   response.write_text(json.dumps(dict(model='muse-spark-1.3',choices=[dict(finish_reason='stop',message=dict(role='assistant',content=json.dumps(plan)))],usage=dict(prompt_tokens=10,completion_tokens=20,total_tokens=30))))
   report=run(binary,'meta',out/('offline-provider-'+name));assert report['published']==(name=='valid');results.append(dict(case='offline-provider-'+name,transport='dummy_curl',report=report))
  assert counter.read_text()=='2'
 finally:os.environ.clear();os.environ.update(env)
(out/'checks.json').write_text(json.dumps(results,indent=2)+'\n')
assert digest(published)==original
print('Evidence:',out)
print(f'PASS {profile}: portable valid pack, 11 rejected plans, overwrite protection, 8 corrupt packages, 2 offline provider cases; prior package unchanged')
