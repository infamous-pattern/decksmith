import unittest
from types import SimpleNamespace
from unittest.mock import patch
import startup

def result(state,load='loaded',code=0):return SimpleNamespace(returncode=code,stdout=f'LoadState={load}\nUnitFileState={state}\n')
class StartupTests(unittest.TestCase):
    def test_transient_missing_and_masked_are_not_off_preferences(self):
        for state,load in [('transient','loaded'),('','not-found'),('masked','masked')]:
            with patch('startup.command',return_value=result(state,load)):
                self.assertFalse(startup.read()['available'])
                with self.assertRaises(RuntimeError):startup.set_enabled(True)
    def test_enable_and_disable_never_start_or_stop_service(self):
        for enabled in (True,False):
            state='enabled' if enabled else 'disabled'
            with patch('startup.command',side_effect=[result('disabled' if enabled else 'enabled'),result(state),result(state)]) as cmd:
                self.assertEqual(startup.set_enabled(enabled)['enabled'],enabled)
                self.assertEqual(cmd.call_args_list[1].args,('enable' if enabled else 'disable',startup.UNIT))
    def test_failed_change_and_failed_read_are_reported(self):
        with patch('startup.command',side_effect=[result('disabled'),result('',code=1)]):
            with self.assertRaises(RuntimeError):startup.set_enabled(True)
        with patch('startup.command',return_value=result('',code=1)):
            with self.assertRaises(RuntimeError):startup.read()
    def test_unconfirmed_state_is_failure(self):
        with patch('startup.command',return_value=result('disabled')):
            with self.assertRaises(RuntimeError):startup.set_enabled(True)
