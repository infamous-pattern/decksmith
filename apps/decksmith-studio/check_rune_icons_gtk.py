"""Native Rune picker/editor check: local fixture, no daemon calls or layout writes.

Run from the repository or installed release root in a GNOME desktop session.
"""
import copy,json,sys,traceback
from pathlib import Path
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path.cwd()/'apps/decksmith-studio'))
import panel,startup
from panel import Gtk,GLib
from editor import ACTIONS
status={'layout':'audio','active_page':0,'display_ready':True,'running':True,'connected':True,'attention':[]}
layout=json.loads(Path('config/audio.json').read_text())
def call(method,args=None):
    if method=='GetLayout':return (json.dumps(layout),)
    if method=='GetStatus':return (json.dumps(status),)
    if method=='PreviewKeys':return bytes([35,40,46])*(8*120*120)
    if method=='PreviewTouch':return bytes([35,40,46])*(800*100),'draft'
    raise ValueError('Unexpected daemon operation: '+method)
panel.call=call;panel.read_status=lambda:status.copy()
startup.read=lambda:{'enabled':False,'available':True,'detail':'Test fixture'}
app=panel.Panel();app.set_application_id('cc.senecal.Decksmith.RuneCheck')
app.set_flags(panel.Gio.ApplicationFlags.NON_UNIQUE)
errors=[];stage=0
def tick():
    global stage
    try:
        ed=app.editor
        if ed.draft is None or ed.pending:return True
        if stage==0:
            ed.shell.navigate('keys');ed.select_key(3)
            ed.label.set_text('My custom label')
            before=copy.deepcopy(ed.draft.data)
            ed.open_icon_library();dialog=ed.icon_dialog
            assert len([i for i in dialog.library.items() if i['id'].startswith(('rune:','tabler:'))])==148
            assert dialog.library.items()[0]['source']=='Rune Outline'
            dialog.cancel.emit('clicked')
            assert ed.draft.data==before,'Cancel changed draft'
            ed.open_icon_library();ed.icon_dialog.escape.get_action().activate(0,ed.icon_dialog,None)
            assert ed.draft.data==before,'Escape changed draft'
            ed.open_icon_library();dialog=ed.icon_dialog
            dialog.search.set_text('Rune');dialog.populate()
            assert dialog.count.get_text().startswith('14 icons')
            key=ed.draft.data['pages'][ed.page]['keys'][ed.key]
            ed.test_label=key['label'];ed.test_action=copy.deepcopy(key['action'])
            stage=1;return True
        if stage==1:
            dialog=ed.icon_dialog
            assert dialog.get_mapped(),'Picker not displayed'
            assert dialog.flow.get_first_child().get_child().get_mapped()
            snapshot=Gtk.Snapshot();ed.snapshot_child(ed.get_child(),snapshot)
            ed.get_renderer().render_texture(snapshot.to_node(),None).save_to_png('/tmp/decksmith-rune-picker.png')
            dialog.choose(dialog.library.preferred('lock'))
            key=ed.draft.data['pages'][ed.page]['keys'][ed.key]
            assert key['label']==ed.test_label and key['action']==ed.test_action
            assert key['icon_source']=='rune:lock' and key['icon_tint']
            assert bytes(key['icon_png']).startswith(b'\x89PNG')
            ed.action.set_selected(ACTIONS.index('media_next'))
            assert key['icon_source']=='rune:player-skip-forward'
            ed.label.set_text('My next track');ed.render()
            assert key['label']=='My next track'
            ed.action.set_selected(ACTIONS.index('system'))
            ed.system_picker.set_selected(ed.system_commands.index('lock'))
            assert key['icon_source']=='rune:lock'
            ed.system_picker.set_selected(ed.system_commands.index('bluetooth'))
            assert key['icon_source']=='tabler:bluetooth'
            print('PASS native Rune picker, visible gallery, Cancel/Escape, custom label/action, media defaults and Tabler fallback',flush=True)
            app.quit();return False
    except BaseException as error:
        traceback.print_exc();errors.append(str(error));app.quit();return False
app.connect('activate',lambda *_:GLib.timeout_add(1200,tick))
app.run(None)
if errors:raise SystemExit('; '.join(errors))
