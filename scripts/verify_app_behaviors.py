#!/usr/bin/env python3
"""Independent QEMU oracles for composed row behaviors; no implicit inference."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
from apps import Studio, build, deliver, host_build, records, schema, states
from app_storage import collect
from verify_apps import submit

GOAL = 'App: name, stock, moved. Add Move: require stock>=1, stock=stock-1, moved=moved+1. Numeric defaults 0.'


def step(op, field, value='', compare='', skip=0):
    return dict(op=op, field=field, value=value, compare=compare, skip=skip)


def field(key, kind, default='', formula=None):
    result = dict(key=key, label=key.title(), kind=kind, required=True, default=default)
    if formula: result['formula'] = formula
    return result


def stock_fixture():
    return dict(id='movement', title='Movement', capability='records.local',
                filter_key='', filter_label='', fields=[
                    field('name', 'text'), field('stock', 'number', '0'),
                    field('moved', 'number', '0'), field('flag', 'check', 'false'),
                    field('status', 'text', 'ready'), field('total', 'calculated', formula='$stock+$moved')],
                behaviors=[dict(id='move', label='Move one', event='press', steps=[
                    step('require', 'stock', '1', 'ge'), step('calculate', 'stock', '$stock-1'),
                    step('calculate', 'moved', '$moved+1'), step('toggle', 'flag'),
                    step('if', 'stock', '2', 'lt', 1), step('set', 'status', 'low')]),
                    dict(id='reject', label='Check rollback', event='press', steps=[
                        step('calculate', 'moved', '$moved+7'), step('require', 'stock', '100', 'ge')])])


def review_fixture():
    return dict(id='reviewer', title='Review desk', capability='records.local',
                filter_key='', filter_label='', fields=[field('name', 'text'),
                    field('points', 'number', '5'), field('accepted', 'check', 'false'),
                    field('label', 'text', 'draft')],
                workflow=dict(initial='open', actions=[dict(id='close', label='Close review',
                    **{'from': 'open'}, to='closed', guard_field='', guard_op='none', guard_value=0)]),
                behaviors=[dict(id='accept', label='Accept', event='press', steps=[
                    step('require', 'label', 'draft', 'eq'), step('require', 'points', '5', 'ge'),
                    step('set', 'accepted', 'true'), step('set', 'label', 'accepted')]),
                    dict(id='revise', label='Revise', event='press', steps=[
                        step('toggle', 'accepted'), step('set', 'label', 'draft')])])


def expect_error(operation, code):
    """Catch only a real UART error frame; unrelated assertions remain failures."""
    try:
        operation()
    except AssertionError as error:
        frame = error.args[0] if error.args else None
        assert isinstance(frame, list) and len(frame) == 4 and frame[0] == '@rai1' and frame[2:] == ['err', code], error
        return
    raise AssertionError(f'Expected guest rejection {code}')


def verify(kernel, binary, output, source='offline'):
    try:
        commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, stderr=subprocess.DEVNULL).strip()
        dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], text=True).strip())
    except (FileNotFoundError, subprocess.CalledProcessError):
        commit = dirty = None
    if source == 'meta': assert commit and dirty is False, 'Freeze sources before the one live request'
    output.mkdir(parents=True, exist_ok=False)
    checkpoint = output / 'device.json'
    report = dict(verified=False, source=source, maximum_live_calls=1 if source == 'meta' else 0,
                  retries=0, source_commit=commit, source_worktree_dirty=dirty,
                  kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(),
                  development_example=True, checks=[])
    app = None
    try:
        app = Studio(kernel, binary, output / 'session', 'meta' if source == 'meta' else 'replay', state=checkpoint)
        g = app.guest

        def approve(spec, capture=None):
            assert g.request('app.key', '0') == 'state=review'
            before = g.request('app.catalog')
            if capture: g.capture(output / (capture + '-fields.ppm'))
            app.press(13)
            if spec.get('workflow'):
                assert g.request('app.key', '0') == 'state=workflow_review'
                assert g.request('app.catalog') == before
                app.press(13)
            for index, _ in enumerate(spec.get('behaviors', [])):
                assert g.request('app.key', '0') == 'state=behavior_review'
                assert g.request('app.catalog') == before
                if capture: g.capture(output / f'{capture}-behavior-{index}.ppm')
                app.press(13)
            assert schema(g, spec['id']) == spec

        def propose(spec, target=None):
            plan = submit(g, 'Compose generic data operations', target)
            plan['spec'] = spec
            deliver(g, plan)
            return plan

        def fill(spec, row):
            for index, f in enumerate(spec['fields']):
                if f['kind'] != 'calculated': g.request('app.value', f'{index}:{row[index].encode().hex()}')

        def select(app_id, row):
            g.request('app.open', app_id); app.press(1003); app.tap(150, 224 + row * 50)
            app.tap(260, 160)
            assert g.request('app.key', '0') == 'state=behaviors'

        def preview(index=0):
            app.tap(250, 265 + index * 94)
            assert g.request('app.key', '0') == 'state=behavior_preview'

        def commit_preview():
            app.tap(240, 720)
            assert g.request('app.key', '0') == 'state=behaviors'

        if source == 'meta':
            for char in GOAL: app.press(ord(char))
            response = app.press(13)
            request_id = dict(part.split('=', 1) for part in response.split('; '))['submit']
            for thread in app.threads:
                thread.join(timeout=45); assert not thread.is_alive()
            plans = [event['plan'] for event in map(json.loads, (output / 'session' / request_id / 'provider.stdout.jsonl').read_text().splitlines()) if event.get('event') == 'app_plan']
            assert len(plans) == 1 and plans[0]['decision'] == 'install'
            spec = plans[0]['spec']; report['plan'] = plans[0]
            assert [f['kind'] for f in spec['fields']] == ['text', 'number', 'number']
            assert len(spec.get('behaviors', [])) == 1 and not spec.get('workflow')
            initial = [['Widget', '3', '0'], ['Empty', '0', '0'], ['Other', '7', '4']]
        else:
            spec = stock_fixture(); report['plan'] = propose(spec)
            initial = [['Widget', '3', '0', 'false', 'ready', '3'], ['Empty', '0', '0', 'false', 'ready', '0'], ['Other', '7', '4', 'false', 'ready', '11']]
        app_id = spec['id']; button_id = spec['behaviors'][0]['id']
        approve(spec, '01-install')
        for rowno, row in enumerate(initial):
            if rowno: app.press(110)
            fill(spec, row); app.press(13)
        assert records(g, app_id) == initial
        select(app_id, 0)
        original = collect(g, schema, records); before_revision = g.fields('app.catalog')['revision']
        g.request('app.behavior.run', f'{before_revision}:{button_id}', error='behavior_preview_required')
        preview(); assert collect(g, schema, records) == original and g.fields('app.catalog')['revision'] == before_revision
        g.capture(output / '02-preview.ppm')
        app.press(27)
        g.request('app.behavior.run', f'{before_revision}:{button_id}', error='behavior_preview_required')
        assert collect(g, schema, records) == original
        select(app_id, 0); preview()
        g.request('app.behavior.run', f'{int(before_revision)-1}:{button_id}', error='behavior_preview_required')
        assert collect(g, schema, records) == original
        # Return through the actual view and obtain a fresh preview before confirmation.
        select(app_id, 0); preview(); commit_preview()
        expected = copy.deepcopy(initial)
        expected[0] = ['Widget', '2', '1'] if source == 'meta' else ['Widget', '2', '1', 'true', 'ready', '3']
        assert records(g, app_id) == expected
        assert int(g.fields('app.catalog')['revision']) == int(before_revision) + 1
        g.request('app.behavior.preview', f'{before_revision}:{button_id}', error='app_stale_revision')
        assert records(g, app_id) == expected
        select(app_id, 0); preview(); commit_preview()
        expected[0] = ['Widget', '1', '2'] if source == 'meta' else ['Widget', '1', '2', 'false', 'low', '3']
        assert records(g, app_id) == expected
        g.capture(output / '03-applied.ppm')
        select(app_id, 2); preview(); commit_preview()
        expected[2] = ['Other', '6', '5'] if source == 'meta' else ['Other', '6', '5', 'true', 'ready', '11']
        assert records(g, app_id) == expected
        report['checks'].append('Preview has no side effects; direct run, canceled/mismatched confirmation and stale preview rejected; native confirmation commits exactly once; sequential calculations match independent oracle')
        select(app_id, 1); before = collect(g, schema, records); revision = g.request('app.catalog')
        expect_error(lambda: app.tap(250, 265), 'behavior_condition_failed')
        assert g.fields('app.behavior.status') == dict(step='1', error='behavior_condition_failed')
        assert collect(g, schema, records) == before and g.request('app.catalog') == revision
        g.capture(output / '04-condition-rejected.ppm')
        if source == 'offline':
            select(app_id, 0); before = collect(g, schema, records); revision = g.request('app.catalog')
            expect_error(lambda: app.tap(250, 359), 'behavior_condition_failed')
            assert g.fields('app.behavior.status') == dict(step='2', error='behavior_condition_failed')
            assert collect(g, schema, records) == before and g.request('app.catalog') == revision
            g.capture(output / '05-rollback.ppm')
            report['checks'].append('Both forward-branch outcomes, checkbox toggle and text set checked; step-two failure after a staged write rolls back every value and revision')
            # Reject even an invalid operation hidden behind an ordinarily skipped branch.
            bad_steps = [
                ([step('if', 'stock', '-100', 'eq', 1), step('set', 'missing', 'bad')], 'behavior_unknown_field'),
                ([step('set', 'total', '7')], 'behavior_read_only_field'),
                ([step('if', 'stock', '0', 'ge', 2), step('set', 'status', 'bad')], 'behavior_bad_skip'),
                ([step('set', 'flag', 'yes')], 'behavior_bad_value'),
            ]
            for steps, code in bad_steps:
                invalid = copy.deepcopy(spec); invalid['behaviors'][0]['steps'] = steps
                before = collect(g, schema, records); revision = g.request('app.catalog')
                expect_error(lambda: propose(invalid, app_id), code)
                assert collect(g, schema, records) == before and g.request('app.catalog') == revision
                assert schema(g, app_id) == spec
            report['checks'].append('Unknown field in skipped branch, derived-field write, invalid skip and malformed boolean rejected before installation')
            # New behavior definition, same existing rows: reviewed, compatible update.
            changed = copy.deepcopy(spec)
            changed['behaviors'][1]['steps'] = [step('set', 'status', 'staging'), step('calculate', 'moved', '2147483647+1')]
            propose(changed, app_id); approve(changed, '06-update')
            assert records(g, app_id) == expected
            select(app_id, 0); before = collect(g, schema, records); revision = g.request('app.catalog')
            expect_error(lambda: app.tap(250, 359), 'formula_overflow')
            assert g.fields('app.behavior.status') == dict(step='2', error='formula_overflow')
            assert collect(g, schema, records) == before and g.request('app.catalog') == revision
            spec = changed
            second = review_fixture(); propose(second); approve(second, '07-second')
            fill(second, ['Document', '5', 'false', 'draft']); app.press(13)
            select(second['id'], 0); preview(); commit_preview()
            assert records(g, second['id']) == [['Document', '5', 'true', 'accepted']]
            assert records(g, app_id) == expected
            select(second['id'], 0); preview(1); commit_preview()
            assert records(g, second['id']) == [['Document', '5', 'false', 'draft']]
            # Same generic behaviors coexist with a workflow, but cannot bypass its terminal lock.
            g.request('app.open', second['id']); app.press(1003); app.press(13); app.press(1013); app.press(13)
            assert states(g, second['id']) == ['closed']
            select(second['id'], 0); before = collect(g, schema, records); revision = g.request('app.catalog')
            expect_error(lambda: app.tap(250, 265), 'workflow_closed')
            assert g.fields('app.behavior.status') == dict(step='0', error='workflow_closed')
            assert collect(g, schema, records) == before and g.request('app.catalog') == revision
            report['checks'].append('Reviewed behavior update preserves records; late arithmetic overflow rolls back; unrelated text/boolean app runs same executor without touching first app; terminal workflow rejects behavior')
        final = collect(g, schema, records)
        app.press(27); g.capture(output / '08-home.ppm')
        assert len(app.threads) == (1 if source == 'meta' else 0)
        app.close(); app = None
        app = Studio(kernel, binary, output / 'restored', 'replay', state=checkpoint)
        assert collect(app.guest, schema, records) == final and not app.threads
        app.guest.request('app.open', app_id); app.press(1003); app.press(13); app.press(1014)
        app.guest.capture(output / '09-restored.ppm')
        report['final_state'] = final
        report['checks'].append('Fresh QEMU restores exact behavior definitions, every field and workflow state without another model call')
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


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--debug', action='store_true')
    parser.add_argument('--source', choices=['offline', 'meta'], default='offline')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    verify(build(args.debug), host_build(args.debug), args.output.resolve(), args.source)
