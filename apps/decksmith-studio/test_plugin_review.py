import unittest
from plugin_review import assignments,PROVIDER

def binding(action='toggle'):
 return {'provider':PROVIDER,'schema':2,'action':PROVIDER+'.'+action,'settings':{'accessoryId':'fixture'}}
class Review(unittest.TestCase):
 def test_keys_shared_dials_and_overrides(self):
  dial={'label':'Fan','plugin_rotation':binding('level'),'plugin_press':binding()}
  layout={'dials':[dial,None,None,None],'pages':[
   {'name':'Home','keys':[{'label':'Lamp','plugin':binding()}]},
   {'name':'Work','keys':[],'dial_overrides':[{'label':'Sensor','plugin_press':binding('status')},None,None,None]}]}
  rows=assignments(layout,[{'id':'fixture','name':'Actual device'}],True)
  self.assertEqual(len(rows),4);self.assertEqual(rows[0],('Home · Key 1','Actual device — Toggle power'))
  self.assertIn('Shared',rows[1][0]);self.assertNotIn('Shared',rows[3][0]);self.assertIn('Status only',rows[3][1])
 def test_missing_target_and_removed_connection_keep_assignment(self):
  rows=assignments({'pages':[{'name':'Home','keys':[{'label':'Lamp','plugin':binding()}]}]},[],False)
  self.assertEqual(len(rows),1);self.assertIn('Accessory unavailable',rows[0][1]);self.assertIn('Connection unavailable',rows[0][1])
 def test_other_providers_omitted(self):
  b=binding();b['provider']='other'
  self.assertEqual(assignments({'pages':[{'keys':[{'plugin':b}]}]},[],True),[])
if __name__=='__main__':unittest.main()
