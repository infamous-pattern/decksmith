"""Run under a desktop session: timeout 20s python3 check_editor_gtk.py.
Uses a local fixture and never calls the daemon or writes a user layout.
"""
import json
from pathlib import Path
from panel import Panel, GLib
from editor import Editor, ACTIONS
import editor as editor_module
editor_module.fetch_website_icon = lambda _url: None

app = Panel()
app.set_application_id('cc.senecal.Decksmith.EditorCheck')
errors = []

def exercise():
    if editor.draft is None or editor.pending:return True
    try:
        assert editor.draft is not None
        assert not editor.appearance.get_expanded()
        editor.appearance.set_expanded(True)
        assert editor.page == 1, 'Opening must follow WORK on the device'
        editor.edit_mode.set_active(True)
        editor.keys[0].emit('clicked')
        assert editor.page == 1 and editor.key == 0
        editor.edit_mode.set_active(False)
        editor.keys[0].emit('clicked')
        assert editor.page == 0, 'Click HOME must navigate'
        while editor.pending:
            GLib.MainContext.default().iteration(True)
        editor.keys[1].emit('clicked')
        assert editor.page == 1, 'Click WORK must navigate'
        while editor.pending:
            GLib.MainContext.default().iteration(True)
        editor.follow_status({'layout':'audio', 'active_page':0, 'display_ready':True})
        assert editor.page == 0
        model = editor.page_picker.get_model()
        for _ in range(30):
            editor.page_picker.set_selected(1)
            assert editor.page == 1
            editor.page_picker.set_selected(0)
            assert editor.page == 0
        assert editor.page_picker.get_model() == model
        assert not editor.draft.dirty
        editor.select_key(7)
        editor.label.set_text('CHECK')
        editor.page_picker.set_selected(1)
        editor.page_picker.set_selected(0)
        assert editor.draft.data['pages'][0]['keys'][7]['label'] == 'CHECK'
        editor.page_name.set_text('DESK')
        assert editor.page_picker.get_model().get_string(0) == 'DESK'
        assert editor.label.get_text() == 'DESK'
        assert editor.draft.data['pages'][1]['keys'][0]['label'] == 'DESK'
        editor.add_page(None)
        assert editor.page == 2
        editor.page_picker.set_selected(1)
        assert editor.page == 1
        editor.page_picker.set_selected(1)
        editor.select_key(6)
        for text in ('T', 'TE', 'TES', 'TEST', 'TEST KEY'):
            editor.label.set_text(text)
            assert editor.save_button.get_sensitive(), (text, editor.status.get_text())
        editor.label.set_text('Test')
        assert editor.label.get_text() == 'Test'
        assert editor.draft.data['pages'][1]['keys'][6]['label'] == 'Test'
        assert editor.save_button.get_sensitive()
        for invalid in ('W'*25, 'TEST!', ''):
            editor.label.set_text(invalid)
            assert not editor.save_button.get_sensitive()
            assert editor.save_button.get_tooltip_text() == editor.status.get_text()
        for position in range(4):
            editor.label_position.set_selected(position)
            assert editor.draft.data['pages'][1]['keys'][6]['label_position']==['hidden','top','middle','bottom'][position]
        editor.label_background.set_selected(1)
        editor.label_color.set_selected(5)
        key=editor.draft.data['pages'][1]['keys'][6]
        assert key['label_background']=='transparent' and key['label_color']=='yellow'
        caption=editor.keys[6].get_child().get_last_child()
        assert caption.get_color().to_string()=='rgb(255,225,60)', 'Selected caption color must override default styling'
        assert caption.has_css_class('transparent')
        editor.label.set_text('mute')
        assert editor.save_button.get_sensitive()
        for kind in ('next_page', 'previous_page'):
            editor.action.set_selected(ACTIONS.index(kind))
            assert editor.draft.data['pages'][1]['keys'][6]['action'] == {'type':kind}
            assert not editor.destination.get_visible()
            assert editor.save_button.get_sensitive()
        editor.action.set_selected(ACTIONS.index('open_application'))
        assert not editor.save_button.get_sensitive()
        assert editor.apps
        editor.app_picker.set_selected(1)
        assert editor.draft.data['pages'][1]['keys'][6]['action']['desktop_id']
        assert editor.save_button.get_sensitive()
        editor.action.set_selected(ACTIONS.index('open_website'))
        editor.url.set_text('file:///tmp/test')
        assert not editor.save_button.get_sensitive()
        editor.url.set_text('https://example.com/')
        assert editor.save_button.get_sensitive()
        editor.page_name.set_text('My Work')
        assert editor.page_name.get_text() == 'MY WORK'
        assert editor.draft.data['pages'][1]['name'] == 'MY WORK'
        assert editor.save_button.get_sensitive(), editor.status.get_text()
        editor.page_name.set_text('Page 1')
        assert editor.page_name.get_text() == 'PAGE 1'
        assert editor.draft.data['pages'][1]['name'] == 'PAGE 1'
        assert editor.save_button.get_sensitive(), editor.status.get_text()
        editor.page_name.set_text('MY WORKS')
        assert editor.save_button.get_sensitive(), editor.status.get_text()
        editor.page_name.set_text('MY WORK A')
        assert not editor.save_button.get_sensitive()
        editor.reset(None)
        editor.move_page(1)
        assert editor.page == 1 and editor.draft.origins == [1,0]
        assert editor.destination.get_selected() == 1
        editor.duplicate_page(None)
        assert editor.page == 2 and editor.page_name.get_text() == 'COPY 1'
        assert editor.destination.get_selected() == 2
        dialog = editor.delete_page(None)
        dialog.emit('response', 'cancel')
        dialog.close()
        assert len(editor.draft.data['pages']) == 3
        dialog = editor.delete_page(None)
        dialog.emit('response', 'delete')
        dialog.close()
        assert len(editor.draft.data['pages']) == 2
        from artwork_dialog import ArtworkDialog
        from io import BytesIO
        from PIL import Image
        source = BytesIO()
        Image.new('RGB',(400,200),'red').save(source,format='PNG')
        applied=[]
        artwork=ArtworkDialog(editor,source.getvalue(),applied.append)
        artwork.fit.set_selected(1)
        artwork.size.set_value(96)
        artwork.background.set_selected(1)
        artwork.use(None)
        assert len(applied)==1
        assert Image.open(BytesIO(bytes(applied[0]))).size==(120,120)
        editor.reset(None)
        assert len(editor.draft.data['pages']) == 2 and not editor.draft.dirty
        from copy import deepcopy
        imported=deepcopy(editor.draft.saved)
        imported['pages'][0]['name']='IMPORT'
        confirmation=editor.confirm_import(imported)
        confirmation.emit('response','load');confirmation.close()
        assert editor.page_name.get_text()=='IMPORT'
        assert all(origin is None for origin in editor.draft.origins)
        assert editor.draft.dirty
        editor.reset(None)
        assert editor.page_name.get_text()=='HOME'
        from copy import deepcopy
        while editor.pending:
            GLib.MainContext.default().iteration(True)
        editor.edit_mode.set_active(True)
        before=deepcopy(editor.draft.data)
        source=editor.key_sources[0]
        assert not editor.pending, 'pending request before drag'
        assert source.emit('prepare',0.,0.) is not None, repr(editor.key_drag)
        token=editor.key_drag[0]
        assert editor.key_targets[5].emit('drop',token,0.,0.)
        keys=editor.draft.data['pages'][editor.page]['keys']
        assert keys[5]==before['pages'][editor.page]['keys'][0]
        assert keys[0]==before['pages'][editor.page]['keys'][5]
        assert editor.key==5 and editor.draft.dirty
        assert not editor.key_targets[2].emit('drop',token,0.,0.)
        source.emit('prepare',0.,0.)
        token=editor.key_drag[0]
        editor.reset(None)
        assert not editor.key_targets[2].emit('drop',token,0.,0.)
        assert not editor.draft.dirty
        editor.edit_mode.set_active(False)
        assert source.emit('prepare',0.,0.) is None
        editor.edit_mode.set_active(True)
        editor.select_key(0)
        original=deepcopy(editor.draft.data['pages'][editor.page]['keys'][0])
        editor.key_tools['copy'].emit('clicked')
        assert not editor.draft.dirty
        editor.page=1;editor.rebuild();editor.select_key(5)
        editor.key_tools['paste'].emit('clicked')
        assert editor.draft.data['pages'][1]['keys'][5]==original
        editor.label.set_text('Changed')
        assert editor.draft.clipboard[0]==original
        editor.key_tools['clear'].emit('clicked')
        assert editor.draft.data['pages'][1]['keys'][5]=={'label':'EMPTY','label_position':'hidden','action':{'type':'none'}}
        editor.reset(None)
        assert not editor.draft.dirty and not editor.key_tools['paste'].get_sensitive()
        editor.key_background.set_selected(7)
        assert editor.draft.data['pages'][editor.page]['keys'][editor.key]['background_color']=='blue'
        from layout_package import encode,decode
        assert decode(encode(editor.draft.data)[0])==editor.draft.data
        editor.label.set_text('Volume Up')
        editor.action.set_selected(ACTIONS.index('volume_up'))
        assert editor.save_button.get_sensitive()
        editor.label.set_text('')
        reason=editor.status.get_text()
        editor.key_tools['copy'].emit('clicked')
        assert editor.status.get_text()==reason and not editor.save_button.get_sensitive()
        editor.reset(None)
        editor.label.set_text('Undo Me')
        assert editor.undo_button.get_sensitive()
        editor.undo_button.emit('clicked')
        assert editor.label.get_text()!='Undo Me'
        editor.redo_button.emit('clicked')
        assert editor.label.get_text()=='Undo Me'
        from gi.repository import Gdk
        assert editor.history_shortcut(None,Gdk.KEY_z,0,Gdk.ModifierType.CONTROL_MASK)
        assert editor.label.get_text()!='Undo Me'
        assert editor.history_shortcut(None,Gdk.KEY_Z,0,Gdk.ModifierType.CONTROL_MASK|Gdk.ModifierType.SHIFT_MASK)
        assert editor.label.get_text()=='Undo Me'
        editor.reset(None)
        from dials import Dials
        dialog=Dials(editor)
        dialog.picker.set_selected(3)
        dialog.rotation.set_selected(2)
        assert dialog.label.get_text()=='Brightness'
        dialog.label.set_text('Display Brightness')
        dialog.picker.set_selected(0)
        dialog.picker.set_selected(3)
        assert dialog.label.get_text()=='Display Brightness'
        dialog.step.set_value(3)
        dialog.press.set_selected(3)
        dialog.use(None)
        assert editor.draft.data['dials'][3]=={'label':'Display Brightness','rotation':'brightness','step':3,'press':{'type':'previous_page'}}
        from layout_package import encode,decode
        assert decode(encode(editor.draft.data)[0])==editor.draft.data
        editor.reset(None)
        for kind in ('media_play_pause','media_previous','media_next'):
            editor.action.set_selected(ACTIONS.index(kind))
            current=editor.draft.data['pages'][editor.page]['keys'][editor.key]
            assert current['action']=={'type':kind}
            from media_artwork import LABELS
            assert current['label']==LABELS[kind] and current['icon_png']
            assert current['label_position']=='bottom'
        editor.label.set_text('My Music')
        assert editor.draft.data['pages'][editor.page]['keys'][editor.key]['label']=='My Music'
        editor.reset(None)
        editor.action.set_selected(ACTIONS.index('audio_adjust'))
        editor.audio_target.select('microphone')
        editor.form_changed()
        assert editor.draft.data['pages'][editor.page]['keys'][editor.key]['action']['target']=='microphone'
        dialog=Dials(editor)
        dialog.target.select('app:application.process.binary=mpz')
        dialog.target.items=[{'id':'system','label':'System sounds'},
                             {'id':'microphone','label':'Default microphone'},
                             {'id':'app:application.process.binary=mpz','name':'mpz','label':'App · mpz · Installed'},
                             {'id':'output:test','label':'Output · Café / Speaker (USB) with a very long name'}]
        dialog.target.select('system')
        dialog.target.set_selected(1)
        assert dialog.label.get_text()=='Default microphone'
        dialog.label.set_text('My Mic')
        dialog.target.select('microphone')  # Inventory refresh must preserve a custom label.
        assert dialog.label.get_text()=='My Mic'
        dialog.picker.set_selected(1)
        dialog.picker.set_selected(0)
        assert dialog.label.get_text()=='My Mic'
        dialog.target.set_selected(3)
        assert dialog.label.get_text()=='Cafe Speaker USB with a'
        assert dialog.apply.get_sensitive()
        dialog.target.set_selected(2)
        assert dialog.label.get_text()=='mpz'
        dialog.label.set_text('Music')
        while dialog.target.loading:
            GLib.MainContext.default().iteration(True)
        from unittest.mock import patch
        options=[{'id':'system','label':'System sounds'},
                 {'id':'microphone','label':'Default microphone'}]
        with patch('audio_target_picker.inventory',return_value=options):
            dialog.target.refresh()
            while dialog.target.loading:GLib.MainContext.default().iteration(True)
            assert dialog.target.value()=='app:application.process.binary=mpz'
            assert dialog.target.visible_items[dialog.target.get_selected()]['label'].startswith('Unavailable')
            assert dialog.label.get_text()=='Music'
            model_changes=[]
            handler=dialog.target.get_model().connect('items-changed',lambda *_:model_changes.append(True))
            dialog.target.refresh()
            while dialog.target.loading:GLib.MainContext.default().iteration(True)
            assert not model_changes, 'Unchanged inventory must not rebuild the dropdown'
            dialog.target.get_model().disconnect(handler)
        options=options+[{'id':'app:application.process.binary=mpz','label':'App · mpz'}]
        with patch('audio_target_picker.inventory',return_value=options):
            dialog.target.refresh()
            while dialog.target.loading:GLib.MainContext.default().iteration(True)
            assert dialog.target.visible_items[dialog.target.get_selected()]['label']=='App · mpz'
            assert dialog.label.get_text()=='Music'
        dialog.rotation.set_selected(1)
        dialog.changed()
        dialog.use(None)
        assert editor.draft.data['dials'][0]['audio_target']=='app:application.process.binary=mpz'
        assert editor.draft.data['dials'][0]['label']=='Music'
        assert decode(encode(editor.draft.data)[0])==editor.draft.data
        editor.reset(None)
        editor.action.set_selected(ACTIONS.index('push_to_talk'))
        editor.audio_target.select('input:fixture-mic',inputs_only=True)
        editor.form_changed()
        assert editor.draft.data['pages'][editor.page]['keys'][editor.key]['action']=={'type':'push_to_talk','target':'input:fixture-mic'}
        editor.draft.validate()
        dialog=Dials(editor)
        dialog.press.set_selected(7)
        assert not dialog.apply.get_sensitive()
        dialog.target.select('input:fixture-mic',inputs_only=True)
        dialog.changed()
        assert dialog.apply.get_sensitive()
        dialog.use(None)
        assert editor.draft.data['dials'][0]['press']=={'type':'push_to_talk','target':'input:fixture-mic'}
        assert decode(encode(editor.draft.data)[0])==editor.draft.data
        editor.reset(None)
        editor.action.set_selected(ACTIONS.index('media_play_pause'))
        editor.media_player.select('org.mpris.MediaPlayer2.mpz')
        editor.form_changed()
        assert editor.draft.data['pages'][editor.page]['keys'][editor.key]['media_player']=='org.mpris.MediaPlayer2.mpz'
        editor.draft.validate()
        dialog=Dials(editor)
        dialog.press.set_selected(4)
        dialog.media_player.select('org.mpris.MediaPlayer2.brave')
        dialog.changed();dialog.use(None)
        assert editor.draft.data['dials'][0]['media_player']=='org.mpris.MediaPlayer2.brave'
        assert decode(encode(editor.draft.data)[0])==editor.draft.data
        editor.media_player.select('');editor.form_changed()
        assert 'media_player' not in editor.draft.data['pages'][editor.page]['keys'][editor.key]
        editor.reset(None)
        # Exercise the integrated surface using real GTK signals and draft history.
        editor.edit_mode.set_active(True)
        editor.keys[0].emit('clicked')
        editor.label.set_text('Keep Key')
        editor.touch_buttons[2].emit('clicked')
        controls=editor.dial_controls
        assert editor.properties.get_visible_child_name()=='dial'
        assert controls.index==2 and not controls.apply.get_visible()
        controls.label.set_text('Keep Dial')
        assert editor.draft.data['dials'][2]['label']=='Keep Dial'
        editor.keys[0].emit('clicked')
        assert editor.label.get_text()=='Keep Key'
        editor.dial_buttons[2].emit('clicked')
        assert controls.label.get_text()=='Keep Dial'
        editor.undo_button.emit('clicked')
        assert controls.label.get_text()!='Keep Dial'
        editor.redo_button.emit('clicked')
        assert controls.label.get_text()=='Keep Dial'
        controls.label.set_text('')
        assert not editor.save_button.get_sensitive()
        editor.keys[1].emit('clicked')
        assert not editor.save_button.get_sensitive(), 'Invalid hidden dial must block saving'
        editor.touch_buttons[2].emit('clicked')
        controls.label.set_text('Recovered')
        assert editor.save_button.get_sensitive()
        editor.page_picker.set_selected(1-editor.page)
        assert editor.properties.get_visible_child_name()=='dial'
        assert controls.label.get_text()=='Recovered'
        # Let asynchronous device-page acknowledgement finish between native clicks.
        def settle_page():
            import time
            deadline=time.monotonic()+3
            while editor.pending and time.monotonic()<deadline:
                GLib.MainContext.default().iteration(False);time.sleep(.005)
            assert not editor.pending
        settle_page()
        # Page-specific settings inherit until explicitly customized.
        shared_label=controls.label.get_text();override_page=editor.page
        controls.scope_button.emit('clicked')
        assert controls.scope_button.get_label()=='Use shared settings'
        controls.label.set_text('Page Only')
        assert editor.draft.data['pages'][override_page]['dial_overrides'][2]['label']=='Page Only'
        editor.page_picker.set_selected(1-override_page)
        settle_page()
        assert controls.label.get_text()==shared_label
        controls.label.set_text('New Shared')
        editor.page_picker.set_selected(override_page)
        settle_page()
        assert controls.label.get_text()=='Page Only'
        controls.scope_button.emit('clicked')
        assert controls.label.get_text()=='New Shared'
        editor.undo_button.emit('clicked')
        assert controls.label.get_text()=='Page Only'
        editor.redo_button.emit('clicked')
        assert controls.label.get_text()=='New Shared'
        editor.reset(None)
        assert not editor.draft.dirty
        editor.keys[0].emit('clicked')
        editor.action_catalog.search.set_text('media next')
        editor.action_catalog.filter()
        visible=[b for b in editor.action_catalog.buttons if b.get_visible()]
        assert len(visible)==1 and 'Next' in visible[0].get_label()
        visible[0].emit('clicked')
        assert editor.draft.data['pages'][editor.page]['keys'][0]['action']['type']=='media_next'
        editor.reset(None)
        while editor.pending:GLib.MainContext.default().iteration(True)
        from concurrent.futures import Future
        measured=editor.device_preview.measure(editor_module.Gtk.Orientation.HORIZONTAL,-1)
        reply=Future();reply.set_result((bytes([30,40,50])*800*100,'live'))
        editor.touch_preview.finish(reply,editor.touch_preview.revision)
        assert editor.touch_preview.picture.get_paintable() is not None
        assert editor.device_preview.measure(editor_module.Gtk.Orientation.HORIZONTAL,-1)==measured, (measured,editor.device_preview.measure(editor_module.Gtk.Orientation.HORIZONTAL,-1),editor.touch_preview.measure(editor_module.Gtk.Orientation.HORIZONTAL,-1))
        assert not editor.touch_preview.message.get_visible()
        editor.touch_buttons[3].emit('clicked')
        assert editor.dial_controls.index==3
        assert not editor.draft.dirty, 'Image hit regions only select; they must not edit'
        editor.dial_controls.label.set_text('Preview Edit')
        assert 'Preview Edit' in editor.touch_preview.desired[0]
        editor.touch_buttons[0].emit('clicked')
        assert editor.draft.data['dials'][3]['label']=='Preview Edit'
        editor.reset(None)
        print('PASS: rendered touch hit regions, draft requests and existing editor interactions', flush=True)
    except BaseException as error:
        errors.append(error)
    app.quit()
    return False

def activate(_app):
    global editor
    source = (Path(__file__).resolve().parents[2]/'config/audio.json').read_text()
    editor = Editor(app, lambda method, *_args: (json.dumps({'layout':'audio', 'active_page':1, 'display_ready':True}) if method == 'GetStatus' else source,))
    GLib.timeout_add(500, exercise)

app.connect('activate', activate)
app.run(None)
if errors:
    raise errors[0]
