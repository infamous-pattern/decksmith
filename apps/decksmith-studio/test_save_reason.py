import unittest
from workspace_shell import save_reason
class SaveReasonTests(unittest.TestCase):
 def test_invalid_draft_explanation_survives_connection_changes(self):
  for status in ({'running':True},{'running':False},{'auto_lock':{'locked':True}}):
   self.assertEqual(save_reason(True,False,False,status,'Key 2 needs a URL.'),'Cannot save: Key 2 needs a URL.')
 def test_reason_clears_after_recovery(self):
  self.assertIn('Start Background',save_reason(True,True,False,{'running':False}))
  self.assertIsNone(save_reason(True,True,False,{'running':True}))
  self.assertIn('Unlock',save_reason(True,True,False,{'auto_lock':{'locked':True}}))
  self.assertIn('No changes',save_reason(False,True,False,{}))
  self.assertIn('wait',save_reason(True,True,True,{}))
