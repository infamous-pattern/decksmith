import unittest
import subprocess
import sys
from unittest.mock import Mock
from audio_inventory import Inventory
from audio_targets import Snapshot, execute, read
from unittest.mock import patch


class BoundedCommandTests(unittest.TestCase):
    def test_reads_output_and_preserves_failure_diagnostics(self):
        from audio_targets import _bounded_command
        self.assertEqual(_bounded_command([sys.executable, '-c', 'print("ready")'], timeout=3), 'ready\n')
        with self.assertRaises(subprocess.CalledProcessError) as failure:
            _bounded_command([sys.executable, '-c', 'import sys;print("failure",file=sys.stderr);sys.exit(4)'], timeout=3)
        self.assertEqual(failure.exception.returncode, 4)
        self.assertIn('failure', failure.exception.stderr)

    def test_limits_output_and_reaps_a_stalled_child(self):
        from audio_targets import _bounded_command
        children=[]
        spawn=subprocess.Popen
        def tracked(*args,**kwargs):
            child=spawn(*args,**kwargs);children.append(child);return child
        with patch('audio_targets.subprocess.Popen', side_effect=tracked):
            with self.assertRaisesRegex(ValueError, 'too large'):
                _bounded_command([sys.executable, '-c', 'import sys;sys.stdout.write("x"*10000)'], limit=128, timeout=3)
            with self.assertRaises(subprocess.TimeoutExpired):
                _bounded_command([sys.executable, '-c', 'import time;time.sleep(5)'], timeout=.1)
        self.assertEqual(len(children),2)
        self.assertTrue(all(child.poll() is not None for child in children))


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.revision = 1
        self.now = 0
        self.watcher = Mock()
        self.watcher.stamp.side_effect = lambda: self.revision
        self.inventory = Inventory(self.watcher, lambda: self.now)
        self.load = Mock(return_value={'sinks': []})

    def test_reuse_until_event_then_refresh_and_watchdog(self):
        self.assertEqual(self.inventory.get('all', self.load), {'sinks': []})
        self.now = 4.99
        self.inventory.get('all', self.load)
        self.assertEqual(self.load.call_count, 1)
        self.revision += 1
        self.inventory.get('all', self.load)
        self.assertEqual(self.load.call_count, 2)
        self.now += 5
        self.inventory.get('all', self.load)
        self.assertEqual(self.load.call_count, 3)

    def test_unready_or_disconnected_listener_never_reuses(self):
        self.inventory.get('all', self.load)
        self.revision = None
        for _ in range(2):
            self.inventory.get('all', self.load)
        self.assertEqual(self.load.call_count, 3)
        self.assertEqual(self.inventory.values, {})
        self.revision = 3
        self.inventory.get('all', self.load)
        self.assertEqual(self.load.call_count, 4)

    def test_query_failure_discards_stale_state(self):
        self.inventory.get('all', self.load)
        self.revision += 1
        self.load.side_effect = RuntimeError('disconnected')
        with self.assertRaises(RuntimeError):
            self.inventory.get('all', self.load)
        self.assertEqual(self.inventory.values, {})

    def test_notification_during_query_does_not_cache_older_snapshot(self):
        def load():
            self.revision += 1
            return {}
        self.inventory.get('all', load)
        self.assertEqual(self.inventory.values, {})

    def test_meter_routing_and_reader_share_query_but_actions_always_refresh(self):
        node = {'index': 7, 'name': 'speaker', 'volume': {'left': {'value': 32768}}, 'mute': False}
        with patch('audio_targets.list_all', return_value={'sinks': [node]}) as load, patch('audio_targets.command',return_value='{}') as command:
            self.assertEqual(read(['output:speaker'], self.inventory)[0]['percent'], 50)
            self.assertEqual(Snapshot(self.inventory).nodes('output:speaker')[1], [node])
            self.assertEqual(load.call_count, 1)
            execute({'type': 'audio_mute', 'target': 'output:speaker'})
            self.assertEqual(load.call_count, 2)
            command.assert_called_with('set-sink-mute', '7', '1')

    def test_default_routing_changes_and_listener_cleanup(self):
        with patch('audio_targets.command', return_value='{"default_sink_name":"first"}') as load:
            self.assertEqual(Snapshot(self.inventory).info()['default_sink_name'], 'first')
            self.revision += 1
            load.return_value = '{"default_sink_name":"second"}'
            self.assertEqual(Snapshot(self.inventory).info()['default_sink_name'], 'second')
        self.inventory.close()
        self.watcher.close.assert_called_once_with()
