"""Running MPRIS players with stable saved choices and asynchronous discovery."""
from concurrent.futures import ThreadPoolExecutor
from gi.repository import Adw,Gtk,GLib
from media import choices
POOL=ThreadPoolExecutor(max_workers=1)

class MediaPicker(Adw.ComboRow):
    def __init__(self,changed):
        super().__init__(title='Media player')
        self.items=[{'id':'','label':'Automatic'}];self.syncing=False;self.loading=False;self.timer=0
        self.set_model(Gtk.StringList.new(['Automatic']))
        self.connect('notify::selected',lambda *_:changed(self) if not self.syncing else None)
        self.connect('map',self.mapped);self.connect('unmap',self.unmapped)
        self.set_tooltip_text('Start playback to discover a player. An explicit choice never falls back to another app. Automatic retains the existing player-selection behavior.')
    def value(self):
        index=self.get_selected()
        return self.items[index]['id'] if index<len(self.items) else ''
    def select(self,target):
        self.syncing=True
        if not any(item['id']==target for item in self.items):
            self.items.append({'id':target,'label':'Unavailable · '+target.removeprefix('org.mpris.MediaPlayer2.')})
        labels=[item['label'] for item in self.items]
        model=self.get_model()
        if [model.get_string(i) for i in range(model.get_n_items())]!=labels:model.splice(0,model.get_n_items(),labels)
        self.set_selected(next(i for i,item in enumerate(self.items) if item['id']==target))
        self.syncing=False
    def mapped(self,*_args):
        self.refresh()
        if not self.timer:self.timer=GLib.timeout_add_seconds(5,self.refresh)
    def unmapped(self,*_args):
        if self.timer:GLib.source_remove(self.timer);self.timer=0
    def refresh(self):
        if self.loading:return True
        self.loading=True
        future=POOL.submit(choices)
        def done():
            target=self.value()
            try:
                items=[{'id':'','label':'Automatic'}]+future.result()
                self.items=items;self.select(target)
            except Exception:pass
            self.loading=False
            return False
        future.add_done_callback(lambda *_:GLib.idle_add(done))
        return True
