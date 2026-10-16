"""Normal saved-device editor for the permanent background service."""
import json,subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from gi.repository import Adw,Gio,Gtk,GLib
from profile_lab import ProfileLab
from profile_client import control_transport
from panel import Panel

class PermanentWorkspace(ProfileLab):
    # These preferences always manage the installed normal unit. They neither
    # start a second daemon nor change per-device saved layout assignments.
    refresh_startup=Panel.refresh_startup
    finish_startup=Panel.finish_startup
    startup_changed=Panel.startup_changed
    quit_requested=Panel.quit_requested
    finish_quit=Panel.finish_quit
    def __init__(self):
        super().__init__(permanent=True)
        self.pool=ThreadPoolExecutor(max_workers=1);self.poll_pool=ThreadPoolExecutor(max_workers=1)
        self.closed=False;self.startup_pending=False;self.startup_updating=False;self.updating=False;self.refreshing=False;self.running=True
        action=Gio.SimpleAction.new('quit-decksmith',None);action.connect('activate',self.quit_requested);self.add_action(action)
    def activate_lab(self,*args):
        first=not hasattr(self,'editor')
        super().activate_lab(*args)
        if hasattr(self,'editor'):
            previous=self.window;self.window=self.editor
            if previous is not self.editor:previous.destroy()
            if first:
                self.refresh_startup();self.refresh_timer=GLib.timeout_add_seconds(1,self.refresh_live)
    def build_home(self):
        super().build_home()
        self.device_group.set_description('Independent saved pages and controls for the selected device.')
        self.service_row=self.background_row;self.service_row.set_subtitle('Running')
        self.service_button=Gtk.Button(label='Stop',valign=Gtk.Align.CENTER,tooltip_text='Stop background controls and blank every connected Deck.')
        self.service_button.connect('clicked',lambda *_:self.service_action('restart' if getattr(self,'restart_runtime',None) else 'stop' if self.running else 'start'));self.service_row.add_suffix(self.service_button)
        self.startup_row=self.integration_rows['Start Decksmith at login'];self.startup_row.connect('notify::active',self.startup_changed)
        self.lock_row=self.integration_rows['Auto-Lock'];self.lock_row.connect('notify::active',self.lock_changed)
    def refresh(self,snapshot=None,status=None):
        super().refresh(snapshot,status)
        self.show_integration(self.editor.shell.status)
    def show_integration(self,status):
        self.running=status.get('running',False);self.updating=True
        self.service_button.set_label('Restart' if getattr(self,'restart_runtime',None) else 'Stop' if self.running else 'Start');self.service_button.set_sensitive(True)
        self.service_row.set_subtitle('Restart needed to apply device changes' if getattr(self,'restart_runtime',None) else 'Running · all saved devices' if self.running else 'Stopped · all Decks blank')
        lock=status.get('auto_lock',{});self.metadata.set_sensitive(self.running and status.get('connected',False) and status.get('input_ready',False) and status.get('display_ready',False) and not lock.get('locked',False) and not status.get('unavailable',False));self.lock_row.set_active(lock.get('enabled',False));self.lock_row.set_sensitive(self.running and bool(lock))
        self.lock_row.set_subtitle('Locked · controls disabled on all devices' if lock.get('locked') else 'Locks every connected Deck with your session' if lock.get('enabled') else 'Off · enable to lock all Decks with your session')
        self.updating=False
    def refresh_live(self):
        if self.closed or not hasattr(self,'editor') or self.editor.pending or self.switching or self.refreshing:return True
        self.refreshing=True;identity=self.selected;client=self.clients[identity];token=dict(client.token or {})
        def task():
            snapshot=json.loads(self.transport('ListDevices')[0]);status=json.loads(client('GetStatus')[0])
            changed=bool(token) and (snapshot.get('runtime')!=token.get('runtime') or status.get('connection')!=token.get('connection') or status.get('binding')!=token.get('binding'))
            profile=self.transport('GetProfile',GLib.Variant('(s)',(identity,))) if changed else None
            return snapshot,status,profile
        future=self.poll_pool.submit(task)
        def finish():
            self.refreshing=False
            if self.closed:return False
            if identity!=self.selected or self.editor.pending or self.switching:return False
            try:
                snapshot,status,profile=future.result()
                if getattr(self,'restart_runtime',None) and snapshot.get('runtime')!=self.restart_runtime:self.restart_runtime=None
                if getattr(self,'restart_runtime',None):status['unavailable']=True
                if profile is not None:
                    value=json.loads(profile[0]);layout=value['layout'];draft=self.editor.draft
                    client.accept(profile)
                    if draft is not None and draft.dirty:
                        if draft.data==layout:draft.saved_now()
                        # Reconnect/restart can refresh context without losing a
                        # local draft. A remotely changed base stays stale so a
                        # later save cannot overwrite another client's edits.
                        if draft.saved!=layout:client.token['revision']=token['revision']
                    else:self.editor.loaded((layout,status))
                self.refresh(snapshot,status);self.editor.follow_status(status)
            except Exception:
                # Keep all typed fields and drafts intact while the unit stops.
                status={'running':False,'connected':False,'unavailable':False}
                self.editor.shell.update_status(status);self.show_integration(status)
            return False
        future.add_done_callback(lambda *_:GLib.idle_add(finish));return True
    def service_action(self,action):
        if action not in ('start','stop','restart'):raise ValueError('Unknown service action')
        self.service_button.set_sensitive(False)
        def task():subprocess.run([str(Path(__file__).resolve().parents[2]/'scripts/decksmith-service.sh'),action],check=True,capture_output=True,timeout=15)
        def done(_):self.running=action in ('start','restart');self.refresh_live()
        self.editor.request(task,done,'Could not change background controls. Your saved profiles and drafts are retained.')
    def lock_changed(self,row,_spec):
        if self.updating or self.editor.pending:return
        enabled=row.get_active();row.set_sensitive(False)
        self.editor.request(lambda:control_transport('SetAutoLock',GLib.Variant('(b)',(enabled,))),lambda _:self.refresh_live(),'Could not save Auto-Lock. Your device layouts are unchanged.')
    def request_error_message(self,error):
        text=str(error)
        if 'profile_saved_apply_failed' in text:
            self.restart_runtime=(self.clients[self.selected].token or {}).get('runtime')
            return 'Layout saved; restart Background controls on Home to apply it. Your draft is retained.'
        if 'live_profile_reload_required' in text:
            self.restart_runtime=(self.clients[self.selected].token or {}).get('runtime')
            return 'Restart Background controls on Home before applying this draft. Your draft is retained.'
        return None
    def guard_before_close(self):
        other=[identity for identity,view in self.shelf.views.items() if identity!=self.selected and view.draft.dirty]
        pending=list(dict.fromkeys(other+list(self.settings_drafts)))
        if not pending:return False
        if getattr(self,'draft_dialog',None):return True
        names=[self.clients[i].name for i in pending if i in self.clients]
        dialog=Adw.AlertDialog(heading='Review unsaved device changes?',body='Save or discard changes for '+', '.join(names)+' before closing Decksmith.')
        dialog.add_response('keep','Keep editing');dialog.add_response('review','Review changes');dialog.set_default_response('review');dialog.set_close_response('keep')
        def response(_dialog,choice):
            self.draft_dialog=None;self.quit_all=False
            if choice=='review':
                identity=pending[0]
                if identity!=self.selected:self.choose(identity)
                self.editor.shell.navigate('home' if identity in self.settings_drafts else 'keys')
        dialog.connect('response',response);self.draft_dialog=dialog;dialog.present(self.editor);return True
    def cleanup_lab(self,*args):
        self.closed=True
        timer=getattr(self,'refresh_timer',None)
        if timer:GLib.source_remove(timer);self.refresh_timer=None
        for name in ('pool','poll_pool'):
            pool=getattr(self,name,None)
            if pool:pool.shutdown(wait=False,cancel_futures=True)
        super().cleanup_lab(*args)
