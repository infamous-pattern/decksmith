import unittest
from concurrent.futures import Future
from types import SimpleNamespace
from unittest.mock import patch
from panel import Panel

class Pool:
    def __init__(self):self.future=Future();self.calls=0
    def submit(self,*args):self.calls+=1;return self.future

class Widget:
    def __init__(self):self.sensitive=True
    def set_sensitive(self,value):self.sensitive=value

class PanelRefreshTests(unittest.TestCase):
    def fixture(self):
        seen=[]
        obj=SimpleNamespace(busy=False,refreshing=False,status_revision=0,running=True,status_pool=Pool(),pool=Pool(),show_status=seen.append)
        for name in ('service_button','layout_row','edit_button','apply'):setattr(obj,name,Widget())
        return obj,seen
    def test_poll_keeps_edit_enabled_and_coalesces(self):
        panel,seen=self.fixture()
        with patch('panel.GLib.idle_add',side_effect=lambda callback:callback()):
            Panel.refresh(panel);Panel.refresh(panel)
            self.assertTrue(panel.edit_button.sensitive)
            self.assertFalse(panel.busy)
            self.assertEqual(panel.status_pool.calls,1)
            panel.status_pool.future.set_result({'running':True})
        self.assertEqual(seen,[{'running':True}]);self.assertFalse(panel.refreshing)
    def test_user_action_during_poll_is_not_dropped_or_overwritten(self):
        panel,seen=self.fixture()
        with patch('panel.GLib.idle_add',side_effect=lambda callback:callback()):
            Panel.refresh(panel)
            Panel.submit(panel,lambda:None,seen.append)
            self.assertEqual(panel.pool.calls,1)
            panel.pool.future.set_result({'new':True})
            panel.status_pool.future.set_result({'old':True})
        self.assertEqual(seen,[{'new':True}])
        self.assertFalse(panel.busy)

class LaunchPromptTests(unittest.TestCase):
    def test_first_reliable_running_status_does_not_prompt_later(self):
        panel=SimpleNamespace(launch_status_checked=False)
        with patch('panel.Adw.AlertDialog') as dialog:
            Panel.offer_start_at_launch(panel,{'running':False,'unavailable':True})
            self.assertFalse(panel.launch_status_checked)
            Panel.offer_start_at_launch(panel,{'running':True})
            Panel.offer_start_at_launch(panel,{'running':False})
            dialog.assert_not_called()
    def test_stopped_prompt_is_once_and_start_never_toggles_to_stop(self):
        for choice,expected in [('later',[]),('start',['start'])]:
            actions=[]
            panel=SimpleNamespace(launch_status_checked=False,window=object(),service_action=actions.append)
            with patch('panel.Adw.AlertDialog') as factory:
                dialog=factory.return_value
                Panel.offer_start_at_launch(panel,{'running':False})
                Panel.offer_start_at_launch(panel,{'running':False})
                factory.assert_called_once()
                dialog.present.assert_called_once_with(panel.window)
                callback=dialog.connect.call_args.args[1]
                panel.running=True # Another client may have started the service meanwhile.
                callback(dialog,choice)
                self.assertEqual(actions,expected)
                self.assertIsNone(panel.launch_prompt)
