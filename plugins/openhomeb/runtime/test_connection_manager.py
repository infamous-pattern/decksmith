import asyncio
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock,patch
from connection_preferences import load,save
from live_panel import LivePanel

class Preferences(unittest.TestCase):
 def test_persist_and_reject_symlink(self):
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'setting.json';self.assertTrue(load(p));save(p,False);self.assertFalse(load(p));self.assertEqual(p.stat().st_mode&0o777,0o600)
   save(p,True);self.assertTrue(load(p));link=Path(d)/'link';link.symlink_to(p)
   with self.assertRaises(OSError):load(link)
   with self.assertRaises(ValueError):save(link,False)

class Manager(unittest.IsolatedAsyncioTestCase):
 async def asyncSetUp(self):
  # Patch manifest location only; no child process or network is started.
  self.p=LivePanel();self.tmp=tempfile.TemporaryDirectory();self.p.preference_path=Path(self.tmp.name)/'setting.json'
  self.p.host.stop=AsyncMock();self.p.connect=AsyncMock();self.p.catalog.refresh=lambda:None
 async def asyncTearDown(self):self.p.temp.cleanup();self.tmp.cleanup()
 async def test_disable_preserves_state_and_blocks_every_input(self):
  self.p.actual={'on':True,'brightness':70};self.p.assigned_states={'test':{'text':'On'}}
  self.assertTrue(await self.p.perform('disable'));self.assertFalse(load(self.p.preference_path));self.assertFalse(self.p.enabled)
  self.assertEqual(self.p.actual,{'on':True,'brightness':70});self.assertEqual(self.p.assigned_states,{})
  for command in ('on','off','up','down','reconnect'):
   self.assertFalse((await self.p.command({'command':command}))['ok'])
  self.assertEqual(self.p.connection_status(),'disabled');self.assertFalse(self.p.assigned_ready())
 async def test_enable_connects_readonly_and_persists(self):
  self.p.enabled=False
  self.assertTrue(await self.p.perform('enable'));self.assertTrue(load(self.p.preference_path));self.p.connect.assert_awaited_once()
 async def test_disabled_start_never_connects(self):
  self.p.enabled=False
  await self.p.start();await asyncio.sleep(.05)
  self.p.connect.assert_not_awaited();self.assertIsNone(self.p.host.process)
  await self.p.close()
 async def test_offline_start_keeps_manager_available(self):
  self.p.connect=AsyncMock(side_effect=OSError('offline'))
  await self.p.start();await asyncio.sleep(.05)
  self.assertEqual(self.p.connection_status(),'disconnected')
  await self.p.close()
 async def test_busy_rejects_disable(self):
  self.p.busy=True;self.assertFalse((await self.p.command({'command':'disable'}))['ok']);self.assertFalse(self.p.preference_path.exists())
 async def test_unconfirmed_has_priority_over_connected(self):
  self.p.manual_recovery=True;self.assertEqual(self.p.connection_status(),'unconfirmed')
 async def test_preference_failure_does_not_claim_disabled(self):
  with patch('connection_preferences.save',side_effect=OSError):self.assertFalse(await self.p.perform('disable'))
  self.assertTrue(self.p.enabled);self.assertEqual(self.p.last_error,'settings_error')

class RefreshScheduling(unittest.IsolatedAsyncioTestCase):
 async def test_busy_actions_do_not_starve_catalog_reads(self):
  p=LivePanel();p.busy=True
  with patch('asyncio.to_thread',new=AsyncMock(return_value=None)) as read:
   task=asyncio.create_task(p.poll_catalog())
   await asyncio.sleep(.01)
   self.assertEqual(read.await_count,1)
   task.cancel();await asyncio.gather(task,return_exceptions=True)
  p.temp.cleanup()
 async def test_old_refresh_cannot_restore_removed_or_replaced_state(self):
  from accessory_catalog import Catalog
  for removed in (True,False):
   p=LivePanel();p.assigned_states={};old=p.catalog;old.states=lambda:{'old':{'text':'On'}}
   entered=asyncio.Event();release=asyncio.Event()
   async def slow_read(*_):entered.set();await release.wait()
   with patch('asyncio.to_thread',new=slow_read):
    task=asyncio.create_task(p.poll_catalog());await entered.wait()
    if removed:p.enabled=False
    else:p.catalog=Catalog('http://new')
    release.set();await asyncio.sleep(.01)
    self.assertEqual(p.assigned_states,{})
    task.cancel();await asyncio.gather(task,return_exceptions=True)
   p.temp.cleanup()

if __name__=='__main__':unittest.main()
