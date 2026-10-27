"""Mutation checks for fixture integrity; no guest or policy results are tested."""

import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'check_review_fixture.py'
SPEC = importlib.util.spec_from_file_location('fixture', SCRIPT)
fixture = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixture)


class FixtureChecks(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.base = Path(self.directory.name) / 'fixture'
        shutil.copytree(fixture.DEFAULT_FIXTURE, self.base)

    def edit_json(self, relative, change):
        path = self.base / relative
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def test_checked_in_fixture(self):
        self.assertEqual(fixture.check_fixture(self.base), (7, 3, 12))

    def test_same_length_source_change(self):
        path = self.base / 'records/notes@1.txt'
        path.write_bytes(path.read_bytes().replace(b'7', b'9'))
        with self.assertRaisesRegex(ValueError, 'digest mismatch'):
            fixture.check_fixture(self.base)

    def test_wrong_index_count(self):
        self.edit_json('expected.json', lambda d: d['goals'][0]['index']['pump'].update({'notes@1': 1}))
        with self.assertRaisesRegex(ValueError, 'incorrect reference index'):
            fixture.check_fixture(self.base)

    def test_unreleased_goal_record(self):
        self.edit_json('scenarios/changing.json', lambda d: d['events'][1].pop('release'))
        with self.assertRaisesRegex(ValueError, 'unreleased record'):
            fixture.check_fixture(self.base)

    def test_wrong_outage_checkpoint(self):
        self.edit_json('expected.json', lambda d: d['checkpoints']['changing'][2].update(required_unavailable=[]))
        with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
            fixture.check_fixture(self.base)

    def test_event_after_deadline(self):
        self.edit_json('scenarios/changing.json', lambda d: d['events'][-1].update(at_ms=60000))
        with self.assertRaisesRegex(ValueError, 'invalid event order/time'):
            fixture.check_fixture(self.base)


if __name__ == '__main__':
    unittest.main()
