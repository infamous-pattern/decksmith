"""Read-only native touch image, coalesced off the GTK main thread."""
from concurrent.futures import ThreadPoolExecutor
from time import monotonic
from gi.repository import Gtk, Gdk, GLib

class TouchPicture(Gtk.Picture):
    def do_measure(self,orientation,for_size):
        # Source pixels must not affect the simulated hardware's preferred size.
        if orientation==Gtk.Orientation.HORIZONTAL:return (0,0,-1,-1)
        return (52,52,-1,-1)

class TouchPreview(Gtk.Overlay):
    def __init__(self,call):
        super().__init__()
        self.retry_after=0.;self.call=call;self.desired=None;self.revision=0;self.busy=False
        self.closed=False;self.rgb=None;self.mode=None;self.rendered_revision=None;self.invalid=False
        self.pool=ThreadPoolExecutor(max_workers=1)
        self.picture=TouchPicture(can_shrink=True,content_fit=Gtk.ContentFit.FILL)
        self.picture.set_size_request(0,52)
        self.set_child(self.picture)
        self.message=Gtk.Label(label='Loading device preview…',halign=Gtk.Align.CENTER,valign=Gtk.Align.CENTER)
        self.message.add_css_class('dim-label');self.add_overlay(self.message)
        self.set_measure_overlay(self.message,False)
        self.note=Gtk.Label(label='Touch-strip preview loading',xalign=0,wrap=True,max_width_chars=48)
        self.note.add_css_class('dim-label')
        self.timer=GLib.timeout_add(50,self.refresh)

    def request(self,payload,page):
        self.invalid=False
        desired=(payload,page)
        if desired!=self.desired:
            self.desired=desired;self.revision+=1
            self.note.set_text('Updating touch-strip preview…')

    def invalidate(self):
        # A partially configured action must not erase the whole device. Keep
        # rendering the last valid layout, explicitly identified as such.
        self.invalid=True
        if self.rgb is None:
            self.message.set_text('Loading last valid preview…' if self.desired else 'Complete the layout to preview')
            self.message.set_visible(True)
        self.note.set_text('Touch strip · last valid layout; complete the invalid field' if self.desired else
                           'Touch preview unavailable · complete the invalid field')

    def refresh(self):
        if self.closed:return False
        if monotonic()<getattr(self,"retry_after",0.) or self.busy or self.desired is None or not self.get_mapped():return True
        revision=self.revision;payload,page=self.desired
        self.busy=True
        future=self.pool.submit(self.call,'PreviewTouch',GLib.Variant('(sy)',(payload,page)))
        future.add_done_callback(lambda result:GLib.idle_add(self.finish,result,revision))
        return True

    def finish(self,future,revision):
        self.busy=False
        if self.closed or revision!=self.revision:return False
        try:
            rgb,mode=future.result();rgb=bytes(rgb)
            if len(rgb)!=800*100*3:raise ValueError('Invalid touch image')
            if rgb!=self.rgb:
                self.picture.set_paintable(Gdk.MemoryTexture.new(800,100,Gdk.MemoryFormat.R8G8B8,GLib.Bytes.new(rgb),800*3))
                self.rgb=rgb
            self.mode=mode;self.rendered_revision=revision;self.message.set_visible(False)
            self.note.set_text('Touch strip · last valid layout; complete the invalid field' if self.invalid else {'live':'Touch strip · current image sent to device',
                'draft':'Touch strip · draft rendered with current device state',
                'draft-missing-target':'Touch strip · draft preview; new audio targets show -- until applied',
                'offline':'Touch strip · device disconnected; live values unavailable'}.get(mode,'Touch strip · rendered preview'))
        except Exception:
            self.retry_after=monotonic()+3.
            # Preserve the last texture through a transient request failure,
            # but never present it as a current measurement. Polling retries.
            self.message.set_text('Device preview unavailable');self.message.set_visible(self.rgb is None)
            self.note.set_text('Touch preview paused · retrying; last image shown' if self.rgb is not None else
                               'Touch preview unavailable · retrying connection')
        return False

    def stop(self):
        if self.closed:return
        self.closed=True;GLib.source_remove(self.timer)
        self.pool.shutdown(wait=False,cancel_futures=True)
