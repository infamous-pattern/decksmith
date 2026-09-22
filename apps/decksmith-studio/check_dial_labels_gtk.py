"""Native dial assignment regression; fixture only, no daemon or layout writes."""
import json
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw
from dials import DialControls
from audio_target_picker import AudioTargetPicker

Gtk.init(); Adw.init()
layout = json.loads((Path(__file__).resolve().parents[2] / 'config/audio.json').read_text())
layout['dials'] = [{'label':'Brightness','rotation':'brightness','step':1,
                    'press':{'type':'none'}} for _ in range(4)]
owner = SimpleNamespace(draft=SimpleNamespace(data=layout), page=0, render=lambda: None)
with patch.object(AudioTargetPicker, 'refresh'), patch('applications.audio_icon_png', return_value=None):
    controls = DialControls(owner, embedded=True)
    controls.picker.set_selected(3)
    assert controls.target.value() == 'system'
    controls.rotation.set_selected(1)
    assert controls.label.get_text() == 'System sounds'
    assert layout['dials'][3]['label'] == 'System sounds'
    assert layout['dials'][3]['rotation'] == 'volume'
    controls.label.set_text('My Alerts')
    controls.target.select('system')  # Inventory sync must keep custom text.
    controls.select()                # Revisiting this dial must keep it too.
    assert controls.label.get_text() == 'My Alerts'
    controls.target.select('microphone')
    controls.rotation.set_selected(2)
    assert controls.label.get_text() == 'Brightness'
    controls.rotation.set_selected(1)
    assert controls.label.get_text() == 'Default microphone'
    assert layout['dials'][3]['audio_target'] == 'microphone'
print('PASS: Brightness to preselected audio target updates label; custom text survives refresh/reselection')
