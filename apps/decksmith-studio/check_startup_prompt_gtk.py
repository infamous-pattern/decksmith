"""Native launch prompt check with fake status and no service/device writes."""
import json,sys,time,traceback
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'apps/decksmith-studio'))
import panel,startup,workspace_shell
from panel import Gtk,GLib,Adw
startup.read=lambda:{'enabled':False,'available':True,'detail':'Disabled'}
workspace_shell.WorkspaceShell.plugins=lambda self:Adw.StatusPage(title='Plugins')
layout=Path('config/audio.json').read_text()
errors=[]
for choice in ('later','start'):
 status={'running':False,'connected':False}
 panel.read_status=lambda:dict(status)
 def call(method,args=None):
  if method=='GetStatus':return (json.dumps(status),)
  if method=='GetLayout':return (layout,)
  if method=='PreviewKeys':return bytes(8*120*120*3)
  if method=='PreviewTouch':return bytes(800*100*3),'draft'
  raise AssertionError('Unexpected write: '+method)
 panel.call=call
 app=panel.Panel();app.set_application_id('cc.senecal.Decksmith.StartPromptTest'+choice)
 actions=[]
 def action(value):
  actions.append(value);status.update(running=True,connected=True,display_ready=True,layout='audio');app.show_status(dict(status))
 app.service_action=action
 began=time.monotonic();stage=[0]
 def tick():
  try:
   if time.monotonic()-began>12:raise AssertionError('Prompt timed out')
   if stage[0]==0:
    if app.launch_prompt is None:return True
    dialog=app.launch_prompt
    assert dialog.get_heading()=='Start background controls?'
    assert dialog.get_mapped()
    dialog.emit('response',choice);dialog.close();stage[0]=1;return True
   assert actions==(['start'] if choice=='start' else [])
   app.show_status({'running':False,'connected':False})
   assert app.launch_prompt is None
   print('PASS native prompt:',choice,flush=True);app.quit();return False
  except Exception as e:
   traceback.print_exc();errors.append(str(e));app.quit();return False
 app.connect('activate',lambda *_:GLib.timeout_add(500,tick))
 app.run(None)
if errors:raise SystemExit('; '.join(errors))
