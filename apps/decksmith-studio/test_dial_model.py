import json,unittest
from pathlib import Path
from copy import deepcopy
from dial_model import defaults,effective,customize,revert,store
from editor_model import Draft
from layout_package import encode,decode

class DialScopeTests(unittest.TestCase):
    def sample(self):return json.loads((Path(__file__).resolve().parents[2]/'config/audio.json').read_text())
    def test_shared_edits_only_reach_inheriting_pages_and_revert_uses_latest(self):
        layout=self.sample();layout['dials']=defaults(layout);before=deepcopy(layout)
        customize(layout,1,2);dial=effective(layout,1)[2];dial['label']='Local';store(layout,1,2,dial)
        shared=effective(layout,0)[2];shared['label']='Shared';store(layout,0,2,shared)
        self.assertEqual(effective(layout,1)[2]['label'],'Local')
        self.assertEqual(effective(layout,0)[2]['label'],'Shared')
        self.assertEqual(effective(layout,1)[0],before['dials'][0])
        revert(layout,1,2);self.assertEqual(effective(layout,1)[2]['label'],'Shared')
        self.assertNotIn('dial_overrides',layout['pages'][1])
    def test_history_duplicate_reorder_and_package_preserve_scope(self):
        draft=Draft(self.sample());customize(draft.data,1,0);draft.checkpoint()
        self.assertTrue(draft.travel());self.assertNotIn('dial_overrides',draft.data['pages'][1])
        self.assertTrue(draft.travel(redo=True));self.assertIsNotNone(draft.data['pages'][1]['dial_overrides'][0])
        self.assertEqual(decode(encode(draft.data)[0]),draft.data)
        copy=draft.duplicate_page(1);self.assertEqual(draft.data['pages'][copy]['dial_overrides'],draft.data['pages'][1]['dial_overrides'])
        draft.move_page(copy,0);self.assertIsNotNone(draft.data['pages'][0]['dial_overrides'][0])
    def test_invalid_hidden_override_is_validated(self):
        draft=Draft(self.sample());customize(draft.data,1,3)
        draft.data['pages'][1]['dial_overrides'][3]['label']=''
        with self.assertRaises(ValueError):draft.validate()
        draft.data['pages'][1]['dial_overrides']=[None]
        with self.assertRaises(ValueError):draft.validate()
    def test_app_icon_survives_overrides_history_and_export(self):
        import struct,zlib
        def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
        png=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',32,32,8,6,0,0,0))+chunk(b'IDAT',zlib.compress((b'\0'+bytes([20,120,200,255])*32)*32))+chunk(b'IEND',b'')
        draft=Draft(self.sample());dial=effective(draft.data,0)[0]
        dial['audio_target']='app:application.process.binary=brave';dial['target_icon_png']=list(png)
        store(draft.data,0,0,dial);draft.checkpoint();draft.validate()
        customize(draft.data,1,0);draft.checkpoint()
        self.assertEqual(decode(encode(draft.data)[0]),draft.data)
        self.assertTrue(draft.travel());self.assertEqual(effective(draft.data,0)[0]['target_icon_png'],list(png))
        draft.data['dials'][0]['target_icon_png']=[1,2,3]
        with self.assertRaises(ValueError):draft.validate()
    def test_legacy_defaults_do_not_mutate_source(self):
        data=self.sample();original=deepcopy(data)
        effective(data,1)[0]['label']='changed'
        self.assertEqual(data,original)
