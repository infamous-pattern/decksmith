import json
import unittest
from pathlib import Path
from copy import deepcopy
from editor_model import Draft
from plugin_bindings import validate, test_page


class Bindings(unittest.TestCase):
    def setUp(self):
        self.original=json.loads((Path(__file__).resolve().parents[2]/'config/navigation.json').read_text())

    def test_page_preserves_original_pages_and_shared_dials(self):
        staged=test_page(self.original)
        self.assertEqual(staged['pages'][:-1],self.original['pages'])
        self.assertEqual(staged.get('dials'),self.original.get('dials'))
        draft=Draft(staged);draft.validate()
        self.assertEqual(staged['pages'][-1]['name'],'HB TEST')
        with self.assertRaises(ValueError):test_page(staged)

    def test_missing_plugin_settings_survive_history_copy_and_export(self):
        draft=Draft(test_page(self.original));page=len(draft.data['pages'])-1
        binding=draft.data['pages'][page]['keys'][0]['plugin']
        binding['provider']='future.provider';binding['schema']=27
        binding['settings']['future']={'array':[True,39,'custom']}
        expected=deepcopy(binding)
        draft.saved_now();draft.copy_key(page,0);draft.paste_key(page,2);draft.checkpoint()
        self.assertEqual(draft.data['pages'][page]['keys'][2]['plugin'],expected)
        draft.travel();draft.travel(True);draft.validate()
        loaded=Draft(json.loads(json.dumps(draft.data)));loaded.validate()
        self.assertEqual(loaded.data['pages'][page]['keys'][2]['plugin'],expected)

    def test_bounds_and_conflicting_builtin(self):
        staged=test_page(self.original);key=staged['pages'][-1]['keys'][0]
        key['action']={'type':'volume_up'}
        with self.assertRaises(ValueError):Draft(staged).validate()
        binding=key['plugin'];binding['settings']={'big':'x'*4096}
        with self.assertRaises(ValueError):validate(binding)
        binding['settings']={};binding['schema']=True
        with self.assertRaises(ValueError):validate(binding)
