"""Compact numeric icon size, with explicit validation and five-point buttons."""
from gi.repository import Gtk,Adw
class IconSizeRow(Adw.ActionRow):
    def __init__(self,changed,invalidated):
        super().__init__(title='Icon size')
        self.changed=changed;self.invalidated=invalidated;self.syncing=False;self.invalid=False;self.value=100
        box=Gtk.Box(spacing=4,valign=Gtk.Align.CENTER)
        self.minus=Gtk.Button(icon_name='list-remove-symbolic',tooltip_text='Smaller icon · 5%')
        self.plus=Gtk.Button(icon_name='list-add-symbolic',tooltip_text='Larger icon · 5%')
        self.entry=Gtk.Entry(width_chars=3,max_width_chars=3,input_purpose=Gtk.InputPurpose.DIGITS)
        self.entry.update_property([Gtk.AccessibleProperty.LABEL],['Icon size percentage'])
        self.minus.connect('clicked',lambda *_:self.entry.set_text(str(max(10,self.value-5))))
        self.plus.connect('clicked',lambda *_:self.entry.set_text(str(min(100,self.value+5))))
        box.append(self.minus);box.append(self.entry);box.append(Gtk.Label(label='%'));box.append(self.plus);self.add_suffix(box)
        self.entry.connect('changed',self.edited);self.set_value(100)
    def set_value(self,value):
        self.syncing=True;self.value=value;self.entry.set_text(str(value));self.invalid=False
        self.entry.remove_css_class('error');self.set_subtitle('');self.buttons();self.syncing=False
    def buttons(self):
        self.minus.set_sensitive(self.value>10);self.plus.set_sensitive(self.value<100)
    def edited(self,*_):
        if self.syncing:return
        text=self.entry.get_text().strip()
        valid=text.isascii() and text.isdigit() and 10<=int(text)<=100 and int(text)%5==0
        self.invalid=not valid
        if not valid:
            self.entry.add_css_class('error');self.set_subtitle('Enter 10–100 in steps of 5. Keeping last valid preview.');self.invalidated();return
        self.value=int(text);self.entry.remove_css_class('error');self.set_subtitle('');self.buttons();self.changed(self.value)
