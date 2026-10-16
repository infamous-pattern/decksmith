"""Normal launch selection and cross-device draft protection without live I/O."""
import unittest
from types import SimpleNamespace
from unittest.mock import Mock,patch
from panel import normal_application
from permanent_workspace import PermanentWorkspace

class PermanentWorkspaceTests(unittest.TestCase):
    def test_normal_launch_uses_permanent_only_after_explicit_mode_discovery(self):
        with patch('profile_client.permanent_available',return_value=False),patch('panel.Panel') as old:
            self.assertIs(normal_application(),old.return_value)
        with patch('profile_client.permanent_available',return_value=True),patch('permanent_workspace.PermanentWorkspace') as new:
            self.assertIs(normal_application(),new.return_value)
    def test_other_device_draft_prevents_close_and_offers_exact_owner(self):
        owner=SimpleNamespace(shelf=SimpleNamespace(views={'other':SimpleNamespace(draft=SimpleNamespace(dirty=True))}),selected='selected',settings_drafts={},clients={'other':SimpleNamespace(name='Second Deck')},editor=Mock(),quit_all=True,choose=Mock())
        with patch('permanent_workspace.Adw.AlertDialog') as factory:
            self.assertTrue(PermanentWorkspace.guard_before_close(owner))
            factory.assert_called_once();factory.return_value.present.assert_called_once_with(owner.editor)
            callback=factory.return_value.connect.call_args.args[1];callback(factory.return_value,'review')
        self.assertFalse(owner.quit_all);owner.choose.assert_called_once_with('other');owner.editor.shell.navigate.assert_called_once_with('keys')
    def test_home_settings_are_protected_even_without_layout_draft(self):
        owner=SimpleNamespace(shelf=SimpleNamespace(views={}),selected='selected',settings_drafts={'selected':{}},clients={'selected':SimpleNamespace(name='Desk')},editor=Mock(),quit_all=False,choose=Mock())
        with patch('permanent_workspace.Adw.AlertDialog') as factory:
            self.assertTrue(PermanentWorkspace.guard_before_close(owner))
            factory.return_value.connect.call_args.args[1](factory.return_value,'review')
        owner.choose.assert_not_called();owner.editor.shell.navigate.assert_called_once_with('home')
    def test_clean_devices_do_not_add_another_close_prompt(self):
        owner=SimpleNamespace(shelf=SimpleNamespace(views={}),selected='selected',settings_drafts={})
        with patch('permanent_workspace.Adw.AlertDialog') as dialog:self.assertFalse(PermanentWorkspace.guard_before_close(owner));dialog.assert_not_called()
    def test_uncertain_apply_requires_explicit_restart_without_false_save_claim(self):
        owner=SimpleNamespace(selected='d',clients={'d':SimpleNamespace(token={'runtime':'old-runtime'})})
        text=PermanentWorkspace.request_error_message(owner,RuntimeError('profile_saved_apply_failed'))
        self.assertIn('Layout saved',text);self.assertEqual(owner.restart_runtime,'old-runtime')
        text=PermanentWorkspace.request_error_message(owner,RuntimeError('live_profile_reload_required'))
        self.assertIn('Restart Background controls',text);self.assertNotIn('Layout saved',text)
        self.assertIsNone(PermanentWorkspace.request_error_message(owner,RuntimeError('other error')))
