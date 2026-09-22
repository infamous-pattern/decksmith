import unittest,tempfile,json
from pathlib import Path
from io import BytesIO
from PIL import Image
from icon_library import Library,ROOT,svg_png
from layout_package import encode,decode
class IconLibraryTests(unittest.TestCase):
    def test_catalog_checksums_render_and_search_offline(self):
        with tempfile.TemporaryDirectory() as root:
            library=Library(root);items=library.items();self.assertEqual(len(items),134)
            for item in items:
                image=Image.open(BytesIO(library.image(item)))
                self.assertEqual(image.size,(120,120));self.assertIn('A',image.getbands())
            self.assertTrue(library.items('microphone'));self.assertTrue(library.items('github'))
            self.assertTrue(all(i['category']=='Development and apps' for i in library.items(category='Development and apps')))
    def test_import_deduplicates_normalized_assets_and_preserves_pixels(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root);source=root/'My icon.png';Image.new('RGBA',(400,400),(240,10,20,180)).save(source)
            library=Library(root/'store');a=library.import_file(source);b=library.import_file(source)
            self.assertEqual(a['id'],b['id']);self.assertEqual(len(list((root/'store').glob('*.png'))),1)
            self.assertTrue(all(abs(a-b)<=1 for a,b in zip(Image.open(BytesIO(library.image(a))).getpixel((60,60)),(240,10,20,180))))
            self.assertEqual(len(library.items('My icon')),1)
    def test_svg_external_content_is_rejected(self):
        for raw in (b'<svg><script/></svg>',b'<svg><image href="file:///etc/passwd"/></svg>',b'<!DOCTYPE svg><svg/>',b'<svg><path style="fill:url(https://example.com)"/></svg>'):
            with self.assertRaises(ValueError):svg_png(raw)
    def test_tabler_asset_package_keeps_license_and_provenance(self):
        from zipfile import ZipFile
        with tempfile.TemporaryDirectory() as root:
            library=Library(root);item=library.items('terminal')[0]
            layout=json.loads((Path(__file__).resolve().parents[2]/'config/audio.json').read_text());key=layout['pages'][0]['keys'][0]
            key.update(icon_source=item['id'],icon_tint=True,icon_png=list(library.image(item)),artwork='application_icon')
            data,_=encode(layout);self.assertEqual(decode(data),layout)
            with ZipFile(BytesIO(data)) as z:self.assertIn((ROOT/'LICENSE').read_bytes(),z.read('asset-notices.txt'))
