"""Coalesced key textures from the same renderer used by the physical device."""
from concurrent.futures import ThreadPoolExecutor
from time import monotonic
from gi.repository import Gtk,Gdk,GLib
class KeyPreview:
    def __init__(self,owner):
        self.frames=[None]*8;self.pictures=[None]*8;self.retry_after=0.;self.last=0.;self.owner=owner;self.desired=None;self.revision=0;self.rendered=None;self.busy=False;self.closed=False
        self.pool=ThreadPoolExecutor(max_workers=1);self.timer=GLib.timeout_add(150,self.refresh)
    def request(self,payload,page):
        if self.desired!=(payload,page):self.desired=(payload,page);self.revision+=1
    def refresh(self):
        if self.closed:return False
        if not any(b.get_mapped() for b in self.owner.keys):return True
        if monotonic()<self.retry_after or self.busy or self.desired is None or (self.rendered==self.revision and monotonic()-self.last<1.):return True
        revision=self.revision;payload,page=self.desired;self.busy=True
        f=self.pool.submit(self.owner.call,'PreviewKeys',GLib.Variant('(sy)',(payload,page)))
        f.add_done_callback(lambda result:GLib.idle_add(self.finish,result,revision));return True
    def finish(self,future,revision):
        self.busy=False
        if self.closed or revision!=self.revision:return False
        try:
            pixels=bytes(future.result())
            if len(pixels)!=8*120*120*3:raise ValueError('Invalid key frame')
            for i,button in enumerate(self.owner.keys):
                frame=pixels[i*43200:(i+1)*43200]
                picture=self.pictures[i]
                if picture is None:
                    picture=Gtk.Picture(can_shrink=True,content_fit=Gtk.ContentFit.FILL);self.pictures[i]=picture
                if self.frames[i]!=frame:
                    picture.set_paintable(Gdk.MemoryTexture.new(120,120,Gdk.MemoryFormat.R8G8B8,GLib.Bytes.new(frame),360));self.frames[i]=frame
                if button.get_child() is not picture:button.set_child(picture)
                button.set_tooltip_text(f"Key {i+1}: {self.owner.draft.data['pages'][self.owner.page]['keys'][i]['label']}")
            self.rendered=revision;self.last=monotonic()
        except Exception:
            self.retry_after=monotonic()+3. # Retain fallback without flooding an unavailable service.
            for button in self.owner.keys:button.set_tooltip_text("Key preview unavailable; showing fallback. Retrying automatically.")
        return False
    def stop(self):
        if self.closed:return
        self.closed=True;GLib.source_remove(self.timer);self.pool.shutdown(wait=False,cancel_futures=True)
