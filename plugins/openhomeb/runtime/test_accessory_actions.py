import asyncio,collections,copy,json,unittest
from types import SimpleNamespace
from accessory_actions import perform,decode
from accessory_catalog import describe
ID='a'*64

def binding(op):return {'provider':'com.infamous-pattern.openhomeb','action':'com.infamous-pattern.openhomeb.'+op,'schema':2,'settings':{'accessoryId':ID}}
class FakeCatalog:
 def __init__(self,kind='Outlet',power='On',value=0):
  self.raw={'type':kind,'uniqueId':ID,'serviceName':'Test','serviceCharacteristics':[{'type':power,'format':'bool' if power=='On' else 'uint8','value':value,'perms':['pr','pw']}]}
 def one(self,identity):return describe(self.raw),copy.deepcopy(self.raw)
class FakeHost:
 def __init__(self,catalog):
  self.catalog=catalog;self.saved={'instances':{'assigned':{'accessoryId':'stale'}}};self.completions={};self.epoch=1;self.writes=[]
  self.ready=asyncio.Event();self.ready.set();self.auth_state='authenticated';self.outcome_unknown=False
 async def disappear(self,c):pass
 async def appear(self,c,a,s):
  assert c not in self.saved['instances'],'stale settings retained';self.settings=s
 async def input(self,c,e,epoch):
  self.writes.append(self.settings.copy())
  for char in self.catalog.raw['serviceCharacteristics']:
   if char['type']==self.settings['characteristicType']:char['value']=self.settings['targetValue']
  self.completions[c]='accepted';return True
class ActionTests(unittest.IsolatedAsyncioTestCase):
 def panel(self,c):return SimpleNamespace(catalog=c,host=FakeHost(c),assigned_states={},busy=True,recovery_required=False,manual_recovery=False,trace=collections.deque(maxlen=16))
 async def test_toggle_reads_current_state_and_never_reuses_previous_settings(self):
  p=self.panel(FakeCatalog())
  self.assertTrue(await perform(p,binding('toggle'),0));self.assertTrue(await perform(p,binding('toggle'),0))
  self.assertEqual([w['targetValue'] for w in p.host.writes],[True,False])
 async def test_readonly_sensor_never_writes(self):
  c=FakeCatalog('MotionSensor','MotionDetected',1);c.raw['serviceCharacteristics'][0]['perms']=['pr'];p=self.panel(c)
  self.assertTrue(await perform(p,binding('status'),0));self.assertFalse(await perform(p,binding('toggle'),0));self.assertEqual(p.host.writes,[])
 async def test_fan_uses_active_and_its_native_speed_step(self):
  c=FakeCatalog('Fanv2','Active',1);c.raw['serviceCharacteristics'].append({'type':'RotationSpeed','format':'float','perms':['pr','pw'],'value':50,'minValue':0,'maxValue':100,'minStep':100/12});p=self.panel(c)
  self.assertTrue(await perform(p,binding('level'),1));self.assertAlmostEqual(p.host.writes[0]['targetValue'],100*7/12)
  self.assertTrue(await perform(p,binding('off'),0));self.assertEqual(p.host.writes[-1]['characteristicType'],'Active');self.assertEqual(p.host.writes[-1]['targetValue'],0)
 async def test_level_does_not_turn_on_off_light(self):
  c=FakeCatalog('Lightbulb');c.raw['serviceCharacteristics'].append({'type':'Brightness','format':'int','perms':['pr','pw'],'value':50,'minValue':0,'maxValue':100});p=self.panel(c)
  self.assertTrue(await perform(p,binding('level'),1));self.assertEqual(p.host.writes,[])
 async def test_lost_confirmation_pauses_without_retry(self):
  p=self.panel(FakeCatalog());original=p.host.input
  async def lost(*args):
   await original(*args);raise RuntimeError('lost acknowledgement')
  p.host.input=lost
  self.assertFalse(await perform(p,binding('toggle'),0))
  self.assertEqual(len(p.host.writes),1);self.assertTrue(p.manual_recovery);self.assertFalse(p.busy)
  self.assertEqual(p.trace[-1]['result'],'unconfirmed')
 async def test_terminal_rejection_refreshes_without_retry_and_allows_new_input(self):
  p=self.panel(FakeCatalog());original=p.host.input
  async def rejected(c,*args):
   p.host.writes.append(p.host.settings.copy());p.host.completions[c]='failed';return False
  p.host.input=rejected
  self.assertFalse(await perform(p,binding('toggle'),0))
  self.assertEqual(len(p.host.writes),1);self.assertFalse(p.manual_recovery);self.assertFalse(p.recovery_required)
  self.assertFalse(p.assigned_states[ID]['on']);self.assertEqual(p.trace[-1]['result'],'rejected_refreshed')
  p.host.input=original
  self.assertTrue(await perform(p,binding('toggle'),0))
  self.assertEqual(len(p.host.writes),2);self.assertTrue(p.assigned_states[ID]['on'])
 async def test_rejection_with_failed_read_stays_blocked_without_retry(self):
  p=self.panel(FakeCatalog())
  async def rejected(c,*args):
   p.host.writes.append(p.host.settings.copy());p.host.completions[c]='failed'
   def offline(identity):raise OSError('private server detail')
   p.catalog.one=offline;return False
  p.host.input=rejected
  self.assertFalse(await perform(p,binding('toggle'),0))
  self.assertTrue(p.manual_recovery);self.assertEqual(len(p.host.writes),1)
  self.assertEqual(p.trace[-1]['result'],'rejected_unavailable')
 async def test_stale_rejection_or_unhealthy_host_does_not_release_gate(self):
  for fault in ('epoch','unknown','offline','auth'):
   with self.subTest(fault=fault):
    p=self.panel(FakeCatalog())
    async def failed(c,*args):
     p.host.writes.append(p.host.settings.copy());p.host.completions[c]='failed'
     if fault=='epoch':p.host.epoch+=1
     if fault=='unknown':p.host.outcome_unknown=True
     if fault=='offline':p.host.ready.clear()
     if fault=='auth':p.host.auth_state='required'
     return False
    p.host.input=failed
    self.assertFalse(await perform(p,binding('toggle'),0));self.assertTrue(p.manual_recovery)
    self.assertEqual(len(p.host.writes),1);self.assertEqual(p.trace[-1]['result'],'unconfirmed')
 async def test_host_change_during_rejection_read_keeps_gate_closed(self):
  p=self.panel(FakeCatalog());read=p.catalog.one
  async def rejected(c,*args):
   p.host.writes.append(p.host.settings.copy());p.host.completions[c]='failed'
   def changed(identity):
    result=read(identity);p.host.epoch+=1;return result
   p.catalog.one=changed;return False
  p.host.input=rejected
  self.assertFalse(await perform(p,binding('toggle'),0));self.assertTrue(p.manual_recovery)
  self.assertEqual(len(p.host.writes),1);self.assertEqual(p.trace[-1]['result'],'rejected_unavailable')
 async def test_cancelled_write_remains_uncertain(self):
  p=self.panel(FakeCatalog());entered=asyncio.Event()
  async def pending(*args):
   p.host.writes.append(p.host.settings.copy());entered.set();await asyncio.Event().wait()
  p.host.input=pending;task=asyncio.create_task(perform(p,binding('toggle'),0))
  await entered.wait();task.cancel()
  with self.assertRaises(asyncio.CancelledError):await task
  self.assertTrue(p.manual_recovery);self.assertFalse(p.busy);self.assertEqual(len(p.host.writes),1)
 async def test_diagnostics_are_bounded_and_exclude_accessory_and_exception_details(self):
  p=self.panel(FakeCatalog())
  def offline(identity):raise OSError('SECRET_TOKEN http://private-server/ alice')
  p.catalog.one=offline
  with self.assertLogs('accessory_actions',level='WARNING') as logs:
   for _ in range(20):self.assertFalse(await perform(p,binding('toggle'),0))
  self.assertEqual(len(p.trace),16);self.assertEqual(p.host.writes,[])
  evidence=json.dumps(list(p.trace))+str(logs.output)
  for private in (ID,'Test','SECRET_TOKEN','private-server','alice'):self.assertNotIn(private,evidence)
  self.assertIn('phase=read reason=device_unavailable',evidence)
 def test_bad_binding_rejected(self):
  for op,ticks in [('level',0),('toggle',1),('arbitrary',0)]:
   with self.assertRaises(ValueError):decode(binding(op),ticks)
if __name__=='__main__':unittest.main()
