"""An inline searchable action chooser for a selected hardware control."""
from gi.repository import Adw, Gtk


class ActionCatalog(Adw.ExpanderRow):
    def __init__(self, entries):
        super().__init__(title='Action', subtitle='Choose an action')
        self.set_tooltip_text('Expand to search actions compatible with this control')
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=8)
        for side in ('top','bottom','start','end'):getattr(box,'set_margin_'+side)(12)
        self.add_row(box)
        self.search=Gtk.SearchEntry(placeholder_text='Search actions…')
        box.append(self.search)
        scroll=Gtk.ScrolledWindow(min_content_height=220,max_content_height=220,
                                 hscrollbar_policy=Gtk.PolicyType.NEVER,
                                 vscrollbar_policy=Gtk.PolicyType.ALWAYS,
                                 overlay_scrolling=False)
        box.append(scroll)
        rows=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=4)
        scroll.set_child(rows);self.groups=[];self.buttons=[]
        for category in dict.fromkeys(item[0] for item in entries):
            title=Gtk.Label(label=category,xalign=0);title.add_css_class('heading')
            rows.append(title);members=[]
            for group,label,callback in entries:
                if group!=category:continue
                button=Gtk.Button(label=label,halign=Gtk.Align.FILL)
                button.add_css_class('flat')
                button.connect('clicked',lambda _b,cb=callback:self.choose(cb))
                rows.append(button);members.append((button,(category+' '+label).casefold()))
                self.buttons.append(button)
            self.groups.append((title,members))
        self.empty=Gtk.Label(label='No matching actions');rows.append(self.empty)
        self.search.connect('search-changed',self.filter)
        self.unavailable=set();self.filter()

    def set_available(self, label, available):
        if available == (label not in self.unavailable):return
        (self.unavailable.discard if available else self.unavailable.add)(label)
        self.filter()

    def set_label(self, label):
        self.set_subtitle(label.removeprefix('Action · '))

    def choose(self,callback):
        self.set_expanded(False);callback()

    def filter(self,*_args):
        query=self.search.get_text().strip().casefold();found=False
        for title,members in self.groups:
            visible=False
            for button,text in members:
                match=button.get_label() not in self.unavailable and all(word in text for word in query.split())
                button.set_visible(match);visible|=match
            title.set_visible(visible);found|=visible
        self.empty.set_visible(not found)
