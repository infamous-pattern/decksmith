"""Edit shared and page-specific dial mappings without changing the active device."""
from copy import deepcopy
from unicodedata import normalize
from audio_target_picker import AudioTargetPicker
from media_picker import MediaPicker
from gi.repository import Adw,Gtk

ROTATIONS=['none','volume','brightness']
PRESSES=['none','mute_toggle','next_page','previous_page','media_play_pause','media_previous','media_next','push_to_talk']

from dial_model import defaults,effective,is_override,customize,revert,store

class DialControls(Gtk.Box):
    def __init__(self,owner,embedded=False):
        super().__init__(orientation=Gtk.Orientation.VERTICAL,spacing=16)
        self.embedded=embedded;self.dialog=None
        self.owner=owner;self.data=effective(owner.draft.data,getattr(owner,"page",0));self.index=0;self.syncing=False
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16)
        for side in ('top','bottom','start','end'):getattr(box,'set_margin_'+side)(0 if embedded else 20)
        self.append(box)
        self.scope_note=Gtk.Label(xalign=0);self.scope_note.add_css_class('caption');box.append(self.scope_note)
        self.scope_button=Gtk.Button();self.scope_button.connect('clicked',self.change_scope);box.append(self.scope_button)
        self.picker=Gtk.DropDown.new_from_strings(['Dial 1','Dial 2','Dial 3','Dial 4'])
        self.picker.connect('notify::selected',self.select);box.append(self.picker)
        group=Adw.PreferencesGroup(title="Assignment");self.group=group;box.append(group)
        self.label=Adw.EntryRow(title='Touch-strip label');self.label.connect('changed',self.changed);group.add(self.label)
        self.rotation=Adw.ComboRow(title='Turn to adjust',model=Gtk.StringList.new(['No action','Audio volume','Device brightness']))
        self.rotation.connect('notify::selected',self.rotation_changed)
        self.target=AudioTargetPicker(self.target_changed);group.add(self.target)
        self.icon_refresh=Gtk.Button(label='Refresh application icon',halign=Gtk.Align.START)
        self.icon_refresh.connect('clicked',self.refresh_icon);group.add(self.icon_refresh)
        self.label_error=Gtk.Label(wrap=True,xalign=0);self.label_error.add_css_class('error');group.add(self.label_error)
        behavior=Adw.PreferencesGroup(title='Behavior');box.append(behavior);behavior.add(self.rotation)
        from homebridge_picker import HomebridgePicker
        self.homebridge=HomebridgePicker(self.assign_homebridge,dial=True);behavior.add(self.homebridge)
        self.remove_homebridge=Gtk.Button(label='Remove Homebridge assignment',halign=Gtk.Align.START)
        self.remove_homebridge.connect('clicked',self.clear_homebridge);behavior.add(self.remove_homebridge)
        self.step=Adw.SpinRow(title='Percent per tick',adjustment=Gtk.Adjustment(value=1,lower=1,upper=10,step_increment=1))
        self.step.connect('notify::value',self.changed);behavior.add(self.step)
        self.press=Adw.ComboRow(title='Dial press',model=Gtk.StringList.new(['No action','Toggle mute','Next page','Previous page','Media · Play / Pause','Media · Previous track','Media · Next track','Push to talk']))
        self.press.connect('notify::selected',self.changed);behavior.add(self.press)
        self.media_player=MediaPicker(self.changed);behavior.add(self.media_player)
        from action_catalog import ActionCatalog
        self.catalog=ActionCatalog(
            [('Turn',label,lambda i=i:self.rotation.set_selected(i)) for i,label in enumerate(['No action','Audio volume','Device brightness'])]+
            [('Press',label,lambda i=i:self.press.set_selected(i)) for i,label in enumerate(['No action','Toggle mute','Next page','Previous page','Play / Pause','Previous track','Next track','Push to talk'])])
        self.catalog.set_visible(False)
        appearance=Adw.PreferencesGroup();box.append(appearance)
        self.appearance_group=Adw.ExpanderRow(title='Appearance',subtitle='Font and colors');appearance.add(self.appearance_group)
        from theme_dialog import StyleRows
        self.style_rows=StyleRows(self.appearance_group,lambda:self.data[self.index].get('appearance',{}),self.style_changed,('font','size','label_color','background_color'))
        reset_style=Gtk.Button(label='Reset to theme');reset_style.connect('clicked',self.reset_style);self.appearance_group.add_row(reset_style)
        self.status=Gtk.Label(label='',wrap=True,xalign=0);box.append(self.status)
        self.apply=Gtk.Button(label='Done');self.apply.add_css_class('suggested-action');self.apply.connect('clicked',self.use);box.append(self.apply)
        self.picker.set_visible(not embedded);self.apply.set_visible(not embedded)
        self.select()
    def select(self,*_args):
        self.index=self.picker.get_selected();dial=self.data[self.index];self.syncing=True
        overridden=is_override(self.owner.draft.data,getattr(self.owner,'page',0),self.index)
        self.group.set_title('Assignment')
        if dial.get('plugin_rotation') or dial.get('plugin_press'):
            self.group.set_title('Assignment · Plugin assignment')
        self.scope_note.set_text('This page · Customized dial' if overridden else 'All pages · Shared dial default')
        self.scope_note.set_tooltip_text('Shared changes affect pages that have not customized this dial.')
        self.scope_button.set_label('Use shared settings' if overridden else 'Customize for this page')
        self.homebridge.sync(dial.get('plugin_rotation'))
        self.remove_homebridge.set_visible(bool(dial.get('plugin_rotation')))
        self.step.set_title('Device steps per click' if dial.get('plugin_rotation',{}).get('schema')==2 else 'Percent per tick')
        self.label.set_text(dial['label']);self.rotation.set_selected(ROTATIONS.index(dial['rotation']))
        self.media_player.select(dial.get('media_player',''))
        self.target.select(dial.get('audio_target','system'),inputs_only=dial['press']['type']=='push_to_talk')
        self.step.set_value(dial['step']);self.press.set_selected(PRESSES.index(dial['press']['type']))
        self.syncing=False;self.validate()
        if hasattr(self,"style_rows"):self.style_rows.sync()
    def clear_homebridge(self,*_):
        self.data[self.index].pop('plugin_rotation',None)
        store(self.owner.draft.data,getattr(self.owner,'page',0),self.index,self.data[self.index])
        self.owner.draft.checkpoint();self.select();self.owner.render()

    def assign_homebridge(self,item,binding):
        from homebridge_picker import label,icon_png
        dial=self.data[self.index];dial['rotation']='none';dial['plugin_rotation']=binding
        dial['label']=label(item['name']);dial['step']=1
        dial.pop('target_icon_png',None)
        icon=icon_png(item['icon'],32)
        if icon:dial['target_icon_png']=icon
        store(self.owner.draft.data,getattr(self.owner,'page',0),self.index,dial)
        self.owner.draft.checkpoint();self.select();self.owner.render()

    def change_scope(self,*_args):
        page=getattr(self.owner,'page',0);layout=self.owner.draft.data
        (revert if is_override(layout,page,self.index) else customize)(layout,page,self.index)
        self.owner.draft.checkpoint()
        self.data=effective(layout,page);self.select();self.owner.render()

    def rotation_changed(self,*_args):
        if self.syncing:return
        self.data[self.index].pop('plugin_rotation',None)
        if ROTATIONS[self.rotation.get_selected()]=='brightness':
            self.syncing=True
            self.label.set_text('Brightness')
            self.syncing=False
        elif ROTATIONS[self.rotation.get_selected()]=='volume':
            # The target may already be selected while hidden by Brightness.
            # Entering audio mode must populate its label even without a
            # target selection notification. Reuse target artwork handling too.
            self.target_changed(self.target)
            return
        self.changed()
    def target_changed(self,picker):
        if self.syncing:return
        index=picker.get_selected()
        if index>=len(picker.visible_items):return
        item=picker.visible_items[index]
        name=item.get('name') or item['label'].split(' · ',1)[-1]
        name=normalize('NFKD',name).encode('ascii','ignore').decode('ascii')
        name=' '.join(''.join(c if c.isalnum() else ' ' for c in name).split())
        # Keep generated labels within the same bounds as handwritten labels.
        self.syncing=True
        self.label.set_text(name[:24].rstrip() or 'Audio')
        self.syncing=False
        from applications import audio_icon_png
        icon=audio_icon_png(item['id'])
        self.data[self.index].pop('target_icon_png',None)
        if icon:self.data[self.index]['target_icon_png']=icon
        self.changed()
    def refresh_icon(self,*_):
        from applications import audio_icon_png
        icon=audio_icon_png(self.target.value())
        self.data[self.index].pop('target_icon_png',None)
        if icon:self.data[self.index]['target_icon_png']=icon
        self.changed()

    def style_changed(self,style):
        if style:self.data[self.index]['appearance']=style
        else:self.data[self.index].pop('appearance',None)
        store(self.owner.draft.data,getattr(self.owner,'page',0),self.index,self.data[self.index]);self.owner.render()
    def reset_style(self,*_):self.style_changed({});self.style_rows.sync()
    def changed(self,*_args):
        if self.syncing:return
        target_icon=self.data[self.index].get('target_icon_png')
        appearance=self.data[self.index].get('appearance')
        plugins={k:v for k,v in self.data[self.index].items() if k in ('plugin_rotation','plugin_press')}
        if _args and _args[0] is self.rotation:plugins.pop('plugin_rotation',None)
        if _args and _args[0] is self.press:plugins.pop('plugin_press',None)
        self.data[self.index]={'label':self.label.get_text(),'rotation':ROTATIONS[self.rotation.get_selected()],'step':round(self.step.get_value()),'press':{'type':PRESSES[self.press.get_selected()]}}
        self.data[self.index].update(plugins)
        if appearance:self.data[self.index]['appearance']=appearance
        if target_icon:self.data[self.index]['target_icon_png']=target_icon
        if self.press.get_selected() in (4,5,6) and self.media_player.value():self.data[self.index]['media_player']=self.media_player.value()
        if self.press.get_selected()==7:
            self.target.select(self.target.value(),inputs_only=True)
            self.data[self.index]['press']['target']=self.target.value()
        elif self.target.inputs_only:self.target.select(self.target.value())
        if self.target.value()!='system':self.data[self.index]['audio_target']=self.target.value()
        self.validate()
        if self.embedded:
            store(self.owner.draft.data,getattr(self.owner,'page',0),self.index,self.data[self.index])
            if _args and _args[0] is self.label:self.owner.history_group=('dial',getattr(self.owner,'page',0),self.index,'label')
            self.owner.render()
    def validate(self):
        valid=all(1<=len(dial['label'])<=24 and dial['label'].strip() and all(c.isascii() and (c.isalnum() or c==' ') for c in dial['label']) for dial in self.data)
        self.icon_refresh.set_visible(self.target.value().startswith('app:') and (self.rotation.get_selected()==1 or self.press.get_selected()==1))
        self.label_error.set_text('Use 1–24 letters, numbers or spaces.' if not valid else '')
        self.label_error.set_visible(not valid)
        self.media_player.set_visible(self.press.get_selected() in (4,5,6))
        self.target.set_visible(self.rotation.get_selected()==1 or self.press.get_selected() in (1,7))
        if any(d['press']['type']=='push_to_talk' and not d.get('audio_target','').startswith('input:') for d in self.data):
            self.apply.set_sensitive(False);self.status.set_text('Choose a named microphone for push to talk.');return
        self.apply.set_sensitive(bool(valid));self.step.set_sensitive(self.rotation.get_selected()!=0 or bool(self.data[self.index].get('plugin_rotation')))
        self.status.set_text(('Save and Apply sends your changes to the device.' if self.embedded else 'Done returns to Edit Layout. Save and Apply sends your changes to the device.') if valid else 'Labels need 1–24 letters, numbers or spaces.')
    def use(self,_button):
        for index,dial in enumerate(self.data):store(self.owner.draft.data,getattr(self.owner,'page',0),index,dial)
        self.owner.render()
        if self.dialog:self.dialog.close()

class Dials(Adw.Dialog):
    """Compatibility dialog using the same properties as the integrated canvas."""
    def __init__(self,owner):
        super().__init__(title='Dial controls',content_width=480,content_height=480)
        self.controls=DialControls(owner)
        self.controls.dialog=self
        self.set_child(self.controls)
        self.present(owner)

    def __getattr__(self,name):
        controls=self.__dict__.get('controls')
        if controls is not None:return getattr(controls,name)
        raise AttributeError(name)
