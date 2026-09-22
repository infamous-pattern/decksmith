"""Native eight-key editor. Saving is an explicit daemon request."""
import json
from uuid import uuid4
from concurrent.futures import ThreadPoolExecutor
import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Adw, Gtk, GLib, Gdk, Gio, Pango, Gsk, Graphene
from audio_target_picker import AudioTargetPicker
from media_picker import MediaPicker
from editor_model import Draft
from action_labels import default_label, uses_default
from media_artwork import LABELS as MEDIA_LABELS, populate as populate_media
from applications import installed, icon_png, website_png
from artwork import dimensions
from website_icon import fetch as fetch_website_icon

LABEL_COLORS = {'default':'#f0e6d7','white':'#ffffff','black':'#000000','red':'#ff4646','orange':'#ff9b3c','yellow':'#ffe13c','green':'#5ae678','blue':'#5aaaff','purple':'#c382ff'}

ACTIONS = ['system','none', 'go_to_page', 'volume_down', 'volume_up', 'mute_toggle', 'volume_adjust', 'next_page', 'previous_page', 'open_application', 'open_website', 'media_play_pause', 'media_previous', 'media_next', 'audio_adjust', 'audio_mute', 'audio_select', 'push_to_talk']
ACTION_LABELS = ['System control','No action', 'Go to page', 'Volume down', 'Volume up', 'Mute output', 'Adjust volume', 'Next page', 'Previous page', 'Open application', 'Open website', 'Media · Play / Pause', 'Media · Previous track', 'Media · Next track', 'Audio · Adjust volume', 'Audio · Toggle mute', 'Audio · Select device', 'Microphone · Push to talk']

class KeyPreviewLayout(Gtk.LayoutManager):
    """Fixed key cells; GTK's default button layout otherwise follows its child."""
    def do_get_request_mode(self,widget):
        return Gtk.SizeRequestMode.CONSTANT_SIZE
    def do_measure(self,widget,orientation,for_size):
        size=getattr(widget,'preview_size',102)
        return (size,size,-1,-1)
    def do_allocate(self,widget,width,height,baseline):
        child=widget.get_child()
        if child is not None:child.allocate(width,height,baseline,None)

class KeyPreviewButton(Gtk.Button):
    """Keep content changes from changing the hardware preview geometry."""
    def __init__(self):
        super().__init__()
        self.set_layout_manager(KeyPreviewLayout())

    def do_snapshot(self,snapshot):
        # Clip full-bleed artwork to the same rounded glass shape as text keys.
        rect=Graphene.Rect()
        rect.init(0,0,self.get_width(),self.get_height())
        rounded=Gsk.RoundedRect()
        rounded.init_from_rect(rect,14)
        snapshot.push_rounded_clip(rounded)
        child=self.get_child()
        if child is not None:self.snapshot_child(child,snapshot)
        snapshot.pop()

class Editor(Adw.ApplicationWindow):
    def __init__(self, owner, call, integrated=False):
        self.integrated=integrated
        self.closed=False
        self.close_after_save=False
        super().__init__(application=owner, transient_for=owner.window, modal=not integrated, title='Decksmith' if integrated else 'Edit layout')
        self.owner, self.call = owner, call
        self.draft = None
        self.page = self.key = 0
        self.control_kind="key"
        self.dial_index=0
        self.dial_controls=None
        self.syncing = False
        self.pending = False
        self.key_drag = None
        self.device_page = None
        self.device_layout = None
        self.pool = ThreadPoolExecutor(max_workers=1)
        # Monitor geometry is an upper bound; the compositor applies the actual
        # work area (panels/docks) when mapping the window, including on Wayland.
        monitor=Gdk.Display.get_default().get_monitor_at_surface(owner.window.get_surface()) if owner.window.get_surface() else None
        bounds=monitor.get_geometry() if monitor else None
        self.set_default_size(min(1440,bounds.width-64) if bounds else 1440,
                              min(900,bounds.height-96) if bounds else 900)
        self.set_size_request(940,600)
        self.connect('close-request', self.close_requested)
        toolbar = Adw.ToolbarView()
        header = Adw.HeaderBar()
        self.save_button = Gtk.Button(label='Save and Apply')
        self.save_button.add_css_class('suggested-action')
        self.save_button.connect('clicked', self.save)
        header.pack_end(self.save_button)
        self.reset_button = Gtk.Button(label='Discard changes')
        self.reset_button.connect('clicked', self.reset)
        header.pack_start(self.reset_button)
        self.undo_button=Gtk.Button(icon_name='edit-undo-symbolic',tooltip_text='Undo · Ctrl+Z')
        self.redo_button=Gtk.Button(icon_name='edit-redo-symbolic',tooltip_text='Redo · Ctrl+Shift+Z')
        self.undo_button.connect('clicked',lambda *_:self.history_step(False))
        self.redo_button.connect('clicked',lambda *_:self.history_step(True))
        header.pack_start(self.undo_button);header.pack_start(self.redo_button)
        shortcuts=Gtk.EventControllerKey()
        shortcuts.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        shortcuts.connect('key-pressed',self.history_shortcut)
        self.add_controller(shortcuts)
        self.history_group=None
        toolbar.add_top_bar(header)
        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        for edge in ('top','bottom','start','end'): getattr(outer,f'set_margin_{edge}')(24)
        toolbar.set_content(outer)
        self.set_content(toolbar)
        intro = Gtk.Label(label='Select a key, touch-strip section, or dial to edit. Turn off Edit keys to try key navigation.', xalign=0, wrap=True)
        intro.add_css_class('dim-label')
        outer.append(intro)
        layout_menu=Gtk.MenuButton(icon_name='open-menu-symbolic',tooltip_text='Layout options')
        layout_popover=Gtk.Popover()
        transfers=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=6)
        for edge in ('top','bottom','start','end'):getattr(transfers,'set_margin_'+edge)(10)
        layout_popover.set_child(transfers);layout_menu.set_popover(layout_popover)
        for title,handler in [('Export layout…',self.export_layout),('Import layout…',self.import_layout),('Restore previous',self.restore_previous)]:
            button=Gtk.Button(label=title)
            button.connect('clicked',lambda button,handler=handler:(layout_popover.popdown(),handler(button)))
            transfers.append(button)
        header.pack_end(layout_menu)
        theme_button=Gtk.Button(label="Theme",tooltip_text="Appearance presets and shared defaults")
        theme_button.connect("clicked",self.open_theme);header.pack_end(theme_button)
        self.body = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=24, vexpand=True)
        outer.append(self.body)
        sidebar=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16,valign=Gtk.Align.START)
        sidebar.set_size_request(300,-1)
        sidebar_scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER,
                                          min_content_width=300,propagate_natural_width=True)
        sidebar_scroll.set_child(sidebar);self.body.append(sidebar_scroll)
        page_heading=Gtk.Box(spacing=8)
        heading=Gtk.Label(label='Pages',xalign=0,hexpand=True)
        heading.add_css_class('title-3');page_heading.append(heading)
        self.add_button=Gtk.Button(label='Add',tooltip_text='Add page')
        self.add_button.connect('clicked',self.add_page);page_heading.append(self.add_button)
        sidebar.append(page_heading)
        self.page_list=Gtk.ListBox(selection_mode=Gtk.SelectionMode.SINGLE)
        self.page_list.add_css_class('boxed-list')
        self.page_list.connect('row-selected',self.page_changed)
        self.page_rows=[]
        page_scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER,
                                      min_content_height=120,max_content_height=280,
                                      propagate_natural_height=True)
        page_scroll.set_child(self.page_list);sidebar.append(page_scroll)
        page_tools=Gtk.Box(spacing=6)
        self.page_tools={}
        for title,icon,handler in [('Move up','go-up-symbolic',lambda _b:self.move_page(-1)),
                                   ('Move down','go-down-symbolic',lambda _b:self.move_page(1)),
                                   ('Duplicate','edit-copy-symbolic',self.duplicate_page),
                                   ('Delete','user-trash-symbolic',self.delete_page)]:
            button=Gtk.Button(icon_name=icon,tooltip_text=title)
            button.connect('clicked',handler);page_tools.append(button)
            self.page_tools[title]=button
        sidebar.append(page_tools)
        left = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16, hexpand=False)
        left.set_valign(Gtk.Align.START)
        # Keep the approved physical geometry in both wide and compact layouts.
        self.device_column=Adw.Clamp(child=left,maximum_size=554,tightening_threshold=554,
                                     hexpand=True,valign=Gtk.Align.START)
        preview_scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER,hexpand=True)
        self.workspace=Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL,spacing=24,hexpand=True,valign=Gtk.Align.START)
        self.workspace.append(self.device_column)
        preview_scroll.set_child(self.workspace);self.body.append(preview_scroll)
        self.edit_mode = Gtk.Switch(active=True, valign=Gtk.Align.CENTER)
        mode_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        mode_row.append(Gtk.Label(label='Edit keys', xalign=0, hexpand=True))
        mode_row.append(self.edit_mode)
        left.append(mode_row)
        from device_preview import DialFace
        self.device_preview=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16)
        self.device_preview.add_css_class('device-preview')
        left.append(self.device_preview)
        brand=Gtk.Label(label='DECKSMITH  +',halign=Gtk.Align.CENTER)
        brand.add_css_class('device-brand');self.device_preview.append(brand)
        self.grid = Gtk.Grid(column_spacing=28, row_spacing=18, column_homogeneous=True, row_homogeneous=True)
        self.grid.set_valign(Gtk.Align.START)
        self.grid.set_halign(Gtk.Align.CENTER)
        self.keys = []
        self.key_sources = []
        self.key_targets = []
        for index in range(8):
            button = KeyPreviewButton()
            button.add_css_class('deck-key')
            button.add_css_class(f'key-slot-{index}')
            button.connect('clicked', lambda _button, key=index: self.activate_key(key))
            source = Gtk.DragSource(actions=Gdk.DragAction.MOVE)
            source.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
            source.connect('prepare', self.prepare_key_drag, index)
            source.connect('drag-end', self.end_key_drag)
            button.add_controller(source)
            target = Gtk.DropTarget.new(str, Gdk.DragAction.MOVE)
            target.connect('drop', self.drop_key, index)
            button.add_controller(target)
            self.key_sources.append(source)
            self.key_targets.append(target)
            button.set_tooltip_text('In Edit keys mode, drag onto another key to swap positions.')
            self.grid.attach(button,index%4,index//4,1,1)
            self.keys.append(button)
        self.device_preview.append(self.grid)
        from key_preview import KeyPreview
        self.key_preview=KeyPreview(self)
        self.connect("destroy",lambda *_:self.key_preview.stop())
        from touch_preview import TouchPreview
        self.touch_preview=TouchPreview(self.call)
        self.connect('destroy',lambda *_:self.touch_preview.stop())
        self.touch_buttons=[];self.dial_buttons=[]
        strip=Gtk.Grid(column_spacing=0,column_homogeneous=True)
        self.touch_preview.add_css_class("device-strip")
        strip.add_css_class("touch-hit-regions")
        dial_row=Gtk.Grid(column_spacing=20,column_homogeneous=True)
        dial_row.add_css_class("device-dials")
        for index in range(4):
            touch=Gtk.Button();touch.set_size_request(0,52)
            touch.connect('clicked',lambda _button,i=index:self.activate_dial(i))
            dial=Gtk.Button(halign=Gtk.Align.CENTER)
            dial.set_child(DialFace())
            dial.update_property([Gtk.AccessibleProperty.LABEL],[f'Dial {index+1}'])
            dial.set_size_request(66,66)
            dial.connect('clicked',lambda _button,i=index:self.activate_dial(i))
            strip.attach(touch,index,0,1,1);dial_row.attach(dial,index,0,1,1)
            self.touch_buttons.append(touch);self.dial_buttons.append(dial)
        self.touch_preview.add_overlay(strip)
        self.touch_preview.set_measure_overlay(strip,False)
        self.device_preview.append(self.touch_preview);self.device_preview.append(dial_row)
        preview_note=self.touch_preview.note
        preview_note.add_css_class('dim-label');left.append(preview_note)
        key_tools=Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL,spacing=8)
        self.key_tools={}
        for title,operation in [('Copy key','copy'),('Paste key','paste'),('Clear key','clear')]:
            button=Gtk.Button(label=title)
            button.connect('clicked',lambda _button,op=operation:self.key_operation(op))
            key_tools.append(button);self.key_tools[operation]=button
        left.append(key_tools)
        self.key_tools_row=key_tools
        self.edit_mode.connect('notify::active',lambda *_args:self.render() if self.draft else None)
        page_group = Adw.PreferencesGroup(title='Page settings')
        self.page_name = Adw.EntryRow(title='Page name')
        self.page_name.set_tooltip_text('Up to 24 uppercase or lowercase letters, numbers or spaces.')
        self.page_name.connect('changed', self.page_name_changed)
        page_group.add(self.page_name)
        self.page_default=Adw.SwitchRow(title='Use as default page')
        self.page_default.connect('notify::active',self.page_default_changed)
        self.page_apps=[app for app in installed() if app.get_id()!='cc.senecal.Decksmith.Studio.desktop']
        self.page_app_ids=[None]+[app.get_id() for app in self.page_apps]
        self.page_app_names=['No automatic switch']+[app.get_display_name() for app in self.page_apps]
        self.page_app=Adw.ComboRow(title='Switch here for',subtitle='Application',model=Gtk.StringList.new(self.page_app_names))
        self.page_app.connect('notify::selected',self.page_application_changed)
        page_group.add(self.page_app)
        page_group.add(self.page_default)
        sidebar.append(page_group)
        self.form = Adw.PreferencesGroup(title='Key 1')
        self.form.set_size_request(320,-1)
        self.form.set_hexpand(True)
        self.properties=Gtk.Stack()
        self.properties.set_hexpand(True)
        self.properties.set_hhomogeneous(True);self.properties.set_vhomogeneous(True)
        self.properties.add_named(self.form,'key')
        self.properties.set_valign(Gtk.Align.START)
        self.workspace.append(self.properties)
        compact=Adw.Breakpoint.new(Adw.BreakpointCondition.parse('max-width: 1160px' if integrated else 'max-width: 1340px'))
        compact.connect('apply',lambda *_:self.arrange_workspace(True))
        compact.connect('unapply',lambda *_:self.arrange_workspace(False))
        self.add_breakpoint(compact)
        self.label = Adw.EntryRow(title='Label')
        self.label.connect('changed', self.form_changed)
        self.form.add(self.label)
        self.label.set_tooltip_text("Up to 24 letters, numbers or spaces. Words wrap automatically.")
        self.appearance=Adw.ExpanderRow(title="Appearance",subtitle="Artwork, label position and colors")
        self.label_position = Adw.ComboRow(title='Label position',model=Gtk.StringList.new(['Hidden','Top','Middle','Bottom']))
        self.label_position.connect('notify::selected',self.form_changed)
        self.appearance.add_row(self.label_position)
        self.label_color=Adw.ComboRow(title='Text color',model=Gtk.StringList.new([name.title() for name in LABEL_COLORS]))
        self.label_color.connect('notify::selected',self.form_changed);self.appearance.add_row(self.label_color)
        self.label_background=Adw.SwitchRow(title='Transparent label background')
        self.label_background.connect('notify::active',self.form_changed);self.form.add(self.label_background)
        self.label_background.set_tooltip_text('Transparent shows the artwork behind the label.')
        self.field_errors={}
        self.add_field_error('label')
        self.key_background=Adw.ComboRow(title='Key background',model=Gtk.StringList.new(['Transparent · page color']+[name.title() for name in LABEL_COLORS if name!='default']))
        self.key_background.set_tooltip_text('Transparent uses the page background; the physical display is opaque. Full-key artwork covers the background.')
        self.key_background.connect('notify::selected',self.form_changed);self.appearance.add_row(self.key_background)
        self.art = Adw.ComboRow(title='Artwork', model=Gtk.StringList.new(['Text only', "Maker’s Mark", 'Custom or automatic image']))
        self.art.connect('notify::selected', self.form_changed)
        self.appearance.add_row(self.art)
        from theme_dialog import StyleRows
        self.key_style_rows=StyleRows(self.appearance,lambda:self.draft.data['pages'][self.page]['keys'][self.key].get('appearance',{}) if self.draft else {},self.set_key_style,('font','size','icon_size'),invalidated=self.render)
        reset_style=Gtk.Button(label="Reset to theme");reset_style.connect("clicked",self.reset_key_style);self.appearance.add_row(reset_style)
        library=Gtk.Button(label="Icon library…");library.connect("clicked",self.open_icon_library);self.appearance.add_row(library)
        image_tools=Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL,spacing=8)
        choose=Gtk.Button(label='Choose image…')
        choose.connect('clicked',self.choose_image);image_tools.append(choose)
        self.refresh_icon=Gtk.Button(label='Refresh website icon')
        self.refresh_icon.connect('clicked',lambda *_args:self.find_website_icon(force=True))
        image_tools.append(self.refresh_icon)
        self.appearance.add_row(image_tools)
        self.action = Adw.ComboRow(title='Action', model=Gtk.StringList.new(ACTION_LABELS))
        self.action.connect('notify::selected', self.form_changed)
        from action_catalog import ActionCatalog
        def category(kind):
            if kind in ('go_to_page','next_page','previous_page'):return 'Navigation'
            if kind=='system':return 'System'
            if kind.startswith('media_'):return 'Media'
            if kind.startswith('open_'):return 'Applications'
            return 'General' if kind=='none' else 'Audio'
        self.action_catalog=ActionCatalog([(category(kind),label,lambda i=i:self.action.set_selected(i))
                                          for i,(kind,label) in enumerate(zip(ACTIONS,ACTION_LABELS))])
        self.form.add(self.action if integrated else self.action_catalog)
        from homebridge_picker import HomebridgePicker
        self.homebridge=HomebridgePicker(self.assign_homebridge)
        self.form.add(self.homebridge)
        self.plugin_remove=Gtk.Button(label='Remove plugin assignment',halign=Gtk.Align.START)
        self.plugin_remove.connect('clicked',self.remove_plugin_assignment);self.form.add(self.plugin_remove)
        from system_controls import LABELS as SYSTEM_LABELS
        self.system_commands=list(SYSTEM_LABELS)
        self.system_picker=Adw.ComboRow(title='System action',model=Gtk.StringList.new(list(SYSTEM_LABELS.values())))
        self.system_picker.connect('notify::selected',self.form_changed);self.form.add(self.system_picker)
        self.system_note=Adw.ActionRow(title='System controls',subtitle='GNOME desktop controls. Amber status means unavailable; green means active. Reboot and Shutdown open desktop confirmation. Night Light switches immediately.')
        self.system_note.set_subtitle_lines(0);self.form.add(self.system_note)
        self.audio_target=AudioTargetPicker(self.form_changed)
        self.form.add(self.audio_target)
        self.media_player=MediaPicker(self.form_changed)
        self.form.add(self.media_player)
        self.apps = self.page_apps
        self.app_picker = Adw.ComboRow(title='Application', model=Gtk.StringList.new(['Choose an application'] + [app.get_display_name() for app in self.apps]))
        self.app_picker.connect('notify::selected', self.application_changed)
        self.form.add(self.app_picker)
        self.add_field_error("application")
        self.url = Adw.EntryRow(title='Website URL · Enter to find its icon')
        self.url.connect('changed', self.form_changed)
        self.form.add(self.url)
        self.add_field_error("url")
        self.url.connect('apply', lambda *_args: self.find_website_icon())
        self.url.set_show_apply_button(True)
        self.url_focus = Gtk.EventControllerFocus()
        self.url_focus.connect('leave', lambda *_args: self.find_website_icon())
        self.url.add_controller(self.url_focus)
        self.icon_pool = ThreadPoolExecutor(max_workers=1)
        self.icon_requests = {}
        self.destination = Adw.ComboRow(title='Destination', model=Gtk.StringList.new([]))
        self.destination.connect('notify::selected', self.form_changed)
        self.form.add(self.destination)
        self.add_field_error("destination")
        self.step = Adw.SpinRow(title='Volume step', adjustment=Gtk.Adjustment(value=5,lower=-20,upper=20,step_increment=1))
        self.step.connect('notify::value', self.form_changed)
        self.form.add(self.step)
        self.add_field_error("step")
        self.form.add(self.appearance)
        self.status = Gtk.Label(label='Loading the active layout…', xalign=0, wrap=True)
        outer.append(self.status)
        from device_preview import CSS as DEVICE_CSS
        self.device_css=Gtk.CssProvider()
        self.device_css.load_from_data(DEVICE_CSS.encode())
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(),self.device_css,Gtk.STYLE_PROVIDER_PRIORITY_USER+3)
        self.caption_css=Gtk.CssProvider()
        extra=''.join(f'.key-caption.color-{name} {{color: {color};}}' for name,color in LABEL_COLORS.items())
        self.caption_css.load_from_data((extra+'.key-caption.transparent {background: transparent;}').encode())
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(),self.caption_css,Gtk.STYLE_PROVIDER_PRIORITY_USER+2)
        self.css = Gtk.CssProvider()
        Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), self.css, Gtk.STYLE_PROVIDER_PRIORITY_USER+1)
        if integrated:
            from workspace_shell import WorkspaceShell
            self.shell=WorkspaceShell(self,owner,toolbar,header,outer,sidebar_scroll,preview_scroll,mode_row,intro,theme_button,layout_menu)
        self.request(self.load_layout, self.loaded)
        if not integrated:self.present()

    def load_layout(self):
        try:return json.loads(self.call('GetLayout')[0]),json.loads(self.call('GetStatus')[0])
        except Exception:
            from pathlib import Path
            import os
            base=Path(os.environ.get('XDG_CONFIG_HOME',str(Path.home()/'.config')))/'decksmith'
            saved=base/'layout.json'
            if saved.exists():return json.loads(saved.read_text()),{'layout':'custom','connected':False}
            from panel import ROOT
            return json.loads((ROOT/'config/audio.json').read_text()),{'layout':'audio','connected':False}

    def cleanup(self):
        if self.closed:return
        self.closed=True
        self.key_preview.stop();self.touch_preview.stop()
        self.pool.shutdown(wait=False,cancel_futures=True)
        self.icon_pool.shutdown(wait=False,cancel_futures=True)
        for provider in (self.css,self.caption_css,self.device_css):
            Gtk.StyleContext.remove_provider_for_display(Gdk.Display.get_default(),provider)
        if hasattr(self,'shell'):self.shell.cleanup()

    def arrange_workspace(self, compact):
        if hasattr(self,'shell'):
            self.shell.arrange(compact)
            return
        # Reorder existing widgets so resize never loses the selection or draft.
        self.workspace.set_orientation(Gtk.Orientation.VERTICAL if compact else Gtk.Orientation.HORIZONTAL)
        self.workspace.reorder_child_after(getattr(self,'property_column',self.properties),None if compact or self.device_column.get_parent() is not self.workspace else self.device_column)
        self.properties.set_hexpand(not compact)
        self.device_column.set_hexpand(compact)
        self.device_column.set_size_request(-1 if compact else 554,-1)

    def request(self, task, done, error_message=None):
        if self.closed:return
        self.pending = True
        self.undo_button.set_sensitive(False)
        self.redo_button.set_sensitive(False)
        self.body.set_sensitive(False)
        self.save_button.set_sensitive(False)
        self.reset_button.set_sensitive(False)
        future = self.pool.submit(task)
        def finish():
            if self.closed:return False
            self.pending = False
            self.body.set_sensitive(True)
            try: done(future.result())
            except Exception as error:
                self.close_after_save=False
                self.owner.quit_all=False
                print(f'Editor request failed: {error}', flush=True)
                self.status.set_text('Could not complete the request. Your draft is still here; check that Decksmith is running.')
                self.body.set_sensitive(self.draft is not None)
                self.save_button.set_sensitive(False)
                self.reset_button.set_sensitive(self.draft is not None)
                if self.draft is not None:
                    self.render()
                self.status.set_text(error_message or "Request failed; changes may have been saved. Your draft is retained. Check the connection before retrying.")
            return False
        future.add_done_callback(lambda _future: GLib.idle_add(finish))

    def loaded(self, data):
        layout, status = data
        self.draft = Draft(layout)
        for page in self.draft.data['pages']:
            for key in page['keys']:
                if key['action']['type'] in MEDIA_LABELS and not key.get('icon_png'):
                    populate_media(key,LABEL_COLORS.get(key.get('background_color'),'#%02x%02x%02x'%tuple(page['background'])),preserve_label=True)
        self.device_layout = status.get('layout')
        page = status.get('active_page')
        if isinstance(page, int) and 0 <= page < len(layout['pages']):
            self.page = self.device_page = page
        # Measure both forms from the first frame, including the first dial click.
        if self.dial_controls is None:
            from dials import DialControls
            self.dial_controls=DialControls(self,embedded=True)
            self.properties.add_named(self.dial_controls,'dial')
        self.rebuild()

    def follow_status(self, status):
        if self.pending or self.draft is None or not status.get('display_ready'):
            return
        if status.get('layout') != self.device_layout:
            return
        page = status.get('active_page')
        if not isinstance(page, int) or page not in self.draft.origins:
            return
        if page == self.device_page:
            return
        self.device_page = page
        draft_page = self.draft.origins.index(page)
        if draft_page != self.page:
            self.page, self.key = draft_page, 0
            self.rebuild()

    def refresh_page_list(self):
        """Update stable rows without stealing focus from the name entry or picker."""
        was_syncing=self.syncing
        self.syncing=True
        pages=self.draft.data['pages']
        while len(self.page_rows)>len(pages):
            row,*_=self.page_rows.pop();self.page_list.remove(row)
        while len(self.page_rows)<len(pages):
            row=Gtk.ListBoxRow()
            box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=4)
            for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(10)
            top=Gtk.Box(spacing=6)
            name=Gtk.Label(xalign=0,hexpand=True,ellipsize=Pango.EllipsizeMode.END)
            badge=Gtk.Label(label='Default');badge.add_css_class('dim-label')
            top.append(name);top.append(badge);box.append(top)
            app=Gtk.Label(xalign=0,ellipsize=Pango.EllipsizeMode.END)
            app.add_css_class('dim-label');box.append(app);row.set_child(box)
            self.page_list.append(row);self.page_rows.append((row,name,app,badge))
        for index,(page,(row,name,app,badge)) in enumerate(zip(pages,self.page_rows)):
            name.set_text(page['name'])
            identity=page.get('application')
            label=(self.page_app_names[self.page_app_ids.index(identity)]
                   if identity in self.page_app_ids and identity else identity or 'No app assigned')
            app.set_text(label);row.set_tooltip_text(page['name']+' · '+label)
            badge.set_visible(index==self.draft.default_page())
        self.page_list.select_row(self.page_rows[self.page][0])
        self.syncing=was_syncing

    def update_page_models(self, names):
        # Keep selection models stable while GTK is delivering selection changes.
        # Replacing a model from notify::selected can continually notify again.
        for widget in (self.destination,):
            model = widget.get_model()
            current = [model.get_string(i) for i in range(model.get_n_items())]
            if current != names:
                model.splice(0, model.get_n_items(), names)

    def rebuild(self):
        self.syncing = True
        names = [page['name'] for page in self.draft.data['pages']]
        self.update_page_models(names)
        self.refresh_page_list()
        self.page_name.set_text(names[self.page])
        is_default=self.page==self.draft.default_page()
        self.page_default.set_active(is_default)
        self.page_default.set_sensitive(not is_default)
        app=self.draft.data['pages'][self.page].get('application')
        if app not in self.page_app_ids:
            self.page_app_ids.append(app);self.page_app_names.append(app+' (not installed)')
            self.page_app.set_model(Gtk.StringList.new(self.page_app_names))
        self.page_app.set_selected(self.page_app_ids.index(app))
        self.syncing = False
        if self.control_kind=='dial':self.select_dial(self.dial_index)
        else:self.select_key(self.key)
        self.add_button.set_sensitive(len(names)<16)
        self.page_tools['Duplicate'].set_sensitive(len(names)<16)
        self.page_tools['Delete'].set_sensitive(len(names)>1)
        self.page_tools['Move up'].set_sensitive(self.page>0)
        self.page_tools['Move down'].set_sensitive(self.page<len(names)-1)

    def key_operation(self, operation):
        if self.draft is None or self.pending or not self.edit_mode.get_active():
            return
        try:
            getattr(self.draft, operation+'_key')(self.page,self.key)
        except ValueError as error:
            self.status.set_text(str(error))
            return
        self.key_drag = None
        self.select_key(self.key)
        try:
            if self.key_style_rows.invalid or (getattr(self,'theme_dialog',None) and self.theme_dialog.rows.invalid):raise ValueError('Enter an icon size from 10–100% in steps of 5.')
            self.draft.validate()
        except ValueError:
            return  # Keep the validation explanation from render visible.
        if operation=='copy':
            self.status.set_text('Key copied. Choose a destination and Paste key to change the layout.')
        elif self.draft.dirty:
            self.status.set_text({'paste':'Key pasted. Save and Apply updates the device.',
                                  'clear':'Key cleared. Save and Apply updates the device.'}[operation])

    def prepare_key_drag(self, source, _x, _y, index):
        self.key_drag = None
        if self.draft is None or self.pending or not self.edit_mode.get_active():
            return None
        keys = self.draft.data['pages'][self.page]['keys']
        token = str(uuid4())
        self.key_drag = (token, keys, index)
        source.set_icon(Gtk.WidgetPaintable.new(self.keys[index]), 0, 0)
        return Gdk.ContentProvider.new_for_value(token)

    def end_key_drag(self, *_args):
        self.key_drag = None

    def drop_key(self, _target, value, _x, _y, index):
        drag = self.key_drag
        self.key_drag = None
        if not drag or self.draft is None or self.pending or not self.edit_mode.get_active():
            return False
        token, keys, origin = drag
        if value != token or keys is not self.draft.data['pages'][self.page]['keys']:
            return False
        keys[origin], keys[index] = keys[index], keys[origin]
        self.select_key(index)
        return True

    def activate_key(self, index):
        if self.draft is None or self.pending:
            return
        action = self.draft.data['pages'][self.page]['keys'][index]['action']
        if not self.integrated and not self.edit_mode.get_active():
            count = len(self.draft.data['pages'])
            target = None
            if action['type'] == 'go_to_page':
                target = action['page']
            elif action['type'] == 'next_page':
                target = (self.page + 1) % count
            elif action['type'] == 'previous_page':
                target = (self.page - 1) % count
            if target is not None:
                self.page_list.select_row(self.page_list.get_row_at_index(target))
                return
        if self.integrated:self.shell.navigate('keys')
        self.select_key(index)

    def select_key(self, index):
        self.control_kind='key'
        self.properties.set_visible_child_name('key')
        self.key = index
        key = self.draft.data['pages'][self.page]['keys'][index]
        self.key_style_rows.sync()
        self.syncing = True
        if key['action']['type']=='system':self.system_picker.set_selected(self.system_commands.index(key['action']['command']))
        self.form.set_title(f'Key {index+1} · '+key['label'])
        self.plugin_remove.set_visible(bool(key.get('plugin')))
        self.homebridge.sync(key.get('plugin'))
        if key.get('plugin'):
            self.form.set_title(f'Key {index+1} · '+key['label']+' · Plugin assignment')
        self.action.set_title('Built-in action (replaces plugin)' if key.get('plugin') else 'Action')
        self.label.set_text(key['label'])
        from themes import effective
        style=effective(self.draft.data,key,self.draft.data['pages'][self.page]['background'])
        self.key_style_rows.rows['icon_size'].set_value(style['icon_size'])
        self.label_position.set_selected(['hidden','top','middle','bottom'].index(style['label_position']))
        self.label_color.set_selected(list(LABEL_COLORS).index(style['label_color']))
        self.label_background.set_active(style['label_background']=='transparent')
        self.key_background.set_selected(list(LABEL_COLORS).index(style['background_color']))
        self.art.set_selected({'makers_mark':1,'application_icon':2}.get(key.get('artwork'),0))
        self.action.set_selected(ACTIONS.index(key['action']['type']))
        self.destination.set_selected(key['action'].get('page',0))
        self.step.set_value(key['action'].get('percent',5))
        self.media_player.select(key.get('media_player',''))
        self.audio_target.select(key['action'].get('target','system'),key['action']['type']=='audio_select',key['action']['type']=='push_to_talk')
        self.url.set_text(key['action'].get('url',''))
        self.app_picker.set_selected(next((i+1 for i,app in enumerate(self.apps) if app.get_id()==key['action'].get('desktop_id')),0))
        self.syncing = False
        self.render()

    def history_shortcut(self,_controller,keyval,_keycode,state):
        if self.integrated and state & Gdk.ModifierType.ALT_MASK and Gdk.KEY_1<=keyval<=Gdk.KEY_5:
            section=list(self.shell.nav)[keyval-Gdk.KEY_1]
            self.shell.navigate(section);self.shell.nav[section].grab_focus()
            return True
        if state & Gdk.ModifierType.CONTROL_MASK and keyval in (Gdk.KEY_s,Gdk.KEY_S):
            if self.draft is not None and not self.pending and self.save_button.get_sensitive():self.save(None)
            return True
        if state & Gdk.ModifierType.CONTROL_MASK and keyval in (Gdk.KEY_z,Gdk.KEY_Z):
            self.history_step(bool(state & Gdk.ModifierType.SHIFT_MASK))
            return True
        return False

    def history_step(self,redo):
        if self.pending or self.draft is None:return
        if self.draft.travel(redo):
            self.key_drag=None
            self.page=min(self.page,len(self.draft.data['pages'])-1)
            self.rebuild()

    def render(self):
        self.refresh_page_list()
        self.draft.checkpoint(self.history_group)
        self.history_group=None
        self.undo_button.set_sensitive(not self.pending and bool(self.draft.undo_stack))
        self.redo_button.set_sensitive(not self.pending and bool(self.draft.redo_stack))
        self.key_tools_row.set_visible(self.edit_mode.get_active())
        self.key_tools_row.set_opacity(1.0 if self.control_kind=='key' else 0.0)
        self.key_tools_row.set_can_target(self.control_kind=='key')
        for operation,button in self.key_tools.items():
            button.set_sensitive(self.control_kind=='key' and self.edit_mode.get_active() and not self.pending and (operation!='paste' or self.draft.clipboard is not None))
        page = self.draft.data['pages'][self.page]
        r,g,b = page['background']
        backgrounds=''.join(f'.deck-key.key-slot-{i} {{background-color: {LABEL_COLORS[key["background_color"]]};}}' for i,key in enumerate(page['keys']) if key.get('background_color'))
        style_css=(backgrounds+f'.deck-key {{padding: 0; min-width: 0; min-height: 0; background-color: rgb({r},{g},{b}); background-image: none; box-shadow: none; color: #f0e6d7; border: 2px solid transparent;}} .deck-key.selected-key {{border-color: @accent_color;}} .key-caption {{color: #f0e6d7; background: alpha(#000000,0.65); padding: 2px 4px; font-weight: bold;}}').encode()
        if getattr(self,'last_style_css',None)!=style_css:
            self.css.load_from_data(style_css);self.last_style_css=style_css
        if not hasattr(self,'fallback_keys'):self.fallback_keys={}
        for index, key in enumerate(page['keys']):
            button = self.keys[index]
            (button.add_css_class if self.control_kind=='key' and index==self.key else button.remove_css_class)('selected-key')
            signature=(json.dumps({k:v for k,v in key.items() if k!='icon_png'},sort_keys=True),id(key.get('icon_png')))
            if self.fallback_keys.get(index)==signature:continue
            self.fallback_keys[index]=signature
            if key.get('artwork') == 'application_icon' and key.get('icon_png'):
                texture = Gdk.Texture.new_from_bytes(GLib.Bytes.new(bytes(key['icon_png'])))
                picture = Gtk.Picture.new_for_paintable(texture)
                picture.set_size_request(72,72)
                picture.set_can_shrink(True)
                button.set_child(picture)
            elif key.get('artwork') == 'makers_mark':
                picture = Gtk.Picture.new_for_paintable(self.owner.logo_texture)
                picture.set_size_request(72,72)
                picture.set_can_shrink(True)
                button.set_child(picture)
            else:
                label = Gtk.Label(label=key['label'])
                label.add_css_class('heading')
                button.set_child(label)
            position=key.get('label_position','hidden' if key.get('artwork') else 'middle')
            if position=='hidden' and not key.get('artwork'):
                button.set_child(Gtk.Label(label=''))
            elif position!='hidden':
                overlay=Gtk.Overlay()
                child=button.get_child()
                button.set_child(None)
                if key.get('artwork'):
                    overlay.set_child(child)
                else:
                    overlay.set_child(Gtk.Box(vexpand=True,hexpand=True))
                caption=Gtk.Label(label=key['label'],halign=Gtk.Align.CENTER,valign={'top':Gtk.Align.START,'middle':Gtk.Align.CENTER,'bottom':Gtk.Align.END}[position])
                caption.set_wrap(True)
                caption.set_wrap_mode(Pango.WrapMode.WORD)
                caption.set_justify(Gtk.Justification.CENTER)
                caption.set_max_width_chars(10)
                caption.add_css_class('key-caption')
                caption.add_css_class('color-'+key.get('label_color','default'))
                if key.get('label_background')=='transparent' or not key.get('artwork'):caption.add_css_class('transparent')
                caption.set_margin_top(8);caption.set_margin_bottom(8)
                overlay.add_overlay(caption)
                button.set_child(overlay)
            button.set_tooltip_text(f"Key {index+1}: {key['label']}")
            (button.add_css_class if self.control_kind=='key' and index==self.key else button.remove_css_class)('selected-key')
        from dials import effective,is_override
        for index,dial in enumerate(effective(self.draft.data,self.page)):
            self.touch_buttons[index].update_property([Gtk.AccessibleProperty.LABEL],[f"Dial {index+1}: {dial['label']}"])
            for button in (self.touch_buttons[index],self.dial_buttons[index]):
                button.set_tooltip_text(f"Dial {index+1}: {dial['label']} · "+("This page" if is_override(self.draft.data,self.page,index) else "Shared settings"))
                (button.add_css_class if self.control_kind=='dial' and index==self.dial_index else button.remove_css_class)('suggested-action')
        style_key=self.draft.data['pages'][self.page]['keys'][self.key]
        custom=bool(style_key.get('appearance')) or any(k in style_key for k in ('label_position','label_color','background_color','label_background'))
        self.appearance.set_subtitle('Custom style · Reset to theme to inherit' if custom else 'Using shared theme defaults')
        self.action_catalog.set_label("Action · "+ACTION_LABELS[self.action.get_selected()])
        action = ACTIONS[self.action.get_selected()]
        self.system_picker.set_visible(action=='system');self.system_note.set_visible(action=='system')
        self.destination.set_visible(action=='go_to_page')
        self.step.set_visible(action in ('volume_adjust','audio_adjust'))
        self.audio_target.set_visible(action in ('audio_adjust','audio_mute','audio_select','push_to_talk'))
        self.media_player.set_visible(action in MEDIA_LABELS)
        self.app_picker.set_visible(action=='open_application')
        self.url.set_visible(action=='open_website')
        self.refresh_icon.set_visible(action=='open_website')
        self.update_field_errors()
        self.reset_button.set_sensitive(self.draft.dirty)
        try:
            if self.key_style_rows.invalid or (getattr(self,'theme_dialog',None) and self.theme_dialog.rows.invalid):raise ValueError('Enter an icon size from 10–100% in steps of 5.')
            self.draft.validate()
            self.layout_valid=True
            self.validation_error=None
            payload=json.dumps(self.draft.data)
            self.touch_preview.request(payload,self.page)
            self.key_preview.request(payload,self.page)
            self.save_button.set_sensitive(self.draft.dirty)
            self.save_button.set_tooltip_text(None if self.draft.dirty else "No changes to save. Copy a key, then paste it into a destination to change the layout.")
            self.status.set_text('Unsaved changes' if self.draft.dirty else 'No unsaved changes')
        except ValueError as error:
            self.layout_valid=False
            self.validation_error=str(error)
            self.touch_preview.invalidate()
            self.status.set_text(str(error))
            self.save_button.set_tooltip_text(str(error))
            self.save_button.set_sensitive(False)

        if hasattr(self,'shell'):self.shell.sync()

    def add_field_error(self,name):
        message=Gtk.Label(xalign=0,wrap=True);message.add_css_class('error')
        message.set_margin_start(12);message.set_margin_end(12);message.set_visible(False)
        self.field_errors[name]=message;self.form.add(message)

    def update_field_errors(self):
        from urllib.parse import urlsplit
        key=self.draft.data['pages'][self.page]['keys'][self.key]
        action=key['action'];kind=action['type'];errors={}
        label=key['label']
        if not (1<=len(label)<=24 and label.strip() and all(c.isascii() and (c.isalnum() or c==' ') for c in label)):
            errors['label']='Use 1–24 letters, numbers or spaces.'
        if kind=='open_application' and not action.get('desktop_id'):errors['application']='Choose an application.'
        if kind=='open_website':
            try:
                url=urlsplit(action.get('url',''))
                valid=url.scheme in ('http','https') and bool(url.hostname) and len(action.get('url',''))<=2048 and not any(c.isspace() or ord(c)<32 for c in action.get('url',''))
            except ValueError:valid=False
            if not valid:errors['url']='Enter a complete address starting with https:// or http://.'
        if kind=='go_to_page' and not 0<=action.get('page',-1)<len(self.draft.data['pages']):errors['destination']='Choose an existing page.'
        if kind in ('volume_adjust','audio_adjust') and action.get('percent',0)==0:errors['step']='Choose a volume step other than zero.'
        for name,widget in self.field_errors.items():
            widget.set_text(errors.get(name,''));widget.set_visible(name in errors)

    def assign_homebridge(self,item,binding):
        from homebridge_picker import label,icon_png
        key=self.draft.data['pages'][self.page]['keys'][self.key]
        key['action']={'type':'none'};key['plugin']=binding;key['label']=label(item['name'])
        key.pop('icon_png',None);key.pop('artwork',None)
        icon=icon_png(item['icon'])
        if icon:key['icon_png']=icon;key['artwork']='application_icon'
        key['label_position']='bottom'
        self.draft.checkpoint();self.select_key(self.key)

    def remove_plugin_assignment(self, *_):
        self.draft.data['pages'][self.page]['keys'][self.key].pop('plugin',None)
        self.draft.checkpoint();self.select_key(self.key)

    def form_changed(self, *_args):
        if self.syncing or self.draft is None: return
        if _args and isinstance(_args[0],Adw.EntryRow):
            self.history_group=(self.page,self.key,id(_args[0]))
        key = self.draft.data['pages'][self.page]['keys'][self.key]
        key['label'] = self.label.get_text()
        sender=_args[0] if _args else None
        if sender is self.label_position:key['label_position']=['hidden','top','middle','bottom'][self.label_position.get_selected()]
        if sender is self.label_color:key['label_color']=list(LABEL_COLORS)[self.label_color.get_selected()]
        if sender is self.label_background:key['label_background']='transparent' if self.label_background.get_active() else 'dark'
        if sender is self.key_background:
            background=list(LABEL_COLORS)[self.key_background.get_selected()]
            if background=='default':key.pop('background_color',None)
            else:key['background_color']=background
        if self.art.get_selected(): key['artwork']={1:'makers_mark',2:'application_icon'}[self.art.get_selected()]
        else: key.pop('artwork',None)
        kind = ACTIONS[self.action.get_selected()]
        previous_action = key['action']
        if sender is self.action:
            key.pop('plugin',None)
        automatic_label=uses_default(key['label'],previous_action,self.draft.data['pages'])
        key['action'] = {'type':kind}
        if kind=='system':key['action']['command']=self.system_commands[self.system_picker.get_selected()]
        if kind in MEDIA_LABELS and self.media_player.value():key['media_player']=self.media_player.value()
        else:key.pop('media_player',None)
        if kind=='go_to_page': key['action']['page']=self.destination.get_selected()
        if kind in ('volume_adjust','audio_adjust'): key['action']['percent']=round(self.step.get_value())
        if kind in ('audio_adjust','audio_mute','audio_select','push_to_talk'):
            self.audio_target.select(self.audio_target.value(),kind=='audio_select',kind=='push_to_talk')
            key['action']['target']=self.audio_target.value()
        if kind=='open_website':
            key['action']['url']=self.url.get_text()
            if previous_action != key['action']:
                key.pop('icon_png',None)
                key.pop('artwork',None)
                self.syncing=True
                self.art.set_selected(0)
                self.syncing=False
        if kind=='open_application':
            selected=self.app_picker.get_selected()
            key['action']['desktop_id']=self.apps[selected-1].get_id() if selected else ''
        action_changed=previous_action!=key['action']
        if kind=='open_application' and selected and action_changed:
            from action_labels import application_label
            key['label']=application_label(self.apps[selected-1].get_display_name())
            automatic_label=True
        if action_changed and automatic_label:
            if kind!='open_application' or not selected:
                key['label']=default_label(key['action'],self.draft.data['pages'])
            if kind!='system' and previous_action['type']=='none' and key.get('label_position')=='hidden':
                key['label_position']='middle'
        if kind=='system' and action_changed:
            from system_artwork import populate
            populate(key)
            self.select_key(self.key)
            return
        if kind in MEDIA_LABELS and previous_action.get('type')!=kind:
            populate_media(key,LABEL_COLORS.get(key.get('background_color'),'#%02x%02x%02x'%tuple(self.draft.data['pages'][self.page]['background'])),preserve_label=True)
            self.select_key(self.key)
            return
        if action_changed and automatic_label:
            key['follow_page_name']=kind=='go_to_page'
            self.select_key(self.key)
            return
        key['follow_page_name'] = kind == 'go_to_page' and key['label'] == self.draft.data['pages'][key['action']['page']]['name']
        self.render()

    def activate_dial(self,index):
        if self.integrated:self.shell.navigate('keys')
        self.select_dial(index)

    def select_dial(self,index):
        if self.draft is None or self.pending:return
        from dials import DialControls,effective
        self.control_kind='dial';self.dial_index=index
        if self.dial_controls is None:
            self.dial_controls=DialControls(self,embedded=True)
            self.properties.add_named(self.dial_controls,'dial')
        controls=self.dial_controls
        controls.data=effective(self.draft.data,self.page)
        controls.syncing=True
        controls.picker.set_selected(index)
        controls.syncing=False
        controls.select()
        self.properties.set_visible_child_name('dial')
        self.render()

    def edit_dials(self,_button):
        self.select_dial(self.dial_index)

    def export_layout(self,_button):
        if self.draft is None or self.pending:return
        try:
            if self.key_style_rows.invalid or (getattr(self,'theme_dialog',None) and self.theme_dialog.rows.invalid):raise ValueError('Enter an icon size from 10–100% in steps of 5.')
            self.draft.validate()
            from layout_package import encode
            data,redacted=encode(self.draft.data)
        except Exception as error:
            self.status.set_text(str(error));return
        dialog=Gtk.FileDialog(title='Export layout',initial_name='My layout.decksmith')
        def chosen(dialog,result):
            try:
                file=dialog.save_finish(result)
                if file is None:return
                from pathlib import Path
                import tempfile,os
                target=Path(file.get_path())
                with tempfile.NamedTemporaryFile(dir=target.parent,delete=False) as stream:
                    stream.write(data);temporary=stream.name
                os.replace(temporary,target)
                self.status.set_text(f'Layout exported. {redacted} website URL(s) had credentials, query strings or fragments removed.')
            except GLib.Error:pass
            except Exception as error:self.status.set_text(str(error))
        dialog.save(self,None,chosen)

    def restore_previous(self,_button):
        if self.draft is None or self.pending:return
        self.request(lambda:self.call('GetPreviousLayout'),lambda result:self.confirm_import(json.loads(result[0])), 'Could not load a previous backup. Your draft and device are unchanged.')

    def confirm_import(self, normalized):
        confirm=Adw.AlertDialog(heading='Load imported layout?',body='This replaces the current draft, including unsaved edits. The device stays unchanged until Save and Apply.')
        confirm.add_response('cancel','Cancel');confirm.add_response('load','Load draft')
        confirm.set_default_response('cancel');confirm.set_close_response('cancel')
        def response(_dialog,choice):
            if choice!='load':return
            previous=self.draft.saved
            self.draft=Draft(normalized)
            self.draft.saved=previous
            self.draft.origins=[None]*len(normalized['pages'])
            self.draft.clear_history()
            self.page=self.key=0
            self.rebuild()
            known={app.get_id() for app in self.apps}
            missing={key['action']['desktop_id'] for page in normalized['pages'] for key in page['keys'] if key['action']['type']=='open_application' and key['action']['desktop_id'] not in known}
            self.status.set_text('Imported draft. Missing applications: '+', '.join(sorted(missing)) if missing else 'Imported draft ready. Save and Apply updates the device; Discard restores your previous layout.')
        confirm.connect('response',response);confirm.present(self)
        return confirm

    def import_layout(self,_button):
        if self.draft is None or self.pending:return
        chooser=Gtk.FileDialog(title='Import layout')
        def chosen(dialog,result):
            try:
                file=dialog.open_finish(result)
                if file is None:return
                from pathlib import Path
                from layout_package import decode,LIMIT
                path=Path(file.get_path())
                if path.stat().st_size>LIMIT:raise ValueError('Choose a layout package smaller than 2 MB.')
                layout=decode(path.read_bytes())
                payload=json.dumps(layout)
                def validated(result):
                    normalized=json.loads(result[0])
                    self.confirm_import(normalized)
                self.request(lambda:self.call('ValidateLayout',GLib.Variant('(s)',(payload,))),validated, 'Import validation failed. Your draft and device are unchanged.')
            except GLib.Error:pass
            except Exception as error:self.status.set_text('Import failed: '+str(error))
        chooser.open(self,None,chosen)

    def open_theme(self,*_):
        if self.integrated:
            self.shell.navigate('pages');self.shell.theme_expander.set_expanded(True)
            return
        if self.draft is not None:
            from theme_dialog import ThemeDialog
            self.theme_dialog=ThemeDialog(self)
    def set_key_style(self,style):
        if self.draft is None:return
        key=self.draft.data['pages'][self.page]['keys'][self.key]
        if style:key['appearance']=style
        else:key.pop('appearance',None)
        self.render()
    def reset_key_style(self,*_):
        from themes import reset
        reset(self.draft.data['pages'][self.page]['keys'][self.key]);self.select_key(self.key)
    def open_icon_library(self,*_):
        from icon_library_dialog import IconLibraryDialog
        key=self.draft.data['pages'][self.page]['keys'][self.key]
        def apply(png,item):
            if not any(key is k for p in self.draft.data['pages'] for k in p['keys']):return
            self.icon_requests[id(key)]='custom'
            key.update(icon_png=list(png),artwork='application_icon',icon_source=item['id'],icon_tint=item['id'].startswith('tabler:'))
            self.select_key(self.key)
        self.icon_dialog=IconLibraryDialog(self,apply)

    def choose_image(self,_button):
        key=self.draft.data['pages'][self.page]['keys'][self.key]
        chooser=Gtk.FileDialog(title='Choose key artwork')
        filters=Gio.ListStore.new(Gtk.FileFilter)
        filter=Gtk.FileFilter(name='Images (PNG, JPEG, WebP, ICO)')
        for pattern in ('*.png','*.jpg','*.jpeg','*.webp','*.ico'):filter.add_pattern(pattern)
        filters.append(filter);chooser.set_filters(filters)
        def selected(dialog,result):
            try:
                file=dialog.open_finish(result)
                if file is None:return
                from pathlib import Path
                from artwork_dialog import ArtworkDialog
                path=Path(file.get_path())
                if path.stat().st_size>8*1024*1024:raise ValueError('Choose an image smaller than 8 MB.')
                data=path.read_bytes()
                def apply(png):
                    if not any(key is k for p in self.draft.data['pages'] for k in p['keys']):return
                    self.icon_requests[id(key)]='custom'
                    key['icon_png']=png;key['artwork']='application_icon';key.pop('icon_source',None);key.pop('icon_tint',None)
                    self.select_key(self.key)
                ArtworkDialog(self,data,apply)
            except GLib.Error:pass
            except Exception as error:self.status.set_text(str(error))
        chooser.open(self,None,selected)

    def find_website_icon(self,force=False):
        if self.draft is None or self.syncing: return
        key = self.draft.data['pages'][self.page]['keys'][self.key]
        if key['action']['type'] != 'open_website': return
        url = key['action'].get('url','')
        if not url.startswith(('http://','https://')) or (not force and self.icon_requests.get(id(key)) == url): return
        self.icon_requests[id(key)] = url
        expected_art = key.get("artwork")
        future = self.icon_pool.submit(fetch_website_icon,url)
        def finish():
            if self.pending:
                GLib.timeout_add(100,finish)
                return False
            try:
                if key.get("artwork") != expected_art or self.icon_requests.get(id(key)) != url: return False
                data = future.result()
                if key['action'] != {'type':'open_website','url':url} or not any(key is candidate for page in self.draft.data['pages'] for candidate in page['keys']): return False
                png = website_png(data) if data else None
                if png:
                    key.pop('icon_source',None);key.pop('icon_tint',None)
                    key['icon_png'] = png
                    key['artwork'] = 'application_icon'
                    if key is self.draft.data['pages'][self.page]['keys'][self.key]:
                        self.select_key(self.key)
                    else: self.render()
                    if min(dimensions(data))<120:
                        self.status.set_text("Website supplied a small icon. Choose a larger image for sharper artwork.")
            except Exception:
                pass
            return False
        future.add_done_callback(lambda _future: GLib.idle_add(finish))

    def application_changed(self, *_args):
        if self.syncing or self.draft is None: return
        selected = self.app_picker.get_selected()
        if selected:
            png = icon_png(self.apps[selected-1])
            key = self.draft.data['pages'][self.page]['keys'][self.key]
            self.syncing = True
            key.pop('icon_source',None);key.pop('icon_tint',None)
            if png:
                key['icon_png'] = png
                self.art.set_selected(2)
            else:
                key.pop('icon_png',None)
                self.art.set_selected(0)
            self.syncing = False
        self.form_changed()

    def page_default_changed(self, *_args):
        if self.syncing or self.draft is None or not self.page_default.get_active():return
        self.draft.set_default_page(self.page)
        self.page_default.set_sensitive(False)
        self.render()

    def page_application_changed(self, *_args):
        if self.syncing or self.draft is None:return
        selected=self.page_app.get_selected()
        if selected>=len(self.page_app_ids):return
        app=self.page_app_ids[selected]
        page=self.draft.data['pages'][self.page]
        if app:page['application']=app
        else:page.pop('application',None)
        # Let ComboRow finish closing its dropdown before updating the preview.
        GLib.idle_add(self.finish_page_application_selection)

    def finish_page_application_selection(self):
        self.render()
        return False

    def page_name_changed(self, *_args):
        if self.syncing or self.draft is None: return
        self.draft.rename_page(self.page, self.page_name.get_text())
        self.syncing=True
        destination=self.destination.get_selected()
        names=[page['name'] for page in self.draft.data['pages']]
        self.update_page_models(names)
        self.refresh_page_list()
        self.destination.set_selected(destination)
        self.label.set_text(self.draft.data["pages"][self.page]["keys"][self.key]["label"])
        self.syncing=False
        self.render()

    def page_changed(self, _list, row):
        if self.syncing or self.draft is None: return
        if row is None:return
        selected = row.get_index()
        if selected == self.page or not 0 <= selected < len(self.draft.data['pages']):
            return
        self.page = selected
        self.rebuild()
        self.show_device_page()

    def show_device_page(self):
        if self.integrated and not self.shell.can_control():
            self.status.set_text('Editing locally · device controls are unavailable.')
            return
        if self.draft.origins[self.page] is None:
            self.status.set_text("Save and Apply to show this new page on the device.")
            return
        page = self.draft.origins[self.page]
        def done(_result):
            self.render()
            if not self.draft.dirty:
                self.status.set_text("Page selected on Stream Deck.")
        self.request(lambda: self.call("ShowPage", GLib.Variant("(y)", (page,))), done)

    def add_page(self, _button):
        self.page=self.draft.add_page()
        self.key=0
        self.rebuild()

    def move_page(self, offset):
        self.page = self.draft.move_page(self.page, self.page + offset)
        self.rebuild()

    def duplicate_page(self, _button):
        self.page = self.draft.duplicate_page(self.page)
        self.key = 0
        self.rebuild()

    def delete_page(self, _button):
        index = self.page
        count = len(self.draft.references_to(index))
        body = f'{count} button(s) on other pages link here. Deleting will set those actions to No action.' if count else 'This page will be removed from your draft.'
        dialog = Adw.AlertDialog(heading=f'Delete {self.draft.data["pages"][index]["name"]}?', body=body + ' Save and Apply updates the device.')
        dialog.add_response('cancel', 'Cancel')
        dialog.add_response('delete', 'Delete page')
        dialog.set_default_response('cancel')
        dialog.set_close_response('cancel')
        dialog.set_response_appearance('delete', Adw.ResponseAppearance.DESTRUCTIVE)
        def response(_dialog, choice):
            if choice == 'delete':
                self.page = self.draft.delete_page(index, clear_links=True)
                self.key = 0
                self.rebuild()
        dialog.connect('response', response)
        dialog.present(self)
        return dialog

    def reset(self, _button):
        self.draft.reset()
        self.page=self.key=0
        self.rebuild()

    def save(self, _button):
        if self.pending or self.draft is None:return
        self.draft.validate()
        payload=json.dumps(self.draft.data)
        self.status.set_text('Saving and applying…')
        def done(_result):
            self.draft.saved_now()
            self.device_layout = "custom"
            self.render()
            self.status.set_text('Saved as My layout and applied to Decksmith.')
            self.owner.refresh()
            if self.close_after_save:self.close_after_save=False;self.close()
        page = self.page
        def apply():
            self.call('SaveLayout', GLib.Variant('(s)', (payload,)))
            if not self.integrated or self.shell.can_control():
                return self.call('ShowPage', GLib.Variant('(y)', (page,)))
        self.request(apply, done)

    def close_requested(self, _window):
        if self.pending:return True
        if self.draft is not None and self.draft.dirty:
            dialog=Adw.AlertDialog(heading='Save changes before closing?',body='Your layout has unsaved changes. Quitting will stop background controls.' if getattr(self.owner,'quit_all',False) else 'Your layout has unsaved changes. Background controls keep running when this window closes.')
            dialog.add_response('keep','Keep editing')
            dialog.add_response('discard','Discard and close')
            dialog.add_response('save','Save and close')
            dialog.set_default_response('keep');dialog.set_close_response('keep')
            dialog.set_response_appearance('discard',Adw.ResponseAppearance.DESTRUCTIVE)
            dialog.set_response_appearance('save',Adw.ResponseAppearance.SUGGESTED)
            dialog.set_response_enabled('save',self.save_button.get_sensitive())
            def response(_dialog,choice):
                if choice=='keep':self.owner.quit_all=False
                if choice=='discard':
                    self.draft.reset();self.close()
                elif choice=='save':
                    self.close_after_save=True;self.save(None)
            dialog.connect('response',response);dialog.present(self)
            return True
        if getattr(self.owner,'quit_all',False):
            self.owner.finish_quit();return True
        self.cleanup()
        return False
