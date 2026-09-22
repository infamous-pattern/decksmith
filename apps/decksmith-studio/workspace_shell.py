"""Native workspace. One editor, one draft, no hidden duplicate previews."""
from gi.repository import Adw, Gtk, Gdk, Gio, GLib, Pango
from i18n import gettext as tr
from validation_announcements import ValidationAnnouncements

SECTIONS=(('home',tr('Home'),'go-home-symbolic'),('pages',tr('Pages'),'view-list-symbolic'),
          ('keys',tr('Keys & Dials'),'input-dialpad-symbolic'),
          ('plugins',tr('Plugins'),'application-x-addon-symbolic'),('about',tr('About'),'help-about-symbolic'))
TIPS={'home':tr('Device status, startup and brightness.'),'pages':tr('Organize pages and assign applications.'),
      'keys':tr('Edit keys, dials and the touch strip.'),
      'plugins':tr('Plugin support is planned for a future release.'),'about':tr('Project information, user guide and support.')}

class WorkspaceShell:
    def __init__(self,ed,owner,toolbar,header,outer,page_scroll,preview_scroll,mode_row,intro,theme_button,layout_menu):
        self.ed=ed;self.owner=owner;self.status={};self.section='home';self.changing_page=False;self.loading_layout=False
        ed.set_modal(False);ed.set_transient_for(None);ed.set_default_size(1280,860);ed.set_size_request(980,680)
        ed.add_css_class('decksmith-workspace')
        self.css=Gtk.CssProvider();self.style=Adw.StyleManager.get_default()
        self.style_handler=self.style.connect('notify::dark',lambda *_:self.colors())
        self.colors();Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(),self.css,Gtk.STYLE_PROVIDER_PRIORITY_USER+4)
        # Move existing controls, never create a second draft or preview worker.
        ed.body.remove(page_scroll);ed.body.remove(preview_scroll)
        toolbar.set_content(None);ed.set_content(None)
        self.root=Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        nav=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=8,width_request=176)
        nav.set_hexpand(False);nav.add_css_class('ds-sidebar');self.root.append(nav)
        brand=Gtk.Label(label='Decksmith',xalign=0);brand.add_css_class('title-3');brand.set_margin_bottom(24);nav.append(brand)
        self.nav={};self.nav_labels={}
        for name,label,icon in SECTIONS:
            b=Gtk.ToggleButton(tooltip_text=TIPS[name]+' · Alt+'+str(len(self.nav)+1));row=Gtk.Box(spacing=10)
            row.append(Gtk.Image.new_from_icon_name(icon));caption=Gtk.Label(label=label,xalign=0,hexpand=True);row.append(caption);self.nav_labels[name]=(caption,label)
            b.set_child(row);b.add_css_class('ds-nav');b.add_css_class('section-'+name)
            b.connect('clicked',lambda _b,n=name:self.navigate(n));nav.append(b);self.nav[name]=b
        spacer=Gtk.Box(vexpand=True);nav.append(spacer)
        slogan=Gtk.Label(label='Open Control.\nForged for Linux.',xalign=0);slogan.add_css_class('dim-label');slogan.add_css_class('caption');nav.append(slogan)
        right=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,hexpand=True,vexpand=True);right.add_css_class('ds-main');self.root.append(right)
        top=Adw.HeaderBar();top.set_show_start_title_buttons(False);right.append(top)
        device=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=3)
        self.device_menu=Gtk.MenuButton(label='Stream Deck +',tooltip_text='Choose Device · current device. Independent multi-device control is planned.')
        pop=Gtk.Popover();info=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=10)
        for edge in ('top','bottom','start','end'):getattr(info,'set_margin_'+edge)(14)
        info.append(Gtk.Label(label='Choose Device',xalign=0));self.device_info=Gtk.Label(label='Stream Deck +',xalign=0);info.append(self.device_info)
        info.append(Gtk.Label(label='One device at a time in this version.',xalign=0,wrap=True))
        pop.set_child(info);self.device_menu.set_popover(pop);device.append(self.device_menu)
        self.connection=Gtk.Label(label='Checking connection…',xalign=0,ellipsize=Pango.EllipsizeMode.END,max_width_chars=32)
        self.connection.add_css_class('caption');device.append(self.connection);top.set_title_widget(device)
        actions=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=4)
        history=Gtk.Box(spacing=6,halign=Gtk.Align.END);self.saved=Gtk.Label(label='Loading…');self.saved.add_css_class('caption');history.append(self.saved)
        for b in (ed.undo_button,ed.redo_button):header.remove(b);history.append(b)
        actions.append(history);saves=Gtk.Box(spacing=6)
        for b in (ed.reset_button,ed.save_button):header.remove(b);saves.append(b)
        ed.reset_button.set_label('Discard');ed.reset_button.set_tooltip_text('Restore the last saved layout for this device.')
        ed.save_button.set_size_request(145,-1);ed.save_button.set_tooltip_text('Save and apply the layout · Ctrl+S')
        actions.append(saves);top.pack_end(actions)
        self.save_reason=Gtk.Label(label='Loading layout…',xalign=0,hexpand=True,ellipsize=Pango.EllipsizeMode.END)
        self.validation_speech=ValidationAnnouncements(GLib.timeout_add,GLib.source_remove,self.announce_validation)
        self.save_reason.set_margin_start(22);self.save_reason.set_margin_end(22)
        self.save_reason.set_size_request(-1,30)
        right.append(self.save_reason)
        self.banner=Adw.Banner();right.append(self.banner)
        self.stack=Gtk.Stack(vexpand=True,hexpand=True,hhomogeneous=True,vhomogeneous=True)
        self.stack.set_transition_type(Gtk.StackTransitionType.NONE)
        self.columns=Gtk.Box(spacing=0,hexpand=True,vexpand=True)
        self.columns.append(self.stack)
        self.preview_scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.AUTOMATIC,min_content_width=598,hexpand=False,vexpand=True)
        self.preview_host=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=12,hexpand=True,valign=Gtk.Align.START)
        self.preview_host.add_css_class('ds-content');self.preview_scroll.set_child(self.preview_host)
        self.columns.append(self.preview_scroll);right.append(self.columns);ed.body=self.stack
        # Existing page list + settings get their own persistent section.
        pagebox=page_scroll.get_child()
        if isinstance(pagebox,Gtk.Viewport):pagebox=pagebox.get_child()
        page_group=pagebox.get_last_child();pagebox.remove(page_group)
        page_content=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=18,hexpand=True,vexpand=True);page_content.add_css_class('ds-content');page_content.add_css_class('section-pages')
        # Scroll the whole Pages form, not a clipped wrapper around its toolbar.
        wrapper=page_scroll.get_child()
        if isinstance(wrapper,Gtk.Viewport):wrapper.set_child(None)
        page_scroll.set_child(None)
        pagebox.set_size_request(-1,-1)
        page_content.append(pagebox)
        settings=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=18,hexpand=True,valign=Gtk.Align.START)
        settings.append(page_group)
        edit=Gtk.Button(label='Edit controls',halign=Gtk.Align.START,tooltip_text='Edit the selected page’s keys and dials.')
        edit.connect('clicked',lambda *_:self.navigate('keys'));settings.append(edit)
        page_content.append(settings)
        pages_scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER,hexpand=True,vexpand=True)
        pages_scroll.set_child(page_content);self.pages_scroll=pages_scroll
        self.stack.add_named(pages_scroll,'pages')
        # Device and property controls retain their models and selections across tabs.
        left=ed.device_column.get_child();left.remove(mode_row)
        picker=Gtk.Box(spacing=10);picker.append(Gtk.Label(label='Page',xalign=0))
        self.page_picker=Gtk.DropDown.new_from_strings([]);self.page_picker.set_hexpand(True);self.page_picker.set_tooltip_text('Choose the page to edit. Unsaved changes are kept.')
        self.page_picker.connect('notify::selected',self.page_selected);picker.append(self.page_picker);left.prepend(picker)
        tools=Gtk.Box(spacing=8)
        header.remove(layout_menu);tools.append(layout_menu)
        header.remove(theme_button)
        appearance_group=Adw.PreferencesGroup(title='Layout appearance',description='Theme and shared defaults apply across all pages.')
        self.theme_expander=Adw.ExpanderRow(title='Themes and defaults',subtitle='Current appearance · Applies to all pages')
        appearance_group.add(self.theme_expander);settings.append(appearance_group)
        self.theme_controls=None
        theme_button.set_tooltip_text('Device colors, fonts and shared defaults. App colors follow GNOME.')
        left.append(tools)
        self.editor_extras=(tools,ed.key_tools_row)
        testrow=Gtk.Box(spacing=8);self.test=Gtk.Button(label='Test action',tooltip_text='Run the saved key action or dial press on the device. Save edits first.')
        self.test.connect('clicked',self.test_action);testrow.append(self.test)
        self.test_note=Gtk.Label(label='Select to edit. Test action runs the saved control.',xalign=0,wrap=True,hexpand=True)
        self.test_note.add_css_class('dim-label');testrow.append(self.test_note)
        # ScrolledWindow has a tiny natural height; explicitly give the editor
        # the remaining page space instead of clipping it to one row.
        preview_scroll.set_vexpand(True)
        key_outer=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=16);key_outer.add_css_class('ds-content');key_outer.add_css_class('section-keys')
        self.context_heading=Gtk.Label(label=tr('Keys & Dials'),xalign=0,ellipsize=Pango.EllipsizeMode.END);self.context_heading.add_css_class('title-1');key_outer.append(self.context_heading);key_outer.append(preview_scroll)
        ed.workspace.remove(ed.properties)
        property_content=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=12,hexpand=True,valign=Gtk.Align.START)
        ed.property_column=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.AUTOMATIC,vscrollbar_policy=Gtk.PolicyType.AUTOMATIC,min_content_width=0,min_content_height=560,max_content_height=560,propagate_natural_width=False,propagate_natural_height=False,hexpand=True)
        ed.property_column.set_child(property_content)
        testrow.set_halign(Gtk.Align.END)
        property_content.append(testrow);property_content.append(ed.properties);ed.workspace.append(ed.property_column)
        self.stack.add_named(key_outer,'keys')
        ed.workspace.remove(ed.device_column)
        self.preview_host.append(ed.device_column)
        ed.device_column.set_size_request(-1,-1);ed.device_column.set_hexpand(True)
        # Home presents existing service/settings controls beside the same device preview.
        home=owner.home_content;box=home.get_child()
        if isinstance(box,Gtk.Viewport):box=box.get_child()
        children=[];child=box.get_first_child()
        while child:children.append(child);child=child.get_next_sibling()
        for child in children:box.remove(child)
        title=Gtk.Label(label='Your workspace',xalign=0);title.add_css_class('title-1');box.append(title)
        for child in children:box.append(child)
        home.add_css_class('section-home');self.stack.add_named(home,'home')
        self.stack.add_named(self.plugins(),'plugins')
        self.stack.add_named(self.about(),'about')
        outer.remove(ed.status)
        ed.status.add_css_class('ds-footer');ed.status.set_margin_start(18);ed.status.set_margin_end(18);ed.status.set_margin_top(10);ed.status.set_margin_bottom(10);right.append(ed.status)
        owner.toasts.set_child(self.root);ed.set_content(owner.toasts)
        # Main-window responsive rule replaces the former editor-only breakpoint.
        self.navigate('home')

    def arrange(self,compact):
        # Scale only on window-width changes, never on tab or control selection.
        self.preview_scroll.set_min_content_width(470 if compact else 598)
        self.ed.grid.set_column_spacing(16 if compact else 28)
        self.ed.grid.set_row_spacing(14 if compact else 18)
        for key in self.ed.keys:
            key.preview_size=68 if compact else 102
            key.get_layout_manager().layout_changed()

    def colors(self):
        colors=('#292d33','#2b302d','#322e2a','#293132','#2f2c33') if self.style.get_dark() else ('#f3f5f8','#f3f6f3','#f8f5f1','#f1f6f6','#f5f3f7')
        css=''' .decksmith-workspace .ds-main {background: @window_bg_color; color: @window_fg_color;}
        .decksmith-workspace .ds-sidebar {padding: 22px 12px; background: @sidebar_bg_color; border-right: 1px solid alpha(@window_fg_color,0.1);}
        .decksmith-workspace .ds-nav {padding: 12px; min-height: 22px; border: none; box-shadow: none; font-weight: 500;}
        .decksmith-workspace .ds-nav:checked {box-shadow: inset 3px 0 @accent_color; font-weight: bold;}
        .decksmith-workspace .ds-nav:focus-visible {outline: 2px solid @accent_color; outline-offset: -4px;}
        .decksmith-workspace .ds-content {padding: 22px;}
        .decksmith-workspace .ds-footer {font-size: 12px;}
        '''
        for (name,_,_),color in zip(SECTIONS,colors):css+=f'.decksmith-workspace .section-{name} {{background-color: {color};}} .decksmith-workspace button.ds-nav.section-{name} {{background: {color}; color: @window_fg_color;}}'
        self.css.load_from_data(css.encode())

    def plugins(self):
        import os
        connection = os.environ.get('DECKSMITH_PLUGIN_LAB_CONNECTION')
        if not connection:
            from pathlib import Path
            runtime=os.environ.get('XDG_RUNTIME_DIR')
            candidate=Path(runtime)/'decksmith-plugin-lab.json' if runtime else None
            managed=Path(os.environ.get('XDG_CONFIG_HOME',str(Path.home()/'.config')))/'systemd/user/decksmith-openhomeb.service'
            if candidate is not None and (candidate.is_file() or managed.is_file()):connection=str(candidate)
        if connection:
            from plugin_lab_page import PluginLabPage
            self.plugin_lab = PluginLabPage(connection)
            self.owner.connect('shutdown', lambda *_: self.plugin_lab.close())
            self.nav['plugins'].set_tooltip_text(tr('Experimental OpenHomeB controls and live status.'))
            return self.plugin_lab
        page=Adw.StatusPage(
            icon_name='application-x-addon-symbolic',
            title=tr('Plugins are on the roadmap'),
            description=tr('Extend Decksmith with additional actions and integrations. Plugin support is planned for a future release.'))
        page.add_css_class('section-plugins')
        return page

    def about(self):
        scroll=Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER);scroll.add_css_class('section-about')
        clamp=Adw.Clamp(maximum_size=540);scroll.set_child(clamp)
        box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=18)
        for side in ('top','bottom','start','end'):getattr(box,'set_margin_'+side)(28)
        clamp.set_child(box)
        logo=Gtk.Picture.new_for_paintable(self.owner.logo_texture);logo.set_can_shrink(True);logo.set_content_fit(Gtk.ContentFit.CONTAIN)
        frame=Gtk.AspectFrame(ratio=1,obey_child=False);frame.set_child(logo)
        logo_clamp=Adw.Clamp(maximum_size=112,tightening_threshold=112,child=frame);box.append(logo_clamp)
        title=Gtk.Label(label='Decksmith');title.add_css_class('title-1');box.append(title)
        box.append(Gtk.Label(label='Open Control, Forged for Linux.'))
        group=Adw.PreferencesGroup();box.append(group)
        for title,detail in [('License','Apache-2.0'),('Appearance','Follows the current GNOME theme'),('Release',self.release_id())]:
            row=Adw.ActionRow(title=title,subtitle=detail);group.add(row)
        from panel import ROOT
        for title,uri in [('User guide',(ROOT/'docs/USER_GUIDE.md').as_uri()),('Source code','https://github.com/infamous-pattern/decksmith'),('Buy me a coffee','https://buymeacoffee.com/infamouspattern')]:
            b=Gtk.LinkButton(uri=uri,label=title,halign=Gtk.Align.CENTER);box.append(b)
        copy=Gtk.Button(label=tr('Copy diagnostic information'),halign=Gtk.Align.CENTER)
        copy.connect('clicked',self.copy_diagnostics);box.append(copy)
        notice=Gtk.Label(label='Elgato and Stream Deck are trademarks of Corsair Memory, Inc. Decksmith is an independent project and is not affiliated with, endorsed by, or sponsored by Elgato or Corsair.',wrap=True,xalign=0,max_width_chars=60)
        notice.add_css_class('dim-label');box.append(notice)
        return scroll

    def release_id(self):
        from panel import ROOT
        import json
        try:return json.loads((ROOT/'release.json').read_text())['id']
        except (OSError,ValueError,KeyError):return 'Development checkout'

    def copy_diagnostics(self,*_):
        import json,platform
        # Deliberately exclude layout contents, URLs, application identities and paths.
        status=self.status
        report={'release':self.release_id(),'system':platform.system(),
                'gtk':f'{Gtk.get_major_version()}.{Gtk.get_minor_version()}.{Gtk.get_micro_version()}',
                'connected':status.get('connected'),'display_ready':status.get('display_ready'),
                'controls_running':status.get('running'),
                'lock_detection':status.get('auto_lock',{}).get('available')}
        self.owner.get_clipboard().set(json.dumps(report,indent=2))
        self.ed.status.set_text('Diagnostic information copied. No layout or personal targets included.')

    def navigate(self,name):
        if name not in self.nav or getattr(self,'navigating',False):return
        self.navigating=True
        self.section=name
        for widget in getattr(self,'editor_extras',()):widget.set_visible(name=='keys')
        for key,b in self.nav.items():
            b.set_active(key==name)
            caption,label=self.nav_labels[key]
            caption.set_markup('<b>'+GLib.markup_escape_text(label)+'</b>' if key==name else GLib.markup_escape_text(label))
        # One permanently visible preview; tab changes never reparent or resize it.
        for section,_,_ in SECTIONS:self.preview_scroll.remove_css_class('section-'+section)
        self.preview_scroll.add_css_class('section-'+name)
        self.stack.set_visible_child_name(name)
        self.navigating=False
        self.sync()

    def page_selected(self,picker,_):
        if self.changing_page or self.ed.draft is None:return
        i=picker.get_selected()
        if i<len(self.ed.page_rows):self.ed.page_list.select_row(self.ed.page_rows[i][0])

    def can_control(self):
        return bool(self.status.get('connected') and self.status.get('display_ready') and not self.status.get('auto_lock',{}).get('locked'))

    def update_status(self,status):
        self.status=status
        text='Locked' if status.get('auto_lock',{}).get('locked') else 'Connected' if status.get('connected') else 'Disconnected'
        self.connection.set_text(text+' · '+('Controls running' if status.get('running') else 'Controls stopped'))
        self.device_info.set_text('Stream Deck + · '+text)
        self.banner.set_title('Device locked · layout editing remains available' if text=='Locked' else 'Device disconnected · your layout remains available' if text=='Disconnected' else '')
        self.banner.set_revealed(text!='Connected')
        ed=self.ed
        # A preset changed outside the editor: refresh only a clean draft, once.
        if ed.draft is not None and not ed.draft.dirty and not ed.pending and status.get('layout') and status['layout']!=ed.device_layout and not self.loading_layout:
            self.loading_layout=True
            def loaded(data):self.loading_layout=False;ed.loaded(data)
            ed.request(ed.load_layout,loaded)
        self.sync()

    def sync(self):
        ed=self.ed
        if self.section!='keys':ed.key_tools_row.set_visible(False)
        if ed.draft is None:self.test.set_sensitive(False);return
        if self.theme_controls is None:
            from theme_dialog import ThemeControls
            self.theme_controls=ThemeControls(ed);ed.theme_dialog=self.theme_controls
            self.theme_expander.add_row(self.theme_controls)
        self.theme_controls.sync_from_draft()
        from themes import PRESETS
        theme=ed.draft.data.get('theme')
        self.theme_expander.set_subtitle((PRESETS[theme['preset']][0] if theme else 'Current appearance')+' · Applies to all pages')
        self.changing_page=True
        names=[p['name'] for p in ed.draft.data['pages']]
        model=self.page_picker.get_model()
        if [model.get_string(i) for i in range(model.get_n_items())]!=names:model.splice(0,model.get_n_items(),names)
        self.page_picker.set_selected(ed.page);self.changing_page=False
        self.saved.set_text('Unsaved preview' if ed.draft.dirty else 'Saved')
        if ed.control_kind=='dial':
            from dial_model import effective
            label=effective(ed.draft.data,ed.page)[ed.dial_index]['label'];control=f'Dial {ed.dial_index+1}'
        else:
            label=ed.draft.data['pages'][ed.page]['keys'][ed.key]['label'];control=f'Key {ed.key+1}'
        caption=control if label==control else f'{control} · {label}'
        self.context_heading.set_text(f'{names[ed.page]} › {caption}')
        ed.form.set_title(f"Key {ed.key+1} · "+ed.draft.data['pages'][ed.page]['keys'][ed.key]['label'])
        blocked=not self.can_control() or ed.pending or ed.draft.dirty
        self.test.set_sensitive(not blocked)
        self.test_note.set_text('Save and Apply before testing edited controls.' if ed.draft.dirty else 'Testing unavailable while the device is locked, disconnected or starting.' if not self.can_control() else 'Runs the selected saved key action or dial press.')
        reason=save_reason(ed.draft.dirty,getattr(ed,'layout_valid',False),ed.pending,
                           self.status,getattr(ed,'validation_error',None))
        enabled=reason is None
        ed.save_button.set_sensitive(enabled)
        text=reason or 'Ready to save and apply · Ctrl+S'
        self.save_reason.set_text(text);self.save_reason.set_tooltip_text(text)
        ed.save_button.set_tooltip_text(text)
        self.validation_speech.update(text if ed.draft.dirty and not ed.pending and not getattr(ed,'layout_valid',False) else None)
        from dial_model import effective
        key_names=tuple(key['label'] for key in ed.draft.data['pages'][ed.page]['keys'])
        dial_names=tuple(dial['label'] for dial in effective(ed.draft.data,ed.page))
        names=(key_names,dial_names)
        if getattr(self,'accessible_names',None)!=names:
            self.accessible_names=names
            for i,label in enumerate(key_names):
                ed.keys[i].update_property([Gtk.AccessibleProperty.LABEL],[f"Key {i+1}: {label}"])
            for i,label in enumerate(dial_names):
                for button in (ed.dial_buttons[i],ed.touch_buttons[i]):
                    button.update_property([Gtk.AccessibleProperty.LABEL],[f"Dial {i+1}: {label}"])

    def test_action(self,_):
        ed=self.ed
        if not self.can_control() or ed.pending or ed.draft is None or ed.draft.dirty:return
        slot=ed.key if ed.control_kind=='key' else ed.dial_index+8
        payload=__import__('json').dumps(ed.draft.data)
        def done(_):ed.render();ed.status.set_text('Test action requested. Live control feedback reports the result.')
        ed.request(lambda:ed.call('TestControl',GLib.Variant('(syy)',(payload,ed.page,slot))),done,
                   'Could not test this control. Check the device, saved page and audio target; push to talk must be tested on the physical device.')

    def announce_validation(self,message):
        if self.ed.get_mapped() and self.ed.is_active():
            self.save_reason.announce(message,Gtk.AccessibleAnnouncementPriority.MEDIUM)

    def cleanup(self):
        self.validation_speech.close()
        if hasattr(self, 'plugin_lab'):self.plugin_lab.close()
        self.style.disconnect(self.style_handler)
        Gtk.StyleContext.remove_provider_for_display(Gdk.Display.get_default(),self.css)

def save_reason(dirty,valid,pending,status,error=None):
    """One consistent explanation for the button and the always-visible status line."""
    if pending:return 'Please wait for the current request to finish.'
    if not valid:return 'Cannot save: '+(error or 'Check the highlighted fields.')
    if not dirty:return 'No changes to save.'
    if status.get('auto_lock',{}).get('locked'):return 'Unlock your desktop to save and apply. Your draft is kept.'
    if not status.get('running',True):return 'Start Background controls on Home to save and apply. Your draft is kept.'
    if status.get('unavailable'):return 'Background controls are unavailable. Your draft is kept; retry after they reconnect.'
    return None
