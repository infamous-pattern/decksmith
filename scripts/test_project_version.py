"""A product-version change must propagate to binary/bundle/catalog metadata."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from project_version import application_version
from package_io import release_files
import localization

SPEC=importlib.util.spec_from_file_location('bundle_builder',Path(__file__).with_name('build-bundle.py'))
BUILDER=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILDER)


class ProductVersionTests(unittest.TestCase):
    def test_version_change_reaches_bundle_identity_manifest_and_catalog_extraction(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            (root/'Cargo.toml').write_text('[workspace.package]\nversion="1.2.3-rc.1"\n')
            self.assertEqual(application_version(root),'1.2.3-rc.1')
            with patch.object(localization.subprocess,'run') as run:
                localization.extract(root)
                self.assertIn('--package-version=1.2.3-rc.1',run.call_args.args[0])
            binaries=root/'binaries';binaries.mkdir()
            for name in ('decksmithd','decksmithctl'):(binaries/name).write_bytes(b'fixture binary')
            files={'apps/decksmith-studio/panel.py':b'panel','scripts/decksmith-install.py':b'installer',
                   'scripts/package_io.py':b'package library','config/audio.json':b'{}',
                   'docs/installation.md':b'guide'}
            for name,data in files.items():
                path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
            for name in ('README.md','LICENSE'):(root/name).write_text('fixture')
            with patch.object(BUILDER,'ROOT',root),patch.object(BUILDER,'compiled_catalogs',return_value={}),\
                 patch.object(BUILDER.subprocess,'run'),patch.object(BUILDER.subprocess,'check_output',side_effect=['a'*40+'\n','']):
                archive=BUILDER.build(root/'output',binaries)
            manifest,_=release_files(archive)
            self.assertEqual(manifest['application_version'],'1.2.3-rc.1')
            self.assertTrue(manifest['id'].startswith('1.2.3-rc.1-'))
            self.assertEqual(manifest['version'],1,'Archive schema must not change with the product version')

    def test_missing_or_unsafe_versions_are_rejected_before_build(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for version in ('../escape','1.0','01.0.0','1.0.0/escape',''):
                (root/'Cargo.toml').write_text('[workspace.package]\nversion='+json.dumps(version)+'\n')
                with self.subTest(version=version),self.assertRaises(ValueError):application_version(root)


if __name__=='__main__':unittest.main()
