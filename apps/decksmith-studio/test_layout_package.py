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

    def mk2(self):return json.loads((Path(__file__).resolve().parents[2]/'config/mk2-default.json').read_text())
    def modify_manifest(self,data,edit):
        result=BytesIO()
        with ZipFile(BytesIO(data)) as source,ZipFile(result,'w',ZIP_DEFLATED) as target:
            for name in source.namelist():
                payload=source.read(name)
                if name=='manifest.json':
                    value=json.loads(payload);edit(value);payload=json.dumps(value).encode()
                target.writestr(name,payload)
        return result.getvalue()
    def test_v2_mk2_preserves_custom_artwork_labels_and_unavailable_provider(self):
        source=self.mk2();key=source['pages'][0]['keys'][7]
        key.update(label='🎵 Meine Musik',icon_png=[1,2,3],label_background='transparent',label_color='yellow')
        key['plugin']={'provider':'missing.provider','action':'future.action','schema':17,'settings':{'keep':[39,True,'Custom']}}
        data,_=encode(source);self.assertEqual(decode(data),source)
        with ZipFile(BytesIO(data)) as archive:
            manifest=json.loads(archive.read('manifest.json'))
            self.assertEqual(manifest['version'],2);self.assertEqual(manifest['model'],'mk2')
            self.assertEqual(manifest['requirements'],{'keys':15,'dials':0,'touch':False})
            self.assertNotIn('serial',manifest);self.assertNotIn('device_id',manifest)
    def test_model_mismatches_and_future_versions_are_rejected_without_truncation(self):
        from copy import deepcopy
        base=self.mk2()
        for edit in [lambda x:x.update(version=99),lambda x:x.update(model='xl'),lambda x:x['pages'][0]['keys'].pop(),
                     lambda x:x.update(dials=[]),lambda x:x.update(audio_dial=True),lambda x:x.update(serial='private')]:
            source=deepcopy(base);edit(source)
            with self.assertRaises(ValueError):encode(source)
        self.assertEqual(len(base['pages'][0]['keys']),15)
    def test_manifest_requirements_must_match_model_and_have_correct_types(self):
        data,_=encode(self.mk2())
        for edit in [lambda x:x.update(model='plus'),lambda x:x['requirements'].update(keys=8),
                     lambda x:x['requirements'].update(dials=False),lambda x:x.update(version=1),lambda x:x.update(device_id='private')]:
            with self.assertRaises(ValueError):decode(self.modify_manifest(data,edit))
    def test_legacy_export_is_explicit_and_only_for_compatible_plus_shapes(self):
        source=self.sample();source.update(version=2,model='plus')
        data,_=encode(source);self.assertEqual(decode(data),source)
        old,_=encode(source,legacy=True);restored=decode(old)
        expected=dict(source);expected.pop('model');expected['version']=1
        self.assertEqual(restored,expected);self.assertEqual(source['version'],2)
        with self.assertRaises(ValueError):encode(self.mk2(),legacy=True)
        source['pages'][0]['keys'][0]['action']={'type':'future_action'}
        with self.assertRaises(ValueError):encode(source,legacy=True)
        source['pages'][0]['keys'][0]['action']={'type':[]}
        with self.assertRaises(ValueError):encode(source,legacy=True)
    def test_v2_maximum_grid_package_has_bounded_capacity_for_all_240_assets(self):
        from copy import deepcopy
        source=self.mk2();page=source['pages'][0];source['pages']=[deepcopy(page) for _ in range(16)]
        for index,page in enumerate(source['pages']):
            page['default']=index==0
            for key_index,key in enumerate(page['keys']):
                # Container-only fixture: image validation remains the native parser's job.
                key['icon_png']=[index,key_index,255]
        data,_=encode(source);self.assertEqual(decode(data),source)
        with ZipFile(BytesIO(data)) as archive:self.assertGreater(len(archive.namelist()),160)
