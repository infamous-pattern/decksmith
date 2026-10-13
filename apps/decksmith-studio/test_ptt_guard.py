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

class SharedGuardianTests(unittest.TestCase):
    def test_reports_only_completed_live_and_mute_writes(self):
        seen=[]
        with patch('ptt_guard.Snapshot'),patch('ptt_guard.heartbeat',side_effect=[True,False]),patch('ptt_guard.select.select',return_value=([],[],[])),patch('ptt_guard.command') as command:
            hold('input:mic',Mock(),seen.append)
        self.assertEqual(seen,['live','muted'])
        self.assertEqual([c.args for c in command.call_args_list],[('set-source-mute','mic','0'),('set-source-mute','mic','1')])
    def test_discovery_failure_still_attempts_named_mute_without_live_ack(self):
        seen=[]
        with patch('ptt_guard.Snapshot') as snapshot,patch('ptt_guard.command') as command:
            snapshot.return_value.nodes.return_value=('sources',[])
            with self.assertRaises(ValueError):hold('input:missing',Mock(),seen.append)
        self.assertEqual(seen,['muted'])
        command.assert_called_once_with('set-source-mute','missing','1')
    def test_failed_live_command_never_reports_live_and_finally_reports_mute(self):
        seen=[]
        with patch('ptt_guard.Snapshot'),patch('ptt_guard.heartbeat',return_value=True),patch('ptt_guard.select.select',return_value=([],[],[])),patch('ptt_guard.command',side_effect=[RuntimeError('failed'),None]):
            with self.assertRaises(RuntimeError):hold('input:mic',Mock(),seen.append)
        self.assertEqual(seen,['muted'])
    def test_standalone_mute_has_bounded_retries_and_no_success_ack_on_failure(self):
        from ptt_guard import mute
        seen=[]
        with patch('ptt_guard.command',side_effect=RuntimeError('failed')) as command,patch('ptt_guard.time.sleep') as sleep:
            with self.assertRaises(RuntimeError):mute('input:mic',seen.append)
        self.assertEqual(command.call_count,3);self.assertEqual(sleep.call_count,2);self.assertEqual(seen,[])
    def test_broken_report_channel_still_attempts_mute(self):
        with patch('ptt_guard.Snapshot'),patch('ptt_guard.heartbeat',return_value=True),patch('ptt_guard.select.select',return_value=([],[],[])),patch('ptt_guard.command') as command,patch('ptt_guard.time.sleep'):
            with self.assertRaises(BrokenPipeError):hold('input:mic',Mock(),Mock(side_effect=BrokenPipeError()))
        self.assertEqual(command.call_args_list[-1].args,('set-source-mute','mic','1'))

    def test_shared_cli_emits_real_newlines_and_standalone_mute_never_opens(self):
        import io,runpy,sys,types
        from pathlib import Path
        from audio_targets import validate
        source=Path(__file__).with_name('ptt_guard.py')
        for mode,expected in [('--shared','live\nmuted\n'),('--mute','muted\n')]:
            output=io.StringIO();command=Mock();snapshot=Mock();snapshot.return_value.nodes.return_value=('sources',[{'name':'mic'}])
            stub=types.SimpleNamespace(Snapshot=snapshot,command=command,validate=validate)
            stream=Mock()
            with patch.dict(sys.modules,{'audio_targets':stub}),patch('sys.argv',[str(source),'input:mic',mode]),patch('sys.stdin',stream),patch('sys.stdout',output),patch('signal.signal'),patch('select.select',side_effect=[([stream.buffer],[],[]),([],[],[]),([stream.buffer],[],[])]),patch('os.read',side_effect=[b'hold\n',b'']):
                runpy.run_path(str(source),run_name='__main__')
            self.assertEqual(output.getvalue(),expected)
            self.assertEqual(command.call_args_list[-1].args,('set-source-mute','mic','1'))
            if mode=='--mute':snapshot.assert_not_called();self.assertEqual(command.call_count,1)
