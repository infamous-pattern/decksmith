import unittest
from concurrent.futures import Future
from types import SimpleNamespace
from unittest.mock import patch
from io import BytesIO
from PIL import Image
from touch_preview import TouchPreview

class Label:
    def set_text(self,text):self.text=text
    def set_visible(self,value):self.visible=value
class Picture:
    def set_paintable(self,value):self.image=value
class Pool:
    def __init__(self):self.calls=0;self.future=Future()
    def submit(self,*_):self.calls+=1;return self.future

class TouchPreviewTests(unittest.TestCase):
    def fixture(self):
        p=SimpleNamespace(closed=False,busy=False,invalid=False,desired=('{}',0),revision=1,rgb=None,
                          picture=Picture(),message=Label(),note=Label(),pool=Pool(),call=lambda *_:None,get_mapped=lambda:True)
        p.finish=lambda *args:TouchPreview.finish(p,*args)
        return p
    def test_texture_preserves_all_source_pixels(self):
        p=self.fixture();rgb=bytes([30,55,90,255,230,110])*40000
        future=Future();future.set_result((rgb,'live'))
        TouchPreview.finish(p,future,1)
        png=p.picture.image.save_to_png_bytes().get_data()
        self.assertEqual(Image.open(BytesIO(png)).convert('RGB').tobytes(),rgb)
        self.assertFalse(p.message.visible)
    def test_stale_result_cannot_overwrite_new_draft(self):
        p=self.fixture();p.revision=2
        future=Future();future.set_result((bytes(240000),'live'))
        TouchPreview.finish(p,future,1)
        self.assertIsNone(p.rgb)
    def test_poll_coalesces_and_does_not_touch_editor(self):
        p=self.fixture()
        with patch('touch_preview.GLib.idle_add',side_effect=lambda fn,*args:fn(*args)):
            TouchPreview.refresh(p);TouchPreview.refresh(p)
            self.assertEqual(p.pool.calls,1)
            p.pool.future.set_result((bytes(240000),'draft'))
            self.assertFalse(p.busy)
    def test_error_preserves_image_with_paused_notice_and_recovers(self):
        p=self.fixture();rgb=bytes([12,24,36])*80000
        good=Future();good.set_result((rgb,'live'));TouchPreview.finish(p,good,1)
        texture=p.picture.image
        bad=Future();bad.set_exception(RuntimeError('offline'))
        TouchPreview.finish(p,bad,1)
        self.assertIs(p.picture.image,texture);self.assertEqual(p.rgb,rgb)
        self.assertIn('paused',p.note.text);self.assertFalse(p.message.visible)
        TouchPreview.finish(p,good,1)
        self.assertNotIn('paused',p.note.text)
    def test_invalid_action_keeps_last_valid_layout_live_and_recovers(self):
        p=self.fixture();TouchPreview.invalidate(p)
        self.assertEqual(p.desired,('{}',0));self.assertEqual(p.revision,1)
        self.assertTrue(p.invalid)
        good=Future();good.set_result((bytes(240000),'draft'))
        TouchPreview.finish(p,good,1)
        self.assertIsNotNone(p.rgb);self.assertIn('last valid layout',p.note.text)
        TouchPreview.request(p,'{"valid":true}',1)
        self.assertFalse(p.invalid);self.assertEqual(p.revision,2)
        TouchPreview.finish(p,good,1)
        self.assertNotIn('current image',p.note.text)
        TouchPreview.finish(p,good,2)
        self.assertNotIn('last valid',p.note.text)
    def test_error_without_any_image_still_reports_unavailable(self):
        p=self.fixture();bad=Future();bad.set_exception(RuntimeError('offline'))
        TouchPreview.finish(p,bad,1)
        self.assertIsNone(p.rgb);self.assertTrue(p.message.visible)
        self.assertIn('unavailable',p.note.text)

    def test_clear_key_then_incomplete_website_keeps_preview_without_allowing_apply(self):
        import json
        from editor_model import Draft
        draft=Draft({'version':1,'audio_dial':True,'pages':[{'name':'HOME','background':[1,2,3],
                    'keys':[{'label':'Key','action':{'type':'none'}} for _ in range(8)]}]})
        draft.clear_key(0,0);draft.validate()
        p=self.fixture();TouchPreview.request(p,json.dumps(draft.data),0)
        good=Future();good.set_result((bytes([30,55,90])*80000,'draft'))
        TouchPreview.finish(p,good,p.revision);texture=p.picture.image
        draft.data['pages'][0]['keys'][0]['action']={'type':'open_website','url':''}
        with self.assertRaises(ValueError):draft.validate()
        TouchPreview.invalidate(p)
        self.assertIs(p.picture.image,texture);self.assertTrue(p.invalid)
        self.assertNotIn('open_website',p.desired[0])
        draft.data['pages'][0]['keys'][0]['action']['url']='https://example.com'
        draft.validate();TouchPreview.request(p,json.dumps(draft.data),0)
        TouchPreview.finish(p,good,p.revision)
        self.assertFalse(p.invalid);self.assertNotIn('last valid',p.note.text)

class PreviewTransportTests(__import__('unittest').TestCase):
    def test_image_transport_keeps_packed_bytes(self):
        from unittest.mock import patch,Mock
        import panel
        pixels=bytes([13,25,47])*800*100
        reply=panel.GLib.Variant('(ays)',(pixels,'live'))
        bus=Mock();bus.call_sync.return_value=reply
        with patch.object(panel.Gio,'bus_get_sync',return_value=bus):
            result=panel.call('PreviewTouch')
        self.assertIsInstance(result[0],bytes)
        self.assertEqual(result,(pixels,'live'))
