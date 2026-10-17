"""Behavioral regression coverage for replacing and sharing key assignments."""
from copy import deepcopy
import json,unittest
from pathlib import Path
from editor_model import Draft,KeyClipboard

ROOT=Path(__file__).resolve().parents[2]
def draft(model):return Draft(json.loads((ROOT/'config'/('mk2-default.json' if model=='mk2' else 'plus-default.json')).read_text()))
def binding():return {'provider':'com.infamous-pattern.openhomeb','action':'com.infamous-pattern.openhomeb.toggle','schema':2,'settings':{'accessoryId':'a'*64}}

class CrossDeviceKeys(unittest.TestCase):
    def test_homebridge_replaces_navigation_and_media_settings_on_both_models(self):
        for model in ('plus','mk2'):
            for kind in ('navigation','media'):
                with self.subTest(model=model,old=kind):
                    d=draft(model);key=d.data['pages'][0]['keys'][1]
                    if kind=='navigation':key.update(action={'type':'go_to_page','page':0},follow_page_name=True)
                    else:key.update(action={'type':'media_play_pause'},media_player='automatic')
                    key['appearance']={'font':'sans','size':'small'};key['background_color']='transparent'
                    d.saved_now();original=deepcopy(d.saved);b=binding()
                    d.assign_plugin_key(0,1,b,'Office light');d.checkpoint();d.validate()
                    self.assertTrue(d.dirty);self.assertEqual(d.saved,original)
                    self.assertNotIn('follow_page_name',key);self.assertNotIn('media_player',key)
                    self.assertEqual(key['appearance'],{'font':'sans','size':'small'})
                    self.assertEqual(key['background_color'],'transparent')
                    self.assertEqual(key['plugin'],b)
                    b['settings']['accessoryId']='b'*64;self.assertEqual(key['plugin']['settings']['accessoryId'],'a'*64)
                    d.travel();self.assertEqual(d.data,original)
                    d.travel(True);d.validate();self.assertEqual(d.data['pages'][0]['keys'][1]['plugin'],binding())

    def test_plugin_key_copies_both_directions_with_independent_drafts_and_history(self):
        for source_model,target_model in [('plus','mk2'),('mk2','plus')]:
            with self.subTest(source=source_model):
                a=draft(source_model);b=draft(target_model);clip=KeyClipboard()
                a.assign_plugin_key(0,1,binding(),'My light');a.data['pages'][0]['keys'][1]['appearance']={'icon_size':85}
                a.saved_now();before=deepcopy(b.data);clip.copy(a,0,1)
                a.data['pages'][0]['keys'][1]['label']='Changed after copy'
                last=len(b.data['pages'][0]['keys'])-1;clip.paste(b,0,last);b.checkpoint();b.validate()
                pasted=b.data['pages'][0]['keys'][last]
                self.assertEqual(pasted['label'],'My light');self.assertEqual(pasted['plugin'],binding())
                self.assertEqual(pasted['appearance'],{'icon_size':85})
                self.assertTrue(b.dirty);self.assertEqual(b.saved,before)
                pasted['plugin']['settings']['accessoryId']='c'*64
                self.assertEqual(a.data['pages'][0]['keys'][1]['plugin'],binding())
                self.assertTrue(b.travel())  # Undo the independent target edit.
                self.assertTrue(b.travel());self.assertEqual(b.data,before)
                self.assertTrue(clip.available)

    def test_cross_device_page_link_maps_by_name_not_source_page_number(self):
        a=draft('plus');b=draft('mk2');clip=KeyClipboard()
        a.data['pages'][0]['keys'][1].update(action={'type':'go_to_page','page':1},follow_page_name=True)
        target_name=a.data['pages'][1]['name'];b.data['pages'][0]['name']=target_name.swapcase()
        for p in b.data['pages'][1:]:
            if p['name'].casefold()==target_name.casefold():p['name']='Other page'
        clip.copy(a,0,1);clip.paste(b,0,1)
        self.assertEqual(b.data['pages'][0]['keys'][1]['action'],{'type':'go_to_page','page':0})
        self.assertEqual(b.data['pages'][0]['keys'][1]['label'],b.data['pages'][0]['name'])

    def test_missing_or_ambiguous_destination_leaves_target_unchanged(self):
        for ambiguous in (False,True):
            a=draft('plus');b=draft('mk2');clip=KeyClipboard()
            a.data['pages'][0]['name']='Source page';clip.copy(a,0,0)
            if ambiguous:
                b.data['pages'][0]['name']='Source page';b.data['pages'][1]['name']='SOURCE PAGE'
            before=deepcopy(b.data)
            with self.assertRaisesRegex(ValueError,'uniquely name'):clip.paste(b,0,1)
            self.assertEqual(b.data,before)

    def test_same_device_navigation_still_tracks_reorder_and_rename(self):
        d=draft('mk2');clip=KeyClipboard();clip.copy(d,0,13)
        d.move_page(0,1);d.rename_page(1,'Renamed');clip.paste(d,0,1)
        self.assertEqual(d.data['pages'][0]['keys'][1]['action']['page'],1)
        self.assertEqual(d.data['pages'][0]['keys'][1]['label'],'Renamed')
        d.delete_page(1,clear_links=True);before=deepcopy(d.data)
        with self.assertRaisesRegex(ValueError,'deleted'):clip.paste(d,0,1)
        self.assertEqual(d.data,before)

    def test_empty_window_clipboard_never_uses_an_old_draft_clipboard(self):
        d=draft('mk2');d.copy_key(0,13)
        with self.assertRaisesRegex(ValueError,'Copy a key first'):KeyClipboard().paste(d,0,1)

if __name__=='__main__':unittest.main()
