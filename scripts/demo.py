#!/usr/bin/env python3
"""Start a saved showcase or a fresh live device; preserve every rehearsal."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import sys
import uuid

sys.dont_write_bytecode = True
from apps import Studio, build, host_build, records, run, schema, states
from app_storage import Checkpoint, collect
from files_demo import ROOT

FIXTURE = ROOT / 'fixtures/microapps/v1/demo-device.json'
PROVENANCE = ROOT / 'fixtures/microapps/v1/demo-provenance.json'


def prepare(output, binary, kernel, source='saved'):
    """Create only a new directory; never reset an existing user's device."""
    output.mkdir(parents=True, exist_ok=False)
    (output / 'exchange/inbox').mkdir(parents=True)
    (output / 'exchange/outbox').mkdir()
    checkpoint = Checkpoint(output / 'device.json', binary, kernel)
    try:
        if source == 'saved':
            checkpoint.path.write_bytes(FIXTURE.read_bytes())
            checkpoint.load()  # Validate checksum, schema, formulas and states.
        elif source == 'meta':
            checkpoint.save(dict(version=1, apps=[]))
        else:
            raise ValueError('Unknown demo source')
    finally:
        checkpoint.close()
    manifest = dict(format='rai-demo-1', source=source,
                    kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),
                    seed_sha256=hashlib.sha256(FIXTURE.read_bytes()).hexdigest() if source == 'saved' else None,
                    provenance=json.loads(PROVENANCE.read_text()) if source == 'saved' else None)
    (output / 'demo.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (output / 'START.txt').write_text(
        'RAI-OS DEMO\n'
        f'Source: {source}. Saved mode restores actual earlier Meta proposals; it does not generate them again.\n'
        'Connect a VNC viewer to localhost:5900. Ctrl-C closes and saves the session.\n'
        'Saved: open Repair Shop > records > Phone C > actions. Delivery is blocked while balance is 4000.\n'
        'Return to records, open Phone C, replace Paid cents with 5000, then Save and Deliver.\n'
        'Home > Quote App > table > right column page: totals and profit run locally.\n'
        'No API is called in saved mode. The screen says replay; its Create button uses only disclosed offline examples.\n'
        'For arbitrary supported generation, start a separate live demo with your own key.\n'
        'Resume with scripts/demo.py resume --output THIS_DIRECTORY; it never resets your data.\n'
        'Limits: 3 apps, 6 fields, 8 records each; host checkpoint and HTTPS; shared EL1 runtime.\n')
    return output


def open_demo(kernel, binary, output, source='saved', resume=False, port=5900, bind='127.0.0.1'):
    if not resume:
        prepare(output, binary, kernel, source)
    manifest = json.loads((output / 'demo.json').read_text())
    if manifest.get('format') != 'rai-demo-1' or manifest.get('source') not in ('saved', 'meta'):
        raise ValueError('Unsupported demo manifest')
    if not (output / 'device.json').is_file():
        raise ValueError('Saved device is missing; refusing to silently start empty')
    source = manifest['source']
    session = output / ('session-' + uuid.uuid4().hex[:12])
    print((output / 'START.txt').read_text(), flush=True)
    run(kernel, binary, session, 'replay' if source == 'saved' else 'meta',
        port=port, bind=bind, state=output / 'device.json', exchange=output / 'exchange')


def verify(kernel, binary, output):
    """Check the published saved demo in real QEMU, with no provider calls."""
    prepare(output, binary, kernel)
    report = dict(verified=False, source='saved_actual_meta_results', live_meta_calls=0,
                  kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(), checks=[])
    seed_bytes = FIXTURE.read_bytes()
    app = None
    try:
        try:
            prepare(output, binary, kernel)
        except FileExistsError:
            pass
        else:
            raise AssertionError('Existing demo must never be reset')
        assert (output / 'device.json').read_bytes() == seed_bytes
        app = Studio(kernel, binary, output / 'check', 'replay',
                     state=output / 'device.json', exchange=output / 'exchange')
        g = app.guest
        expected = json.loads(seed_bytes)['state']
        assert collect(g, schema, records) == expected
        g.capture(output / '01-launcher.ppm')
        # Navigate through the actual guest pointer targets, not a host UI.
        app.tap(132, 270); app.press(1003)
        assert g.request('app.key', '0') == 'state=records'
        g.capture(output / '02-orders.ppm')
        app.tap(150, 324); app.tap(235, 160)
        before = collect(g, schema, records); revision = g.fields('app.catalog')['revision']
        # The native card produces the error and the guest renders its reason.
        g.request('app.tap', '250:426', error='workflow_guard_failed')
        assert collect(g, schema, records) == before and g.fields('app.catalog')['revision'] == revision
        g.capture(output / '03-blocked.ppm')
        app.press(1003); app.tap(150, 324); app.tap(150, 420)
        # Replace payment with normal keystrokes, as a person in the viewer would.
        for _ in range(4): app.press(8)
        for char in '5000': app.press(ord(char))
        app.press(13)
        assert records(g, 'repairshop')[2][3:] == ['5000', '0']
        app.tap(235, 160); app.tap(250, 426)
        assert states(g, 'repairshop')[2] == 'delivered'
        g.capture(output / '04-delivered.ppm')
        workshop = collect(g, schema, records)['apps'][0]
        report['checks'].append('Saved Meta workflow: native unpaid delivery rejected unchanged; payment and delivery succeed locally')
        app.press(27); app.tap(350, 270); app.press(1007); app.tap(400, 650)
        assert g.fields('app.totals', 'quote:all')['totals'].endswith('profit:285')
        g.capture(output / '05-profit.ppm')
        app.tap(150, 255); app.tap(150, 500)
        for _ in range(3): app.press(8)
        for char in '175': app.press(ord(char))
        app.press(13); app.press(1007); app.tap(400, 650)
        assert records(g, 'quote')[0][-2:] == ['175', '200']
        assert g.fields('app.totals', 'quote:all')['totals'].endswith('profit:260')
        assert collect(g, schema, records)['apps'][0] == workshop
        g.capture(output / '06-local-edit.ppm'); app.tap(405, 160)
        exported = next((output / 'exchange/outbox').glob('quote-*.csv'))
        assert list(csv.reader(io.StringIO(exported.read_text(), newline=''))) == [
            [f['key'] for f in schema(g, 'quote')['fields']]] + records(g, 'quote')
        final = collect(g, schema, records)
        assert not app.threads
        app.close(); app = None
        app = Studio(kernel, binary, output / 'restarted', 'replay',
                     state=output / 'device.json', exchange=output / 'exchange')
        assert collect(app.guest, schema, records) == final and not app.threads
        app.tap(132, 270); app.press(1003)
        app.guest.capture(output / '07-restarted.ppm')
        assert FIXTURE.read_bytes() == seed_bytes
        report['checks'].append('Two apps restored; exact local profit 285 -> 260; workshop unchanged; CSV and reboot match; no model requests')
        report['checks'].append('Existing demo directory refuses reset; tracked fixture remains byte-identical')
        report['final_state'] = final
        report['verified'] = True
    except BaseException as error:
        report['error'] = str(error) or type(error).__name__
        raise
    finally:
        try:
            if app is not None: app.close()
        finally:
            (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
            print(json.dumps(report, indent=2))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['saved', 'live', 'resume', 'verify'])
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--debug', action='store_true')
    parser.add_argument('--port', default=5900, type=int)
    args = parser.parse_args()
    kernel, binary = build(args.debug), host_build(args.debug)
    output = args.output.resolve()
    try:
        if args.action == 'verify':
            verify(kernel, binary, output)
        else:
            open_demo(kernel, binary, output, 'meta' if args.action == 'live' else 'saved',
                      resume=args.action == 'resume', port=args.port)
    except KeyboardInterrupt:
        pass


if __name__ == '__main__':
    main()
