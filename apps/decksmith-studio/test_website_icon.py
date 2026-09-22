import unittest
from unittest.mock import patch
from io import BytesIO
from PIL import Image
from website_icon import fetch
from artwork import render,dimensions

def png(size):
    output=BytesIO();Image.new('RGBA',size,'red').save(output,format='PNG');return output.getvalue()

class IconTests(unittest.TestCase):
    def test_prefers_large_declared_icon(self):
        large=png((144,144))
        html=b'<link rel="icon" href="/small.png" sizes="16x16"><link rel="icon" href="/large.png" sizes="144x144">'
        with patch('website_icon.read',side_effect=[(html,'https://example.com/'),(large,'https://example.com/large.png')]) as read:
            self.assertEqual(fetch('https://example.com/'),large)
            self.assertEqual(read.call_args.args[0],'https://example.com/large.png')
    def test_invalid_candidate_falls_back(self):
        large=png((128,128))
        with patch('website_icon.read',side_effect=[(b'<link rel="icon" href="/bad">','https://example.com/'),(b'not an image','https://example.com/bad'),(large,'https://example.com/favicon.ico')]):
            self.assertEqual(fetch('https://example.com/'),large)
    def test_failure_keeps_text_fallback(self):
        with patch('website_icon.read',side_effect=OSError):
            self.assertIsNone(fetch('https://example.com/'))
    def test_fit_preserves_proportions_and_crop_fills_square(self):
        data=png((400,200))
        fitted=Image.open(BytesIO(bytes(render(data,100,False,'#000000'))))
        cropped=Image.open(BytesIO(bytes(render(data,100,True,'#000000'))))
        self.assertEqual(fitted.size,(120,120))
        self.assertEqual(fitted.getpixel((60,15)),(0,0,0))
        self.assertEqual(cropped.getpixel((60,15)),(255,0,0))
    def test_ico_uses_largest_frame(self):
        output=BytesIO();Image.new('RGBA',(256,256),'red').save(output,format='ICO',sizes=[(16,16),(128,128),(256,256)])
        self.assertEqual(dimensions(output.getvalue()),(256,256))

class RedirectSecurityTests(unittest.TestCase):
    def test_redirects_cannot_change_to_non_web_schemes_or_include_credentials(self):
        from website_icon import WebRedirects
        from urllib.request import Request
        request=Request('https://example.com/icon')
        for url in ('file:///etc/passwd','ftp://example.com/icon','http://user:secret@example.com/icon','data:image/png;base64,AA=='):
            with self.subTest(url=url),self.assertRaises(ValueError):
                WebRedirects().redirect_request(request,None,302,'Found',{},url)
        redirected=WebRedirects().redirect_request(request,None,302,'Found',{},'https://example.org/icon.png')
        self.assertEqual(redirected.full_url,'https://example.org/icon.png')
