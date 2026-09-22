"""Explicit desktop confirmation, without an automatic shutdown countdown."""
import sys
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Gtk,Adw,Gio,GLib
from system_controls import Controls
from i18n import gettext as tr

class Confirmation(Adw.Application):
    def __init__(self,command):
        super().__init__(application_id='cc.senecal.Decksmith.SystemConfirmation')
        self.command=command;self.connect('activate',self.activate)
    def activate(self,*_):
        if hasattr(self,'window'):self.window.present();return
        self.window=Adw.ApplicationWindow(application=self,title='Decksmith',default_width=420,default_height=160)
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16)
        for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(24)
        self.window.set_content(box)
        title=tr('Restart') if self.command=='reboot' else tr('Shut down')
        question=tr('Restart this computer?') if self.command=='reboot' else tr('Shut down this computer?')
        box.append(Gtk.Label(label=question,css_classes=['title-2']))
        self.status=Gtk.Label(label=tr('Save your work first. Nothing happens until you confirm.'),wrap=True);box.append(self.status)
        buttons=Gtk.Box(spacing=12,halign=Gtk.Align.END);box.append(buttons)
        cancel=Gtk.Button(label=tr('Cancel'));cancel.connect('clicked',lambda *_:self.quit());buttons.append(cancel)
        self.confirm=Gtk.Button(label=title,css_classes=['destructive-action']);self.confirm.connect('clicked',self.execute);buttons.append(self.confirm)
        key=Gtk.EventControllerKey();key.connect('key-pressed',lambda _,key,*args:self.escape(key));self.window.add_controller(key)
        self.window.present();cancel.grab_focus()
    def escape(self,key):
        from gi.repository import Gdk
        if key==Gdk.KEY_Escape:self.quit();return True
        return False
    def execute(self,*_):
        self.confirm.set_sensitive(False)
        from concurrent.futures import ThreadPoolExecutor
        self.pool=ThreadPoolExecutor(max_workers=1)
        future=self.pool.submit(confirmed_power,self.command)
        def done():
            try:future.result();self.quit()
            except Exception:self.status.set_text(tr('The desktop blocked this request. Save your work and check running applications, then try again.'));self.confirm.set_sensitive(True)
            self.pool.shutdown(wait=False);return False
        future.add_done_callback(lambda _:GLib.idle_add(done))

def confirmed_power(command,controls=None):
    if command not in ('reboot','shutdown'):raise ValueError('Unsupported action')
    c=controls or Controls()
    # Respect GNOME application inhibitors in addition to logind's inhibitors.
    if c.call(False,'org.gnome.SessionManager','/org/gnome/SessionManager','org.gnome.SessionManager','IsInhibited',GLib.Variant('(u)',(1,)))[0]:raise RuntimeError('Session inhibited')
    c.call(True,'org.freedesktop.login1','/org/freedesktop/login1','org.freedesktop.login1.Manager','Reboot' if command=='reboot' else 'PowerOff',GLib.Variant('(b)',(True,)),timeout=120000)
if __name__=='__main__':
    command=sys.argv[1] if len(sys.argv)==2 else ''
    if command not in ('reboot','shutdown'):raise SystemExit(2)
    Confirmation(command).run([])
