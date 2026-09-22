"""Nonblocking audio inventory selector; saved targets survive temporary absence."""
from concurrent.futures import ThreadPoolExecutor
from gi.repository import Adw,Gtk,GLib
from audio_targets import inventory
from audio_monitor import MONITOR
from audio_apps import cached
POOL=ThreadPoolExecutor(max_workers=1)

class AudioTargetPicker(Adw.ComboRow):
    def __init__(self,changed):
        super().__init__(title='Audio target')
        self.items=[{'id':'system','label':'System sounds'},{'id':'microphone','label':'Default microphone'}]
        self.items.extend({'id':app['id'],'name':app['name'],'label':'App · '+app['name']+' · Installed'} for app in cached())
        self.visible_items=self.items[:];self.syncing=False;self.devices_only=False;self.inputs_only=False;self.loading=False;self.refresh_pending=False
        self.set_model(Gtk.StringList.new([item['label'] for item in self.items]))
        self.connect('notify::selected',lambda *_:changed(self) if not self.syncing else None)
        self.refresh_button=Gtk.Button(icon_name='view-refresh-symbolic',tooltip_text='Refresh apps and audio devices',valign=Gtk.Align.CENTER)
        self.refresh_button.connect('clicked',lambda *_:self.refresh());self.add_suffix(self.refresh_button)
        self.set_tooltip_text('System sounds adjusts desktop alerts; named outputs adjust listening volume. Installed audio apps are discovered automatically. Audio detected means a matching playback stream exists. Installed apps can be assigned before playback.')
        self.connect('map',lambda *_:MONITOR.add(self))
        self.connect('unmap',lambda *_:MONITOR.remove(self))
        self.refresh()
    def value(self):
        index=self.get_selected()
        return self.visible_items[index]['id'] if index<len(self.visible_items) else 'system'
    def select(self,target,devices_only=False,inputs_only=False):
        self.syncing=True;self.devices_only=devices_only;self.inputs_only=inputs_only
        self.visible_items=[item for item in self.items if (item['id'].startswith('input:') if inputs_only else not devices_only or item['id'].startswith(('output:','input:')))]
        if not any(item['id']==target for item in self.visible_items):
            label='Choose a microphone' if inputs_only and not target.startswith('input:') else 'Choose an audio device' if devices_only and target in ('system','microphone') else 'Unavailable · '+target.split(':',1)[-1].split('=',1)[-1]
            self.visible_items.append({'id':target,'label':label})
        self.get_model().splice(0,self.get_model().get_n_items(),[item['label'] for item in self.visible_items])
        self.set_selected(next(i for i,item in enumerate(self.visible_items) if item['id']==target))
        self.syncing=False
    def refresh(self):
        if self.loading:
            self.refresh_pending=True
            return
        self.loading=True;self.refresh_button.set_sensitive(False)
        future=POOL.submit(inventory)
        def done():
            target=self.value()
            try:
                items=future.result()
                if items!=self.items:
                    self.items=items
                    self.select(target,self.devices_only,self.inputs_only)
                self.set_tooltip_text('System sounds adjusts desktop alerts; named outputs adjust listening volume. Installed audio apps are discovered automatically. Audio detected means a matching playback stream exists. Installed apps can be assigned before playback.')
            except Exception:self.set_tooltip_text('Audio discovery unavailable. Check the audio service, then refresh.')
            self.loading=False;self.refresh_button.set_sensitive(True)
            if self.refresh_pending:
                self.refresh_pending=False
                self.refresh()
            return False
        future.add_done_callback(lambda *_:GLib.idle_add(done))
