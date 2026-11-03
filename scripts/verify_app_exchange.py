#!/usr/bin/env python3
"""Real guest CSV review, atomic import, export and recovery; no inference."""
import argparse
import csv
import io
import json
from pathlib import Path
import sys
sys.dont_write_bytecode=True
from apps import Studio,deliver,schema,records,build,host_build,ROOT
from verify_apps import submit
from app_exchange import parse,encode,Exchange

# Reuse an actual prior Meta result, explicitly without regenerating it.
def example():return json.loads((ROOT/'fixtures/microapps/v1/exchange-example.json').read_text())['spec']

def verify(kernel,binary,out):
    out.mkdir(parents=True,exist_ok=False);state=out/'device.json';exchange=out/'exchange';spec=example();app_id=spec['id']
    app=Studio(kernel,binary,out/'session','replay',state=state,exchange=exchange);g=app.guest;checks=[]
    try:
        p=submit(g,'Restore a previously generated app');p['spec']=spec;deliver(g,p);app.press(13)
        original=['Existing','1','50','50']
        for i,value in enumerate(original[:3]):g.request('app.value',f'{i}:{value.encode().hex()}')
        app.press(13);saved=records(g,app_id);before=g.request('app.catalog')
        # Imported columns can be reordered and CSV quoting preserves commas/quotes.
        incoming=b'unitc,item,qty\r\n125,"Cable, blue",3\r\n80,"Adapter ""pro""",2\r\n'
        inbox=exchange/'inbox'/f'{app_id}.csv';inbox.write_bytes(incoming)
        app.tap(310,160);assert g.request('app.key','0')=='state=import_review'
        assert records(g,app_id)==saved and g.request('app.catalog')==before
        g.capture(out/'preview-first.ppm');app.press(9);g.capture(out/'preview-second.ppm')
        app.press(27);assert records(g,app_id)==saved
        # New request: confirm the actual guest primary button, once.
        g.request('app.open',app_id);app.press(1011);app.tap(240,720)
        expected=saved+[['Cable, blue','3','125','375'],['Adapter "pro"','2','80','160']]
        assert records(g,app_id)==expected
        assert int(g.fields('app.catalog')['revision'])==int(dict(x.split('=') for x in before.split('; '))['revision'])+1
        app.tap(400,650);g.capture(out/'imported-table.ppm');checks.append('Native preview and cancel; confirmed batch increments revision once and preserves existing rows')
        # Export contains all rows/columns, regardless of the visible table page.
        app.tap(405,160);exports=list((exchange/'outbox').glob('*.csv'));assert len(exports)==1
        exported=exports[0].read_bytes();assert list(csv.reader(io.StringIO(exported.decode(),newline='')))==[[f['key'] for f in spec['fields']]]+expected
        g.capture(out/'exported.ppm');g.request('app.open',app_id);app.press(1012)
        assert len(list((exchange/'outbox').glob('*.csv')))==1 and exports[0].read_bytes()==exported
        checks.append('Export includes exact quoted text and derived results; repeated export does not overwrite')
        # A valid first row followed by an overflow must append neither row.
        before=g.request('app.catalog');saved=records(g,app_id)
        for data in [b'item,qty,unitc\nGood,1,1\nBad,2147483647,2\n',b'item,qty,unitc,total\nBad,2,2,999\n',b'item,qty,qty\nX,1,1\n',b'item,qty,unitc\nX,1\n',b'item,qty,unitc\n'+b'X,1,1\n'*6]:
            inbox.write_bytes(data);g.request('app.open',app_id);app.press(1011)
            assert g.request('app.key','0')=='state=notice'
            assert records(g,app_id)==saved and g.request('app.catalog')==before
        checks.append('Later-row overflow, forged derived values, duplicate headers, malformed widths and capacity excess rejected unchanged')
        # Full exports are accepted only when their computed values match.
        inbox.write_bytes(exported);g.request('app.open',app_id);app.press(1011);assert g.request('app.key','0')=='state=import_review';app.press(27)
        assert records(g,app_id)==saved
        # Late/duplicate/incomplete staging cannot bypass the GUI approval.
        g.request('app.open',app_id);event=dict(p.split('=',1) for p in g.request('app.key','1011').split('; '));rid=event['request']
        g.request('app.import.begin',f'{rid}:1');g.request('app.import.cell',f'{rid}:0:0:58');g.request('app.import.cell',f'{rid}:0:0:58',error='app_import_duplicate')
        g.request('app.import.ready',rid,error='app_import_incomplete');g.request('app.key','27');g.request('app.import.ready',rid,error='app_stale_transfer')
        assert records(g,app_id)==saved
        checks.append('Full CSV round-trip accepted; duplicate, incomplete and cancelled staging rejected')
        for data in [b'x'*16385,b'item,qty,unitc\nX,1,\xff\n',b'item,qty,unitc\n"line\nbreak",1,2\n']:
            try:parse(data,spec)
            except ValueError:pass
            else:raise AssertionError('Invalid CSV accepted')
        inbox.unlink();inbox.symlink_to(exports[0]);g.request('app.open',app_id);app.press(1011);assert g.request('app.key','0')=='state=notice';assert records(g,app_id)==saved
        assert not app.threads
    finally:app.close()
    restored=Studio(kernel,binary,out/'restored','replay',state=state,exchange=exchange)
    try:
        assert records(restored.guest,app_id)==expected and schema(restored.guest,app_id)==spec
        restored.guest.request('app.open',app_id);restored.press(1007);restored.press(1008);restored.guest.capture(out/'restored.ppm')
        assert not restored.threads;checks.append('Imported rows and formulas survive fresh QEMU recovery without inference')
    finally:restored.close()
    report=dict(verified=True,live_meta_calls=0,schema_source='Prior Meta result d20f98f, restored without inference',checks=checks,final_rows=expected)
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args();verify(build(a.debug),host_build(a.debug),a.output.resolve())
