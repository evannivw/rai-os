#!/usr/bin/env python3
"""Offline real-QEMU update review, atomic migration, navigation and saved-state checks.

Definitions below are deterministic verifier fixtures, not model output. This
script never submits a prompt through the host planner or reads an API key.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
from app_storage import collect
from apps import ROOT, Studio, build, deliver, host_build, records, schema, states
from verify_app_behaviors import expect_error, field
from verify_apps import submit

LONG_FORMULA = '($base_amount + $base_amount + $base_amount + $base_amount)  / 4'


def repair_definition():
    return dict(
        id='repairdesk', title='Repair desk', capability='records.local',
        filter_key='', filter_label='', fields=[
            field('customer', 'text'), field('parts', 'number', '0'),
            field('labor', 'number', '0'), field('paid', 'number', '0'),
            field('due', 'calculated', formula='$parts+$labor-$paid')],
        workflow=dict(initial='ready', actions=[dict(
            id='deliver', label='Deliver device', **{'from': 'ready'}, to='delivered',
            guard_field='due', guard_op='eq', guard_value=0)]))


def quote_definition():
    return dict(
        id='quotecheck', title='Quote check', capability='records.local',
        filter_key='', filter_label='', fields=[
            field('item', 'text'), field('base_amount', 'number', '0'),
            field('total', 'calculated', formula='$base_amount*2')])


def raster(path):
    magic, size, maximum, pixels = path.read_bytes().split(b'\n', 3)
    assert (magic, size, maximum) == (b'P6', b'480 800', b'255')
    assert len(pixels) == 480 * 800 * 3
    return pixels


def region(pixels, x, y, width, height):
    return b''.join(pixels[((y + row) * 480 + x) * 3:
                           ((y + row) * 480 + x + width) * 3]
                    for row in range(height))


def verify(kernel, binary, output):
    output.mkdir(parents=True, exist_ok=False)
    checkpoint = output / 'device.json'
    head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT,
                          capture_output=True, text=True)
    dirty = subprocess.run(['git', 'status', '--porcelain'], cwd=ROOT,
                           capture_output=True, text=True)
    report = dict(
        verified=False, source='explicit_offline_verifier_fixtures',
        live_meta_calls=0, ai_advantage_measured=False,
        graphics='real_QEMU_guest_framebuffer',
        source_commit=head.stdout.strip() if head.returncode == 0 else None,
        source_worktree_dirty=bool(dirty.stdout.strip()) if dirty.returncode == 0 else None,
        kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),
        host_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
        checks=[], captures=[], proposals=[])
    app = None
    phase = 'start'
    try:
        app = Studio(kernel, binary, output / 'session', 'replay', state=checkpoint)
        g = app.guest

        def snapshot():
            return (g.request('app.catalog'), collect(g, schema, records),
                    checkpoint.read_bytes() if checkpoint.exists() else None)

        def unchanged(before):
            assert snapshot() == before, 'Installed definition, rows, states, revision or checkpoint changed before approval'

        def capture(name):
            path = output / f'{name}.ppm'
            g.capture(path)
            report['captures'].append(path.with_suffix('.png').name)
            return raster(path)

        def proposal(spec, target=None):
            # Direct guest submit obtains real request/revision identifiers but
            # deliberately bypasses Studio's provider dispatch.
            p = submit(g, 'Verifier: review the proposed application change.', target)
            p['spec'] = copy.deepcopy(spec)
            p['reason'] = 'Offline verifier fixture. Review before applying.'
            report['proposals'].append(copy.deepcopy(p))
            return p

        def approve(spec):
            before = snapshot()
            modes = ['review']
            if spec.get('workflow'): modes.append('workflow_review')
            modes += ['behavior_review'] * len(spec.get('behaviors', []))
            if spec.get('transfer'): modes.append('move_review')
            modes += ['page_review'] * len(spec.get('pages', []))
            for mode in modes:
                assert g.request('app.key', '0') == f'state={mode}'
                unchanged(before)
                app.press(13)
            assert schema(g, spec['id']) == spec
            assert app.storage_error is None

        def fill_rows(spec, rows):
            for row_index, row in enumerate(rows):
                if row_index: app.press(110)
                for index, f in enumerate(spec['fields']):
                    if f['kind'] != 'calculated':
                        g.request('app.value', f'{index}:{row[index].encode().hex()}')
                app.press(13)
            assert records(g, spec['id']) == rows

        def select_row(app_id, row):
            g.request('app.open', app_id)
            app.press(1003)
            app.tap(150, 224 + row * 50)
            app.tap(235, 160)
            assert g.request('app.key', '0') == 'state=workflow'

        phase = 'install two-row repair fixture'
        repair = repair_definition()
        deliver(g, proposal(repair))
        capture('01-create-review')
        approve(repair)
        original_rows = [['A', '1000', '2000', '2000', '1000'],
                         ['B', '1000', '1000', '1000', '1000']]
        fill_rows(repair, original_rows)
        assert states(g, repair['id']) == ['ready', 'ready']
        select_row(repair['id'], 0)
        before = snapshot()
        g.request('app.workflow.run', f"{g.fields('app.catalog')['revision']}:deliver",
                  error='workflow_guard_failed')
        unchanged(before)

        phase = 'formula update review and navigation'
        changed = copy.deepcopy(repair)
        changed['fields'][4]['formula'] = '$parts+$labor/2-$paid'
        before = snapshot()
        pending = proposal(changed, repair['id'])
        deliver(g, pending)
        unchanged(before)
        initial_pixels = capture('02-formula-before-after')
        app.tap(100, 567)
        previous_pixels = capture('03-previous-field')
        assert previous_pixels != initial_pixels, 'Pointer previous did not change the visible field'
        app.tap(380, 567)
        assert capture('04-pointer-return') == initial_pixels
        for key in [1008, 1006, 9, 1008, 9]: app.press(key)
        assert capture('05-keyboard-cycle') == initial_pixels
        app.press(1005)
        assert capture('06-keyboard-previous') == previous_pixels
        app.press(1009)
        app.press(1008)
        app.press(1006)
        assert capture('07-keyboard-return') == initial_pixels
        unchanged(before)
        report['checks'].append('First changed calculation is captured; pointer and keyboard paging cycle all fields without mutating installed state')

        phase = 'cancel, late and stale proposal rejection'
        app.press(27)
        unchanged(before)
        g.request('app.ready', f"{pending['request_id']}:Late reply", error='app_stale_request')
        g.request('app.formula', f"{pending['request_id']}:due:31", error='app_stale_request')
        unchanged(before)
        stale = proposal(changed, repair['id'])
        g.request('app.begin',
                  f"{stale['request_id']}:{stale['base_revision'] - 1}:{repair['id']}:526570616972:records.local",
                  error='app_stale_revision')
        app.press(27)
        unchanged(before)

        phase = 'invalid migration on second row'
        invalid = copy.deepcopy(repair)
        invalid['fields'][4]['formula'] = '$parts/($labor-1000)'
        p = proposal(invalid, repair['id'])
        expect_error(lambda: deliver(g, p), 'formula_div_zero')
        unchanged(before)
        app.press(27)
        report['checks'].append('Cancel, late reply, stale revision and division by zero in the second existing row preserve exact definitions, records, states, revision and checkpoint bytes')

        phase = 'approve compatible formula update'
        deliver(g, proposal(changed, repair['id']))
        unchanged(before)
        app.press(13)
        assert g.request('app.key', '0') == 'state=workflow_review'
        capture('08-unchanged-delivery-guard')
        unchanged(before)
        app.press(13)
        expected_rows = [['A', '1000', '2000', '2000', '0'],
                         ['B', '1000', '1000', '1000', '500']]
        assert records(g, repair['id']) == expected_rows
        assert schema(g, repair['id']) == changed
        assert [r[:4] for r in expected_rows] == [r[:4] for r in original_rows]
        assert states(g, repair['id']) == ['ready', 'ready']
        repair = changed
        select_row(repair['id'], 1)
        before = snapshot()
        g.request('app.workflow.run', f"{g.fields('app.catalog')['revision']}:deliver",
                  error='workflow_guard_failed')
        unchanged(before)
        app.press(1007)
        app.press(1008)
        capture('09-recomputed-records')
        report['formula_transition'] = dict(before=original_rows, after=expected_rows,
                                            unchanged_guard=repair['workflow'])
        report['checks'].append('Approval recomputes 1000 to 0 and 1000 to 500, preserves every user input and process state, and retains the unpaid-delivery guard')

        phase = 'append field after calculation'
        added = copy.deepcopy(repair)
        added['fields'].append(field('note', 'text', 'checked'))
        before = snapshot()
        deliver(g, proposal(added, repair['id']))
        unchanged(before)
        capture('10-added-field-before-after')
        approve(added)
        expected_rows = [row + ['checked'] for row in expected_rows]
        assert records(g, repair['id']) == expected_rows
        repair = added

        phase = 'no-op definition review'
        before = snapshot()
        deliver(g, proposal(repair, repair['id']))
        unchanged(before)
        capture('11-no-field-changes')
        approve(repair)
        assert collect(g, schema, records) == before[1]
        report['checks'].append('An appended field is reviewed separately after the calculation; existing rows receive its default; identical-definition approval preserves all data')

        phase = 'unrelated application and maximum-length formula'
        quote = quote_definition()
        deliver(g, proposal(quote))
        approve(quote)
        quote_rows = [['Bolt', '80', '160'], ['Cable', '125', '250']]
        fill_rows(quote, quote_rows)
        assert len(LONG_FORMULA.encode('ascii')) == 64
        long_spec = copy.deepcopy(quote)
        long_spec['fields'][2]['formula'] = LONG_FORMULA
        before = snapshot()
        deliver(g, proposal(long_spec, quote['id']))
        first_tail = capture('12-maximum-formula')
        unchanged(before)
        app.press(27)
        # Change only the final byte. A different raster in its exact glyph box
        # proves the 64th byte is visible, rather than silently clipped. Small
        # type uses a seven-pixel advance; line two begins at (42, 474).
        alternative = copy.deepcopy(long_spec)
        alternative['fields'][2]['formula'] = LONG_FORMULA[:-1] + '2'
        deliver(g, proposal(alternative, quote['id']))
        second_tail = capture('13-maximum-formula-last-byte')
        assert region(first_tail, 147, 474, 7, 18) != region(second_tail, 147, 474, 7, 18), 'Final formula character is not visible'
        assert region(first_tail, 42, 456, 336, 18) == region(second_tail, 42, 456, 336, 18)
        unchanged(before)
        app.press(27)
        deliver(g, proposal(long_spec, quote['id']))
        approve(long_spec)
        quote = long_spec
        quote_rows = [['Bolt', '80', '80'], ['Cable', '125', '125']]
        assert records(g, quote['id']) == quote_rows
        assert records(g, repair['id']) == expected_rows
        report['maximum_formula_bytes'] = len(LONG_FORMULA)
        report['checks'].append('A second unrelated application uses the same update path; a 64-byte formula renders its final character and recomputes records independently')

        phase = 'page-only update retains page review'
        page_spec = copy.deepcopy(quote)
        page_spec['pages'] = [dict(title='Summary', blocks=[dict(
            kind='metric', title='Total', value='total', label_field='',
            aggregate='sum', scope='all')])]
        before = snapshot()
        deliver(g, proposal(page_spec, quote['id']))
        capture('14-page-only-no-field-changes')
        app.press(13)
        assert g.request('app.key', '0') == 'state=page_review'
        capture('15-page-review-still-required')
        unchanged(before)
        app.press(27)
        unchanged(before)
        deliver(g, proposal(page_spec, quote['id']))
        approve(page_spec)
        quote = page_spec
        assert records(g, quote['id']) == quote_rows
        assert g.fields('app.page.values', f"{quote['id']}:0") == {'metrics': '205'}
        report['checks'].append('Page-only changes still require a separate page approval; cancellation is exact and the installed metric is computed from saved data')

        phase = 'brightness review does not actuate'
        light = dict(id='lightlevel', title='Light level', capability='display.brightness',
                     fields=[field('level', 'number', '50')], filter_key='', filter_label='')
        deliver(g, proposal(light))
        approve(light)
        before = snapshot()
        light['fields'][0]['default'] = '75'
        deliver(g, proposal(light, light['id']))
        capture('16-brightness-default-review')
        unchanged(before)
        assert g.fields('app.display')['brightness'] == '100'
        approve(light)
        assert g.fields('app.display')['brightness'] == '100'
        assert records(g, light['id']) == []
        report['checks'].append('Brightness default changes use the same review without actuating the display or inventing records')

        phase = 'delivery guard and persistent restart'
        select_row(repair['id'], 0)
        g.request('app.workflow.run', f"{g.fields('app.catalog')['revision']}:deliver")
        app.persist()
        assert states(g, repair['id']) == ['delivered', 'ready']
        assert records(g, repair['id']) == expected_rows
        final = collect(g, schema, records)
        assert not app.threads and app.provider_calls == 0
        assert app.storage_error is None
        app.close()
        app = Studio(kernel, binary, output / 'restored', 'replay', state=checkpoint)
        assert collect(app.guest, schema, records) == final
        assert not app.threads and app.provider_calls == 0
        app.guest.request('app.open', repair['id'])
        app.press(1003)
        app.guest.capture(output / '17-restored-records.ppm')
        report['captures'].append('17-restored-records.png')
        report['final_state'] = final
        report['checks'].append('Zero-due delivery succeeds; exact definitions, calculated records and workflow states survive a fresh QEMU boot with zero provider calls')
        report['verified'] = True
    except BaseException as error:
        report['failed_phase'] = phase
        report['error'] = str(error) or type(error).__name__
        raise
    finally:
        try:
            if app: app.close()
        finally:
            (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
            print(json.dumps({key: value for key, value in report.items()
                              if key not in {'proposals', 'final_state', 'formula_transition'}}, indent=2))
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--debug', action='store_true')
    parser.add_argument('--kernel', type=Path, help='Use an existing guest binary without rebuilding it')
    parser.add_argument('--binary', type=Path, help='Use an existing host validator without rebuilding it')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    verify(args.kernel.resolve() if args.kernel else build(args.debug),
           args.binary.resolve() if args.binary else host_build(args.debug),
           args.output.resolve())
