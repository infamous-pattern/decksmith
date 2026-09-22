"""Native inline confirmation, cancellation and preserved-assignment review."""
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Adw,GLib
import plugin_lab_page as module
import plugin_review
calls=[]
class Fake:
 def __init__(self,path):self.removed=False
 def request(self,command=None):
  return dict(enabled=not self.removed,server='' if self.removed else 'http://fixture:8581',username='',has_saved_password=not self.removed,connection='not_configured' if self.removed else 'connected',device_count=0 if self.removed else 1,busy=False,setup={'state':'removed' if self.removed else 'idle','message':'Connection removed. Assignments preserved.'})
 def setup(self,data):calls.append(data);self.removed=True;return {'ok':True}
module.LabClient=Fake
plugin_review.read_review=lambda client:[('Home · Key 1','Lamp — Toggle power'+(' · Connection unavailable' if client.removed else ''))]
app=Adw.Application(application_id='cc.senecal.Decksmith.RemovalCheck');errors=[];stage=0

def activate(app):
 global window,page
 window=Adw.ApplicationWindow(application=app,default_width=900,default_height=850)
 page=module.PluginLabPage('/unused');window.set_content(page);window.present();GLib.timeout_add(100,tick)
def tick():
 global stage
 if page.pending or page.state is None:return True
 try:
  if stage==0:
   assert len(page.review_rows)==1
   page.remove_button.emit('clicked');assert page.removal_box.get_visible()
   assert not page.delete_password.get_active()
   page.cancel_remove.emit('clicked');assert not calls and not page.removal_box.get_visible()
   page.remove_button.emit('clicked');page.delete_password.set_active(True)
   page.confirm_remove.emit('clicked');stage=1
  elif stage==1:
   assert calls==[{'operation':'remove','confirm':True,'delete_password':True}]
   assert page.health.get_subtitle()=='Not configured'
   assert not page.toggle.get_sensitive() and not page.reconnect.get_sensitive()
   assert not page.removal_box.get_visible() and page.address.get_text()==''
   page.fetch(review=True);stage=2
  else:
   assert len(page.review_rows)==1 and 'Connection unavailable' in page.review_rows[0].get_subtitle()
   assert page.test_connection.get_sensitive()
   print('PASS: inline Cancel sends nothing; confirmed removal, optional password deletion and preserved unavailable assignments',flush=True)
   page.close();app.quit();return False
 except Exception as e:errors.append(repr(e));page.close();app.quit();return False
 return True
app.connect('activate',activate);app.run(None)
if errors:raise SystemExit(str(errors))
