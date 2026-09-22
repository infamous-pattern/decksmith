"""Real loopback HTTP authorization checks; no Homebridge/device access."""
import http.client
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

ROOT=Path(__file__).resolve().parents[1]
SERVER=r'''
import asyncio,sys,types
from pathlib import Path
runtime=Path(sys.argv[1]);sys.path.insert(0,str(runtime))
for name,values in {
 'adjustment_host':{'BINARY':None,'PacedAdjustmentTestHost':object},
 'live_on_off_trial':{'BASE':'http://example.invalid','ROOT':runtime,'TARGET':'test','reader':None},
 'paced_host':{'BRIGHTNESS':'brightness','SET':'set'},
}.items():
 module=types.ModuleType(name);module.__dict__.update(values);sys.modules[name]=module
import live_panel
class Panel:
 def __init__(self,**kwargs):self.poll_task=True;self.enabled=True;self.catalog=self
 async def start(self):pass
 async def close(self):pass
 def snapshot(self):return {'available':True,'test':True}
 async def command(self,data):return {'ok':True}
 async def setup_command(self,data):return {'ok':True}
live_panel.LivePanel=Panel
asyncio.run(live_panel.main(Path(sys.argv[2]),managed=True))
'''

class CompanionHttp(unittest.TestCase):
 def test_token_is_not_disclosed_and_all_state_requires_authentication(self):
  with tempfile.TemporaryDirectory() as tmp:
   descriptor=Path(tmp)/'connection.json'
   process=subprocess.Popen([sys.executable,'-I','-c',SERVER,str(ROOT/'plugins/openhomeb/runtime'),str(descriptor)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
   try:
    deadline=time.monotonic()+8
    while not descriptor.exists() and process.poll() is None and time.monotonic()<deadline:time.sleep(.05)
    self.assertTrue(descriptor.exists(),'Test server did not start')
    info=json.loads(descriptor.read_text());port=int(info['url'].rsplit(':',1)[1]);token=info['token']
    def request(path,headers=None,method='GET'):
     c=http.client.HTTPConnection('127.0.0.1',port,timeout=3)
     try:
      c.request(method,path,body=b'{}' if method=='POST' else None,headers=headers or {})
      r=c.getresponse();return r.status,r.read(),r.getheader('Referrer-Policy')
     finally:c.close()
    for path in ('/','/state','/catalog','/?token=wrong'):
     status,body,_=request(path);self.assertNotEqual(status,200);self.assertNotIn(token.encode(),body)
    headers={'X-Lab-Token':token}
    self.assertEqual(request('/state',headers)[0],200)
    status,body,policy=request('/?token='+token)
    self.assertEqual(status,200);self.assertIn(token.encode(),body);self.assertEqual(policy,'no-referrer')
    self.assertEqual(request('/command',method='POST')[0],403)
    self.assertEqual(request('/command',dict(headers,Origin='https://untrusted.example'),method='POST')[0],403)
    self.assertEqual(request('/command',headers,method='POST')[0],200)
    self.assertEqual(request('/state',dict(headers,Host='untrusted.example'))[0],403)
   finally:
    process.terminate()
    try:process.communicate(timeout=5)
    except subprocess.TimeoutExpired:process.kill();process.communicate()
