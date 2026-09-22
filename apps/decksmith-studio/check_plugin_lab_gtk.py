"""Native tab lifecycle check; fake bridge, no device or Homebridge writes."""
import gi
gi.require_version('Gtk', '4.0');gi.require_version('Adw', '1')
from gi.repository import Adw, Gtk, GLib
import plugin_lab_page as module
import time

calls = []
class FakeClient:
    def __init__(self, path):self.enabled=True
    def request(self, command=None):
        calls.append(command)
        if command is not None:
            if command in ('disable','enable'):self.enabled=command=='enable'
            return {'ok':True}
        return {'enabled':self.enabled,'server':'http://192.0.2.19:8581','connection':'connected' if self.enabled else 'disabled','device_count':61,'target':'Main_LED’s', 'ready':True, 'busy':False, 'fresh':True,
                'feedback_fresh':True, 'actual':{'on':False,'brightness':39},
                'feedback':'39%', 'message':'Connected'}
module.LabClient = FakeClient
app = Adw.Application(application_id='cc.senecal.Decksmith.PluginLabCheck')
errors = [];stage = 0;started = time.monotonic()
def activate(app):
    global window, stack, page
    window = Adw.ApplicationWindow(application=app, title='Plugin integration check', default_width=900, default_height=720)
    stack = Gtk.Stack();window.set_content(stack)
    page = module.PluginLabPage('/unused');stack.add_named(page,'plugins')
    stack.add_named(Gtk.Label(label='Other page'),'other');window.present()
    GLib.timeout_add(100, tick)
def tick():
    global stage, hidden_at, hidden_count
    try:
        assert time.monotonic()-started < 12, 'GTK test timed out'
        if stage==0:
            if page.pending or page.state is None:return True
            assert page.health.get_subtitle()=='Connected'
            assert page.count.get_subtitle()=='61'
            assert page.toggle.get_label()=='Disable'
            page.toggle.emit('clicked');stage=1
        elif stage==1:
            if page.pending:return True
            assert calls.count('disable')==1
            assert page.toggle.get_label()=='Enable' and not page.reconnect.get_sensitive()
            stack.set_visible_child_name('other');hidden_at=time.monotonic();hidden_count=len(calls);stage=2
        elif stage==2:
            if time.monotonic()-hidden_at<1.5:return True
            assert page.timer is None and len(calls)==hidden_count
            stack.set_visible_child_name('plugins');stage=3
        else:
            if page.pending:return True
            assert len(calls)>hidden_count
            for status in ('device_unavailable','unconfirmed','disconnected'):
                value=dict(page.state,connection=status,enabled=True)
                page.complete(value);assert page.health.get_subtitle()==module.STATUS[status][0]
            page.close();assert page.closed and page.timer is None
            print('PASS: manager status, disable, recovery messages, hidden-tab polling and cleanup',flush=True)
            app.quit();return False
    except Exception as e:
        errors.append(str(e));page.close();app.quit();return False
    return True
app.connect('activate',activate);app.run(None)
if errors:raise SystemExit(str(errors))
