import unittest
from accessory_catalog import describe,caption

def service(kind,chars):return {'type':kind,'uniqueId':'a'*64,'serviceName':'Example','serviceCharacteristics':chars}
def char(kind,value=0,write=True,fmt='bool',**extra):return {'type':kind,'value':value,'perms':['pr']+(['pw'] if write else []),'format':fmt,**extra}
class CatalogTests(unittest.TestCase):
 def test_lights_and_sockets_default_toggle(self):
  for kind in ('Lightbulb','Outlet'):
   i=describe(service(kind,[char('On')]))
   self.assertEqual(i['default'],'toggle');self.assertNotIn('level',i['operations'])
 def test_read_only_power_cannot_be_controlled(self):
  i=describe(service('Outlet',[char('On',write=False)]));self.assertEqual(i['operations'],['status'])
 def test_fan_has_independent_speed_capability(self):
  i=describe(service('Fanv2',[char('Active',fmt='uint8'),char('RotationSpeed',50,fmt='float',minValue=0,maxValue=100,minStep=8.3333)]))
  self.assertEqual(i['power'],'Active');self.assertIn('level',i['operations'])
 def test_camera_writable_properties_are_status_only(self):
  i=describe(service('CameraOperatingMode',[char('HomeKitCameraActive',1)]));self.assertEqual(i['operations'],['status'])
  self.assertIsNone(describe(service('CameraRTPStreamManagement',[char('Active',1)])))
 def test_sensor_status(self):
  i=describe(service('MotionSensor',[char('MotionDetected',1,write=False)]));self.assertEqual(caption(i),'Motion');self.assertEqual(i['operations'],['status'])
 def test_unknown_or_invalid_characteristics_are_not_actions(self):
  self.assertIsNone(describe(service('Unknown',[char('On')])))
  i=describe(service('Lightbulb',[char('On'),char('Brightness',50,fmt='float',minValue=0,maxValue=float('nan'))]));self.assertNotIn('level',i['operations'])
if __name__=='__main__':unittest.main()
