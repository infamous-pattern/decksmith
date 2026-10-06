"""Exercise generic assignments through the private IPC socket, without devices."""
import asyncio
import collections
import json
import os
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock,patch
from hardware_bridge import HardwareBridge
from test_accessory_actions import FakeCatalog,FakeHost,binding

class AccessoryBridgeTests(unittest.IsolatedAsyncioTestCase):
 async def asyncSetUp(self):
  self.directory=tempfile.TemporaryDirectory()
  self.environment=patch.dict(os.environ,{'XDG_RUNTIME_DIR':self.directory.name});self.environment.start()
  c=FakeCatalog();h=FakeHost(c);h.stop=AsyncMock()
  self.p=SimpleNamespace(catalog=c,host=h,assigned_states={},busy=False,enabled=True,
   recovery_required=False,manual_recovery=False,trace=collections.deque(maxlen=16))
  self.p.assigned_ready=lambda:not self.p.manual_recovery
  self.p.snapshot=lambda:{'ready':self.p.assigned_ready(),'actual':None}
  self.bridge=HardwareBridge(self.p);await self.bridge.start()
 async def asyncTearDown(self):
  await self.bridge.close();self.environment.stop();self.directory.cleanup()
 async def request(self,payload):
  r,w=await asyncio.open_unix_connection(self.bridge.path)
  try:
   w.write(json.dumps(payload).encode()+b'\n');await w.drain()
   return json.loads(await asyncio.wait_for(r.readline(),2))
  finally:w.close();await w.wait_closed()
 def payload(self):return {'binding':binding('toggle'),'ticks':0}
 async def test_rejected_command_returns_false_then_new_input_works(self):
  original=self.p.host.input
  async def rejected(c,*args):
   self.p.host.writes.append(self.p.host.settings.copy());self.p.host.completions[c]='failed';return False
  self.p.host.input=rejected
  self.assertEqual(await self.request(self.payload()),{'ok':False})
  self.assertEqual(len(self.p.host.writes),1);self.assertTrue(self.p.assigned_ready())
  self.p.host.input=original
  self.assertEqual(await self.request(self.payload()),{'ok':True})
  self.assertEqual(len(self.p.host.writes),2)
 async def test_eof_cancels_write_and_blocks_new_input(self):
  entered=asyncio.Event()
  async def pending(*args):
   self.p.host.writes.append(self.p.host.settings.copy());entered.set();await asyncio.Event().wait()
  self.p.host.input=pending
  r,w=await asyncio.open_unix_connection(self.bridge.path)
  w.write(json.dumps(self.payload()).encode()+b'\n');await w.drain();await asyncio.wait_for(entered.wait(),2)
  w.close();await w.wait_closed()
  async with asyncio.timeout(2):
   while not self.p.manual_recovery:await asyncio.sleep(.01)
  self.assertEqual(await self.request(self.payload()),{'ok':False})
  self.assertEqual(len(self.p.host.writes),1);self.p.host.stop.assert_awaited_once()
 async def test_busy_and_malformed_requests_never_write_or_log_payload(self):
  self.p.busy=True
  with self.assertLogs('hardware_bridge',level='WARNING') as logs:
   self.assertEqual(await self.request(self.payload()),{'ok':False})
   self.assertEqual(await self.request({'private':'SECRET_TOKEN private-server'}),{'ok':False})
  self.assertEqual(self.p.host.writes,[])
  self.assertIn('reason=busy',str(logs.output));self.assertIn('reason=invalid_input',str(logs.output))
  for value in ('SECRET_TOKEN','private-server',binding('toggle')['settings']['accessoryId']):
   self.assertNotIn(value,str(logs.output))
  self.assertEqual(Path(self.bridge.path).stat().st_mode&0o777,0o600)
