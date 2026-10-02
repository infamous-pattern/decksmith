import unittest
from text_labels import generated_label, valid_label


class TextLabelTests(unittest.TestCase):
    def test_accepts_emoji_sequences_and_unicode_names(self):
        for text in ('🔊', '👩‍💻 Build', '🇺🇸', '🎚️ Studio', 'Café'):
            self.assertTrue(valid_label(text), text)

    def test_rejects_empty_control_and_overlong_labels(self):
        for text in ('', '   ', 'Bad\nLabel', 'x' * 25, '😀' * 25):
            self.assertFalse(valid_label(text), repr(text))

    def test_generated_defaults_keep_emoji_and_remain_bounded(self):
        self.assertEqual(generated_label('🔊 — Living Room'), '🔊 Living Room')
        self.assertEqual(generated_label('🎥 Main LED’s'), '🎥 Main LED s')
        self.assertLessEqual(len(generated_label('😀' * 30)), 24)


if __name__ == '__main__':
    unittest.main()
