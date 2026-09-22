"""Inline accessory assignment. Discovery reads only; assignment edits the draft."""
import os
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from unicodedata import normalize
from gi.repository import Adw,Gtk,GLib,Gdk,GdkPixbuf
from plugin_lab_client import LabClient

POOL=ThreadPoolExecutor(max_workers=1,thread_name_prefix='homebridge-discovery')
LABELS={'toggle':'Toggle on / off','on':'Turn on','off':'Turn off','level':'Adjust level','status':'Show status'}

def binding(item,operation):
    if operation not in item['operations']:raise ValueError('Unsupported capability')
    return {'provider':'com.infamous-pattern.openhomeb','action':'com.infamous-pattern.openhomeb.'+operation,'schema':2,'settings':{'accessoryId':item['id']}}

def label(name):
    name=normalize('NFKD',name).encode('ascii','ignore').decode()
    return ' '.join(''.join(c if c.isalnum() else ' ' for c in name).split())[:24].strip() or 'Homebridge'

def icon_png(name,size=120):
    try:
        theme=Gtk.IconTheme.get_for_display(Gdk.Display.get_default())
        icon=theme.lookup_icon(name,['preferences-system-symbolic'],round(size*.8),1,Gtk.TextDirection.NONE,0)
        source=GdkPixbuf.Pixbuf.new_from_file_at_scale(icon.get_file().get_path(),round(size*.8),round(size*.8),True)
        canvas=GdkPixbuf.Pixbuf.new(GdkPixbuf.Colorspace.RGB,True,8,size,size);canvas.fill(0)
        pixels=source.get_pixels();channels=source.get_n_channels();stride=source.get_rowstride()
        tinted=bytearray()
        for y in range(source.get_height()):
            for x in range(source.get_width()):
                alpha=pixels[y*stride+x*channels+3] if channels==4 else 255
                tinted.extend((235,235,235,alpha))
        source=GdkPixbuf.Pixbuf.new_from_bytes(GLib.Bytes.new(bytes(tinted)),GdkPixbuf.Colorspace.RGB,True,8,source.get_width(),source.get_height(),source.get_width()*4)
        source.copy_area(0,0,source.get_width(),source.get_height(),canvas,(size-source.get_width())//2,(size-source.get_height())//2)
        ok,data=canvas.save_to_bufferv('png',[],[])
        return list(data) if ok else None
    except Exception:return None

class HomebridgePicker(Adw.ExpanderRow):
    def __init__(self,assign,dial=False):
        super().__init__(title='Homebridge accessory',subtitle='Choose a device and its supported action')
        self.assign=assign;self.dial=dial;self.items=[];self.visible=[];self.ops=[];self.saved=None;self.pending=False;self.syncing=False
        self.search=Adw.EntryRow(title='Search accessories');self.search.connect('changed',self.filter);self.add_row(self.search)
        self.targets=Adw.ComboRow(title='Accessory',model=Gtk.StringList.new(['Refresh to discover devices']))
        self.targets.set_use_subtitle(True)
        self.targets.connect('notify::selected',self.select);self.add_row(self.targets)
        self.actions=Adw.ComboRow(title='Action',model=Gtk.StringList.new([]));self.add_row(self.actions)
        self.note=Adw.ActionRow(title='Discovery is read-only',subtitle='Assigning changes the draft. Save and Apply activates it.');self.note.set_subtitle_lines(0);self.add_row(self.note)
        row=Gtk.Box(spacing=8,margin_top=8,margin_bottom=8)
        self.refresh=Gtk.Button(label='Refresh accessories');self.refresh.connect('clicked',self.fetch);row.append(self.refresh)
        self.use=Gtk.Button(label='Use assignment');self.use.set_sensitive(False);self.use.connect('clicked',self.apply);row.append(self.use);self.add_row(row)
        self.connect('notify::expanded',lambda *_:self.fetch() if self.get_expanded() and not self.items else None)
    def sync(self,saved):
        self.saved=saved
        self.set_subtitle('Assigned · '+saved['action'].rsplit('.',1)[-1] if saved else 'Choose a device and its supported action')
        self.filter()
    def fetch(self,*_):
        if self.pending:return
        self.pending=True;self.refresh.set_sensitive(False);self.use.set_sensitive(False)
        def work():return LabClient(Path(os.environ['XDG_RUNTIME_DIR'])/'decksmith-plugin-lab.json').catalog()
        def done(f):
            try:data=f.result()
            except Exception:data=None
            GLib.idle_add(self.complete,data)
        POOL.submit(work).add_done_callback(done)
    def complete(self,data):
        self.pending=False;self.refresh.set_sensitive(True)
        if not data or not data['available']:
            self.items=[];self.note.set_subtitle('Homebridge unavailable. Saved assignments are retained. Use Reconnect on the Plugins tab.');self.filter();return False
        self.items=data['items'];self.note.set_subtitle('Only supported actions are offered. Cameras and sensors show status; no video stream.');self.filter();return False
    def filter(self,*_):
        query=self.search.get_text().casefold()
        self.visible=[i for i in self.items if (not self.dial or 'level' in i['operations']) and query in (i['name']+' '+i['kind']).casefold()]
        saved_id=(self.saved or {}).get('settings',{}).get('accessoryId')
        self.syncing=True
        self.targets.set_model(Gtk.StringList.new([i['name']+' · '+i['kind'] for i in self.visible] or ['No matching accessories']))
        self.targets.set_selected(next((n for n,i in enumerate(self.visible) if i['id']==saved_id),0));self.syncing=False;self.select()
    def select(self,*_):
        if self.syncing:return
        n=self.targets.get_selected();item=self.visible[n] if n<len(self.visible) else None
        self.ops=([o for o in item['operations'] if (o=='level')==self.dial] if item else [])
        self.actions.set_model(Gtk.StringList.new([('Adjust brightness' if item['kind']=='Light' else 'Adjust fan speed') if o=='level' else LABELS[o] for o in self.ops]))
        old=(self.saved or {}).get('action','').rsplit('.',1)[-1]
        same=item and item['id']==(self.saved or {}).get('settings',{}).get('accessoryId')
        self.actions.set_selected(self.ops.index(old) if same and old in self.ops else 0)
        self.use.set_sensitive(bool(self.ops) and not self.pending)
    def apply(self,*_):
        n=self.targets.get_selected();a=self.actions.get_selected()
        if self.pending or n>=len(self.visible) or a>=len(self.ops):return
        item=self.visible[n];self.assign(item,binding(item,self.ops[a]))
