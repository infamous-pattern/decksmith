#!/usr/bin/env python3
"""Opt-in native profile editor lab; only the separate VirtualDeck endpoint."""
import json,os,tempfile
from pathlib import Path
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Adw,Gtk,GLib,Gdk,GdkPixbuf
from editor import Editor
from profile_client import DeviceClient,transport

class ProfileLab(Adw.Application):
    def __init__(self):
        if os.environ.get('DECKSMITH_ISOLATED_TEST')!='1':raise ValueError('Isolated test required')
        self.isolation=tempfile.TemporaryDirectory(prefix='decksmith-profile-editor-')
        for variable,name in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_CACHE_HOME','cache'),('XDG_STATE_HOME','state'),('XDG_RUNTIME_DIR','runtime')]:
            directory=Path(self.isolation.name)/name;directory.mkdir(mode=0o700);os.environ[variable]=str(directory)
        os.environ['PULSE_SERVER']='unix:'+str(Path(self.isolation.name)/'no-audio.sock')
        os.environ.pop('DECKSMITH_PLUGIN_LAB_CONNECTION',None)
        # The endpoint is a disposable session bus; desktop audio/plug-in
        # selectors cannot reach the user's normal runtime sockets or settings.
        super().__init__(application_id='cc.senecal.Decksmith.ProfileEditorLab')
        self.quit_all=False;self.editors={};self.clients={};self.selected=None;self.switching=False
        self.connect('activate',self.activate_lab);self.connect('shutdown',self.cleanup_lab)
    def activate_lab(self,*_):
        if self.selected:
            self.editor.present();return
        self.window=Adw.ApplicationWindow(application=self)
        from pathlib import Path
        sheet=GdkPixbuf.Pixbuf.new_from_file(str(Path(__file__).resolve().parents[2]/'brand/decksmith-makers-mark-brand-sheet.png'))
        self.logo_texture=Gdk.Texture.new_for_pixbuf(sheet.new_subpixbuf(840,48,212,212).copy())
        snapshot=json.loads(transport('ListDevices')[0])
        if snapshot.get('mode')!='virtual-lab' or snapshot.get('api_version')!=2:raise ValueError('Wrong lab service')
        self.devices=snapshot['devices'];self.generation=snapshot['generation']
        for device in self.devices:self.clients[device['id']]=DeviceClient(device,transport)
        self.choose(snapshot['selected'])
    def choose(self,identity):
        if self.switching or identity==self.selected:return
        old=getattr(self,'editor',None)
        if old is not None and old.pending:
            self.switching=True
            old.device_picker.set_selected(next(i for i,d in enumerate(self.devices) if d['id']==self.selected))
            self.switching=False;return
        self.switching=True
        try:
            # Switching keeps independent in-memory drafts, histories and pending
            # preview work; hidden editors pause preview polling through mapping.
            snapshot=json.loads(transport('SelectDevice',GLib.Variant('(st)',(identity,self.generation)))[0])
            self.generation=snapshot['generation']
            ed=self.editors.get(identity)
            if ed is None or ed.closed:
                client=self.clients[identity];ed=Editor(self,client)
                client.page=lambda ed=ed:ed.page
                self.editors[identity]=ed
                picker=Gtk.DropDown.new_from_strings([d['name'] for d in self.devices]);ed.device_picker=picker
                picker.set_tooltip_text('Choose Device · each device keeps its own saved layout and unsaved draft')
                picker.connect('notify::selected',lambda widget,*_:self.choose(self.devices[widget.get_selected()]['id']) if not self.switching else None)
                ed.header.pack_start(picker)
                ed.set_title('Decksmith · Virtual device profiles')
                ed.set_modal(False)
            ed.device_picker.set_selected(next(i for i,d in enumerate(self.devices) if d['id']==identity))
            if old is not None:old.set_visible(False)
            self.selected=identity;self.editor=ed;ed.present()
        finally:self.switching=False
    def refresh(self):
        snapshot=json.loads(transport('ListDevices')[0]);self.generation=snapshot['generation']
    def cleanup_lab(self,*_):
        for ed in self.editors.values():ed.cleanup()
        from audio_monitor import MONITOR
        MONITOR.close()
        self.isolation.cleanup()
    def finish_quit(self):self.quit()

if __name__=='__main__':
    if os.environ.get('DECKSMITH_ISOLATED_TEST')!='1':raise SystemExit('Only available in the isolated VirtualDeck lab.')
    ProfileLab().run(None)
