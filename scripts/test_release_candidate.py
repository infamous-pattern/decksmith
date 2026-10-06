"""Candidate admission rejects source drift, bootstrap replacement and tampering."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from package_io import digest, read_archive, write_archive

SPEC = importlib.util.spec_from_file_location(
    'candidate', Path(__file__).with_name('prepare-release-candidate.py'))
CANDIDATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CANDIDATE)
SOURCE = 'a' * 40


class CandidateAdmission(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.archive = self.root / 'bundle.tar.gz'
        self.output = self.root / 'candidate'
        self.files = {
            'bin/decksmithd': b'daemon', 'bin/decksmithctl': b'client',
            'apps/decksmith-studio/panel.py': b'panel', 'config/audio.json': b'{}',
            'scripts/decksmith-install.py': b'installer',
            'scripts/package_io.py': b'package library',
            'docs/installation.md': b'guide',
        }
        self.manifest = {
            'format': 'decksmith-release', 'version': 1, 'id': '0.1.0-fixture',
            'source_commit': SOURCE, 'source_dirty': False, 'platform': 'Linux',
            'architecture': 'x86_64', 'tested_distribution': 'Fedora 44',
            'files': {name: digest(data) for name, data in self.files.items()},
        }
        self.write_bundle()
        for name, member in [('decksmith-install.py', 'scripts/decksmith-install.py'),
                             ('package_io.py', 'scripts/package_io.py'),
                             ('INSTALL.md', 'docs/installation.md')]:
            (self.root / name).write_bytes(self.files[member])

    def write_bundle(self):
        write_archive(self.archive, self.files | {
            'release.json': json.dumps(self.manifest).encode()})

    def prepare(self, source=SOURCE):
        return CANDIDATE.prepare(self.archive, self.output, source)

    def test_complete_staging_is_byte_identical_and_covers_every_download(self):
        original = self.archive.read_bytes()
        self.prepare()
        self.assertEqual((self.output / CANDIDATE.SUBJECTS[0]).read_bytes(), original)
        self.assertEqual(set(path.name for path in self.output.iterdir()),
                         set(CANDIDATE.SUBJECTS) | {'SHA256SUMS', 'candidate.json'})
        lines = (self.output / 'SHA256SUMS').read_text().splitlines()
        self.assertEqual(len(lines), 4)
        for line in lines:
            checksum, name = line.split('  ')
            self.assertEqual(checksum, digest((self.output / name).read_bytes()))
        self.assertEqual(json.loads((self.output / 'candidate.json').read_text())[
            'source_commit'], SOURCE)

    def test_dirty_wrong_source_or_platform_never_stage(self):
        for field, value in [('source_dirty', True), ('source_dirty', None),
                             ('source_commit', 'b' * 40), ('architecture', 'aarch64'),
                             ('tested_distribution', 'Ubuntu')]:
            original = self.manifest[field]
            self.manifest[field] = value
            self.write_bundle()
            with self.assertRaises(ValueError):
                self.prepare()
            self.assertFalse(self.output.exists())
            self.manifest[field] = original
        with self.assertRaises(ValueError):
            self.prepare('main')

    def test_tampered_member_or_replaced_bootstrap_is_rejected(self):
        files = read_archive(self.archive)
        files['bin/decksmithd'] = b'tampered'
        write_archive(self.archive, files)
        with self.assertRaises(ValueError):
            self.prepare()
        self.write_bundle()
        (self.root / 'package_io.py').write_bytes(b'replaced')
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_missing_or_symlinked_bootstrap_is_rejected(self):
        path = self.root / 'package_io.py'
        path.unlink()
        with self.assertRaises(OSError):
            self.prepare()
        path.symlink_to(self.root / 'decksmith-install.py')
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_existing_destination_is_preserved(self):
        self.output.mkdir()
        marker = self.output / 'keep'
        marker.write_bytes(b'existing')
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual(marker.read_bytes(), b'existing')

    def test_input_replacement_during_validation_cannot_change_staged_bytes(self):
        original = self.archive.read_bytes()
        validate = CANDIDATE.release_files
        def replace_input(snapshot):
            result = validate(snapshot)
            self.archive.write_bytes(b'replaced after verification')
            return result
        with patch.object(CANDIDATE, 'release_files', side_effect=replace_input):
            self.prepare()
        self.assertEqual((self.output / CANDIDATE.SUBJECTS[0]).read_bytes(), original)
