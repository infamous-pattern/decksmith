import json,unittest
from pathlib import Path
from editor_model import Draft

class HistoryBudgetTests(unittest.TestCase):
    def test_large_artwork_history_is_bounded_and_restores_exact_bytes(self):
        data=json.loads((Path(__file__).resolve().parents[2]/'config/audio.json').read_text())
        pixels=list(bytes(range(256))*256)
        data['pages'][0]['keys'][0]['icon_png']=pixels
        d=Draft(data);d.HISTORY_BYTES=20000
        for i in range(70):
            d.data['pages'][0]['keys'][0]['label']=f'Edit {i}';d.checkpoint()
        self.assertLessEqual(sum(map(len,d.undo_stack))+sum(map(len,d.redo_stack)),d.HISTORY_BYTES)
        self.assertTrue(d.travel());self.assertEqual(d.data['pages'][0]['keys'][0]['label'],'Edit 68')
        self.assertEqual(d.data['pages'][0]['keys'][0]['icon_png'],pixels)
        self.assertTrue(d.travel(True));self.assertEqual(d.data['pages'][0]['keys'][0]['label'],'Edit 69')
