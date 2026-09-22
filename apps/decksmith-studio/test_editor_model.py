import unittest
from editor_model import Draft

class DraftTests(unittest.TestCase):
    def sample(self):
        return {'version': 1, 'audio_dial': True, 'pages': [{'name': 'HOME', 'background': [1,2,3],
            'keys': [{'label': 'HOME', 'action': {'type':'go_to_page','page':0}} for _ in range(8)]}]}
    def test_page_name_case_digits_history_and_persistence(self):
        import json
        source=self.sample();draft=Draft(source)
        draft.rename_page(0,'Joplin Notes 2');draft.checkpoint();draft.validate()
        self.assertEqual(draft.data['pages'][0]['name'],'Joplin Notes 2')
        self.assertEqual(draft.data['pages'][0]['keys'][0]['label'],'Joplin Notes 2')
        self.assertEqual(source['pages'][0]['name'],'HOME')
        self.assertTrue(draft.travel());self.assertEqual(draft.data['pages'][0]['name'],'HOME')
        self.assertTrue(draft.travel(True))
        restored=Draft(json.loads(json.dumps(draft.data)));restored.validate()
        self.assertEqual(restored.data['pages'][0]['name'],'Joplin Notes 2')
        for name in ('','   ','Bad!','1'*25):
            restored.rename_page(0,name)
            with self.assertRaisesRegex(ValueError,'Page names'):restored.validate()

    def test_default_selection_undo_reorder_duplicate_delete_and_roundtrip(self):
        import json
        draft=Draft(self.sample());draft.add_page();draft.checkpoint()
        draft.set_default_page(1);draft.checkpoint();self.assertEqual(draft.default_page(),1)
        self.assertTrue(draft.travel());self.assertEqual(draft.default_page(),0)
        self.assertTrue(draft.travel(True));self.assertEqual(draft.default_page(),1)
        restored=Draft(json.loads(json.dumps(draft.data)));self.assertEqual(restored.default_page(),1)
        restored.duplicate_page(1);self.assertFalse(restored.data['pages'][2].get('default',False))
        restored.move_page(1,0);self.assertEqual(restored.default_page(),0)
        restored.delete_page(0,clear_links=True);self.assertEqual(restored.default_page(),0)
        self.assertEqual(sum(bool(p.get('default')) for p in restored.data['pages']),1)
        restored.validate();restored.data['pages'][1]['default']=True
        with self.assertRaisesRegex(ValueError,'one default'):restored.validate()

    def test_application_assignment_follows_page_and_duplicate_is_unassigned(self):
        draft=Draft(self.sample());draft.data['pages'][0]['application']='brave-browser.desktop'
        copy=draft.duplicate_page(0);self.assertNotIn('application',draft.data['pages'][copy])
        draft.move_page(0,1);self.assertEqual(draft.data['pages'][1]['application'],'brave-browser.desktop')
        draft.validate();draft.data['pages'][0]['application']='brave-browser.desktop'
        with self.assertRaisesRegex(ValueError,'only one page'):draft.validate()
        draft.reset();self.assertNotIn('application',draft.data['pages'][0])

    def test_draft_changes_do_not_mutate_source_and_reset_restores_it(self):
        source = self.sample()
        draft = Draft(source)
        draft.data['pages'][0]['keys'][0]['label'] = 'MUTE'
        self.assertEqual(source['pages'][0]['keys'][0]['label'], 'HOME')
        self.assertTrue(draft.dirty)
        draft.reset()
        self.assertFalse(draft.dirty)
    def test_adding_and_renaming_page_preserves_numeric_destinations(self):
        draft = Draft(self.sample())
        index = draft.add_page()
        draft.data['pages'][0]['keys'][0]['action']['page'] = index
        draft.data['pages'][index]['name'] = 'WORK'
        draft.validate()
        self.assertEqual(draft.data['pages'][0]['keys'][0]['action']['page'], 1)
        self.assertTrue(all(k['action']['type']=='none' for k in draft.data['pages'][1]['keys']))
    def test_rename_updates_matching_labels_across_pages_and_discard_restores(self):
        draft = Draft(self.sample())
        draft.add_page()
        keys = draft.data['pages'][1]['keys']
        keys[0] = {'label':'HOME', 'action':{'type':'go_to_page','page':0}}
        keys[1] = {'label':'HOME', 'action':{'type':'none'}}
        keys[2] = {'label':'BACK', 'action':{'type':'go_to_page','page':0}}
        draft.saved_now()
        draft.rename_page(0, 'DESK')
        self.assertEqual(draft.data['pages'][0]['keys'][0]['label'], 'DESK')
        self.assertEqual([k['label'] for k in keys[:3]], ['DESK','DESK','BACK'])
        self.assertEqual(keys[0]['action'], {'type':'go_to_page','page':0})
        draft.validate()
        draft.reset()
        self.assertEqual(draft.data['pages'][1]['keys'][0]['label'], 'HOME')

    def test_linked_label_recovers_stale_text_and_survives_rename(self):
        source = self.sample()
        source['pages'][0]['name'] = 'TUESDAYS'
        source['pages'][0]['keys'][1]['label'] = 'WORK'
        source['pages'][0]['keys'][1]['follow_page_name'] = True
        draft = Draft(source)
        self.assertEqual(draft.data['pages'][0]['keys'][1]['label'], 'TUESDAYS')
        draft.rename_page(0, 'FRIDAY')
        self.assertEqual(draft.data['pages'][0]['keys'][1]['label'], 'FRIDAY')

    def test_page_names_and_labels_allow_digits_and_spaces(self):
        draft = Draft(self.sample())
        draft.rename_page(0, 'PAGE 1')
        draft.validate()
        self.assertEqual(draft.data['pages'][0]['keys'][0]['label'], 'PAGE 1')

    def test_reorder_preserves_targets_and_device_mapping(self):
        draft = Draft(self.sample())
        draft.add_page()
        draft.saved_now()
        draft.move_page(0,1)
        self.assertEqual(draft.origins, [1,0])
        self.assertEqual(draft.data['pages'][1]['keys'][0]['action']['page'],1)
        draft.reset()
        self.assertEqual(draft.origins,[0,1])
        self.assertEqual(draft.data['pages'][0]['keys'][0]['action']['page'],0)

    def test_duplicate_is_independent_and_self_links_follow_copy(self):
        draft = Draft(self.sample())
        index = draft.duplicate_page(0)
        self.assertEqual(index,1)
        key = draft.data['pages'][1]['keys'][0]
        self.assertEqual(key['action']['page'],1)
        self.assertEqual(key['label'],'COPY 1')
        key['label']='EDIT'
        self.assertEqual(draft.data['pages'][0]['keys'][0]['label'],'HOME')
        self.assertEqual(draft.origins,[0,None])

    def test_delete_checks_links_and_remaps_remaining_destinations(self):
        draft = Draft(self.sample())
        draft.add_page()
        draft.add_page()
        draft.data['pages'][2]['keys'][0]['action']={'type':'go_to_page','page':0}
        draft.data['pages'][2]['keys'][1]['action']={'type':'go_to_page','page':2}
        with self.assertRaises(ValueError): draft.delete_page(0)
        draft.delete_page(0,clear_links=True)
        self.assertEqual(draft.data['pages'][1]['keys'][0]['action'],{'type':'none'})
        self.assertEqual(draft.data['pages'][1]['keys'][1]['action']['page'],1)
        draft.delete_page(0,clear_links=True)
        with self.assertRaises(ValueError): draft.delete_page(0,clear_links=True)
        draft.validate()

    def test_invalid_draft_cannot_be_saved_and_success_clears_dirty(self):
        draft = Draft(self.sample())
        draft.data['pages'][0]['keys'][0]['label'] = 'bad!'
        with self.assertRaises(ValueError): draft.validate()
        draft.reset()
        draft.data['pages'][0]['keys'][0]['action'] = {'type':'volume_adjust','percent':0}
        with self.assertRaises(ValueError): draft.validate()
        draft.data['pages'][0]['keys'][0]['action']['percent'] = -2
        draft.validate()
        draft.saved_now()
        self.assertFalse(draft.dirty)

    def test_copied_navigation_tracks_reorder_and_rename_but_rejects_deleted_target(self):
        draft=Draft(self.sample())
        draft.add_page()
        draft.copy_key(0,0)
        draft.move_page(0,1)
        draft.rename_page(1,'NEW HOME')
        draft.paste_key(0,3)
        self.assertEqual(draft.data['pages'][0]['keys'][3]['action'],{'type':'go_to_page','page':1})
        self.assertEqual(draft.data['pages'][0]['keys'][3]['label'],'NEW HOME')
        draft.delete_page(1,clear_links=True)
        with self.assertRaises(ValueError):draft.paste_key(0,4)

    def test_history_restores_pages_and_invalidates_redo_after_edit(self):
        draft=Draft(self.sample())
        original=draft.data.copy()
        draft.add_page();draft.checkpoint()
        draft.clear_key(0,0);draft.checkpoint()
        self.assertTrue(draft.travel())
        self.assertEqual(draft.data['pages'][0]['keys'][0]['action']['type'],'go_to_page')
        self.assertTrue(draft.travel());self.assertEqual(len(draft.data['pages']),1)
        self.assertTrue(draft.travel(True));self.assertEqual(len(draft.data['pages']),2)
        draft.rename_page(1,'NEW');draft.checkpoint()
        self.assertFalse(draft.travel(True))
        draft.reset();self.assertFalse(draft.undo_stack)

if __name__ == '__main__': unittest.main()
