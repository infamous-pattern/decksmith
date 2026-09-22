import json,unittest
from pathlib import Path
from io import BytesIO
from zipfile import ZipFile,ZIP_DEFLATED
from layout_package import encode,decode

class PackageTests(unittest.TestCase):
    def sample(self):return json.loads((Path(__file__).resolve().parents[2]/'config/audio.json').read_text())
    def test_roundtrip_preserves_layout_and_embedded_assets(self):
        source=self.sample();source['pages'][0]['keys'][5]['icon_png']=[1,2,3];source['pages'][0]['keys'][5]['label_position']='bottom';source['pages'][0]['keys'][5]['label']='YouTube';source['pages'][0]['keys'][5]['label_color']='yellow';source['pages'][0]['keys'][5]['label_background']='transparent'
        source['pages'][1]['name']='Joplin Notes 2';source['pages'][1]['default']=True;source['pages'][1]['application']='joplin.desktop'
        data,removed=encode(source)
        self.assertEqual(removed,0);self.assertEqual(decode(data),source)
        with ZipFile(BytesIO(data)) as archive:
            self.assertIn('behavior.json',archive.namelist());self.assertIn('appearance.json',archive.namelist())
    def test_export_sanitizes_url_without_modifying_source(self):
        source=self.sample();action={'type':'open_website','url':'https://user:secret@example.com/path?token=private#secret'}
        source['pages'][0]['keys'][5]['action']=action
        data,removed=encode(source)
        self.assertEqual(removed,1)
        self.assertEqual(decode(data)['pages'][0]['keys'][5]['action']['url'],'https://example.com/path')
        self.assertIn('private',action['url'])
    def test_modified_content_rejected(self):
        data,_=encode(self.sample());output=BytesIO()
        with ZipFile(BytesIO(data)) as source, ZipFile(output,'w',ZIP_DEFLATED) as target:
            for name in source.namelist():target.writestr(name,b'{}' if name=='behavior.json' else source.read(name))
        with self.assertRaises(ValueError):decode(output.getvalue())
    def test_invalid_archive_rejected(self):
        with self.assertRaises(Exception):decode(b'not a zip')
