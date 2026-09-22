import unittest
from unittest.mock import Mock,patch
from system_controls import Controls,COMMANDS,LABELS
from system_confirm import confirmed_power
class SystemControlTests(unittest.TestCase):
 def test_unknown_command_rejected_before_bus_access(self):
  c=Controls();c.call=Mock()
  with self.assertRaises(ValueError):c.execute('run_shell')
  c.call.assert_not_called()
 def test_power_actions_only_open_confirmation(self):
  for command in ('reboot','shutdown'):
   c=Controls();c.state=Mock(return_value={'available':True});c.call=Mock()
   with patch('system_controls.Gio.Settings.sync'),patch('subprocess.Popen') as spawn:c.execute(command)
   self.assertEqual(spawn.call_args.args[0][-1],command);c.call.assert_not_called()
 def test_inhibitor_blocks_confirmed_power(self):
  c=Mock();c.call.return_value=(True,)
  with self.assertRaises(RuntimeError):confirmed_power('reboot',c)
  self.assertEqual(c.call.call_count,1)
 def test_confirmed_power_uses_nonforced_authorized_method(self):
  c=Mock();c.call.side_effect=[(False,),()];confirmed_power('shutdown',c)
  self.assertEqual(c.call.call_args.args[4],'PowerOff');self.assertEqual(c.call.call_args.args[5].unpack(),(True,))
 def test_labels_and_icons_complete(self):
  from system_artwork import image
  self.assertEqual(set(LABELS),COMMANDS)
  for command,label in LABELS.items():
   self.assertLessEqual(len(label),24);self.assertTrue(image(command).startswith(b'\x89PNG'))

 def test_assignment_inherits_theme_and_preserves_custom_styles(self):
  from system_artwork import populate
  from themes import effective
  key={'action':{'type':'system','command':'lock'},'label':'Lock'}
  populate(key)
  self.assertNotIn('label_position',key)
  self.assertNotIn('label_background',key)
  layout={'theme':{'preset':'light','appearance':{'label_position':'top','label_background':'dark'}}}
  style=effective(layout,key)
  self.assertEqual(style['icon_size'],75)
  self.assertEqual(style['background'],[236,240,245])
  self.assertEqual(style['label_position'],'top')
  self.assertEqual(style['label_background'],'dark')
  layout['theme']['appearance']['icon_size']=60
  self.assertEqual(effective(layout,key)['icon_size'],60)
  key.update(label_position='bottom',background_color='blue',appearance={'icon_size':50})
  populate(key)
  style=effective(layout,key)
  self.assertEqual(style['icon_size'],50)
  self.assertEqual(style['label_position'],'bottom')
  self.assertEqual(style['background'],[90,170,255])
