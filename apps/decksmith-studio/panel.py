#!/usr/bin/env python3
"""Thin native control-panel client; the Rust daemon owns all device access."""
import json
import sys
sys.dont_write_bytecode=True
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import gi

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Adw, Gtk, Gio, GLib, GdkPixbuf, Gdk

ROOT = Path(__file__).resolve().parents[2]
BUS = 'cc.senecal.Decksmith'
PATH = '/cc/senecal/Decksmith'
IFACE = BUS + '.Control1'
LAYOUTS = ['audio', 'navigation', 'custom']

def call(method, args=None):
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    reply = bus.call_sync(BUS, PATH, IFACE, method, args, None,
                          Gio.DBusCallFlags.NO_AUTO_START, 2500, None)
    if method == 'PreviewKeys':
        return reply.get_child_value(0).get_data_as_bytes().get_data()
    if method == 'PreviewTouch':
        # Avoid converting 240,000 image bytes into individual Python integers.
        return (reply.get_child_value(0).get_data_as_bytes().get_data(),
                reply.get_child_value(1).get_string())
    return reply.unpack()

def read_status():
    # The daemon is the authoritative fast path. Do not spawn systemctl every
    # second while its session-bus endpoint is healthy.
    try:
        status=json.loads(call('GetStatus')[0])
        if status.get('api_version')!=1:raise ValueError('Unsupported interface')
        return dict(status,running=True)
    except Exception:
        result=subprocess.run(['systemctl','--user','is-active','decksmith.service'],capture_output=True,text=True,timeout=3)
        active=result.stdout.strip()=='active'
        return {'running':active,'connected':False,'unavailable':active}

class Panel(Adw.Application):
    def __init__(self):
        super().__init__(application_id='cc.senecal.Decksmith.Studio')
        self.pool = ThreadPoolExecutor(max_workers=1)
        from audio_apps import refresh as discover_audio_apps
        self.pool.submit(discover_audio_apps)
        self.status_pool = ThreadPoolExecutor(max_workers=1)
        self.refreshing = False
        self.status_revision = 0
        self.busy = False
        self.updating = False
        self.running = False
        self.last_brightness = None
        self.startup_updating = False
        self.startup_pending = False
        self.quit_all=False
        self.launch_status_checked=False
        self.launch_prompt=None
        quit_action=Gio.SimpleAction.new('quit-decksmith',None)
        quit_action.connect('activate',self.quit_requested);self.add_action(quit_action)
        self.connect('activate', self.activate)
        self.connect('shutdown', self.shutdown_resources)

    def quit_requested(self,*_):
        ed=getattr(self,'editor',None)
        if ed is None or ed.pending:return
        self.quit_all=True
        ed.present();ed.close()

    def finish_quit(self):
        ed=self.editor
        def stop():
            subprocess.run(['systemctl','--user','stop','decksmith.service'],check=True,capture_output=True,timeout=15)
        def done(_):
            self.quit_all=False
            ed.close()
        ed.request(stop,done,'Could not stop background controls. Decksmith remains open; try Quit again.')

    def shutdown_resources(self, *_):
        timer=getattr(self,'refresh_timer',None)
        if timer:GLib.source_remove(timer)
        self.pool.shutdown(wait=False,cancel_futures=True)
        self.status_pool.shutdown(wait=False,cancel_futures=True)
        editor=getattr(self,'editor',None)
        if editor is not None:editor.cleanup()

    def activate(self, _app):
        if hasattr(self, 'window'):
            self.refresh_startup()
            self.window.present()
            return
        self.window = Adw.ApplicationWindow(application=self, title='Decksmith')
        self.window.set_default_size(1200, 820)
        toolbar = Adw.ToolbarView()
        toolbar.add_top_bar(Adw.HeaderBar())
        self.toasts = Adw.ToastOverlay()
        toolbar.set_content(self.toasts)
        self.window.set_content(toolbar)
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=24)
        for setter in ('set_margin_top', 'set_margin_bottom', 'set_margin_start', 'set_margin_end'):
            getattr(box, setter)(24)
        scroll = Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER)
        scroll.set_child(box)
        self.toasts.set_child(scroll)
        hero = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=18)
        sheet = GdkPixbuf.Pixbuf.new_from_file(str(ROOT / 'brand/decksmith-makers-mark-brand-sheet.png'))
        logo = sheet.new_subpixbuf(840, 48, 212, 212).copy()
        self.logo_texture = Gdk.Texture.new_for_pixbuf(logo)
        picture = Gtk.Picture.new_for_paintable(self.logo_texture)
        picture.set_size_request(80, 80)
        hero.append(picture)
        titles = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5, valign=Gtk.Align.CENTER)
        title = Gtk.Label(label='Decksmith', xalign=0)
        title.add_css_class('title-1')
        titles.append(title)
        subtitle = Gtk.Label(label='Open Control, Forged for Linux.', xalign=0)
        subtitle.add_css_class('dim-label')
        titles.append(subtitle)
        hero.append(titles)
        self.home_logo = hero  # The full brand treatment lives on About.
        group = Adw.PreferencesGroup(title='Your device')
        self.device_row = Adw.ActionRow(title='Choose Device', subtitle='Stream Deck + · checking connection…')
        self.device_row.set_tooltip_text('Current Stream Deck +. Independent multi-device control is planned for a later milestone.')
        self.device_row.add_prefix(Gtk.Image.new_from_icon_name('input-gaming-symbolic'))
        group.add(self.device_row)
        self.service_row = Adw.ActionRow(title='Background controls', subtitle='Checking service…')
        self.service_button = Gtk.Button(label='Start', valign=Gtk.Align.CENTER)
        self.service_button.connect('clicked', self.service_clicked)
        self.service_row.add_suffix(self.service_button)
        group.add(self.service_row)
        self.startup_row = Adw.SwitchRow(title='Start Decksmith at login', subtitle='Checking login settings…')
        self.startup_row.set_sensitive(False)
        self.startup_row.connect('notify::active', self.startup_changed)
        group.add(self.startup_row)
        self.lock_row = Adw.SwitchRow(title='Auto-Lock', subtitle='Disable device controls when Fedora is locked')
        self.lock_row.set_sensitive(False)
        self.lock_row.connect('notify::active', self.lock_changed)
        group.add(self.lock_row)
        box.append(group)
        settings = Adw.PreferencesGroup(title='Device settings')
        self.layout_row = Adw.ComboRow(title='Saved layout', model=Gtk.StringList.new(['Audio controls', 'Navigation', 'My layout']))
        self.layout_row.connect('notify::selected', self.layout_changed)
        settings.add(self.layout_row)
        self.brightness_row = Adw.ActionRow(title='Brightness', subtitle='Choose a level, then apply')
        settings.add(self.brightness_row)
        controls = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8, valign=Gtk.Align.CENTER)
        self.brightness_input = Gtk.SpinButton.new_with_range(0, 100, 1)
        self.brightness_input.set_numeric(True)
        self.brightness_input.set_digits(0)
        self.brightness_input.set_width_chars(3)
        self.brightness_input.set_value(50)
        self.brightness_input.update_property([Gtk.AccessibleProperty.LABEL], ['Brightness percentage'])
        controls.append(self.brightness_input)
        controls.append(Gtk.Label(label='%'))
        self.apply = Gtk.Button(label='Apply', valign=Gtk.Align.CENTER)
        self.apply.add_css_class('suggested-action')
        self.apply.connect('clicked', self.brightness_clicked)
        controls.append(self.apply)
        self.brightness_row.add_suffix(controls)
        box.append(settings)
        editor_group = Adw.PreferencesGroup()
        editor_row = Adw.ActionRow(title='Pages and buttons', subtitle='Customize key labels, artwork and actions')
        self.edit_button = Gtk.Button(label='Edit layout', valign=Gtk.Align.CENTER)
        self.edit_button.connect('clicked', self.open_editor)
        editor_row.add_suffix(self.edit_button)
        editor_group.add(editor_row)
        self.automatic_row=Adw.ActionRow(title='Automatic page switching',subtitle='Checking GNOME integration…')
        editor_group.add(self.automatic_row)
        box.append(editor_group)
        from control_feedback import FeedbackView
        self.feedback=FeedbackView();box.append(self.feedback)
        note = Gtk.Label(label='Closing this window leaves your controls running.\nLogin startup runs in the background without opening this window.', wrap=True, xalign=0)
        note.add_css_class('dim-label')
        box.append(note)
        for widget in (self.service_button,self.layout_row,self.edit_button,self.apply,self.brightness_input):
            widget.set_sensitive(False)
        self.home_content = scroll
        self.toasts.set_child(None)
        toolbar.set_content(None)
        previous = self.window
        from editor import Editor
        self.editor = Editor(self, call, integrated=True)
        self.window = self.editor
        previous.destroy()
        self.editor.present()
        self.refresh_startup()
        self.refresh()
        self.refresh_timer = GLib.timeout_add_seconds(1, self.refresh)
        if os.environ.get('DECKSMITH_PANEL_SNAPSHOT'):
            GLib.timeout_add_seconds(3, self.snapshot)

    def refresh_startup(self):
        if self.startup_pending:
            return
        from startup import read
        self.startup_pending = True
        future = self.pool.submit(read)
        future.add_done_callback(lambda result: GLib.idle_add(self.finish_startup, result))

    def finish_startup(self, future):
        self.startup_pending = False
        self.startup_updating = True
        try:
            state = future.result()
            self.startup_confirmed = state['enabled']
            self.startup_row.set_active(state['enabled'])
            self.startup_row.set_sensitive(state['available'])
            self.startup_row.set_subtitle(state['detail'])
        except Exception:
            self.startup_row.set_active(getattr(self, 'startup_confirmed', False))
            self.startup_row.set_sensitive(True)
            self.startup_row.set_subtitle('Could not confirm login settings. Try again.')
            self.toasts.add_toast(Adw.Toast.new('Could not save or read the login preference.'))
        self.startup_updating = False
        return False

    def startup_changed(self, row, _spec):
        if self.startup_updating or self.startup_pending:
            return
        from startup import set_enabled
        self.startup_pending = True
        row.set_sensitive(False)
        future = self.pool.submit(set_enabled, row.get_active())
        future.add_done_callback(lambda result: GLib.idle_add(self.finish_startup, result))

    def snapshot(self):
        try:
            paintable = Gtk.WidgetPaintable.new(self.window)
            snapshot = Gtk.Snapshot()
            paintable.snapshot(snapshot, self.window.get_width(), self.window.get_height())
            texture = self.window.get_renderer().render_texture(snapshot.to_node(), None)
            texture.save_to_png(os.environ['DECKSMITH_PANEL_SNAPSHOT'])
        except Exception as error:
            print(f'Panel snapshot unavailable: {error}', flush=True)
        return False

    def submit(self, function, callback):
        if self.busy:
            return
        self.busy = True
        self.status_revision += 1
        self.service_button.set_sensitive(False)
        self.layout_row.set_sensitive(False)
        self.edit_button.set_sensitive(False)
        self.apply.set_sensitive(False)
        future = self.pool.submit(function)
        def finish():
            self.busy = False
            try:
                callback(future.result())
            except Exception as error:
                print(f'Panel request failed: {error}', flush=True)
                self.toasts.add_toast(Adw.Toast.new('Could not apply this change. Check the service and device connection.'))
                self.service_button.set_sensitive(True)
            return False
        future.add_done_callback(lambda _future: GLib.idle_add(finish))

    def refresh(self):
        if self.busy or self.refreshing:
            return True
        self.refreshing = True
        revision = self.status_revision
        future = self.status_pool.submit(read_status)
        def finish():
            self.refreshing = False
            if self.busy or revision != self.status_revision:
                return False
            try:
                self.show_status(future.result())
            except Exception as error:
                print(f'Panel status check failed: {error}', flush=True)
                self.show_status({'running':self.running,'connected':False,'unavailable':True})
            return False
        future.add_done_callback(lambda _future: GLib.idle_add(finish))
        return True

    def show_status(self, status):
        self.updating = True
        self.running = status['running']
        available = self.running and not status.get('unavailable')
        connected = status.get('connected', False)
        self.service_row.set_subtitle('Running' if self.running else 'Stopped')
        self.service_button.set_label('Stop' if self.running else 'Start')
        self.service_button.set_sensitive(True)
        if status.get('unavailable'):
            text = 'Control interface unavailable'
        elif connected:
            text = 'Connected' if status.get('display_ready') else 'Connected · updating displays'
        else:
            text = 'Waiting for device' if self.running else 'Start background controls to connect'
        self.device_row.set_subtitle('Stream Deck + · '+text)
        focus=status.get('automatic_pages',{})
        self.automatic_row.set_subtitle('Ready · assign applications in Pages' if focus.get('available') else 'Enable the Decksmith GNOME extension; sign in again after an extension update')
        self.feedback.update(status)
        count=len({n['check']['slot'] for n in status.get('attention',[])}) if connected else 0
        if count:self.service_row.set_subtitle(f'Running · {count} control'+('s' if count!=1 else '')+' need attention')
        lock = status.get('auto_lock', {})
        self.lock_row.set_active(lock.get('enabled', False))
        self.lock_row.set_sensitive(available and bool(lock))
        if not lock:
            lock_text = 'Restart updated background controls to use Auto-Lock'
        elif lock.get('enabled') and lock.get('locked'):
            lock_text = 'Locked · controls disabled' if lock.get('available') else 'Lock state unavailable · controls held locked'
        elif lock.get('available'):
            lock_text = 'Ready · locks with your session' if lock.get('enabled') else 'Off · enable to lock controls with your session'
        else:
            lock_text = 'Session lock detection unavailable'
        self.lock_row.set_subtitle(lock_text)
        if lock.get('locked'):
            self.device_row.set_subtitle('Locked · controls disabled' if connected else 'Locked · waiting for device')
            available = False
        self.layout_row.set_sensitive(available)
        self.edit_button.set_sensitive(True)
        self.brightness_input.set_sensitive(available and connected)
        self.apply.set_sensitive(available and connected)
        if status.get('layout') in LAYOUTS:
            self.layout_row.set_selected(LAYOUTS.index(status['layout']))
        value = status.get('brightness')
        if value != self.last_brightness:
            if value is not None:
                self.brightness_input.set_value(value)
            self.last_brightness = value
        self.brightness_row.set_subtitle(f'Saved level: {value}%' if value is not None else 'Not set by Decksmith yet · choose a level, then apply')
        self.updating = False
        editor = getattr(self, "editor", None)
        if editor is not None and editor.get_visible():
            editor.follow_status(status)
            if hasattr(editor,"shell"):editor.shell.update_status(status)
        self.offer_start_at_launch(status)

    def offer_start_at_launch(self,status):
        # Wait for a reliable first status, then ask at most once per launch.
        if self.launch_status_checked or status.get('unavailable'):
            return
        self.launch_status_checked=True
        if status.get('running'):
            return
        from i18n import gettext as tr
        dialog=Adw.AlertDialog(heading=tr('Start background controls?'),
            body=tr('Background controls are stopped. Start them to connect your Stream Deck and use its keys and dials.'))
        dialog.add_response('later',tr('Not now'))
        dialog.add_response('start',tr('Start controls'))
        dialog.set_response_appearance('start',Adw.ResponseAppearance.SUGGESTED)
        dialog.set_default_response('start');dialog.set_close_response('later')
        def response(_dialog,choice):
            self.launch_prompt=None
            if choice=='start':self.service_action('start')
        dialog.connect('response',response)
        self.launch_prompt=dialog
        dialog.present(self.window)

    def change(self, method, args):
        def task():
            call(method, args)
            return read_status()
        self.submit(task, self.show_status)

    def open_editor(self, _button):
        self.editor.shell.navigate('keys')

    def lock_changed(self, row, _spec):
        if not self.updating and not self.busy:
            row.set_sensitive(False)
            self.change('SetAutoLock', GLib.Variant('(b)', (row.get_active(),)))

    def brightness_clicked(self, _button):
        self.change('SetBrightness', GLib.Variant('(y)', (round(self.brightness_input.get_value()),)))

    def layout_changed(self, row, _spec):
        if not self.updating and not self.busy:
            if getattr(self,'editor',None) and self.editor.draft and self.editor.draft.dirty:
                self.toasts.add_toast(Adw.Toast.new('Save or discard your layout edits before changing the saved layout.'))
                self.refresh()
                return
            self.change('SetLayout', GLib.Variant('(s)', (LAYOUTS[row.get_selected()],)))

    def service_clicked(self, _button):
        self.service_action('stop' if self.running else 'start')

    def service_action(self,action):
        def task():
            subprocess.run([str(ROOT / 'scripts/decksmith-service.sh'), action], check=True,
                           capture_output=True, text=True, timeout=8)
            return read_status()
        self.submit(task, self.show_status)

if __name__ == '__main__':
    Panel().run(None)
