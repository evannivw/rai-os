"""Explicit user-selected CSV exchange. No model calls or model-selected paths."""
import csv
import io
import os
from pathlib import Path
import re
import stat
import tempfile

MAX_BYTES=16384

def parse(data,spec):
    if len(data)>MAX_BYTES:raise ValueError('CSV exceeds 16384 bytes.')
    if spec['capability']!='records.local':raise ValueError('This app does not hold records.')
    try:rows=list(csv.reader(io.StringIO(data.decode('ascii'),newline=''),strict=True))
    except (UnicodeError,csv.Error):raise ValueError('CSV must contain valid ASCII rows.') from None
    fields=spec['fields'];keys=[f['key'] for f in fields];inputs=[f['key'] for f in fields if f['kind']!='calculated']
    if not rows:raise ValueError('CSV needs a header and at least one row.')
    header=rows[0]
    if len(set(header))!=len(header) or set(header) not in (set(inputs),set(keys)):
        raise ValueError('CSV header must name all input fields, or all fields, exactly once.')
    if not 1<=len(rows)-1<=8:raise ValueError('Import requires 1 to 8 rows.')
    for row in rows[1:]:
        if len(row)!=len(header):raise ValueError('CSV rows must match the header width.')
        for value in row:
            if len(value)>32 or any(not 32<=ord(c)<=126 for c in value):raise ValueError('Cells require up to 32 printable ASCII characters.')
    return [keys.index(key) for key in header],rows[1:]

def encode(spec,rows):
    if spec['capability']!='records.local':raise ValueError('This app does not hold records.')
    stream=io.StringIO(newline='');writer=csv.writer(stream,lineterminator='\r\n')
    writer.writerow([f['key'] for f in spec['fields']]);writer.writerows(rows)
    return stream.getvalue().encode('ascii')

class Exchange:
    def __init__(self,root):
        self.root=Path(root).resolve()
        self.inbox=self.root/'inbox';self.outbox=self.root/'outbox'
        self.inbox.mkdir(parents=True,exist_ok=True);self.outbox.mkdir(parents=True,exist_ok=True)
    def read(self,app):
        if not re.fullmatch('[a-z0-9_]{1,12}',app):raise ValueError('Invalid installed app ID.')
        fd=os.open(self.inbox/f'{app}.csv',os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
        with os.fdopen(fd,'rb') as file:
            if not stat.S_ISREG(os.fstat(file.fileno()).st_mode):raise ValueError('Import requires a regular CSV file.')
            data=file.read(MAX_BYTES+1)
        return data
    def write(self,app,revision,data):
        if not re.fullmatch('[a-z0-9_]{1,12}',app) or not revision.isdecimal():raise ValueError('Invalid export identity.')
        name=f'{app}-r{revision}.csv';dest=self.outbox/name
        fd,temp=tempfile.mkstemp(prefix='.rai-export-',dir=self.outbox)
        try:
            with os.fdopen(fd,'wb') as file:file.write(data);file.flush();os.fsync(file.fileno())
            # Publish a complete file atomically without overwriting any existing path.
            os.link(temp,dest,follow_symlinks=False)
            directory=os.open(self.outbox,os.O_RDONLY)
            try:os.fsync(directory)
            finally:os.close(directory)
        finally:os.unlink(temp)
        return name
    def transfer(self,g,event,schema,records):
        app=event['app'];rid=event['request'];revision=event['revision']
        if g.fields('app.catalog')['revision']!=revision:raise ValueError('The app changed. Request the transfer again.')
        spec=schema(g,app)
        if event['transfer']=='import':
            columns,rows=parse(self.read(app),spec)
            g.request('app.import.begin',f'{rid}:{len(rows)}')
            for rowno,row in enumerate(rows):
                for column,value in zip(columns,row):g.request('app.import.cell',f'{rid}:{rowno}:{column}:{value.encode("ascii").hex()}')
            g.request('app.import.ready',rid)
        else:
            name=self.write(app,revision,encode(spec,records(g,app)))
            g.request('app.transfer.notice',f'{rid}:Exported all saved rows to outbox/{name}.')
