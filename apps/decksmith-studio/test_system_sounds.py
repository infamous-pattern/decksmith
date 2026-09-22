import ctypes as C
import unittest
from unittest.mock import patch
import system_sounds as sounds
import audio_targets as targets

class SystemSoundsTests(unittest.TestCase):
    def test_defaults_and_bounded_adjustment_preserve_routing_and_balance(self):
        info = sounds.default_info()
        self.assertEqual(sounds.state(info)['percent'], 100)
        info.volume.channels = info.channel_map.channels = 2
        info.volume.values[0] = 32768; info.volume.values[1] = 16384
        info.device = b'saved-output'; info.mute = 1
        sounds.adjusted(info, {'type':'audio_adjust', 'percent':10})
        self.assertEqual(sounds.state(info)['percent'], 60)
        self.assertAlmostEqual(info.volume.values[0] / info.volume.values[1], 2, places=3)
        self.assertEqual(info.device, b'saved-output'); self.assertEqual(info.mute, 1)
        for _ in range(10): sounds.adjusted(info, {'type':'audio_adjust', 'percent':20})
        self.assertEqual(sounds.state(info)['percent'], 100)
        for _ in range(10): sounds.adjusted(info, {'type':'audio_adjust', 'percent':-20})
        self.assertEqual(sounds.state(info)['percent'], 0)
        sounds.adjusted(info, {'type':'audio_adjust', 'percent':5})
        self.assertEqual(sounds.state(info)['percent'], 5)

    def test_mute_preserves_volume_and_rejects_bad_actions(self):
        info = sounds.default_info()
        sounds.adjusted(info, {'type':'audio_mute'})
        self.assertTrue(sounds.state(info)['muted'])
        self.assertEqual(sounds.state(info)['percent'], 100)
        sounds.adjusted(info, {'type':'audio_mute'})
        self.assertFalse(sounds.state(info)['muted'])
        for step in (0,21,-21,True,'5'):
            with self.assertRaises(ValueError): sounds.adjusted(info, {'type':'audio_adjust','percent':step})
        with self.assertRaises(ValueError): sounds.adjusted(info, {'type':'audio_select'})

    def test_idle_read_and_actions_never_access_output_or_other_apps(self):
        value = {'percent':42,'muted':False,'icon':'speaker'}
        with patch.object(targets,'command') as command, patch.object(targets,'listing') as listing, patch.object(sounds,'read',return_value=value) as read:
            self.assertEqual(targets.read(['system','system']),[value,value])
            read.assert_called_once(); command.assert_not_called(); listing.assert_not_called()
        with patch.object(targets,'command') as command, patch.object(sounds,'execute') as execute:
            action={'type':'audio_mute','target':'system'}
            targets.execute(action); execute.assert_called_once_with(action); command.assert_not_called()
        with patch.object(sounds,'read',side_effect=ValueError('offline')),patch.object(targets,'command') as command:
            self.assertEqual(targets.read(['system']),[None]); command.assert_not_called()

    def test_event_meter_excludes_browser_and_output_streams(self):
        nodes=[{'index':1,'properties':{'media.role':'event'}},
               {'index':2,'properties':{'media.role':'music'}},
               {'index':3,'properties':{'application.name':'Brave'}}]
        with patch.object(targets,'listing',return_value=nodes):
            kind,selected=targets.Snapshot().nodes('system')
        self.assertEqual(kind,'sink-inputs'); self.assertEqual([n['index'] for n in selected],[1])

    def test_write_replaces_only_one_event_record_and_applies_immediately(self):
        client = sounds.Client(); seen=[]
        def write(context,mode,pointer,count,apply,callback,data):
            self.assertEqual(mode,2); self.assertEqual(count,1); self.assertEqual(apply,1)
            self.assertEqual(C.cast(pointer,C.POINTER(sounds.Info)).contents.name,sounds.EVENT)
            seen.append(True); callback(None,1,None); return 7
        with patch.object(client,'ready'),patch.object(client,'operation'),patch.object(client,'write_op',side_effect=write):
            client.write(sounds.default_info())
        self.assertEqual(seen,[True])

    def test_health_is_available_even_without_event_stream(self):
        from control_health import probe
        with patch.object(sounds,'read',return_value={'percent':60,'muted':False}), patch.object(targets,'listing') as listing:
            result=probe([{'kind':'audio','target':'system','label':'Mine'}])[0]
            self.assertEqual(result['status'],'available'); self.assertEqual(result['target_name'],'System sounds')
            listing.assert_not_called()

    def test_live_event_update_and_disappearing_alert(self):
        import subprocess
        from unittest.mock import Mock
        connection=Mock();connection.read.return_value=sounds.default_info()
        events=[{'index':10,'properties':{'media.role':'event'}}]
        with patch.object(sounds,'client',return_value=connection),patch.object(targets,'listing',return_value=events),patch.object(targets,'command') as command:
            sounds.execute({'type':'audio_adjust','target':'system','percent':-5})
            command.assert_called_once_with('set-sink-input-volume','10','95%')
            self.assertEqual(sounds.state(connection.write.call_args.args[0])['percent'],95)
        with patch.object(sounds,'client',return_value=connection),patch.object(targets,'listing',side_effect=[events,[]]),patch.object(targets,'command',side_effect=subprocess.CalledProcessError(1,'pactl')):
            sounds.execute({'type':'audio_mute','target':'system'})
        connection.read.side_effect=ValueError('offline')
        with patch.object(sounds,'client',return_value=connection),patch.object(targets,'command') as command:
            with self.assertRaises(ValueError):sounds.execute({'type':'audio_mute','target':'system'})
            command.assert_not_called()
