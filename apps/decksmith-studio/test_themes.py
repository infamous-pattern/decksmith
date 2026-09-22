import unittest,json
from copy import deepcopy
from pathlib import Path
from io import BytesIO
from zipfile import ZipFile
from editor_model import Draft
from themes import PRESETS,effective,reset,set_theme,validate,encode_theme,decode_theme
from layout_package import encode,decode

class ThemeTests(unittest.TestCase):
    def sample(self):return json.loads((Path(__file__).resolve().parents[2]/'config/audio.json').read_text())
    def test_presets_and_resets_preserve_every_action_and_caption(self):
        layout=self.sample();before=deepcopy(layout)
        for preset in PRESETS:
            set_theme(layout,preset)
            for p in layout['pages']:
                for key in p['keys']:
                    key['appearance']={'font':'mono','label_color':'yellow'};reset(key)
            validate(layout)
            self.assertEqual([[k['action'] for k in p['keys']] for p in layout['pages']],[[k['action'] for k in p['keys']] for p in before['pages']])
            self.assertEqual([[k['label'] for k in p['keys']] for p in layout['pages']],[[k['label'] for k in p['keys']] for p in before['pages']])
    def test_legacy_customizations_override_theme_until_explicit_reset(self):
        layout=self.sample();key=layout['pages'][0]['keys'][0];key.update(label_color='yellow',background_color='blue')
        set_theme(layout,'light');self.assertEqual(effective(layout,key)['label_color'],'yellow')
        reset(key);self.assertEqual(effective(layout,key)['color'],[24,30,38])
        self.assertEqual(effective(layout,key)['background'],[236,240,245])
    def test_style_edits_history_and_import_export_keep_actions(self):
        draft=Draft(self.sample());set_theme(draft.data,'dark');draft.checkpoint()
        draft.data['theme']['appearance']={'font':'serif','size':'large'};draft.checkpoint()
        self.assertTrue(draft.travel());self.assertEqual(draft.data['theme']['appearance'],{})
        self.assertTrue(draft.travel(True));self.assertEqual(draft.data['theme']['appearance']['font'],'serif')
        draft.data['dials']=[{'label':f'Dial {i}','rotation':'none','press':{'type':'none'},'step':1,'appearance':{'font':'mono'}} for i in range(4)]
        draft.data['pages'][1]['dial_overrides']=[deepcopy(draft.data['dials'][0]),None,None,None]
        data,_=encode(draft.data);self.assertEqual(decode(data),draft.data)
        with ZipFile(BytesIO(data)) as z:
            behavior=json.loads(z.read('behavior.json'))
            self.assertNotIn('theme',behavior);self.assertNotIn('appearance',behavior['dials'][0]);self.assertNotIn('appearance',behavior['pages'][1]['dial_overrides'][0])
    def test_unknown_fields_cannot_smuggle_actions_into_theme(self):
        layout=self.sample();layout['theme']={'preset':'dark','appearance':{'action':'mute'}}
        with self.assertRaises(ValueError):validate(layout)

    def test_standalone_theme_roundtrip_and_reject_behavior(self):
        theme={'preset':'light','appearance':{'font':'mono','size':'large'}}
        self.assertEqual(decode_theme(encode_theme(theme)),theme)
        payload=json.loads(encode_theme(theme));payload['actions']=[{'type':'mute'}]
        with self.assertRaises(ValueError):decode_theme(json.dumps(payload).encode())
        for raw in (b'[]',b'null',b'{}',b'x'*65537):
            with self.assertRaises(ValueError):decode_theme(raw)
        payload.pop('actions');payload['theme']['appearance']['font']=[]
        with self.assertRaises(ValueError):decode_theme(json.dumps(payload).encode())

    def test_icon_size_steps_reset_history_and_packages(self):
        draft=Draft(self.sample());set_theme(draft.data,'dark');draft.data['theme']['appearance']['icon_size']=90
        key=draft.data['pages'][0]['keys'][0];key['appearance']={'icon_size':50};draft.checkpoint()
        self.assertEqual(effective(draft.data,key)['icon_size'],50)
        reset(key);draft.checkpoint();self.assertEqual(effective(draft.data,key)['icon_size'],90)
        self.assertTrue(draft.travel());self.assertEqual(draft.data['pages'][0]['keys'][0]['appearance']['icon_size'],50)
        packed,_=encode(draft.data);self.assertEqual(decode(packed),draft.data)
        for value in (9,11,99,101,'50',True,50.0):
            draft.data['pages'][0]['keys'][0]['appearance']['icon_size']=value
            with self.assertRaises(ValueError):validate(draft.data)
