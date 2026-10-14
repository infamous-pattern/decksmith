"""Isolated native two-device editor acceptance; opens no HID/audio or user data."""
import json,os,subprocess,sys,tempfile,time,traceback
from pathlib import Path
from profile_lab import ProfileLab
from profile_client import connected_transport as transport
from gi.repository import Gtk,GLib
if os.environ.get('DECKSMITH_ISOLATED_TEST')!='1':raise SystemExit('Isolated test required')
binary=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
tmp=tempfile.TemporaryDirectory(prefix='decksmith-native-profiles-')
log=(out/'profile-service.log').open('w')
service=subprocess.Popen([str(binary),'--connected-workspace-lab',str(Path(tmp.name)/'store'),'--seconds','75'],stdout=log,stderr=subprocess.STDOUT)
for _ in range(60):
    try:transport('ListDevices');break
    except Exception:
        if service.poll() is not None:raise RuntimeError('Profile lab stopped')
        time.sleep(.1)
else:raise RuntimeError('Profile lab unavailable')
app=ProfileLab(connected=True);state={'phase':0};errors=[];deadline=time.monotonic()+45

def screenshot(ed,name):
    paint=Gtk.WidgetPaintable.new(ed);snap=Gtk.Snapshot();paint.snapshot(snap,ed.get_width(),ed.get_height())
    node=snap.to_node()
    if node is None:return False
    ed.get_renderer().render_texture(node,None).save_to_png(str(out/(name+'.png')))
    return True

def exercise():
    try:
        if time.monotonic()>deadline:raise RuntimeError('Native profile check timed out')
        ed=getattr(app,'editor',None)
        if ed is None or ed.pending or ed.draft is None:return True
        phase=state['phase']
        if phase>0:assert ed is state['editor'],'Device selection created another editor'
        if phase>0:assert (ed.get_width(),ed.get_height())==state['size'],'Window resized during device selection'
        assert 'failed' not in ed.status.get_text().lower(),ed.status.get_text()
        assert getattr(ed,'layout_valid',False),ed.status.get_text()
        if phase==0:
            assert ed.key_count==8 and ed.device_model=='plus'
            assert sum(b.get_visible() for b in ed.keys)==8
            state['editor']=ed;state['size']=(ed.get_width(),ed.get_height())
            ed.shell.navigate('keys')
            state['plus']=app.selected;state['mk2']=next(d['id'] for d in app.devices if d['model']=='mk2')
            ed.edit_mode.set_active(True);ed.select_key(7);ed.label.set_text('Plus draft 🎵')
            assert ed.draft.dirty;ed.draft.data['pages'][0]['keys'][7]['action']={'type':'audio_adjust','target':'app:application.name=New fixture','percent':1};ed.draft.validate();state['plus_draft']=ed.draft
            app.name.set_text('Desk draft');app.adjust_brightness(1)
            app.choose(state['mk2']);state['phase']=1
        elif phase==1:
            assert len(ed.keys)==15 and ed.device_model=='mk2';assert not ed.touch_preview.get_visible();assert not ed.dial_row.get_visible()
            ed.edit_mode.set_active(True);ed.select_key(14);ed.label.set_text('Fifteen 🎵');assert ed.draft.dirty
            assert len(ed.draft.data['pages'][0]['keys'])==15
            # Read-only preview must not paint another device or publish a draft.
            profile=json.loads(transport('GetProfile',GLib.Variant('(s)',(state['mk2'],)))[0]);assert profile['layout']['pages'][0]['keys'][14]['label']!='Fifteen 🎵'
            ed.save(None);state['phase']=2
        elif phase==2:
            assert not ed.shell.test.get_sensitive(),'Virtual test enabled live action execution'
            assert not ed.draft.dirty;profile=json.loads(transport('GetProfile',GLib.Variant('(s)',(state['mk2'],)))[0]);assert profile['layout']['pages'][0]['keys'][14]['label']=='Fifteen 🎵'
            if any(f is None for f in ed.key_preview.frames) or ed.key_preview.rendered!=ed.key_preview.revision:return True
            assert len(ed.key_preview.frames)==15 and all(len(f)==72*72*3 for f in ed.key_preview.frames)
            ok,bounds=ed.keys[-1].compute_bounds(ed)
            assert ok and bounds.get_y()+bounds.get_height()<ed.get_height()-50,'MK.2 last row is clipped'
            if not screenshot(ed,'profile-mk2'):return True
            app.choose(state['plus']);state['phase']=3
        elif phase==3:
            assert ed.draft is state['plus_draft'];assert ed.draft.dirty
            assert ed.draft.data['pages'][0]['keys'][7]['label']=='Plus draft 🎵';assert ed.touch_preview.get_visible()
            assert app.name.get_text()=='Desk draft' and app.desired_brightness==51,'Home settings draft was lost'
            ed.save(None);state['phase']=4
        elif phase==4:
            assert not ed.draft.dirty
            if any(f is None for f in ed.key_preview.frames) or ed.key_preview.rendered!=ed.key_preview.revision:return True
            assert len(ed.key_preview.frames)==8 and all(len(f)==120*120*3 for f in ed.key_preview.frames)
            if not screenshot(ed,'profile-plus'):return True
            for identity,label,index in [(state['plus'],'Plus draft 🎵',7),(state['mk2'],'Fifteen 🎵',14)]:
                profile=json.loads(transport('GetProfile',GLib.Variant('(s)',(identity,)))[0]);assert profile['layout']['pages'][0]['keys'][index]['label']==label
            assert state['plus_draft'].undo_stack==[]
            app.editor.shell.navigate('home');app.save_metadata();state['phase']=5
        elif phase==5:
            assert app.clients[state['plus']].brightness==51
            status=json.loads(transport('ListDevices')[0]);assert next(d['brightness'] for d in status['devices'] if d['id']==state['mk2'])==50
            app.editor.shell.navigate('keys');app.choose(state['mk2']);state['rounds']=0;state['phase']=6
        elif phase==6:
            assert ed is state['editor'];state['rounds']+=1
            status=json.loads(transport('ListDevices')[0]);count,pending=status['history'];assert not pending
            if count>8:
                app.archive_history();state['phase']=7
            else:app.choose(state['plus'] if app.selected==state['mk2'] else state['mk2'])
            assert state['rounds']<16
        elif phase==7:
            status=json.loads(transport('ListDevices')[0]);assert status['history']==[8,False]
            assert ed is state['editor'];assert not ed.draft.dirty
            (out/'native-connected-editor-result.json').write_text(json.dumps({'status':'passed','checks':['live_save_and_apply','provider_set_refresh','single_workspace','stable_window_size','native_model_geometry','15th_key_edit_save','independent_draft_retention','separate_saves','native_model_sized_textures','touch_capabilities','scoped_home_brightness','verified_history_archive','legacy_plus_preview'],'physical_hid':False,'audio_access':False,'native_actions':False})+'\n')
            state['phase']=8;app.quit();return False
        return True
    except Exception as e:
        errors.append(str(e));traceback.print_exc();app.quit();return False
app.connect('activate',lambda *_:GLib.timeout_add(150,exercise))
try:app.run(None)
finally:
    service.terminate()
    try:service.wait(timeout=5)
    except subprocess.TimeoutExpired:service.kill();service.wait()
    if service.returncode!=0:errors.append('Live profile host did not stop cleanly')
    result=json.loads((Path(tmp.name)/'store/live-editor-result.json').read_text());assert result['peer_effects_while_other_save_pending']>0,result
    (out/'connected-editor-runtime-result.json').write_text(json.dumps(result,indent=2)+'\n')
    tmp.cleanup();log.close()
if state['phase']!=8 or not (out/'native-connected-editor-result.json').is_file():errors.append('Native profile acceptance did not complete')
if errors:raise SystemExit('; '.join(errors))
print('Native live editor, provider refresh and independent peer effects passed')
