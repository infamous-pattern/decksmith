import io
import json
import unittest
from unittest.mock import Mock,patch
import launch
import shared_builtin as shared
from system_controls import Controls


def abort():raise ValueError('Stale action scope')


class SharedBuiltinTests(unittest.TestCase):
    def test_application_discovery_precedes_grant_and_stale_never_launches(self):
        app=Mock();app.should_show.return_value=True
        with patch.object(launch.Gio.DesktopAppInfo,'new',return_value=app) as discover:
            with self.assertRaisesRegex(ValueError,'Stale'):
                launch.launch('application','org.example.Test.desktop',authorize=lambda:(discover.assert_called_once(),abort()))
        app.launch.assert_not_called();app.launch_uris_as_manager.assert_not_called()

    def test_website_pins_handler_before_grant_and_keeps_literal_uri(self):
        app=Mock();events=[]
        app.launch_uris_as_manager.side_effect=lambda urls,*args:events.append(('write',urls)) or True
        with patch.object(launch.Gio.AppInfo,'get_default_for_uri_scheme',side_effect=lambda scheme:events.append(('discover',scheme)) or app):
            launch.launch('website','https://example.com/?q=a&b=2',authorize=lambda:events.append(('grant',)))
        self.assertEqual(events,[('discover','https'),('grant',),('write',['https://example.com/?q=a&b=2'])])
        app.launch_uris_as_manager.reset_mock()
        with patch.object(launch.Gio.AppInfo,'get_default_for_uri_scheme',return_value=None):
            with self.assertRaisesRegex(ValueError,'unavailable'):
                launch.launch('website','https://example.com/',authorize=lambda:self.fail('No grant for missing handler'))
        app.launch_uris_as_manager.assert_not_called()

    def test_guarded_app_child_has_independent_session_and_confirmation_retains_consent(self):
        app=Mock()
        launch.detached_launch(app,[])
        args=app.launch_uris_as_manager.call_args.args
        # PyGObject's child setup bridge requires tuple user_data; None silently
        # skips the callback with a TypeError. Verified by the private GI fixture.
        self.assertEqual(args[4],())
        with patch.object(launch.os,'setsid') as detach, \
             patch.object(launch.os,'open',return_value=9), \
             patch.object(launch.os,'dup2'),patch.object(launch.os,'close'):
            args[3](*args[4])
        detach.assert_called_once_with()
        with patch.object(launch.os,'setsid',side_effect=OSError),patch.object(launch.os,'_exit',side_effect=SystemExit(126)) as terminate:
            with self.assertRaises(SystemExit):args[3](*args[4])
        terminate.assert_called_once_with(126)
        c=Controls(authorize=lambda:None);c.state=Mock(return_value={'available':True})
        with patch('system_controls.Gio.Settings.sync'),patch('subprocess.Popen') as spawn:c.execute('shutdown')
        self.assertTrue(spawn.call_args.kwargs['start_new_session'])
        self.assertEqual(spawn.call_args.args[0][-1],'shutdown')

    def test_legacy_uri_launch_keeps_default_handler_path(self):
        with patch.object(launch.Gio.AppInfo,'launch_default_for_uri',return_value=True) as invoke,patch.object(launch.Gio.AppInfo,'get_default_for_uri_scheme') as discover:
            launch.launch('website','https://example.com/')
        invoke.assert_called_once_with('https://example.com/',None)
        discover.assert_not_called()

    def test_child_stdio_isolation_failure_prevents_exec(self):
        for operation in ('open', 'dup2', 'close'):
            with self.subTest(operation=operation), \
                 patch.object(launch.os,'setsid'), \
                 patch.object(launch.os,'open',return_value=9) as open_fd, \
                 patch.object(launch.os,'dup2') as duplicate, \
                 patch.object(launch.os,'close') as close_fd, \
                 patch.object(launch.os,'_exit',side_effect=SystemExit(126)) as terminate:
                {'open':open_fd,'dup2':duplicate,'close':close_fd}[operation].side_effect=OSError
                with self.assertRaises(SystemExit):launch.independent_child()
                terminate.assert_called_once_with(126)

    def test_launch_helper_aborts_after_read_without_executing(self):
        writes=[]
        def launcher(kind,value,*,authorize):
            authorize();writes.append((kind,value))
        output=io.StringIO()
        with self.assertRaisesRegex(ValueError,'Stale'):
            shared.execute({'type':'open_application','desktop_id':'org.example.Test.desktop'},io.StringIO('{"decision":"abort"}\n'),output,launcher=launcher)
        self.assertFalse(writes)
        self.assertEqual(json.loads(output.getvalue()),{'status':'ready'})

    def test_invalid_payloads_and_plugins_rejected_without_adapter(self):
        for action in [dict(type='open_application',desktop_id='../evil.desktop'),dict(type='system',command='shell'),dict(type='plugin'),dict(type='open_website',url='https://example.com/',extra=1)]:
            with self.assertRaises(ValueError):shared.execute(action,io.StringIO(),io.StringIO(),launcher=lambda *a,**k:self.fail('Unexpected adapter'))

    def test_system_mutations_granted_after_bus_discovery_and_stale_blocks_call(self):
        bus=Mock();events=[]
        c=Controls(authorize=lambda:events.append('grant'))
        with patch('system_controls.Gio.bus_get_sync',side_effect=lambda *a:events.append('discover') or bus):
            c.call(False,'name','/path','interface','Lock')
        self.assertEqual(events,['discover','grant'])
        c.authorize=abort;bus.call_sync.reset_mock()
        with self.assertRaisesRegex(ValueError,'Stale'):c.call(False,'name','/path','interface','Set')
        bus.call_sync.assert_not_called()

    def test_power_confirmation_and_dnd_do_not_write_after_stale_discovery(self):
        for command in ('shutdown','reboot','dnd'):
            c=Controls(authorize=abort);c.state=Mock(return_value={'available':True,'active':False});setting=Mock();c.setting=Mock(return_value=setting)
            with patch('subprocess.Popen') as spawn,patch('system_controls.Gio.Settings.sync'):
                with self.assertRaisesRegex(ValueError,'Stale'):c.execute(command)
            spawn.assert_not_called();setting.set_boolean.assert_not_called()

    def test_partial_multiwrite_then_abort_is_uncertain_and_helper_ends(self):
        def execute(action,source,output,*,controls,grant):
            grant();grant()
        source=io.StringIO('{"type":"system","command":"night_light"}\n{"decision":"apply"}\n{"decision":"abort"}\n')
        output=io.StringIO()
        with patch.object(shared,'execute',execute):shared.serve(source,output)
        self.assertEqual(json.loads(output.getvalue().splitlines()[-1]),{'status':'error','code':'uncertain'})

if __name__=='__main__':unittest.main()
