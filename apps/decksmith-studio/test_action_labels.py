import unittest
from action_labels import LABELS, default_label, uses_default
from media_artwork import populate

class ActionLabelTests(unittest.TestCase):
    def test_empty_and_previous_default_change_but_custom_text_does_not(self):
        for label in ('EMPTY','Empty',''):
            self.assertTrue(uses_default(label,{'type':'none'},[]))
        self.assertTrue(uses_default('Volume Up',{'type':'volume_up'},[]))
        self.assertFalse(uses_default('Speakers',{'type':'volume_up'},[]))
        self.assertTrue(uses_default('Empty',{'type':'volume_up'},[]))
    def test_every_default_fits_key_validation(self):
        for label in LABELS.values():
            self.assertTrue(0<len(label)<=24)
            self.assertTrue(all(c.isascii() and (c.isalnum() or c==' ') for c in label))
        self.assertEqual(default_label({'type':'go_to_page','page':0},[{'name':'Joplin Notes 2'}]),'Joplin Notes 2')
    def test_media_artwork_does_not_overwrite_custom_label(self):
        key={'label':'My Music','action':{'type':'media_next'}}
        populate(key,preserve_label=True)
        self.assertEqual(key['label'],'My Music');self.assertTrue(key['icon_png'])

    def test_application_names_fit_device_and_keep_identity(self):
        from action_labels import application_label
        self.assertEqual(application_label('Brave Web Browser'),'Brave Web Browser')
        self.assertEqual(application_label('Café — Player'),'Cafe Player')
        self.assertLessEqual(len(application_label('An application with an extremely long name')),24)
