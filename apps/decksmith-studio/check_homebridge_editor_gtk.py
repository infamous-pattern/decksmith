"""Fixture-only integration check; no daemon writes or accessory commands."""
import json
from pathlib import Path
from panel import Panel,GLib
from editor import Editor
from homebridge_picker import binding
app=Panel();app.set_application_id('cc.senecal.Decksmith.HomebridgeEditorCheck');errors=[]
def exercise():
 if editor.draft is None or editor.pending:return True
 try:
  item={'id':'a'*64,'name':'Office Socket','kind':'Socket','icon':'power-plug-symbolic','operations':['toggle','on','off','status']}
  editor.select_key(0);editor.assign_homebridge(item,binding(item,'toggle'))
  key=editor.draft.data['pages'][editor.page]['keys'][0]
  assert key['label']=='Office Socket' and key['plugin']['schema']==2
  editor.label.set_text('My Socket');assert key['label']=='My Socket' and key['plugin']['schema']==2
  editor.draft.validate()
  editor.touch_buttons[0].emit('clicked')
  fan={**item,'name':'Ceiling Fan','kind':'Fan','operations':['toggle','on','off','level','status']}
  editor.dial_controls.assign_homebridge(fan,binding(fan,'level'))
  assert editor.dial_controls.data[0]['plugin_rotation']['schema']==2
  editor.dial_controls.label.set_text('My Fan');editor.draft.validate()
  editor.dial_controls.remove_homebridge.emit('clicked')
  assert 'plugin_rotation' not in editor.dial_controls.data[0]
  editor.select_key(0);editor.action.set_selected(1)
  # No action was already selected; explicit Remove always clears the plugin.
  editor.remove_plugin_assignment();assert 'plugin' not in key
  print('PASS: full editor key/dial assignment, custom labels, validation and removal',flush=True)
 except BaseException as e:errors.append(e)
 app.quit();return False
def activate(_):
 global editor
 source=(Path(__file__).resolve().parents[2]/'config/audio.json').read_text()
 editor=Editor(app,lambda method,*_args:(json.dumps({'layout':'audio','active_page':1,'display_ready':True}) if method=='GetStatus' else source,))
 GLib.timeout_add(500,exercise)
app.connect('activate',activate);app.run(None)
if errors:raise errors[0]
