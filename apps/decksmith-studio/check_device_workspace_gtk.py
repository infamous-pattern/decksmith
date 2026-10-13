"""Fresh native saved-device enrollment with no HID or real audio access."""
import json,os,subprocess,sys,tempfile,time,traceback
from pathlib import Path
from device_workspace import DeviceWorkspace
from profile_client import workspace_transport as transport
from gi.repository import GLib,Gtk
if os.environ.get('DECKSMITH_ISOLATED_TEST')!='1':raise SystemExit('Isolated test required')
binary=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
tmp=tempfile.TemporaryDirectory(prefix='decksmith-native-saved-')
log=(out/'device-workspace-service.log').open('w')
service=subprocess.Popen([str(binary),'--workspace-lab',str(Path(tmp.name)/'store'),'--seconds','90'],stdout=log,stderr=subprocess.STDOUT)
for _ in range(60):
    try:transport('ListDevices');break
    except Exception:
        if service.poll() is not None:raise RuntimeError('Workspace fixture stopped')
        time.sleep(.1)
else:raise RuntimeError('Workspace fixture unavailable')
app=DeviceWorkspace();state={'phase':0};errors=[];deadline=time.monotonic()+60

def shot(ed,name):
    paint=Gtk.WidgetPaintable.new(ed);snap=Gtk.Snapshot();paint.snapshot(snap,ed.get_width(),ed.get_height());node=snap.to_node()
    if node is None:return False
    ed.get_renderer().render_texture(node,None).save_to_png(str(out/(name+'.png')));return True

def exercise():
    try:
        if time.monotonic()>deadline:raise RuntimeError('Native saved workspace timed out')
        phase=state['phase'];ed=getattr(app,'editor',None)
        if phase==0:
            if not hasattr(app,'enrollment_snapshot'):return True
            snapshot=app.enrollment_snapshot;assert snapshot['devices']==[]
            app.enroll(snapshot,snapshot['discovery']['candidates'][0],'Plus 🎵',import_plus=True);state['phase']=1;return True
        if ed is None or ed.pending or ed.draft is None or app.enrollment_pending or app.switching:return True
        assert ed.layout_valid,ed.status.get_text()
        if phase==1:
            state['editor']=ed;state['size']=(ed.get_width(),ed.get_height());state['plus']=app.selected
            assert ed.key_count==8 and ed.call.workspace_only
            status=json.loads(ed.call('GetStatus')[0]);assert not status['connected'] and not status['display_ready'] and status['detected']
            assert ed.save_button.get_label()=='Save profile'
            ed.shell.navigate('keys');ed.edit_mode.set_active(True);ed.select_key(7);ed.label.set_text('Plus draft 🎵');state['plus_draft']=ed.draft
            app.name.set_text('Retained name');app.adjust_brightness(1)
            snapshot=json.loads(transport('ListDevices')[0]);app.enroll(snapshot,snapshot['discovery']['candidates'][1],'MK.2 🎵');state['phase']=2
        elif phase==2:
            assert ed is state['editor'] and ed.draft is state['plus_draft'] and ed.draft.dirty
            state['mk2']=next(d['id'] for d in app.devices if d['model']=='mk2');app.choose(state['mk2']);state['phase']=3
        elif phase==3:
            assert ed is state['editor'] and (ed.get_width(),ed.get_height())==state['size']
            assert ed.key_count==15 and not ed.touch_preview.get_visible() and not ed.dial_row.get_visible()
            ed.edit_mode.set_active(True);ed.select_key(14);ed.label.set_text('Fifteen 🎵')
            assert ed.save_button.get_sensitive(),'Offline save blocked'
            assert not ed.shell.test.get_sensitive()
            ed.save(None);state['phase']=4
        elif phase==4:
            assert not ed.draft.dirty
            if any(f is None for f in ed.key_preview.frames) or ed.key_preview.rendered!=ed.key_preview.revision:return True
            assert len(ed.key_preview.frames)==15 and all(len(f)==72*72*3 for f in ed.key_preview.frames)
            if not shot(ed,'workspace-mk2'):return True
            app.choose(state['plus']);state['phase']=5
        elif phase==5:
            assert ed is state['editor'] and ed.draft is state['plus_draft'] and ed.draft.dirty
            assert app.name.get_text()=='Retained name' and app.desired_brightness==51
            ed.save(None);state['phase']=6
        elif phase==6:
            assert not ed.draft.dirty
            for identity,index,label in [(state['plus'],7,'Plus draft 🎵'),(state['mk2'],14,'Fifteen 🎵')]:
                profile=json.loads(transport('GetProfile',GLib.Variant('(s)',(identity,)))[0]);assert profile['layout']['pages'][0]['keys'][index]['label']==label
            snapshot=json.loads(transport('ListDevices')[0]);assert all(not s['connected'] for s in snapshot['sessions'])
            if any(f is None for f in ed.key_preview.frames) or ed.key_preview.rendered!=ed.key_preview.revision:return True
            if not shot(ed,'workspace-plus'):return True
            (out/'native-device-workspace-result.json').write_text(json.dumps({'status':'passed','checks':['explicit_initial_enrollment','prepared_plus_import','mk2_own_defaults','live_enrollment_keeps_draft','single_editor','stable_window_size','offline_save','device_addressed_saves','retained_home_draft','model_geometry','read_only_discovery_status','actions_disabled'],'physical_hid':False,'real_audio_access':False})+'\n')
            state['phase']=7;app.quit();return False
        return True
    except Exception as e:errors.append(str(e));traceback.print_exc();app.quit();return False
app.connect('activate',lambda *_:GLib.timeout_add(150,exercise))
try:app.run(None)
finally:
    service.terminate()
    try:service.wait(timeout=5)
    except subprocess.TimeoutExpired:service.kill();service.wait()
    if service.returncode!=0:errors.append('Workspace service did not stop cleanly')
    tmp.cleanup();log.close()
if state['phase']!=7 or not (out/'native-device-workspace-result.json').is_file():errors.append('Native workspace acceptance did not complete')
if errors:raise SystemExit('; '.join(errors))
print('Native saved-device workspace passed')
