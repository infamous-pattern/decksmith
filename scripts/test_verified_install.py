"""Check the trust boundary before any downloaded installer code executes."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from package_io import digest, write_archive

SPEC = importlib.util.spec_from_file_location(
    'verified_install', Path(__file__).with_name('verify-install.py'))
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)
SOURCE = 'a' * 40


class VerifiedInstallTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.downloads = self.root / 'downloads'
        self.downloads.mkdir()
        self.executed = self.root / 'executed.json'
        installer = ('import json,sys\nfrom pathlib import Path\n'
                     f'Path({str(self.executed)!r}).write_text(json.dumps(sys.argv))\n').encode()
        self.files = {
            'bin/decksmithd': b'daemon', 'bin/decksmithctl': b'client',
            'apps/decksmith-studio/panel.py': b'panel', 'config/audio.json': b'{}',
            'scripts/decksmith-install.py': installer,
            'scripts/package_io.py': Path(__file__).with_name('package_io.py').read_bytes(),
            'docs/installation.md': b'guide',
        }
        self.manifest = {'format': 'decksmith-release', 'version': 1,
                         'id': '0.1.0-fixture', 'source_commit': SOURCE,
                         'source_dirty': False, 'platform': 'Linux',
                         'architecture': 'x86_64',
                         'tested_distribution': VERIFIER.target_config(getattr(self, 'target', 'fedora44'))[0],
                         'files': {name: digest(data) for name, data in self.files.items()}}
        self.target = getattr(self, 'target', 'fedora44')
        self.refresh()
        self.commands = []
        self.real_run = subprocess.run

    def refresh(self):
        downloads = VERIFIER.target_config(self.target)[2]
        write_archive(self.downloads / downloads[0], self.files | {
            'release.json': json.dumps(self.manifest).encode()})
        for name, member in [('decksmith-install.py', 'scripts/decksmith-install.py'),
                             ('package_io.py', 'scripts/package_io.py'),
                             ('INSTALL.md', 'docs/installation.md')]:
            (self.downloads / name).write_bytes(self.files[member])
        hashes = {name: digest((self.downloads / name).read_bytes())
                  for name in VERIFIER.target_config(self.target)[2]}
        (self.downloads / 'SHA256SUMS').write_text(''.join(
            value + '  ' + name + '\n' for name, value in hashes.items()))
        (self.downloads / 'candidate.json').write_text(json.dumps({
            'format': 'decksmith-release-candidate', 'version': 1,
            'bundle': self.manifest['id'], 'source_commit': SOURCE, 'files': hashes, 'target': self.target}))

    def accepted(self, command, **kwargs):
        self.commands.append(command)
        if command[0] == 'gh':
            self.assertIn('--deny-self-hosted-runners', command)
            self.assertEqual(command[command.index('--source-digest') + 1], SOURCE)
            self.assertEqual(command[command.index('--signer-workflow') + 1], VERIFIER.target_config(self.target)[1])
            return subprocess.CompletedProcess(command, 0)
        return self.real_run(command, **kwargs)

    def verify(self, **kwargs):
        return VERIFIER.verify_install(self.downloads, SOURCE, target=self.target, **kwargs)

    def test_any_signature_failure_prevents_import_and_install(self):
        # Even a package library that crashes if imported must never run before
        # every subject passes; fail each position independently.
        (self.downloads / 'package_io.py').write_bytes(b'raise AssertionError("executed unverified code")')
        for failed in VERIFIER.target_config(self.target)[3]:
            self.commands.clear()
            def reject(command, **kwargs):
                self.commands.append(command)
                if Path(command[3]).name == failed:
                    raise subprocess.CalledProcessError(1, command)
                return subprocess.CompletedProcess(command, 0)
            with self.subTest(subject=failed), patch.object(VERIFIER.subprocess, 'run', side_effect=reject):
                with self.assertRaises(subprocess.CalledProcessError):
                    self.verify(install=True, stage_root=self.root / 'stage')
                self.assertFalse(self.executed.exists())
                self.assertTrue(all(command[0] == 'gh' for command in self.commands))

    def test_install_uses_only_verified_private_snapshot_and_isolated_python(self):
        # Unlisted import traps in the original download directory are excluded.
        (self.downloads / 'hashlib.py').write_bytes(b'raise AssertionError("untrusted import")')
        with patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
            result = self.verify(install=True, stage_root=self.root / 'stage')
        self.assertTrue(result['installed'] and result['staged'])
        self.assertEqual(len(self.commands), 7)
        command = self.commands[-1]
        self.assertIn('-I', command)
        self.assertNotIn('--activate', command)
        self.assertNotIn('--enable', command)
        arguments = json.loads(self.executed.read_text())
        self.assertEqual(arguments[1:3], ['--stage-root', str(self.root / 'stage')])
        self.assertEqual(arguments[3], 'install')
        self.assertNotEqual(Path(arguments[0]).parent, self.downloads)
        self.assertFalse(Path(arguments[0]).exists(), 'Temporary snapshot was not cleaned up')

    def test_source_replacement_cannot_change_verified_execution_bytes(self):
        def replace(command, **kwargs):
            if command[0] == 'gh':
                (self.downloads / 'decksmith-install.py').write_bytes(b'raise AssertionError("replacement ran")')
            return self.accepted(command, **kwargs)
        with patch.object(VERIFIER.subprocess, 'run', side_effect=replace):
            self.verify(install=True, stage_root=self.root / 'stage')
        self.assertTrue(self.executed.exists())

    def test_missing_links_fifo_and_oversize_fail_without_execution(self):
        path = self.downloads / 'INSTALL.md'
        for mode in ('missing', 'link', 'fifo', 'oversize'):
            path.unlink(missing_ok=True)
            if mode == 'link':path.symlink_to(self.downloads / 'decksmith-install.py')
            if mode == 'fifo':os.mkfifo(path)
            if mode == 'oversize':
                with path.open('wb') as stream:stream.truncate(VERIFIER.LIMITS[path.name] + 1)
            with self.subTest(mode=mode), patch.object(VERIFIER.subprocess, 'run') as run:
                with self.assertRaises((OSError, ValueError)):
                    self.verify(install=True, stage_root=self.root / 'stage')
                run.assert_not_called()
            self.assertFalse(self.executed.exists())

    def test_metadata_checksum_and_archive_mismatches_prevent_install(self):
        for mode in ('incomplete', 'duplicate', 'altered', 'source', 'structure', 'dirty', 'bootstrap'):
            self.refresh()
            if mode in ('incomplete', 'duplicate'):
                path = self.downloads / 'SHA256SUMS'
                rows = path.read_text().splitlines()
                path.write_text('\n'.join(rows[:1] if mode == 'incomplete' else rows + rows[:1]) + '\n')
            elif mode == 'altered':
                (self.downloads / 'INSTALL.md').write_bytes(b'changed')
            elif mode == 'source':
                path = self.downloads / 'candidate.json'
                data = json.loads(path.read_text()); data['source_commit'] = 'b' * 40
                path.write_text(json.dumps(data))
            elif mode == 'structure':
                (self.downloads / 'candidate.json').write_text('[]')
            else:
                if mode == 'dirty':self.manifest['source_dirty'] = True
                else:
                    self.files['scripts/decksmith-install.py'] = b'changed bootstrap'
                    self.manifest['files']['scripts/decksmith-install.py'] = digest(b'changed bootstrap')
                self.refresh()
                if mode == 'bootstrap':
                    (self.downloads / 'decksmith-install.py').write_bytes(b'other standalone')
                    hashes = {name: digest((self.downloads / name).read_bytes()) for name in VERIFIER.target_config(self.target)[2]}
                    (self.downloads / 'SHA256SUMS').write_text(''.join(h + '  ' + n + '\n' for n,h in hashes.items()))
                    path = self.downloads / 'candidate.json'
                    data = json.loads(path.read_text()); data['files'] = hashes
                    path.write_text(json.dumps(data))
            with self.subTest(mode=mode), patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
                with self.assertRaises(ValueError):self.verify(install=True, stage_root=self.root / 'stage')
            self.assertFalse(self.executed.exists())
            self.manifest['source_dirty'] = False

    def test_full_commit_is_required_and_verify_only_has_no_install_side_effect(self):
        with patch.object(VERIFIER.subprocess, 'run') as run:
            for value in ('main', 'v1.0.0', 'a' * 39):
                with self.assertRaises(ValueError):VERIFIER.verify_install(self.downloads, value)
            run.assert_not_called()
        with patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
            result = self.verify()
        self.assertFalse(result['installed'])
        self.assertFalse(self.executed.exists())
        self.assertEqual(len(self.commands), 6)

    def test_saved_signing_bundle_is_snapshotted_and_supplied_for_every_subject(self):
        bundle = self.root / 'attestation.json'; bundle.write_text('{}')
        with patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
            self.verify(bundle=bundle)
        for command in self.commands:
            argument = Path(command[command.index('--bundle') + 1])
            self.assertNotEqual(argument, bundle)


    def test_missing_target_only_accepts_legacy_fedora_metadata(self):
        path = self.downloads / 'candidate.json'
        metadata = json.loads(path.read_text()); metadata.pop('target')
        path.write_text(json.dumps(metadata))
        with patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
            if self.target == 'fedora44':
                self.assertEqual(self.verify()['target'], 'fedora44')
            else:
                with self.assertRaises(ValueError):self.verify()


class DebianVerifiedInstallTests(VerifiedInstallTests):
    target = 'debian13'

    def test_wrong_target_never_installs(self):
        path = self.downloads / 'candidate.json'
        metadata = json.loads(path.read_text())
        metadata['target'] = 'fedora44'
        path.write_text(json.dumps(metadata))
        with patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
            with self.assertRaises(ValueError):
                self.verify(install=True, stage_root=self.root / 'stage')
        self.assertFalse(self.executed.exists())

    def test_debian_distribution_must_match_even_when_signed(self):
        self.manifest['tested_distribution'] = 'Fedora 44'
        self.refresh()
        with patch.object(VERIFIER.subprocess, 'run', side_effect=self.accepted):
            with self.assertRaises(ValueError):self.verify()

    def test_pinned_debian_signer_is_distinct_and_unknown_target_fails(self):
        self.assertNotEqual(VERIFIER.target_config(self.target)[1], VERIFIER.WORKFLOW)
        with patch.object(VERIFIER.subprocess, 'run') as run:
            with self.assertRaises(ValueError):VERIFIER.verify_install(self.downloads, SOURCE, target='unknown')
            run.assert_not_called()


class UbuntuVerifiedInstallTests(DebianVerifiedInstallTests):
    target = 'ubuntu2604'

    def test_debian_signer_and_archive_are_not_ubuntu(self):
        ubuntu = VERIFIER.target_config(self.target)
        debian = VERIFIER.target_config('debian13')
        self.assertNotEqual(ubuntu[1], debian[1])
        self.assertNotEqual(ubuntu[2][0], debian[2][0])


class CosmicVerifiedInstallTests(DebianVerifiedInstallTests):
    target = 'popos2404'

    def test_gnome_preview_signers_and_archives_cannot_be_substituted(self):
        cosmic = VERIFIER.target_config(self.target)
        self.assertEqual(cosmic[0], 'Ubuntu 24.04')
        self.assertTrue(cosmic[1].endswith('/popos-candidate.yml'))
        for target in ('fedora44', 'debian13', 'ubuntu2604'):
            other = VERIFIER.target_config(target)
            self.assertNotEqual(cosmic[1], other[1])
            self.assertNotEqual(cosmic[2][0], other[2][0])


if __name__ == '__main__':unittest.main()
