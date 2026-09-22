import tempfile,time,unittest,os
from pathlib import Path
from unittest.mock import AsyncMock,patch
from connection_preferences import read,write
from live_panel import LivePanel
from accessory_catalog import Catalog

class Removal(unittest.IsolatedAsyncioTestCase):
 async def asyncSetUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.p=LivePanel()
  self.p.preference_path=Path(self.tmp.name)/'decksmith/openhomeb.json'
  self.p.connection_config={'enabled':True,'server':'http://fixture:8581','username':'fixture','credential_profile':'fixture-secret'}
  write(self.p.preference_path,self.p.connection_config)
  self.p.host.stop=AsyncMock();self.p.setup.store.delete=AsyncMock()
  self.p.catalog.token='invented-token';self.p.catalog.password='invented-password'
  self.p.read=lambda:None
 async def asyncTearDown(self):self.p.temp.cleanup();self.tmp.cleanup()
 async def remove(self,delete=False):
  await self.p.setup.remove({'operation':'remove','confirm':True,'delete_password':delete})
 async def test_removal_persists_and_never_reconnects(self):
  await self.remove()
  self.assertEqual(self.p.connection_status(),'not_configured');self.assertFalse(self.p.assigned_ready())
  self.assertIsNone(self.p.catalog.token);self.assertEqual(self.p.catalog.password,'');self.assertIsNone(self.p.read)
  self.p.setup.store.delete.assert_not_awaited();self.assertEqual(read(self.p.preference_path)['credential_profile'],'fixture-secret')
  with patch.dict(os.environ,{'XDG_CONFIG_HOME':self.tmp.name}):other=LivePanel(managed=True)
  other.connect=AsyncMock()
  try:
   await other.start();other.connect.assert_not_awaited();self.assertEqual(other.connection_status(),'not_configured')
   for name in ('enable','reconnect','on','off','up','down'):
    self.assertFalse((await other.command({'command':name}))['ok'])
   self.assertEqual(other.catalog.base,'')
  finally:await other.close()
 async def test_delete_only_current_profile(self):
  await self.remove(True);self.p.setup.store.delete.assert_awaited_once_with('fixture-secret')
  self.assertNotIn('credential_profile',read(self.p.preference_path))
 async def test_delete_failure_stays_removed_and_can_retry(self):
  self.p.setup.store.delete.side_effect=OSError('locked')
  await self.remove(True)
  self.assertEqual(self.p.connection_status(),'not_configured');self.assertIn('could not be confirmed',self.p.setup.result['message'])
  self.assertIn('credential_profile',read(self.p.preference_path))
  self.p.setup.store.delete.side_effect=None
  await self.remove(True);self.assertNotIn('credential_profile',read(self.p.preference_path))
 async def test_write_failure_preserves_connection(self):
  old=dict(self.p.connection_config)
  with patch('connection_setup.write',side_effect=OSError):await self.remove(True)
  self.assertEqual(self.p.connection_config,old);self.assertTrue(self.p.enabled)
  self.p.host.stop.assert_not_awaited();self.p.setup.store.delete.assert_not_awaited()
 async def test_requires_confirmation_and_invalidates_candidate(self):
  self.p.setup.candidate={'id':'old','expires':time.monotonic()+60}
  await self.p.setup.remove({'operation':'remove','confirm':False,'delete_password':True})
  self.assertTrue(self.p.enabled);self.p.host.stop.assert_not_awaited()
  await self.remove();self.assertIsNone(self.p.setup.candidate)
 async def test_explicit_setup_restores_removed_connection(self):
  await self.remove();self.p.connect=AsyncMock()
  self.p.setup.candidate={'id':'new','expires':time.monotonic()+60,'catalog':Catalog('http://replacement:8581')}
  await self.p.setup.apply({'operation':'apply','candidate':'new'})
  self.assertTrue(self.p.enabled);self.assertNotIn('removed',read(self.p.preference_path));self.p.connect.assert_awaited_once()
 async def test_failed_setup_restores_removed_state(self):
  await self.remove();self.p.connect=AsyncMock(side_effect=OSError)
  self.p.setup.candidate={'id':'new','expires':time.monotonic()+60,'catalog':Catalog('http://replacement:8581')}
  await self.p.setup.apply({'operation':'apply','candidate':'new'})
  self.assertFalse(self.p.enabled);self.assertTrue(read(self.p.preference_path)['removed'])
  self.assertEqual(self.p.connection_status(),'not_configured')
if __name__=='__main__':unittest.main()
