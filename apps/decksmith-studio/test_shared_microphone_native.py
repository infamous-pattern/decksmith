import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import shared_microphone_native as mic

class Client:
    def __init__(self):
        self.node = {'name': 'fixture-mic', 'index': 31, 'serial': '51', 'muted': True}
        self.writes = []
        self.on_read = None
        self.failed = False
    def read_source(self, selector):
        if self.on_read: self.on_read()
        if self.failed: raise ValueError('lost source')
        return dict(self.node)
    def set_mute(self, index, muted):
        self.writes.append((index, muted))
        self.node['muted'] = muted

class NativeMicrophoneTests(unittest.TestCase):
    def setUp(self):
        self.client = Client()
        self.pin = {'server': [4, 8], 'name': 'fixture-mic', 'index': 31, 'serial': '51'}
        self.reports = []
    def run_guard(self, mode, decisions):
        with patch.object(mic, 'allowed', return_value=True), patch.object(mic, 'still_allowed', return_value=True), patch.object(mic, 'decision', side_effect=decisions):
            mic.execute(self.client, self.pin, mode, io.BytesIO(), self.reports.append, lambda: [4, 8])
    def test_hold_opens_and_eof_mutes_exact_source(self):
        self.run_guard('hold', ['hold', None])
        self.assertEqual(self.client.writes, [(31, False), (31, True)])
        self.assertEqual(self.reports, ['live', 'muted'])
    def test_manual_unmute_persists_only_after_commit(self):
        self.run_guard('toggle', ['hold', 'commit'])
        self.assertEqual(self.client.writes, [(31, False)])
        self.assertFalse(self.client.node['muted'])
        self.assertEqual(self.reports, ['live', 'committed'])
    def test_manual_unmute_lost_commit_mutes(self):
        self.run_guard('toggle', [None])
        self.assertEqual(self.client.writes, [(31, False), (31, True)])
        self.assertEqual(self.reports, ['live', 'muted'])
    def test_manual_mute_commits_and_never_opens(self):
        self.client.node['muted'] = False
        self.run_guard('toggle', ['commit'])
        self.assertEqual(self.client.writes, [(31, True)])
        self.assertEqual(self.reports, ['muted', 'committed'])
    def test_release_during_final_identity_read_never_opens(self):
        with patch.object(mic, 'allowed', return_value=True), patch.object(mic, 'still_allowed', return_value=False):
            mic.execute(self.client, self.pin, 'hold', io.BytesIO(), self.reports.append, lambda: [4, 8])
        self.assertEqual(self.client.writes, [(31, True)])
        self.assertEqual(self.reports, ['muted'])
    def test_replacement_or_new_server_receives_no_writes(self):
        for field, value in [('serial', '52'), ('index', 32), ('name', 'other')]:
            client = Client(); client.node[field] = value
            with self.assertRaises(ValueError):
                mic.execute(client, self.pin, 'mute', io.BytesIO(), self.reports.append, lambda: [4, 8])
            self.assertEqual(client.writes, [])
        with self.assertRaises(ValueError):
            mic.execute(self.client, self.pin, 'mute', io.BytesIO(), self.reports.append, lambda: [4, 9])
        self.assertEqual(self.client.writes, [])
    def test_failed_live_write_or_report_still_attempts_mute(self):
        reports = []
        def report(state):
            reports.append(state)
            if state == 'live': raise BrokenPipeError()
        with patch.object(mic, 'allowed', return_value=True), patch.object(mic, 'still_allowed', return_value=True):
            with self.assertRaises(BrokenPipeError):
                mic.execute(self.client, self.pin, 'hold', io.BytesIO(), report, lambda: [4, 8])
        self.assertEqual(self.client.writes, [(31, False), (31, True)])
    def test_cached_pin_never_rebinds_on_retry(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'pin.json'
            pin = mic.pin_source(self.client, 'input:fixture-mic', path, lambda: [4, 8])
            self.assertEqual(pin, self.pin)
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)
            self.client.node['serial'] = '52'
            with self.assertRaises(ValueError):
                mic.pin_source(self.client, 'input:fixture-mic', path, lambda: [4, 8])
            self.assertEqual(json.loads(path.read_text()), self.pin)
    def test_failed_initial_discovery_reserves_an_unusable_pin(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'pin.json'
            self.client.failed = True
            with self.assertRaises(ValueError):
                mic.pin_source(self.client, 'input:fixture-mic', path, lambda: [4, 8])
            self.client.failed = False
            with self.assertRaises(ValueError):
                mic.pin_source(self.client, 'input:fixture-mic', path, lambda: [4, 8])
            self.assertEqual(path.read_text(), '')

    def test_missing_target_or_default_alias_does_not_create_pin(self):
        with tempfile.TemporaryDirectory() as root:
            for target in ('microphone', 'input:@DEFAULT_SOURCE@', 'input:31', 'output:fixture'):
                path = Path(root) / 'pin.json'
                with self.assertRaises(ValueError):
                    mic.pin_source(self.client, target, path, lambda: [4, 8])
                self.assertFalse(path.exists())
    def test_malformed_or_early_commit_is_not_a_hold_authorization(self):
        stream = type('Stream', (), {'fileno': lambda self: 8})()
        for data, expected in [(b'hold\n', 'hold'), (b'hold\ncommit\n', 'commit'), (b'commit\nhold\n', None), (b'hold', None), (b'', None)]:
            with patch.object(mic.select, 'select', return_value=([stream], [], [])), patch.object(mic.os, 'read', return_value=data):
                self.assertEqual(mic.decision(stream), expected)
        with patch.object(mic, 'decision', return_value='commit'):
            self.assertFalse(mic.allowed(stream))

if __name__ == '__main__': unittest.main()
