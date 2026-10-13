#!/usr/bin/env python3
"""One native workspace for the explicit VirtualDeck profile test endpoint."""
import json,os,tempfile
from pathlib import Path
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Adw,Gtk,GLib,Gdk,GdkPixbuf,Pango
from editor import Editor
from profile_client import DeviceClient,transport,workspace_transport
from profile_drafts import DraftShelf

class ProfileLab(Adw.Application):
    def __init__(self,*,workspace=False):
        self.workspace=workspace;self.transport=workspace_transport if workspace else transport;self.isolation=None
        if not workspace:
            if os.environ.get('DECKSMITH_ISOLATED_TEST')!='1':raise ValueError('Isolated test required')
            self.isolation=tempfile.TemporaryDirectory(prefix='decksmith-profile-editor-')
            for variable,name in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache'),('XDG_STATE_HOME','state'),('XDG_RUNTIME_DIR','runtime')]:
                directory=Path(self.isolation.name)/name;directory.mkdir(mode=0o700);os.environ[variable]=str(directory)
            os.environ['PULSE_SERVER']='unix:'+str(Path(self.isolation.name)/'no-audio.sock')
            os.environ.pop('DECKSMITH_PLUGIN_LAB_CONNECTION',None)
        super().__init__(application_id='cc.senecal.Decksmith.DeviceWorkspaceEditor' if workspace else 'cc.senecal.Decksmith.ProfileEditorLab')
        self.quit_all=False;self.clients={};self.selected=None;self.switching=False;self.shelf=DraftShelf();self.settings_drafts={};self.home_sync=False;self.enrollment_pending=False
        self.connect('activate',self.activate_lab);self.connect('shutdown',self.cleanup_lab)
    def activate_lab(self,*_):
        if self.selected:self.editor.present();return
        self.window=Adw.ApplicationWindow(application=self);self.toasts=Adw.ToastOverlay()
        sheet=GdkPixbuf.Pixbuf.new_from_file(str(Path(__file__).resolve().parents[2]/'brand/decksmith-makers-mark-brand-sheet.png'))
        self.logo_texture=Gdk.Texture.new_for_pixbuf(sheet.new_subpixbuf(840,48,212,212).copy())
        snapshot=json.loads(self.transport('ListDevices')[0])
        if snapshot.get('mode')!=('saved-workspace' if self.workspace else 'virtual-lab') or snapshot.get('api_version')!=2:raise ValueError('Wrong lab service')
        self.devices=snapshot['devices'];self.generation=snapshot['generation'];self.selected=snapshot['selected']
        for device in self.devices:self.clients[device['id']]=DeviceClient(device,self.transport)
        if self.selected is None:
            if not self.workspace:raise ValueError('Empty virtual lab')
            self.build_enrollment(snapshot);return
        self.build_home()
        client=self.clients[self.selected];self.editor=Editor(self,client,integrated=True)
        for c in self.clients.values():c.page=lambda:self.editor.page
        self.editor.set_title('Decksmith · Saved device profiles' if self.workspace else 'Decksmith · Virtual device workspace');self.refresh();self.editor.present()
        if self.workspace:self.editor.save_button.set_label('Save profile')
    def open_enrollment(self):
        ed=self.editor
        if ed.pending:return
        ed.request(lambda:json.loads(self.transport('ListDevices')[0]),lambda snapshot:self.build_enrollment(snapshot,ed.shell.device_menu.get_popover()),'Device discovery is unavailable. Existing drafts are retained.')
    def build_enrollment(self,snapshot,popover=None):
        self.enrollment_snapshot=snapshot
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=12)
        for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(18)
        box.append(Gtk.Label(label='Add a saved device profile',xalign=0))
        note=Gtk.Label(label='Each device has its own pages. These profiles do not change physical controls yet.',wrap=True,xalign=0,max_width_chars=45);box.append(note)
        discovery=snapshot.get('discovery',{})
        self.enrollment_note=Gtk.Label(xalign=0,wrap=True,max_width_chars=45);box.append(self.enrollment_note)
        if not discovery.get('available'):self.enrollment_note.set_text('Device discovery is unavailable. Refresh to try again.')
        count=0
        for candidate in discovery.get('candidates',[]):
            association=candidate.get('association',{})
            if association!={'state':'unassigned','value':'new_device'}:continue
            count+=1;model=candidate['model'];name='Stream Deck +' if model=='plus' else 'Stream Deck MK.2'
            button=Gtk.Button(label='Add '+name+' · detected device '+str(candidate['slot']+1))
            button.connect('clicked',lambda _b,c=candidate,n=name:self.enroll(snapshot,c,n));box.append(button)
            if model=='plus' and snapshot.get('legacy_import_prepared'):
                imported=Gtk.Button(label='Add Stream Deck + from prepared layout');imported.connect('clicked',lambda _b,c=candidate,n=name:self.enroll(snapshot,c,n,import_plus=True));box.append(imported)
        if discovery.get('available') and not count:self.enrollment_note.set_text('No new uniquely identified device is available. Previously saved profiles remain in Choose Device.')
        refresh=Gtk.Button(label='Refresh detected devices');refresh.connect('clicked',lambda *_:self.open_enrollment() if hasattr(self,'editor') else self.build_enrollment(json.loads(self.transport('ListDevices')[0])));box.append(refresh)
        if popover:
            back=Gtk.Button(label='Back to saved devices');back.connect('clicked',lambda *_:(self.install_device_picker(self.editor.shell),self.refresh()));box.append(back);popover.set_child(box)
        else:
            view=Adw.ToolbarView();view.add_top_bar(Adw.HeaderBar());view.set_content(box);self.window.set_content(view);self.window.set_default_size(650,420);self.window.present()
    def enroll(self,snapshot,candidate,name,*,import_plus=False):
        # Explicit model/slot choice. No automatic enrollment, import or fallback.
        if self.enrollment_pending:return
        self.enrollment_pending=True
        args=GLib.Variant('(sttysyb)',(snapshot['runtime'],snapshot['discovery']['revision'],snapshot['generation'],candidate['slot'],name,50,import_plus))
        def task():return self.transport('EnrollDevice',args)
        def done(_):
            self.enrollment_pending=False
            if hasattr(self,'editor'):
                self.devices=json.loads(self.transport('ListDevices')[0])['devices'];self.install_device_picker(self.editor.shell);self.refresh();self.editor.shell.device_menu.get_popover().popdown()
            else:
                self.window.close();self.activate_lab()
        def failed():self.enrollment_pending=False;return False
        if hasattr(self,'editor'):
            def checked_task():
                try:return task()
                except Exception:GLib.idle_add(failed);raise
            self.editor.request(checked_task,done,'Device was not added. Refresh discovery before retrying; your saved profiles and drafts are retained.')
        else:
            # Enrollment runs off the UI thread; focus and controls stay stable.
            import threading
            def run():
                try:task();GLib.idle_add(done,None)
                except Exception:
                    GLib.idle_add(failed);GLib.idle_add(self.enrollment_note.set_text,'Device was not added. Refresh discovery before retrying.')
            threading.Thread(target=run,daemon=True).start()
    def build_home(self):
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=18)
        for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(22)
        self.home_content=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER);self.home_content.set_child(box)
        group=Adw.PreferencesGroup(title='Your device',description='Saved device profiles · your current background controls keep running. Physical changes are not applied yet.' if self.workspace else 'Virtual test workspace · no physical Deck or desktop settings are changed.');box.append(group)
        self.name=Adw.EntryRow(title='Device name');self.name.connect('changed',lambda *_:self.keep_settings());group.add(self.name)
        group.add(Adw.ActionRow(title='Background controls',subtitle='Managed by your current Decksmith service' if self.workspace else 'Virtual controls running · managed by the isolated test host'))
        for title in ('Start Decksmith at login','Auto-Lock'):
            row=Adw.SwitchRow(title=title,subtitle='Use the main Decksmith app for desktop integration' if self.workspace else 'Desktop integration is unavailable in this virtual test');row.set_sensitive(False);group.add(row)
        brightness=Adw.ActionRow(title='Brightness',subtitle='Percentage for the selected device')
        controls=Gtk.Box(spacing=8,valign=Gtk.Align.CENTER)
        self.percent=Gtk.Label(label='50%',width_chars=4)
        for label,offset in [('−',-1),('+',1)]:
            b=Gtk.Button(label=label,tooltip_text='Decrease brightness' if offset<0 else 'Increase brightness')
            b.connect('clicked',lambda _b,n=offset:self.adjust_brightness(n));controls.append(b)
            if offset<0:controls.append(self.percent)
        brightness.add_suffix(controls);group.add(brightness)
        self.metadata=Gtk.Button(label='Save device settings' if self.workspace else 'Apply device settings',halign=Gtk.Align.START);self.metadata.connect('clicked',self.save_metadata);box.append(self.metadata)
        discard=Gtk.Button(label='Discard device settings changes',halign=Gtk.Align.START);discard.connect('clicked',self.discard_settings);box.append(discard)
        history=Adw.PreferencesGroup(title='Saved profile history');box.append(history)
        self.history=Adw.ActionRow(title='Verified history');history.add(self.history)
        self.archive=Gtk.Button(label='Archive older saves',halign=Gtk.Align.START,tooltip_text='Verify older saves in a separate private archive before removing them from the active store.')
        self.archive.connect('clicked',self.archive_history);box.append(self.archive)
    def install_device_picker(self,shell):
        shell.device_menu.set_tooltip_text('Choose Device · independent saved profiles and retained drafts')
        content=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=8)
        for edge in ('top','bottom','start','end'):getattr(content,'set_margin_'+edge)(12)
        content.append(Gtk.Label(label='Choose Device',xalign=0));self.device_buttons={}
        for device in self.devices:
            button=Gtk.Button(label=device['name'],tooltip_text='Select this device without discarding another device’s draft')
            button.connect('clicked',lambda _b,i=device['id']:self.choose(i));content.append(button);self.device_buttons[device['id']]=button
        self.device_picker_content=content
        if self.workspace:
            add=Gtk.Button(label='Add detected device',tooltip_text='Refresh discovery and create an independent saved profile.');add.connect('clicked',lambda *_:self.open_enrollment());content.append(add)
        shell.device_menu.get_popover().set_child(content)
        self.caption=Gtk.Label(ellipsize=Pango.EllipsizeMode.END,max_width_chars=26,xalign=0)
        title=Gtk.Box(spacing=8);title.append(self.caption);title.append(Gtk.Image.new_from_icon_name('pan-down-symbolic'));shell.device_menu.set_child(title)
    def choose(self,identity):
        ed=self.editor
        if self.switching or identity==self.selected or ed.pending or ed.draft is None:return
        if identity not in self.clients:return
        self.shelf.keep(self.selected,ed,self.clients[self.selected].token['revision'])
        client=self.clients[identity];self.switching=True
        def read():
            try:
                snapshot=json.loads(self.transport('SelectDevice',GLib.Variant('(st)',(identity,self.generation)))[0])
                layout=json.loads(client('GetLayout')[0]);status=json.loads(client('GetStatus')[0])
                return snapshot,layout,status
            except Exception:
                GLib.idle_add(self.selection_failed);raise
        def loaded(result):
            snapshot,layout,status=result;self.generation=snapshot['generation'];self.selected=identity
            ed.call=client;ed.touch_preview.rebind(client);ed.control_kind='key';ed.page=ed.key=0
            view=self.shelf.restore(identity,layout,client.token)
            ed.loaded((layout,status))
            if view:
                ed.draft=view.draft
                active=status.get('active_page');ed.page=ed.draft.origins.index(active) if active in ed.draft.origins else min(view.page,len(ed.draft.data['pages'])-1)
                ed.key=min(view.key,ed.key_count-1);ed.control_kind=view.kind if ed.device_model=='plus' else 'key';ed.dial_index=view.dial
                ed.rebuild()
            self.switching=False;self.refresh();ed.shell.device_menu.get_popover().popdown()
        ed.request(read,loaded,'Could not select the device. Your current draft is retained; refresh the saved state before retrying.')
    def selection_failed(self):self.switching=False;return False
    def keep_settings(self):
        if self.home_sync or not self.selected or not hasattr(self,'desired_brightness'):return
        client=self.clients[self.selected]
        if (self.name.get_text(),self.desired_brightness)==(client.name,client.brightness):
            self.settings_drafts.pop(self.selected,None);return
        old=self.settings_drafts.get(self.selected)
        self.settings_drafts[self.selected]={'name':self.name.get_text(),'brightness':self.desired_brightness,
            'base':old['base'] if old else (client.name,client.brightness),
            'revision':old['revision'] if old else client.token['revision'] if client.token else 1}
    def discard_settings(self,*_):self.settings_drafts.pop(self.selected,None);self.refresh()
    def adjust_brightness(self,offset):
        self.desired_brightness=max(0,min(100,self.desired_brightness+offset));self.percent.set_text(f'{self.desired_brightness}%');self.keep_settings()
    def save_metadata(self,*_):
        ed=self.editor;client=self.clients[self.selected]
        if ed.pending or client.token is None:return
        token=dict(client.token);draft=self.settings_drafts.get(self.selected)
        if draft:token['revision']=draft['revision']
        token=json.dumps(token,separators=(',',':'));name=self.name.get_text();brightness=self.desired_brightness
        def task():return client.accept(self.transport('SaveMetadata',GLib.Variant('(sssy)',(client.id,token,name,brightness))))
        def done(_):self.settings_drafts.pop(client.id,None);self.refresh();ed.render();ed.status.set_text('Device settings saved; your layout draft is retained.')
        ed.request(task,done,'Device settings could not be confirmed. Your layout draft is retained; reload saved state before retrying.')
    def archive_history(self,*_):
        ed=self.editor
        if ed.pending:return
        expected=self.generation
        def task():return json.loads(self.transport('ArchiveHistory',GLib.Variant('(t)',(expected,)))[0])
        def done(result):self.refresh();ed.render();ed.status.set_text(f"Archived {result['archived']} verified saves; {result['retained']} retained in the active store.")
        ed.request(task,done,'Archiving needs attention. Your active saved profile is intact; verify the archive before resuming.')
    def refresh(self):
        snapshot=json.loads(self.transport('ListDevices')[0]);self.generation=snapshot['generation'];self.devices=snapshot['devices']
        client=self.clients[self.selected];summary=next(d for d in self.devices if d['id']==self.selected)
        client.name=summary['name'];client.brightness=summary['brightness'];draft=self.settings_drafts.get(self.selected)
        if draft and draft['base']==(client.name,client.brightness) and client.token:draft['revision']=client.token['revision']
        self.home_sync=True
        self.name.set_text(draft['name'] if draft else client.name)
        self.desired_brightness=draft['brightness'] if draft else client.brightness;self.percent.set_text(f'{self.desired_brightness}%')
        self.home_sync=False;self.caption.set_text(client.name)
        for device in self.devices:
            if device['id'] not in self.clients:
                self.clients[device['id']]=DeviceClient(device,self.transport);self.clients[device['id']].page=lambda:self.editor.page
            if device['id'] not in self.device_buttons:
                button=Gtk.Button(label=device['name']);button.connect('clicked',lambda _b,i=device['id']:self.choose(i));self.device_picker_content.append(button);self.device_buttons[device['id']]=button
            self.device_buttons[device['id']].set_label(device['name'])
        status=json.loads(client('GetStatus')[0]);self.editor.shell.update_status(status)
        caption,label=self.editor.shell.nav_labels['keys'];self.editor.shell.nav_labels['keys']=(caption,'Keys' if client.device_model=='mk2' else 'Keys & Dials')
        self.editor.shell.navigate(self.editor.shell.section)
        count,pending=snapshot['history'];self.history.set_subtitle(f'{count} active generations · archive pending' if pending else f'{count} active generations · older archives are retained separately')
        self.archive.set_label('Resume verified archive' if pending else 'Archive older saves');self.archive.set_sensitive(pending or count>8)
    def cleanup_lab(self,*_):
        if hasattr(self,'editor'):self.editor.cleanup()
        from audio_monitor import MONITOR
        MONITOR.close()
        if self.isolation:self.isolation.cleanup()
    def finish_quit(self):self.quit()
    def get_clipboard(self):return self.editor.get_clipboard()

if __name__=='__main__':
    if os.environ.get('DECKSMITH_ISOLATED_TEST')!='1':raise SystemExit('Only available in the isolated VirtualDeck lab.')
    ProfileLab().run(None)
