"""Bounded, searchable local icon gallery; applying only changes artwork."""
from gi.repository import Gtk,Adw,GLib,Gdk,Gio
from icon_library import Library,FULL_SET_URL

class IconLibraryDialog(Adw.Dialog):
    def __init__(self,owner,apply,library=None):
        super().__init__(title='Icon library',content_width=720,content_height=620)
        self.library=library or Library();self.apply_icon=apply;self.page=0;self.timer=None
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=12)
        for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(20)
        self.set_child(box)
        self.cancel=Gtk.Button(label='Cancel',halign=Gtk.Align.START)
        self.cancel.connect('clicked',self.dismiss);box.append(self.cancel)
        shortcuts=Gtk.ShortcutController(scope=Gtk.ShortcutScope.MANAGED)
        shortcuts.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        self.escape=Gtk.Shortcut.new(Gtk.KeyvalTrigger.new(Gdk.KEY_Escape,0),Gtk.CallbackAction.new(self.dismiss))
        shortcuts.add_shortcut(self.escape);self.add_controller(shortcuts)
        self.search=Gtk.SearchEntry(placeholder_text='Search technology, apps and imported icons');box.append(self.search)
        self.search.connect('search-changed',self.changed)
        cats=['All categories']+sorted({i['category'] for i in self.library.items()})
        self.categories=cats;self.category=Gtk.DropDown.new_from_strings(cats);self.category.connect('notify::selected',self.changed);box.append(self.category)
        scroll=Gtk.ScrolledWindow(vexpand=True,hscrollbar_policy=Gtk.PolicyType.NEVER);box.append(scroll)
        self.flow=Gtk.FlowBox(selection_mode=Gtk.SelectionMode.NONE,max_children_per_line=6,min_children_per_line=3,column_spacing=6,row_spacing=6);scroll.set_child(self.flow)
        row=Gtk.Box(spacing=10);box.append(row)
        self.previous=Gtk.Button(label='Previous');self.previous.connect('clicked',lambda *_:self.turn(-1));row.append(self.previous)
        self.count=Gtk.Label(hexpand=True);row.append(self.count)
        self.next=Gtk.Button(label='Next');self.next.connect('clicked',lambda *_:self.turn(1));row.append(self.next)
        add=Gtk.Button(label='Import local icon…');add.connect('clicked',self.import_icon);box.append(add)
        self.full_set=Gtk.LinkButton.new_with_label(FULL_SET_URL,'Get the full Tabler icon set ↗');box.append(self.full_set)
        note=Gtk.Label(label=f"Tabler {self.library.catalog['version']} · MIT · Opens the official download page. Downloaded icons can be imported above.",wrap=True,xalign=0);note.add_css_class('dim-label');box.append(note)
        self.status=Gtk.Label(wrap=True,xalign=0);box.append(self.status)
        self.populate();self.present(owner)
    def dismiss(self,*_):
        self.close();return True
    def changed(self,*_):self.page=0;self.populate()
    def turn(self,delta):self.page+=delta;self.populate()
    def populate(self):
        while (child:=self.flow.get_first_child()) is not None:self.flow.remove(child)
        index=self.category.get_selected();items=self.library.items(self.search.get_text(),self.categories[index] if index else None)
        self.page=max(0,min(self.page,max(0,(len(items)-1)//48)));chunk=items[self.page*48:(self.page+1)*48]
        self.previous.set_sensitive(self.page>0);self.next.set_sensitive((self.page+1)*48<len(items))
        self.count.set_text(f'{len(items)} icons · page {self.page+1} of {max(1,(len(items)+47)//48)}')
        for item in chunk:
            button=Gtk.Button(tooltip_text=item['name']+' · '+item['source']);content=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=5)
            try:
                texture=Gdk.Texture.new_from_bytes(GLib.Bytes.new(self.library.image(item,64,'#3088bd')))
                picture=Gtk.Picture.new_for_paintable(texture);picture.set_size_request(60,60);picture.set_can_shrink(True);content.append(picture)
            except Exception:content.append(Gtk.Image.new_from_icon_name('image-missing-symbolic'))
            label=Gtk.Label(label=item['name'],wrap=True,max_width_chars=12);content.append(label);button.set_child(content)
            button.connect('clicked',lambda _,item=item:self.choose(item));self.flow.append(button)
    def choose(self,item):
        try:self.apply_icon(self.library.image(item),item);self.close()
        except Exception as e:self.status.set_text(str(e))
    def import_icon(self,*_):
        chooser=Gtk.FileDialog(title='Import icon (SVG, PNG, JPEG, WebP or ICO)')
        def chosen(dialog,result):
            try:
                file=dialog.open_finish(result)
                if file:
                    item=self.library.import_file(file.get_path());self.category.set_selected(0);self.search.set_text(item['name']);self.populate();self.status.set_text('Imported locally. Select the icon to use it.')
            except GLib.Error:pass
            except Exception as e:self.status.set_text(str(e))
        chooser.open(self.get_root(),None,chosen)
