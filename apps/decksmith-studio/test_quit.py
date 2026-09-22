import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch
from panel import Panel

class QuitTests(unittest.TestCase):
    def test_quit_uses_normal_close_guard(self):
        ed=Mock(pending=False);owner=SimpleNamespace(editor=ed,quit_all=False)
        Panel.quit_requested(owner)
        self.assertTrue(owner.quit_all);ed.present.assert_called_once();ed.close.assert_called_once()

    def test_stop_precedes_close_and_does_not_disable_login(self):
        ed=Mock();owner=SimpleNamespace(editor=ed,quit_all=True)
        ed.request.side_effect=lambda task,done,message:done(task())
        with patch('panel.subprocess.run') as run:
            Panel.finish_quit(owner)
        self.assertEqual(run.call_args.args[0],['systemctl','--user','stop','decksmith.service'])
        self.assertFalse(owner.quit_all);ed.close.assert_called_once()

    def test_failed_stop_does_not_close(self):
        ed=Mock();owner=SimpleNamespace(editor=ed,quit_all=True)
        ed.request.side_effect=lambda task,done,message:done(task())
        with patch('panel.subprocess.run',side_effect=RuntimeError('unavailable')):
            with self.assertRaises(RuntimeError):Panel.finish_quit(owner)
        ed.close.assert_not_called()
