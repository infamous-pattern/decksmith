"""Private-bus VirtualDeck check for explicit saved-control testing."""
import json,os,subprocess,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
assert os.environ.get('DECKSMITH_ISOLATED_TEST')=='1'
assert os.environ.get('DBUS_SESSION_BUS_ADDRESS','').startswith('unix:path=/tmp/dbus-')
sys.path.insert(0,str(ROOT/'apps/decksmith-studio'))
from panel import call,GLib
with tempfile.TemporaryDirectory(prefix='decksmith-control-') as tmp:
 env=dict(os.environ,XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',XDG_STATE_HOME=tmp+'/state',XDG_RUNTIME_DIR=tmp)
 log=open(tmp+'/daemon.log','w')
 daemon=subprocess.Popen([str(ROOT/'target/release/decksmithd'),'--virtual-service',str(ROOT/'config/navigation.json'),'--seconds','20'],env=env,stdout=log,stderr=log)
 try:
  def ready(page):
   until=time.monotonic()+10
   while time.monotonic()<until:
    try:
     s=json.loads(call('GetStatus')[0])
     if s['display_ready'] and s['active_page']==page:return
    except Exception:pass
    time.sleep(.05)
   raise RuntimeError('VirtualDeck did not settle')
  ready(0);layout=call('GetLayout')[0]
  call('TestControl',GLib.Variant('(syy)',(layout,0,1)));ready(1)
  altered=json.loads(layout);altered['pages'][1]['keys'][0]['label']='Unsaved'
  try:call('TestControl',GLib.Variant('(syy)',(json.dumps(altered),1,0)))
  except GLib.Error:pass
  else:raise AssertionError('Unsaved action must be rejected')
  assert json.loads(call('GetStatus')[0])['active_page']==1
  assert not (Path(tmp)/'config/decksmith/layout.json').exists()
  print('PASS explicit saved-control navigation; unsaved layout rejected; no layout writes')
 finally:daemon.terminate();daemon.wait(timeout=8);log.close()
