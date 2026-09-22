import unittest
from unittest.mock import Mock,patch
from ptt_guard import hold

class PushToTalkTests(unittest.TestCase):
    def test_release_mutes_the_same_named_mic(self):
        with patch('ptt_guard.Snapshot') as snapshot,patch('ptt_guard.heartbeat',side_effect=[True,True,False]),patch('ptt_guard.select.select',return_value=([],[],[])),patch('ptt_guard.command') as command:
            snapshot.return_value.nodes.return_value=('sources',[{'name':'mic'}])
            hold('input:mic',Mock())
        self.assertEqual([c.args for c in command.call_args_list],[('set-source-mute','mic','0'),('set-source-mute','mic','1')])
    def test_no_heartbeat_never_opens_mic(self):
        with patch('ptt_guard.Snapshot'),patch('ptt_guard.heartbeat',return_value=False),patch('ptt_guard.command') as command:
            hold('input:mic',Mock())
        command.assert_called_once_with('set-source-mute','mic','1')
    def test_release_during_discovery_never_opens_mic(self):
        stream=Mock()
        with patch('ptt_guard.Snapshot'),patch('ptt_guard.heartbeat',return_value=True),patch('ptt_guard.select.select',return_value=([stream],[],[])),patch('ptt_guard.os.read',return_value=b''),patch('ptt_guard.command') as command:
            hold('input:mic',stream)
        command.assert_called_once_with('set-source-mute','mic','1')
    def test_error_after_open_still_mutes(self):
        with patch('ptt_guard.Snapshot'),patch('ptt_guard.heartbeat',side_effect=[True,RuntimeError('lost parent')]),patch('ptt_guard.select.select',return_value=([],[],[])),patch('ptt_guard.command') as command:
            with self.assertRaises(RuntimeError):hold('input:mic',Mock())
        self.assertEqual(command.call_args_list[-1].args,('set-source-mute','mic','1'))
    def test_rejects_default_and_output_targets(self):
        for target in ('microphone','system','output:mic','app:application.name=Zoom'):
            with self.assertRaises(ValueError):hold(target,Mock())
