"""Native setup workflow with invented credentials and no network."""
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Adw,Gtk,GLib
import plugin_lab_page as module
class Fake:
 def __init__(self,path):self.server='http://old:8581';self.result={'state':'idle'}
 def request(self,command=None):
  return {'enabled':True,'server':self.server,'username':'','connection':'connected','device_count':1,'busy':False,'setup':dict(self.result)}
 def setup(self,data):
  if data['operation']=='test':
   self.result={'state':'tested','message':'Verified','server':data['server'],'username':data['username'],'candidate':'fixture'}
  else:self.server=self.result['server'];self.result={'state':'saved','message':'Saved'}
  return {'ok':True}
module.LabClient=Fake
app=Adw.Application(application_id='cc.senecal.Decksmith.ConnectionSetupCheck');errors=[];stage=0
def activate(app):
 global window,page
 window=Adw.ApplicationWindow(application=app,default_width=850,default_height=800);page=module.PluginLabPage('/unused');window.set_content(page);window.present();GLib.timeout_add(100,tick)
def tick():
 global stage
 if page.pending or page.state is None:return True
 try:
  if stage==0:
   page.setup_section.set_expanded(True);page.address.set_text('http://new:8581');page.username.set_text('fixture');page.password.set_text('invented password');page.test_connection.emit('clicked');stage=1
  elif stage==1:
   assert page.save_connection.get_sensitive();assert page.server.get_subtitle()=='http://old:8581'
   page.password.set_text('changed invented password');page.complete(page.state)
   assert not page.save_connection.get_sensitive(),'Edited credentials must invalidate old test'
   page.test_connection.emit('clicked');stage=2
  elif stage==2:
   assert page.save_connection.get_sensitive();page.save_connection.emit('clicked');stage=3
  else:
   assert page.server.get_subtitle()=='http://new:8581';assert page.password.get_text()==''
   page.complete(dict(page.state,setup={'state':'two_factor','message':'Enter code'}));assert page.otp.get_visible()
   page.close();assert page.password.get_text()=='' and page.otp.get_text()==''
   print('PASS: read-only test, stale test invalidation, explicit save, password clearing and two-factor prompt',flush=True);app.quit();return False
 except Exception as e:errors.append(repr(e));page.close();app.quit();return False
 return True
app.connect('activate',activate);app.run(None)
if errors:raise SystemExit(str(errors))
