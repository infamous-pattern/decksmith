import asyncio
from test_live_panel import LivePanelTests,FaultHandler
from test_host import until
import live_panel

class Unavailable(FaultHandler):
 def do_GET(self):
  if self.server.mode=='offline':return self._json(503,{'error':'fixture unavailable'})
  return super().do_GET()

class Reliability(LivePanelTests):
 async def test_network_outage_recovers_without_writes(self):
  self.server.RequestHandlerClass=Unavailable
  self.panel.recovery_task=asyncio.create_task(self.panel.recover())
  original_pid=self.panel.host.process.pid
  self.server.mode='offline'
  await until(lambda:not self.panel.snapshot()['ready'],10)
  self.assertFalse((await self.panel.command({'command':'up'}))['ok'])
  self.server.mode='normal'
  await until(lambda:self.panel.snapshot()['ready'],20)
  self.assertEqual(self.server.puts,0)
  self.assertEqual(self.panel.host.process.pid,original_pid)
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
