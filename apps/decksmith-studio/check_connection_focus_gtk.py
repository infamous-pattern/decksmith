"""Connection drafts retain focus and selection across asynchronous status polls."""
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Adw,GLib
import plugin_lab_page as module
class Fake:
 def __init__(self,path):pass
 def request(self,command=None):
  return dict(enabled=True,server='http://old:8581',username='',connection='connected',device_count=1,busy=False,setup={'state':'idle'})
module.LabClient=Fake
app=Adw.Application(application_id='cc.senecal.Decksmith.ConnectionFocusCheck')
errors=[];stage=0;focus=None

def activate(app):
 global window,page
 window=Adw.ApplicationWindow(application=app,default_width=850,default_height=800)
 page=module.PluginLabPage('/unused');window.set_content(page);window.present()
 GLib.timeout_add(100,tick)
def tick():
 global stage,focus
 if page.pending or page.state is None:return True
 try:
  if stage==0:
   page.setup_section.set_expanded(True);page.address.set_text('http://draft:8581')
   page.address.grab_focus();page.address.select_region(7,12);focus=window.get_focus()
   assert focus is not None
  elif stage<=40:
   assert window.get_focus()==focus,'Status refresh stole focus'
   assert page.address.get_sensitive(),'Read-only refresh disabled entry'
   assert page.address.get_text()=='http://draft:8581'
   assert page.address.get_selection_bounds()==(7,12),'Selection changed'
   page.fetch()
   assert page.address.get_sensitive(),'Pending refresh disabled entry'
   assert window.get_focus()==focus,'Pending refresh stole focus'
  else:
   page.complete(None)
   assert page.address.get_sensitive() and window.get_focus()==focus,'Failed refresh interrupted editing'
   assert page.address.get_text()=='http://draft:8581'
   print('PASS: focus, selection and draft survive repeated polls and unavailable status',flush=True)
   page.close();app.quit();return False
  stage+=1
 except Exception as e:errors.append(repr(e));page.close();app.quit();return False
 return True
app.connect('activate',activate);app.run(None)
if errors:raise SystemExit(str(errors))
