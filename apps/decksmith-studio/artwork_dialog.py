"""Preview source artwork and apply one final key-sized render."""
from gi.repository import Adw,Gtk,Gdk,GLib
from artwork import render,dimensions

class ArtworkDialog(Adw.Dialog):
    def __init__(self,owner,data,apply):
        super().__init__(title='Key artwork',content_width=440,content_height=420)
        self.data=data;self.apply_image=apply
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16)
        for side in ('top','bottom','start','end'): getattr(box,'set_margin_'+side)(20)
        self.set_child(box)
        width,height=dimensions(data)
        box.append(Gtk.Label(label=f'Source: {width} × {height} pixels · Key: 120 × 120',wrap=True))
        if min(width,height)<120:box.append(Gtk.Label(label='Small source image: choose a larger original for sharper artwork.',wrap=True))
        self.preview=Gtk.Picture(width_request=120,height_request=120,halign=Gtk.Align.CENTER)
        self.preview.set_can_shrink(False);box.append(self.preview)
        self.fit=Gtk.DropDown.new_from_strings(['Fit entire image','Crop to square'])
        self.fit.connect('notify::selected',self.update);box.append(self.fit)
        box.append(Gtk.Label(label='Artwork size',xalign=0))
        self.size=Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL,48,120,1)
        self.size.set_value(104);self.size.set_draw_value(True)
        self.size.connect('value-changed',self.update);box.append(self.size)
        self.background=Gtk.DropDown.new_from_strings(['Dark background','Black background','White background'])
        self.background.connect('notify::selected',self.update);box.append(self.background)
        button=Gtk.Button(label='Use artwork');button.add_css_class('suggested-action')
        button.connect('clicked',self.use);box.append(button)
        self.update();self.present(owner)
    def update(self,*_args):
        if not hasattr(self,'background'):return
        self.png=render(self.data,self.size.get_value(),self.fit.get_selected()==1,['#1e2227','#000000','#ffffff'][self.background.get_selected()])
        self.preview.set_paintable(Gdk.Texture.new_from_bytes(GLib.Bytes.new(bytes(self.png))))
    def use(self,_button):
        self.apply_image(self.png);self.close()
