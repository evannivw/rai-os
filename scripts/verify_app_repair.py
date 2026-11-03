#!/usr/bin/env python3
"""Check bounded correction with real Rust validation and real QEMU, offline first."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
from apps import Studio, build, deliver, host_build, records, schema
from app_storage import collect
from files_demo import ROOT
from verify_apps import submit
from verify_app_behaviors import GOAL, expect_error, field, step


def example():
    return dict(id='movement', title='Movement', capability='records.local',
                fields=[field('name', 'text'), field('stock', 'number', '0'), field('moved', 'number', '0')],
                filter_key='', filter_label='', behaviors=[dict(id='move', label='Move one', event='press', steps=[
                    step('require', 'stock', '1', 'ge'), step('calculate', 'stock', '$stock-1'),
                    step('calculate', 'moved', '$moved+1')])])


def invalid():
    value = example()
    value['behaviors'][0]['steps'][1]['value'] = '$stock -'
    return value


class ScriptedStudio(Studio):
    """Only proposal transport is scripted; Rust and guest checks remain real."""
    def __init__(self, *args, proposals, after=None, transport_error=False, adapter_error=None, **kwargs):
        self.proposals = proposals
        self.after = after
        self.transport_error = transport_error
        self.adapter_error = adapter_error
        self.invocations = []
        super().__init__(*args, source='replay', **kwargs)
        self.repair_enabled = True

    def _provider_attempt(self, r, request_path, feedback_path, attempt_dir):
        index = len(self.invocations)
        self.invocations.append(dict(index=index + 1, feedback=feedback_path is not None))
        assert index < len(self.proposals), 'Unexpected additional correction attempt'
        if self.adapter_error:
            cmd = ['synthetic-adapter-fault', self.adapter_error]
            if self.adapter_error == 'timeout':
                raise subprocess.TimeoutExpired(cmd, 40, output=b'{"event":"partial"}\n', stderr=b'partial stderr\n')
            stdout, stderr = {'null': ('null\n', ''),
                              'missing-plan': ('{"event":"app_plan"}\n', ''),
                              'stderr-array': ('', '[]\n')}[self.adapter_error]
            result = subprocess.CompletedProcess(cmd, 0, stdout, stderr)
        elif self.transport_error:
            result = subprocess.CompletedProcess(['synthetic-transport-fault'], 1,
                json.dumps(dict(event='provider_diagnostic', source='synthetic_transport_fault',
                                outcome='rejected', error='meta_transport_failed')) + '\n',
                json.dumps(dict(event='error', error='meta_transport_failed')) + '\n')
        else:
            supplied = self.proposals[index]
            proposal = dict(request_id=r['request_id'], base_revision=r['base_revision'],
                            decision='install', spec=copy.deepcopy(supplied),
                            reason='Disclosed offline proposal for correction verification.')
            if supplied is None:
                proposal.update(decision='clarify', spec=None, reason='Clarify the intended arithmetic.')
            path = attempt_dir / 'supplied.json'
            path.write_text(json.dumps(proposal, indent=2) + '\n')
            if feedback_path is None:
                cmd = [str(self.binary), str(self.kernel), '--app-plan-replay', str(request_path), str(path)]
            else:
                feedback = json.loads(feedback_path.read_text())
                assert feedback['attempt'] == index + 1 and feedback['code'] == 'formula_syntax'
                assert feedback['proposal']['request_id'] == r['request_id']
                assert feedback['proposal']['base_revision'] == r['base_revision']
                assert feedback['proposal']['spec'] == self.proposals[index - 1]
                cmd = [str(self.binary), str(self.kernel), '--app-repair-replay', str(request_path), str(feedback_path), str(path)]
            env = dict(os.environ)
            env.pop('MODEL_API_KEY', None)
            env.pop('RAI_VNC_PASSWORD', None)
            result = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True, timeout=20)
        if self.after is not None:
            self.after(self, index + 1, result)
        return result


def start(app):
    for char in GOAL:
        app.press(ord(char))
    reply = app.press(13)
    rid = dict(item.split('=', 1) for item in reply.split('; '))['submit']
    for thread in app.threads:
        thread.join(timeout=115)
        assert not thread.is_alive(), 'Bounded correction thread did not finish'
    folder = app.output / rid
    events = [json.loads(line) for line in (folder / 'provider.stdout.jsonl').read_text().splitlines()]
    return folder, events


def empty(app):
    assert app.guest.fields('app.catalog') == dict(revision='1', apps='-')
    assert not app.checkpoint.path.exists()


def use_accepted(app, spec, output):
    g = app.guest
    assert [f['kind'] for f in spec['fields']] == ['text', 'number', 'number']
    assert spec['capability'] == 'records.local' and not spec.get('workflow')
    assert len(spec.get('behaviors', [])) == 1
    assert g.request('app.key', '0') == 'state=review'
    empty(app)
    g.capture(output / '01-accepted-review.ppm')
    app.press(13)
    assert g.request('app.key', '0') == 'state=behavior_review'
    empty(app)
    g.capture(output / '02-program-review.ppm')
    app.press(13)
    assert schema(g, spec['id']) == spec
    rows = [['Widget', '3', '0'], ['Empty', '0', '0'], ['Other', '7', '4']]
    for rowno, row in enumerate(rows):
        if rowno: app.press(110)
        for i, value in enumerate(row): g.request('app.value', f'{i}:{value.encode().hex()}')
        app.press(13)
    assert records(g, spec['id']) == rows
    def select(row):
        g.request('app.open', spec['id']); app.press(1003); app.tap(150, 224 + 50 * row); app.tap(260, 160)
    select(0)
    revision = int(g.fields('app.catalog')['revision'])
    app.tap(250, 265)
    assert g.request('app.key', '0') == 'state=behavior_preview'
    assert records(g, spec['id']) == rows
    g.capture(output / '03-corrected-preview.ppm')
    app.tap(240, 720)
    rows[0] = ['Widget', '2', '1']
    assert records(g, spec['id']) == rows and int(g.fields('app.catalog')['revision']) == revision + 1
    select(2); app.tap(250, 265); app.tap(240, 720)
    rows[2] = ['Other', '6', '5']
    assert records(g, spec['id']) == rows
    select(1); before = g.request('app.catalog')
    expect_error(lambda: app.tap(250, 265), 'behavior_condition_failed')
    assert records(g, spec['id']) == rows and g.request('app.catalog') == before
    g.capture(output / '04-denied-empty.ppm')
    return collect(g, schema, records)


def verify(kernel, binary, output, source='offline'):
    try:
        commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, stderr=subprocess.DEVNULL).strip()
        dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], text=True).strip())
    except (FileNotFoundError, subprocess.CalledProcessError):
        commit = dirty = None
    if source == 'meta':
        assert commit and dirty is False, 'Commit sources before the bounded live request'
    output.mkdir(parents=True, exist_ok=False)
    report = dict(verified=False, source=source, source_commit=commit, source_worktree_dirty=dirty,
                  maximum_live_calls=3 if source == 'meta' else 0,
                  kernel_sha256=hashlib.sha256(kernel.read_bytes()).hexdigest(), checks=[], cases=[])
    app = initial_app = None
    try:
        checkpoint = output / 'success-device.json'
        if source == 'meta':
            app = Studio(kernel, binary, output / 'success', 'meta', state=checkpoint)
        else:
            def capture_attempt(app, attempt, result):
                with app.lock:
                    empty(app)
                    if attempt == 2: app.guest.capture(output / '00-correcting.ppm')
            app = ScriptedStudio(kernel, binary, output / 'success', state=checkpoint,
                                 proposals=[invalid(), example()], after=capture_attempt)
        initial_app = app
        folder, events = start(app)
        success_trace = json.loads((folder / 'loop.json').read_text())
        report['success_trace'] = success_trace
        report['events'] = events
        report['actual_attempts'] = app.provider_calls
        report['generation_calls'] = app.provider_calls if source == 'meta' else 0
        assert success_trace['state'] == 'review', success_trace
        assert len(success_trace['attempts']) == app.provider_calls
        accepted = [e for e in events if e.get('event') == 'app_plan']
        failures = [e for e in events if e.get('event') == 'app_validation_failed']
        assert 1 <= app.provider_calls <= 3
        assert len(accepted) == 1 and accepted[0]['plan']['decision'] == 'install', 'No accepted installation after bounded correction; inspect retained diagnostics'
        if source == 'offline':
            assert len(app.invocations) == 2 and len(failures) == 1
            assert failures[0]['code'] == 'formula_syntax' and failures[0]['execution'] is False
            assert failures[0]['repairable'] is True
            assert [event['attempt'] for event in events] == [1, 2]
            assert failures[0]['proposal']['spec'] == invalid()
            assert accepted[0]['plan']['spec'] == example()
        spec = accepted[0]['plan']['spec']
        final = use_accepted(app, spec, output)
        request_count = app.provider_calls
        app.close(); app = None
        app = Studio(kernel, binary, output / 'restored', 'replay', state=checkpoint)
        assert collect(app.guest, schema, records) == final and app.provider_calls == 0
        app.guest.request('app.open', spec['id']); app.press(1003)
        app.guest.capture(output / '05-restored.ppm')
        app.close(); app = None
        report['checks'].append('Only the accepted proposal reaches guest review; both approvals required; exact arithmetic on different rows, denied empty stock and reboot verified without further inference')
        report['final_state'] = final
        if source == 'offline':
            def run_case(name, proposals, expected_calls, after=None, transport_error=False, adapter_error=None, initial_calls=0, expected_event=None, expect_empty=True):
                nonlocal app
                directory = output / name
                app = ScriptedStudio(kernel, binary, directory, proposals=proposals, after=after,
                                     transport_error=transport_error, adapter_error=adapter_error, state=output / (name + '-device.json'))
                app.provider_calls = initial_calls
                case_folder, case_events = start(app)
                trace = json.loads((case_folder / 'loop.json').read_text())
                expected_state = {'exhausted': 'exhausted', 'scope-denied': 'not_repairable',
                    'cancel-invalid': 'cancelled_or_stale', 'cancel-valid': 'cancelled_or_stale',
                    'replaced-revision': 'cancelled_or_stale', 'transport-stop': 'provider_failed',
                    'session-budget': 'session_limit', 'clarify': 'clarify',
                    'nonrepairable': 'not_repairable', 'adapter-null': 'failed',
                    'adapter-missing-plan': 'failed', 'adapter-stderr-array': 'failed',
                    'adapter-timeout': 'failed'}[name]
                assert trace['state'] == expected_state, (name, trace)
                assert len(trace['attempts']) == expected_calls
                assert len(list(case_folder.glob('attempt-*'))) == expected_calls
                for index in range(1, expected_calls + 1):
                    attempt_folder = case_folder / f'attempt-{index:02d}'
                    assert (attempt_folder / 'provider.stdout.jsonl').exists()
                    assert (attempt_folder / 'provider.stderr.jsonl').exists()
                assert len(app.invocations) == expected_calls, (name, app.invocations)
                assert app.provider_calls == initial_calls + expected_calls
                if expect_empty: empty(app)
                if name.startswith('adapter-'):
                    assert app.guest.request('app.key', '0') == 'state=notice'
                    if adapter_error == 'timeout':
                        assert 'partial' in (case_folder / 'attempt-01' / 'provider.stdout.jsonl').read_text()
                        assert 'partial stderr' in (case_folder / 'attempt-01' / 'provider.stderr.jsonl').read_text()
                assert not any(e.get('event') == 'app_plan' and isinstance(e.get('plan'), dict) for e in case_events) or name in ('cancel-valid', 'clarify')
                if expected_event:
                    assert any(e.get('event') == 'app_validation_failed' and e.get('code') == expected_event for e in case_events), (name, case_events)
                if name == 'replaced-revision':
                    catalog = app.guest.fields('app.catalog')
                    assert catalog['revision'] == '2' and catalog['apps'].split(':', 1)[0] == 'replacement'
                if name == 'session-budget':
                    app.press(1002)
                    for char in GOAL: app.press(ord(char))
                    app.press(13)
                    assert app.provider_calls == 8 and len(app.invocations) == 1
                    assert app.guest.request('app.key', '0') == 'state=notice'
                    empty(app)
                app.guest.capture(output / (name + '.ppm'))
                report['cases'].append(dict(name=name, attempts=expected_calls, provider_counter=app.provider_calls, final_catalog=app.guest.fields('app.catalog'), trace=trace))
                app.close(); app = None
            run_case('exhausted', [invalid()] * 3, 3, expected_event='formula_syntax')
            weakened = example(); weakened['behaviors'][0]['steps'].pop(0)
            run_case('scope-denied', [invalid(), weakened], 2, expected_event='app_repair_scope_denied')
            def cancel(app, attempt, result):
                if attempt == 1: app.press(27)
            run_case('cancel-invalid', [invalid(), example()], 1, after=cancel)
            run_case('cancel-valid', [example()], 1, after=cancel)
            def replace_request(app, attempt, result):
                if attempt != 1: return
                with app.lock:
                    plan = submit(app.guest, 'Replacement request')
                    plan['spec'] = dict(id='replacement', title='Replacement', capability='records.local',
                                        fields=[field('name', 'text', 'Local')], filter_key='', filter_label='')
                    deliver(app.guest, plan); app.guest.request('app.key', '13')
                    assert app.guest.fields('app.catalog')['revision'] == '2'
            run_case('replaced-revision', [invalid(), example()], 1, after=replace_request, expect_empty=False)
            run_case('transport-stop', [None], 1, transport_error=True)
            run_case('session-budget', [invalid(), example()], 1, initial_calls=7)
            run_case('clarify', [invalid(), None], 2)
            unsupported = example(); unsupported['capability'] = 'network'
            run_case('nonrepairable', [unsupported], 1, expected_event='app_capability_denied')
            for fault in ('null', 'missing-plan', 'stderr-array', 'timeout'):
                run_case('adapter-' + fault, [None], 1, adapter_error=fault)
            report['checks'].append('Three attempts maximum; guard-removal repair denied; canceled or replaced requests never deliver or trigger another attempt; transport and unsupported capability stop immediately; session counter limits corrections; clarification does not force installation; malformed adapter envelopes and synthetic timeout stop with notice and retained diagnostics')
        report['generation_calls'] = request_count if source == 'meta' else 0
        report['verified'] = True
    except BaseException as error:
        report['error'] = str(error) or type(error).__name__
        raise
    finally:
        try:
            if app is not None and app is initial_app and not app.closed:
                report['actual_attempts'] = app.provider_calls
                report['generation_calls'] = app.provider_calls if source == 'meta' else 0
                try: report['final_catalog'] = app.guest.fields('app.catalog')
                except (AssertionError, OSError) as error: report['final_catalog_error'] = type(error).__name__
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
