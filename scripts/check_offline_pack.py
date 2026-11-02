#!/usr/bin/env python3
"""Independent fixed-fixture intent, bytes, permissions and offline-reader check."""
import argparse
import hashlib
import json
from html.parser import HTMLParser
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from check_bundle import unique
ROOT=Path(__file__).resolve().parents[1]
FIXTURE=ROOT/'fixtures/offline_mission/v1'


def load(path,limit=65536):
    assert path.is_file() and not path.is_symlink()
    data=path.read_bytes();assert len(data)<=limit
    return json.loads(data,object_pairs_hook=unique)


def check_plan(plan):
    expected=load(FIXTURE/'expected.json');request=load(FIXTURE/'request.json');catalog=load(FIXTURE/'catalog.json')
    assert set(plan)=={'request_goal','request_version','catalog_version','permissions_version','decision','selected','evidence','pending','reason'}
    assert plan['request_goal']==request['goal'] and type(plan['request_version']) is int and plan['request_version']==request['request_version']
    for key in ['catalog_version','permissions_version']:assert type(plan[key]) is int and plan[key]==catalog[key]
    assert plan['decision']==expected['decision']=='publish'
    assert isinstance(plan['selected'],list) and sorted(plan['selected'])==expected['selected']
    sources={}
    for row in catalog['records']:
        if row['read_allowed']:
            data=(FIXTURE/row['path']).read_bytes()
            assert len(data)==row['byte_length'] and hashlib.sha256(data).hexdigest()==row['sha256'],'Fixture changed'
            sources[row['id']]=data
    assert set(plan['selected'])<=sources.keys() and len(set(plan['selected']))==len(plan['selected'])
    assert isinstance(plan['reason'],str) and 0<len(plan['reason'].strip()) and len(plan['reason'].encode())<=240
    assert not any(ord(c)<32 or 127<=ord(c)<=159 for c in plan['reason'])
    for name in ['evidence','pending']:
        assert isinstance(plan[name],list);seen=set()
        for q in plan[name]:
            assert set(q)=={'source','quote'} and q['source'] in plan['selected']
            assert isinstance(q['quote'],str) and 0<len(q['quote'].strip()) and len(q['quote'].encode())<=300
            assert not any(ord(c)<32 or 127<=ord(c)<=159 for c in q['quote'])
            assert q['quote'] in sources[q['source']].decode()
            pair=(q['source'],q['quote']);assert pair not in seen;seen.add(pair)
    assert 2<=len(plan['evidence'])<=6 and len(plan['pending'])<=4
    assert sorted((q['source'],q['quote']) for q in plan['pending'])==sorted((q['source'],q['quote']) for q in expected['pending'])
    for needed in expected['required_evidence']:
        assert any(q['source']==needed['source'] and needed['contains'] in q['quote'] for q in plan['evidence']),'Missing release/dependency evidence'
    return request,catalog,sources


class Reader(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True);self.stack=[];self.blocks=[];self.active=None;self.links=[];self.ids=set();self.style='';self.csp=False;self.text=''
    def handle_starttag(self,tag,attrs):
        assert tag in {'html','head','meta','title','style','body','main','header','div','h1','h2','br','p','section','blockquote','a','nav','details','summary','article','pre','footer'}
        a=dict(attrs);assert len(a)==len(attrs) and not any(k.startswith('on') or k in ['src','srcdoc','style'] for k in a)
        if 'id' in a:assert a['id'] not in self.ids;self.ids.add(a['id'])
        if tag=='a':self.links.append(a['href'])
        if tag=='meta' and a.get('http-equiv','').lower()=='content-security-policy':
            assert a['content']=="default-src 'none'; style-src 'unsafe-inline'; connect-src 'none'; base-uri 'none'; form-action 'none'";self.csp=True
        marker=next((k for k in ['data-document','data-pending','data-source'] if k in a),None)
        if marker:
            assert self.active is None;self.active=[tag,marker,a[marker],'']
        if tag not in ['meta','br']:self.stack.append(tag)
    def handle_endtag(self,tag):
        assert self.stack and self.stack.pop()==tag,'Unbalanced reader'
        if self.active and self.active[0]==tag:self.blocks.append(self.active);self.active=None
    def handle_data(self,data):
        self.text+=data
        if self.stack and self.stack[-1]=='style':self.style+=data
        if self.active:self.active[3]+=data


def check_package(folder):
    assert folder.is_dir() and not folder.is_symlink()
    paths=list(folder.rglob('*'));assert all(not p.is_symlink() for p in paths),'Symlink in package'
    plan=load(folder/'plan.json',8192);request,catalog,sources=check_plan(plan)
    expected=load(FIXTURE/'expected.json');rows=[r for r in catalog['records'] if r['id'] in expected['selected']];rows.sort(key=lambda r:r['id'])
    want={'index.html','manifest.json','plan.json',*[r['path'] for r in rows]}
    assert {str(p.relative_to(folder)) for p in paths if p.is_file()}==want,'Unexpected/missing package file'
    assert {str(p.relative_to(folder)) for p in paths if p.is_dir()}=={'documents'}
    manifest=load(folder/'manifest.json')
    assert manifest==dict(format='rai-offline-pack-v1',execution_location='host',request_version=request['request_version'],catalog_version=1,permissions_version=1,goal=request['goal'],records=rows)
    for row in rows:
        data=(folder/row['path']).read_bytes();assert data==sources[row['id']] and hashlib.sha256(data).hexdigest()==row['sha256'],'Source bytes changed'
    html=(folder/'index.html').read_bytes();assert len(html)<=131072
    reader=Reader();reader.feed(html.decode());reader.close();assert not reader.stack and reader.active is None and reader.csp
    assert 'url(' not in reader.style.lower() and '@import' not in reader.style.lower()
    for href in reader.links:
        assert href[1:] in reader.ids if href.startswith('#') else href in want
    documents=[b for b in reader.blocks if b[1]=='data-document'];assert len(documents)==len(rows)
    assert {b[2]:b[3] for b in documents}=={r['id']:sources[r['id']].decode() for r in rows}
    for kind,name in [('data-pending','pending'),('data-source','evidence')]:
        assert sorted((b[2],b[3]) for b in reader.blocks if b[1]==kind)==sorted((q['source'],q['quote']) for q in plan[name])
    assert 'PRIVATE_FIXTURE_SENTINEL' not in reader.text and 'This package is generated by the host bridge' in reader.text
    return dict(documents=len(rows),source_bytes=sum(r['byte_length'] for r in rows),pending_actions=len(plan['pending']),external_resources=0,execution_location='host')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('package',type=Path);a=p.parse_args()
    try:print(json.dumps(check_package(a.package)))
    except (AssertionError,OSError,ValueError,KeyError,TypeError) as e:p.exit(1,'REJECTED: '+(str(e) or type(e).__name__)+'\n')
