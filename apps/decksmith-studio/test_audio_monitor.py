import unittest
from unittest.mock import Mock,patch
from audio_monitor import AudioMonitor,TOPOLOGY

class AudioMonitorTests(unittest.TestCase):
    def test_inventory_events_exclude_volume_and_discovery_clients(self):
        for event in (b"Event 'new' on sink-input #5",b"Event 'remove' on source #2",b"Event 'change' on server #0"):
            self.assertIsNotNone(TOPOLOGY.search(event))
        for event in (b"Event 'change' on sink #2",b"Event 'new' on client #99",b"Event 'remove' on client #99"):
            self.assertIsNone(TOPOLOGY.search(event))
    def test_burst_events_share_one_refresh(self):
        monitor=AudioMonitor();picker=Mock();monitor.pickers.add(picker)
        with patch('audio_monitor.GLib.timeout_add',return_value=5) as timer:
            for _ in range(30):monitor.schedule()
        timer.assert_called_once()
        monitor.dispatch();picker.refresh.assert_called_once()
        self.assertEqual(monitor.debounce,0)
    def test_fragmented_events_are_buffered(self):
        monitor=AudioMonitor();source=Mock();source.fileno.return_value=1
        with patch('audio_monitor.os.read',side_effect=[b"Event 'new' on sink-",b"input #5\n"]),patch.object(monitor,'schedule') as schedule:
            self.assertTrue(monitor.receive(source,0));schedule.assert_not_called()
            self.assertTrue(monitor.receive(source,0));schedule.assert_called_once()
    def test_last_picker_closes_subscription(self):
        monitor=AudioMonitor();a=Mock();b=Mock()
        with patch.object(monitor,'start') as start,patch.object(monitor,'close') as close,patch('audio_monitor.GLib.timeout_add_seconds',return_value=42):
            monitor.add(a);monitor.process=Mock();monitor.add(b)
            start.assert_called_once()
            monitor.remove(a);close.assert_not_called()
            monitor.remove(b);close.assert_called_once()

    def test_recheck_recovers_without_any_audio_event(self):
        monitor=AudioMonitor();picker=Mock()
        with patch.object(monitor,'start'),patch('audio_monitor.GLib.timeout_add_seconds',return_value=42) as timer:
            monitor.add(picker)
            picker.refresh.assert_called_once()
            timer.assert_called_once_with(2,monitor.reconcile)
            picker.refresh.reset_mock()
            self.assertTrue(monitor.reconcile())
            picker.refresh.assert_called_once()
        with patch('audio_monitor.GLib.source_remove') as remove:
            monitor.close()
            remove.assert_called_once_with(42)
