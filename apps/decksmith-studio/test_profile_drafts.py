import json,unittest
from pathlib import Path
from types import SimpleNamespace
from copy import deepcopy
from profile_drafts import DraftShelf
from editor_model import Draft

class DraftShelfTests(unittest.TestCase):
    def setUp(self):
        self.layout=json.loads((Path(__file__).resolve().parents[2]/'config/mk2-default.json').read_text());self.shelf=DraftShelf()
        self.editor=SimpleNamespace(draft=Draft(self.layout),page=1,key=14,control_kind='key',dial_index=0)
        self.editor.draft.data['pages'][1]['keys'][14]['label']='My edit 🎵';self.editor.draft.checkpoint()
    def test_switch_retains_original_draft_history_and_fifteenth_key(self):
        self.shelf.keep('one',self.editor,3);token={'revision':3}
        view=self.shelf.restore('one',self.layout,token)
        self.assertIs(view.draft,self.editor.draft);self.assertEqual(view.key,14);self.assertTrue(view.draft.undo_stack)
        self.assertIsNone(self.shelf.restore('two',self.layout,token))
    def test_remote_layout_change_retains_old_revision_instead_of_silently_rebasing(self):
        self.shelf.keep('one',self.editor,3);changed=deepcopy(self.layout);changed['pages'][0]['name']='Remote'
        token={'revision':4};self.shelf.restore('one',changed,token);self.assertEqual(token['revision'],3)
        self.assertEqual(self.editor.draft.data['pages'][1]['keys'][14]['label'],'My edit 🎵')
    def test_metadata_only_revision_can_use_new_context_with_same_saved_layout(self):
        self.shelf.keep('one',self.editor,3);token={'revision':4};self.shelf.restore('one',self.layout,token);self.assertEqual(token['revision'],4)
        self.editor.draft.saved_now();self.assertIsNone(self.shelf.restore('one',self.editor.draft.data,token))
    def test_bounded_shelf_never_evicts_another_unsaved_draft(self):
        for i in range(8):self.shelf.keep(str(i),self.editor,i)
        with self.assertRaises(ValueError):self.shelf.keep('ninth',self.editor,9)
        self.assertIs(self.shelf.views['0'].draft,self.editor.draft)

    def test_clean_layout_with_redo_history_survives_device_switch(self):
        self.editor.draft.travel();self.assertFalse(self.editor.draft.dirty);self.assertTrue(self.editor.draft.redo_stack)
        self.shelf.keep('one',self.editor,3);view=self.shelf.restore('one',self.layout,{'revision':3})
        self.assertIs(view.draft,self.editor.draft);view.draft.travel(True)
        self.assertEqual(view.draft.data['pages'][1]['keys'][14]['label'],'My edit 🎵')
