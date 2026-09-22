"""Searchable, grouped actions for a selected hardware control."""
from gi.repository import Gtk

class ActionCatalog(Gtk.MenuButton):
    def __init__(self, entries):
        super().__init__(label='Choose action…')
        self.set_tooltip_text('Search actions compatible with this control')
        self.popover=Gtk.Popover();self.set_popover(self.popover)
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=8)
        for side in ('top','bottom','start','end'):getattr(box,'set_margin_'+side)(12)
        self.popover.set_child(box)
        self.search=Gtk.SearchEntry(placeholder_text='Search actions…')
        box.append(self.search)
        scroll=Gtk.ScrolledWindow(min_content_height=260,max_content_height=400,
                                 hscrollbar_policy=Gtk.PolicyType.NEVER)
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
        self.filter()

    def choose(self,callback):
        self.popover.popdown();callback()

    def filter(self,*_args):
        query=self.search.get_text().strip().casefold();found=False
        for title,members in self.groups:
            visible=False
            for button,text in members:
                match=all(word in text for word in query.split())
                button.set_visible(match);visible|=match
            title.set_visible(visible);found|=visible
        self.empty.set_visible(not found)
