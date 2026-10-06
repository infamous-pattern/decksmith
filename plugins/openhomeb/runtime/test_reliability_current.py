import asyncio
import copy
import test_live_panel
from test_live_panel import FaultHandler
from test_host import until
from unittest.mock import patch
from accessory_catalog import Catalog
from accessory_actions import perform
from test_accessory_actions import binding,ID
import live_panel

class Unavailable(FaultHandler):
 def do_GET(self):
  if self.server.mode=='offline':return self._json(503,{'error':'fixture unavailable'})
  return super().do_GET()
 def do_PUT(self):
  if self.server.mode=='rejectwrite':
   self.server.rejected_attempts+=1
   return self._json(503,{'error':'fixture rejected'})
  return super().do_PUT()

class Reliability(test_live_panel.LivePanelTests):
 async def asyncSetUp(self):
  await super().asyncSetUp()
  # The current runtime reads through Catalog.one, rather than the old
  # reader() callback. Inject faults into the path it actually uses.
  original=Catalog.one
  def read(catalog,identity):
   if self.read_failed:raise OSError('fixture unavailable')
   return original(catalog,identity)
  self.catalog_fault=patch.object(Catalog,'one',read);self.catalog_fault.start()
 async def asyncTearDown(self):
  try:await super().asyncTearDown()
  finally:self.catalog_fault.stop()
 async def test_network_outage_recovers_without_writes(self):
  self.server.RequestHandlerClass=Unavailable
  self.panel.recovery_task=asyncio.create_task(self.panel.recover())
  self.server.mode='offline'
  await until(lambda:not self.panel.snapshot()['ready'],10)
  self.assertFalse((await self.panel.command({'command':'up'}))['ok'])
  self.server.mode='normal'
  await until(lambda:self.panel.snapshot()['ready'],20)
  self.assertEqual(self.server.puts,0)
  # A whole-server outage can require a supervised reconnect. The contract
  # is restored readiness without replay, not preservation of a child PID.
  self.assertTrue(self.panel.host.ready.is_set())
  self.assertEqual(self.panel.host.auth_state,'authenticated')
 async def test_full_manager_restart_does_not_replay(self):
  await self.command('up');self.assertEqual(self.server.puts,1)
  await self.panel.close()
  self.panel=live_panel.LivePanel();await self.panel.start()
  await until(lambda:self.panel.snapshot()['ready'])
  await asyncio.sleep(2.5)
  self.assertEqual(self.server.puts,1);self.assertEqual(self.panel.actual['brightness'],45)
 async def test_slow_or_broken_read_never_writes(self):
  self.server.mode='malformed'
  self.assertTrue((await self.panel.command({'command':'up'}))['ok'])
  await until(lambda:not self.panel.busy,10)
  self.assertEqual(self.server.puts,0)
  self.server.mode='normal';await self.command('reconnect')
  self.assertTrue(self.panel.snapshot()['ready']);self.assertEqual(self.server.puts,0)
 async def test_plugin_negative_acknowledgement_allows_new_generic_input(self):
  # Exercise the pinned plugin, not just a simulated host completion.
  service=copy.deepcopy(self.server.services[0]);service['uniqueId']=ID
  self.server.services.append(service);self.server.RequestHandlerClass=Unavailable
  self.server.rejected_attempts=0;self.server.mode='rejectwrite'
  self.panel.busy=True
  self.assertFalse(await perform(self.panel,binding('toggle'),0))
  self.assertEqual(self.panel.host.completions['assigned'],'failed')
  self.assertFalse(self.panel.manual_recovery);self.assertEqual(self.server.rejected_attempts,1)
  self.assertEqual(self.server.puts,0)
  self.server.mode='normal';self.panel.busy=True
  self.assertTrue(await perform(self.panel,binding('toggle'),0))
  self.assertEqual(self.server.puts,1);self.assertEqual(self.server.rejected_attempts,1)
