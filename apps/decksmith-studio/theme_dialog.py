"""Native appearance defaults, independently editable from action mappings."""
from gi.repository import Gtk,Adw,GLib
from themes import PRESETS,STYLE_FIELDS,FONTS,SIZES,COLORS,set_theme,encode_theme,decode_theme

CHOICES={'font':(['Theme']+list(FONTS.values()),[None]+list(FONTS)),
         'size':(['Theme']+list(SIZES.values()),[None]+list(SIZES)),
         'label_color':(['Theme']+[k.title() for k in COLORS if k!='default'],[None]+[k for k in COLORS if k!='default']),
         'background_color':(['Theme']+[k.title() for k in COLORS if k!='default'],[None]+[k for k in COLORS if k!='default']),
         'label_position':(['Theme','Hidden','Top','Middle','Bottom'],[None,'hidden','top','middle','bottom']),
         'label_background':(['Theme','Dark','Transparent'],[None,'dark','transparent'])}
LABELS={'font':'Font','size':'Text size','label_color':'Text color','background_color':'Background color','label_position':'Key label position','label_background':'Key label background'}
class StyleRows:
    def __init__(self,group,get_style,changed,fields=None,invalidated=lambda:None):
        self.get_style=get_style;self.changed=changed;self.syncing=False;self.rows={}
        for name in fields or (*CHOICES,'icon_size'):
            if name=='icon_size':
                from icon_size_row import IconSizeRow
                row=IconSizeRow(self.edit_icon_size,invalidated);group.add(row) if hasattr(group,'add') else group.add_row(row);self.rows[name]=row
                continue
            labels,values=CHOICES[name];row=Adw.ComboRow(title=LABELS[name],model=Gtk.StringList.new(labels))
            if name=='font':
                row.set_enable_search(True)
                row.set_tooltip_text('Bundled fonts work offline. Viking Runes converts letters decoratively; saved text stays unchanged.')
            row.connect('notify::selected',lambda r,_p,n=name:self.edit(n,r));group.add(row) if hasattr(group,'add') else group.add_row(row);self.rows[name]=row
        self.sync()
    def sync(self):
        self.syncing=True;style=self.get_style()
        for name,row in self.rows.items():
            if name=='icon_size':row.set_value(style.get(name,100));continue
            values=CHOICES[name][1];value=style.get(name);row.set_selected(values.index(value) if value in values else 0)
        self.syncing=False
    @property
    def invalid(self):return any(getattr(row,'invalid',False) for row in self.rows.values())
    def edit_icon_size(self,value):
        if self.syncing:return
        style=dict(self.get_style());style['icon_size']=value;self.changed(style)
    def edit(self,name,row):
        if self.syncing:return
        style=dict(self.get_style());value=CHOICES[name][1][row.get_selected()]
        if value is None:style.pop(name,None)
        else:style[name]=value
        self.changed(style)

class ThemeControls(Gtk.Box):
    def __init__(self,owner):
        super().__init__(orientation=Gtk.Orientation.VERTICAL,spacing=16)
        self.owner=owner;self.syncing=True
        box=self
        for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(12)
        note=Gtk.Label(label='Appearance only. Existing artwork and custom styles stay in place. Use Reset to theme on a key or dial to inherit these defaults. Save and Apply sends changes to the device.',wrap=True,xalign=0);box.append(note)
        group=Adw.PreferencesGroup();box.append(group)
        self.preset=Adw.ComboRow(title='Preset',model=Gtk.StringList.new(['Keep current appearance']+[v[0] for v in PRESETS.values()]));group.add(self.preset)
        theme=owner.draft.data.get('theme');self.preset.set_selected(1+list(PRESETS).index(theme['preset']) if theme else 0)
        self.preset.connect('notify::selected',self.preset_changed)
        self.defaults=Adw.PreferencesGroup(title='Shared defaults');box.append(self.defaults)
        self.rows=StyleRows(self.defaults,self.style,self.changed,invalidated=owner.render);self.defaults.set_sensitive(theme is not None)
        reset=Gtk.Button(label='Reset shared defaults');reset.connect('clicked',self.reset_defaults);box.append(reset)
        export=Gtk.Button(label='Export theme…');export.connect('clicked',self.export_theme);box.append(export)
        imp=Gtk.Button(label='Import theme…');imp.connect('clicked',self.import_theme);box.append(imp)
        self.status=Gtk.Label(wrap=True,xalign=0);box.append(self.status)
        self.syncing=False
        self.snapshot=None
        self.sync_from_draft()

    def sync_from_draft(self):
        from copy import deepcopy
        theme=self.owner.draft.data.get('theme')
        if theme==self.snapshot:return
        self.syncing=True
        self.preset.set_selected(1+list(PRESETS).index(theme['preset']) if theme else 0)
        self.defaults.set_sensitive(theme is not None)
        self.rows.sync();self.snapshot=deepcopy(theme)
        self.syncing=False

    def style(self):return self.owner.draft.data.get('theme',{}).get('appearance',{})
    def changed(self,style=None):
        if self.syncing:return
        if style is not None:self.owner.draft.data['theme']['appearance']=style
        if self.owner.control_kind=='dial':self.owner.select_dial(self.owner.dial_index)
        else:self.owner.select_key(self.owner.key)
        self.status.set_text('Draft appearance updated. Actions unchanged.')
    def preset_changed(self,*_):
        if self.syncing:return
        index=self.preset.get_selected();set_theme(self.owner.draft.data,list(PRESETS)[index-1] if index else None)
        self.defaults.set_sensitive(bool(index));self.rows.sync();self.changed()
    def reset_defaults(self,*_):
        if self.owner.draft.data.get('theme') is not None:self.owner.draft.data['theme']['appearance']={};self.rows.sync();self.changed()
    def export_theme(self,*_):
        theme=self.owner.draft.data.get('theme')
        if theme is None:self.status.set_text('Choose a preset first.');return
        data=encode_theme(theme)
        chooser=Gtk.FileDialog(title='Export appearance theme',initial_name='Decksmith-theme.json')
        def chosen(dialog,result):
            try:
                file=dialog.save_finish(result)
                if file:file.replace_contents(data,None,False,0,None);self.status.set_text('Theme exported without actions.')
            except GLib.Error:pass
        chooser.save(self.owner,None,chosen)
    def import_theme(self,*_):
        chooser=Gtk.FileDialog(title='Import appearance theme')
        def chosen(dialog,result):
            try:
                file=dialog.open_finish(result)
                if not file:return
                from pathlib import Path
                path=Path(file.get_path())
                if path.stat().st_size>65536:raise ValueError('Theme file is too large.')
                theme=decode_theme(path.read_bytes());self.owner.draft.data['theme']=theme
                self.syncing=True;self.preset.set_selected(1+list(PRESETS).index(theme['preset']));self.syncing=False
                self.defaults.set_sensitive(True);self.rows.sync();self.changed()
            except GLib.Error:pass
            except Exception as e:self.status.set_text(str(e))
        chooser.open(self.owner,None,chosen)


class ThemeDialog(Adw.Dialog):
    """Compatibility container for the standalone editor."""
    def __init__(self,owner):
        super().__init__(title='Theme and defaults',content_width=450,content_height=620)
        controls=ThemeControls(owner);self.rows=controls.rows
        scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER,child=controls)
        self.set_child(scroll)
        done=Gtk.Button(label='Done');done.connect('clicked',lambda *_:self.close());controls.append(done)
        self.connect('closed',lambda *_:owner.render())
        self.present(owner)
