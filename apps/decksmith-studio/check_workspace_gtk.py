"""Native shell regression check. Run from the repo root; fake writes only.
--render uses the live daemon's read-only image methods. No device actions are sent.
"""
import sys,json,time,traceback,resource
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'apps/decksmith-studio'))
import panel
from panel import Gtk,GLib
import startup
startup.read=lambda:{'enabled':False,'available':True,'detail':'Starts quietly at login'}
status={'layout':'audio','active_page':0,'display_ready':True,'running':True,'connected':True,'auto_lock':{'enabled':True,'locked':False,'available':True},'automatic_pages':{'available':True},'attention':[]}
layout=json.loads(Path('config/audio.json').read_text())
real_call=panel.call
calls=[]
def call(method,args=None):
 calls.append(method)
 if '--render' in sys.argv and method in ('PreviewKeys','PreviewTouch'):return real_call(method,args)
 if method=='GetLayout':return (json.dumps(layout),)
 if method=='GetStatus':return (json.dumps(status),)
 if method=='PreviewKeys':return bytes([35,40,46])*(8*120*120)
 if method=='PreviewTouch':return bytes([35,40,46])*(800*100),'draft'
 if method=='SaveLayout':layout.clear();layout.update(json.loads(args.unpack()[0]));return ()
 if method in ('ShowPage','TestControl'):return ()
 raise ValueError(method)
panel.call=call;panel.read_status=lambda:status.copy()
Gtk.Settings.get_default().set_property('gtk-enable-animations',False)
if '--large-text' in sys.argv:Gtk.Settings.get_default().set_property('gtk-xft-dpi',144*1024)
if '--compact' in sys.argv:
 import workspace_shell
 original_shell=workspace_shell.WorkspaceShell.__init__
 def compact_shell(self,ed,*args):
  original_shell(self,ed,*args);ed.set_default_size(1024,768)
 workspace_shell.WorkspaceShell.__init__=compact_shell
if '--dark' in sys.argv:panel.Adw.StyleManager.get_default().set_color_scheme(panel.Adw.ColorScheme.FORCE_DARK)
import workspace_shell
def placeholder_plugins(self):
 return panel.Adw.StatusPage(icon_name='application-x-addon-symbolic',title='Plugins are on the roadmap')
if '--managed-plugins' not in sys.argv:workspace_shell.WorkspaceShell.plugins=placeholder_plugins
app=panel.Panel();app.set_application_id('cc.senecal.Decksmith.WorkspaceCheck');app.set_flags(panel.Gio.ApplicationFlags.NON_UNIQUE)
errors=[];began=time.monotonic();stage=0;perf_start=None;perf_cpu=None;perf_calls=None

def snapshot(ed,name):
 assert ed.shell.root.get_width()<=ed.get_width(), [(name,ed.shell.stack.get_child_by_name(name).measure(Gtk.Orientation.HORIZONTAL,-1)) for name in ed.shell.nav]
 assert ed.device_preview.translate_coordinates(ed,0,0)[0]+ed.device_preview.get_width()<=ed.get_width(), (ed.device_preview.translate_coordinates(ed,0,0),ed.device_preview.get_width(),ed.get_width())
 print('GEOMETRY',ed.get_width(),ed.get_height(),ed.get_mapped(),ed.shell.root.get_width(),ed.shell.root.get_height(),flush=True)
 ed.allocate(ed.get_width(),ed.get_height(),-1,None)
 snap=Gtk.Snapshot();ed.snapshot_child(ed.get_child(),snap)
 ed.get_renderer().render_texture(snap.to_node(),None).save_to_png(str(Path.cwd()/('local/ui-'+name+'.png')))

def visible_fields(ed):
 adjustment=ed.property_column.get_hadjustment()
 assert adjustment.get_upper()<=adjustment.get_page_size()+1, 'Settings require horizontal scrolling'
 field=ed.label if ed.control_kind=='key' else ed.dial_controls.label
 assert field.translate_coordinates(ed,0,0)[0]>=ed.property_column.translate_coordinates(ed,0,0)[0], 'Field label clipped on the left'

def stable_geometry(ed):
 return (ed.get_width(),ed.get_height(),tuple((w.get_width(),w.get_height(),w.translate_coordinates(ed,0,0))
     for w in (ed.device_preview,ed.grid,ed.property_column,ed.workspace)))

def tick():
 global stage,perf_start,perf_cpu,perf_calls
 try:
  if time.monotonic()-began>60:raise RuntimeError('Workspace timeout')
  ed=app.editor
  if ed.draft is None or ed.pending:return True
  ed.allocate(ed.get_width(),ed.get_height(),-1,None)
  if stage==0:
   assert ed.shell.section=='home'
   if '--compact' in sys.argv:assert ed.get_width()<=1050,ed.get_width()
   assert ed.shell.root.get_mapped(), 'Native shell must actually be visible'
   ed.shared_preview_geometry=(ed.device_preview.get_width(),ed.device_preview.translate_coordinates(ed,0,0))
   snapshot(ed,'home');ed.shell.navigate('keys');stage=1;return True
  if stage==1:
   assert (ed.device_preview.get_width(),ed.device_preview.translate_coordinates(ed,0,0))==ed.shared_preview_geometry, (ed.shared_preview_geometry,(ed.device_preview.get_width(),ed.device_preview.translate_coordinates(ed,0,0)))
   viewport=ed.workspace.get_parent()
   scroller=viewport.get_parent()
   assert scroller.get_height()>250, f'Keys editor viewport collapsed: {scroller.get_height()}px'
   ed.select_key(3);ed.label.set_text('Test label');assert ed.draft.dirty
   ed.label.set_text('');assert ed.field_errors['label'].get_visible();assert not ed.save_button.get_sensitive()
   assert 'Cannot save:' in ed.shell.save_reason.get_text()
   ed.label.grab_focus();focus=ed.get_focus();reason=ed.shell.save_reason.get_text()
   ed.shell.update_status(status.copy())
   assert ed.get_focus() is focus and ed.shell.save_reason.get_text()==reason
   ed.label.set_text('Test label');assert not ed.field_errors['label'].get_visible()
   assert 'Key 4' in ed.shell.context_heading.get_text()
   assert ed.shell.saved.get_text()=='Unsaved preview'
   ed.shell.navigate('pages');ed.shell.navigate('plugins');ed.shell.navigate('about');ed.shell.navigate('keys')
   assert all(b.get_active()==(n=='keys') for n,b in ed.shell.nav.items())
   assert ed.key==3 and ed.label.get_text()=='Test label';assert not ed.shell.test.get_sensitive()
   for i,name in enumerate(ed.shell.nav):
    assert ed.history_shortcut(None,panel.Gdk.KEY_1+i,0,panel.Gdk.ModifierType.ALT_MASK)
    assert ed.shell.section==name and ed.shell.nav[name].has_focus()
   ed.shell.navigate('keys');ed.shell.nav['keys'].grab_focus()
   ed.allocate(ed.get_width(),ed.get_height(),-1,None);visible_fields(ed)
   snapshot(ed,'keys');ed.history_step(False);assert ed.label.get_text()!='Test label'
   stage=11;return True
  if stage==11:
   ed.shell.nav['keys'].grab_focus()
   visited=set()
   for _ in range(180):
    ed.shell.root.child_focus(Gtk.DirectionType.TAB_FORWARD)
    focused=ed.get_focus()
    if focused is not None:
     if any(focused is button or focused.is_ancestor(button) for button in ed.keys):visited.add('keys')
     if any(focused is button or focused.is_ancestor(button) for button in ed.dial_buttons):visited.add('dials')
   assert visited=={'keys','dials'},visited
   ed.shell.nav['keys'].grab_focus()
   ed.shell.preview_scroll.get_vadjustment().set_value(0)
   stage=12;return True
  if stage==12:
   ed.geometry_before=stable_geometry(ed)
   ed.select_dial(1);stage=2;return True
  if stage==2:
   geometry=stable_geometry(ed)
   assert geometry==ed.geometry_before, f'Key/dial size shift: {ed.geometry_before} -> {geometry}'
   visible_fields(ed)
   snapshot(ed,'dials');ed.select_key(3);stage=21;return True
  if stage==21:
   geometry=stable_geometry(ed)
   assert geometry==ed.geometry_before, f'Dial/key size shift: {ed.geometry_before} -> {geometry}'
   ed.shell.navigate('pages');stage=3;return True
  if stage==3:
   for button in ed.page_tools.values():
    x,y=button.translate_coordinates(ed,0,0)
    assert button.get_mapped() and button.get_height()>=24
    assert x>=0 and x+button.get_width()<=ed.get_width()
    assert y>=84 and y+button.get_height()<ed.get_height()-30
   ed.shell.theme_expander.set_expanded(True)
   theme_before=json.dumps(ed.draft.data.get('theme'),sort_keys=True)
   controls=ed.shell.theme_controls
   preset_before=controls.preset.get_selected()
   chosen=2 if preset_before!=2 else 3
   controls.preset.set_selected(chosen)
   assert ed.draft.dirty and ed.get_visible_dialog() is None
   assert controls.preset.get_selected()==chosen
   ed.history_step(False)
   assert json.dumps(ed.draft.data.get('theme'),sort_keys=True)==theme_before
   assert controls.preset.get_selected()==preset_before
   ed.history_step(True);assert controls.preset.get_selected()==chosen
   ed.history_step(False)
   font_row=controls.rows.rows['font']
   original_font=font_row.get_selected()
   if ed.draft.data.get('theme') is not None:
    font_row.set_selected(font_row.get_model().get_n_items()-1)
    assert ed.draft.data['theme']['appearance']['font']=='viking_runes'
    ed.history_step(False);assert font_row.get_selected()==original_font
   snapshot(ed,'pages');ed.shell.navigate('about');stage=4;return True
  if stage==4:
   snapshot(ed,'about');ed.shell.navigate('keys');ed.select_key(0)
   ed.shell.update_status(dict(status,connected=False));assert not ed.shell.test.get_sensitive();assert ed.label.get_sensitive()
   ed.shell.update_status(status.copy());assert ed.shell.test.get_sensitive()
   if ed.apps:
    from action_labels import application_label
    from editor import ACTIONS
    ed.action.set_selected(ACTIONS.index('open_application'));ed.app_picker.set_selected(1)
    assert ed.label.get_text()==application_label(ed.apps[0].get_display_name())
    ed.label.set_text('My launcher');ed.render();assert ed.label.get_text()=='My launcher'
   from editor import ACTIONS
   ed.label.set_text('EMPTY')
   ed.action.set_selected(ACTIONS.index('system'))
   ed.system_picker.set_selected(ed.system_commands.index('dnd'))
   assert ed.label.get_text()=='Do Not Disturb'
   ed.label.set_text('Quiet time');ed.render();assert ed.label.get_text()=='Quiet time'
   ed.label.set_text('Saved test');assert ed.draft.dirty
   app.activate_action('quit-decksmith',None)
   assert app.quit_all
   dialog=ed.get_visible_dialog();assert dialog is not None
   dialog.emit('response','keep');dialog.close();assert ed.draft.dirty and not ed.closed and not app.quit_all
   assert ed.history_shortcut(None,panel.Gdk.KEY_s,0,panel.Gdk.ModifierType.CONTROL_MASK)
   stage=41;return True
  if stage==41:
   assert not ed.draft.dirty and 'SaveLayout' in calls
   ed.shell.navigate('plugins');stage=5;perf_start=time.monotonic();perf_cpu=time.process_time();perf_calls=len(calls);return True
  if stage==5:
   if time.monotonic()-perf_start<4:return True
   assert list(ed.shell.nav)==['home','pages','keys','plugins','about']
   if '--managed-plugins' not in sys.argv:
    assert ed.shell.stack.get_visible_child().get_title()=='Plugins are on the roadmap'
    assert ed.shell.stack.get_visible_child().get_child() is None
   if '--managed-plugins' in sys.argv:
    ed.shell.plugin_lab.setup_section.set_expanded(True)
    ed.shell.plugin_lab.show_removal()
   snapshot(ed,'plugins')
   assert ed.device_column.get_parent() is ed.shell.preview_host
   assert ed.device_preview.get_mapped()
   assert ed.shell.stack.translate_coordinates(ed,0,0)[0] < ed.device_column.translate_coordinates(ed,0,0)[0]
   cpu=100*(time.process_time()-perf_cpu)/(time.monotonic()-perf_start)
   rss=next(line for line in Path('/proc/self/status').read_text().splitlines() if line.startswith('VmRSS:'))
   print(f'PERFORMANCE shared preview, idle CPU={cpu:.2f}% of one core, {rss}',flush=True)
   print('PASS native shell, draft retention, history, status gating and five sections',flush=True)
   app.quit();return False
 except BaseException as e:traceback.print_exc();errors.append(str(e));app.quit();return False
def setup(*_):
 if '--dark' in sys.argv:panel.Adw.StyleManager.get_default().set_color_scheme(panel.Adw.ColorScheme.FORCE_DARK)
 if '--compact' in sys.argv:app.editor.set_default_size(1024,768)
 app.editor.test_paint=Gtk.WidgetPaintable.new(app.editor)
 GLib.timeout_add(1800,tick)
if '--inspect' not in sys.argv:app.connect('activate',setup)
app.run(None)
if errors:raise SystemExit('; '.join(errors))
