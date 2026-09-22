"""Native picker check using fixtures only; no Homebridge writes."""
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Adw,Gtk,GLib
from homebridge_picker import HomebridgePicker,label,icon_png
calls=[];errors=[]
items=[{'id':'a'*64,'name':'Office Socket','kind':'Socket','icon':'power-plug-symbolic','operations':['toggle','on','off','status']},
 {'id':'b'*64,'name':'Fan','kind':'Fan','icon':'weather-windy-symbolic','operations':['toggle','on','off','level','status']},
 {'id':'c'*64,'name':'Motion','kind':'Motion sensor','icon':'motion-sensor-symbolic','operations':['status']}]
app=Adw.Application(application_id='cc.senecal.Decksmith.PickerCheck')
def activate(app):
 global window,key,dial
 window=Adw.ApplicationWindow(application=app,default_width=700,default_height=700)
 box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL);window.set_content(box)
 group=Adw.PreferencesGroup();box.append(group)
 key=HomebridgePicker(lambda i,b:calls.append((i,b)));dial=HomebridgePicker(lambda i,b:calls.append((i,b)),dial=True)
 group.add(key);group.add(dial)
 for p in (key,dial):p.complete({'available':True,'items':items});p.set_expanded(True)
 window.present();GLib.timeout_add(200,check)
def check():
 try:
  assert key.ops[0]=='toggle';key.apply();assert calls[-1][1]['action'].endswith('.toggle')
  key.search.set_text('Motion');assert key.ops==['status'];key.apply();assert calls[-1][1]['action'].endswith('.status')
  assert len(dial.visible)==1 and dial.ops==['level'];dial.apply()
  saved=calls[-1][1];dial.sync(saved);dial.complete(None);assert dial.saved==saved and not dial.use.get_sensitive()
  assert label('Main_LED’s')=='Main LEDs'
  assert icon_png('weather-windy-symbolic')
  print('PASS: native inline key/dial pickers, capability filtering, defaults, unavailable preservation, icon rendering',flush=True)
 except Exception as e:errors.append(repr(e))
 app.quit();return False
app.connect('activate',activate);app.run(None)
if errors:raise SystemExit(str(errors))
