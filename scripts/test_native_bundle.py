"""Cross-distribution packaging must not mislabel or accept incompatible binaries."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from build_platform import bundle_platform

SPEC = importlib.util.spec_from_file_location('native_builder', Path(__file__).with_name('build-bundle.py'))
BUILDER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILDER)
SPEC = importlib.util.spec_from_file_location('doctor', Path(__file__).with_name('runtime-doctor.py'))
DOCTOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DOCTOR)


class NativeBundleTests(unittest.TestCase):
    def environment(self, distribution, version):
        return patch('platform.freedesktop_os_release', return_value={'ID': distribution, 'VERSION_ID': version})

    def test_debian_environment_does_not_claim_fedora(self):
        with self.environment('debian', '13'), patch('platform.system', return_value='Linux'), \
                patch('platform.machine', return_value='x86_64'), patch('platform.libc_ver', return_value=('glibc', '2.41')):
            result = bundle_platform()
        self.assertEqual(result['tested_distribution'], 'Debian 13')
        self.assertEqual(result['build_environment']['libc_version'], '2.41')
        self.assertEqual(result['architecture'], 'x86_64')

    def test_ubuntu_native_label_preserves_full_release_version(self):
        with self.environment('ubuntu', '26.04'), patch('platform.system', return_value='Linux'):
            result = bundle_platform()
        self.assertEqual(result['tested_distribution'], 'Ubuntu 26.04')
        self.assertEqual(result['build_environment']['distribution'], 'ubuntu')
        self.assertEqual(result['build_environment']['version'], '26.04')

    def test_fedora_candidate_target_remains_exact(self):
        with self.environment('fedora', '44'), patch('platform.system', return_value='Linux'):
            self.assertEqual(bundle_platform()['tested_distribution'], 'Fedora 44')

    def test_missing_distribution_is_rejected(self):
        with patch('platform.system', return_value='Linux'), patch('platform.freedesktop_os_release', return_value={}):
            with self.assertRaises(ValueError):
                bundle_platform()

    def test_incompatible_binaries_never_create_a_bundle(self):
        with tempfile.TemporaryDirectory() as temporary, self.environment('debian', '13'), \
                patch('platform.system', return_value='Linux'), \
                patch.object(BUILDER.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, ['decksmithd'], stderr='GLIBC_2.43 not found')):
            destination = Path(temporary) / 'bundle'
            with self.assertRaises(subprocess.CalledProcessError):
                BUILDER.build(destination, Path(temporary) / 'fedora-binaries')
            self.assertFalse(destination.exists())

    def test_runtime_hints_use_the_distribution_package_names(self):
        for distribution in ('debian', 'ubuntu'):
            with self.environment(distribution, '13'):
                hints = DOCTOR.package_hints()
                self.assertEqual(hints['libpulse.so.0'], 'libpulse0')
                self.assertIn('python3-gi-cairo', hints['artwork'])
        with self.environment('fedora', '44'):
            self.assertEqual(DOCTOR.package_hints()['libpulse.so.0'], 'pulseaudio-libs')

    def test_pop_os_inherits_ubuntu_runtime_package_hints(self):
        with patch('platform.freedesktop_os_release', return_value={
                'ID': 'pop', 'VERSION_ID': '24.04', 'ID_LIKE': 'ubuntu debian'}):
            hints = DOCTOR.package_hints()
            self.assertEqual(hints['gdbus'], 'libglib2.0-bin')
            self.assertEqual(hints['libudev.so.1'], 'libudev1')
            self.assertEqual(hints['libpulse.so.0'], 'libpulse0')
            self.assertIn('gir1.2-adw-1', hints['artwork'])

    def test_incompatible_client_is_rejected_even_with_a_working_daemon(self):
        def execute(args, **kwargs):
            if Path(args[0]).name == 'decksmithctl':
                raise subprocess.CalledProcessError(1, args, stderr='GLIBC_2.43 not found')
            return subprocess.CompletedProcess(args, 0, stdout='', stderr='')
        with tempfile.TemporaryDirectory() as temporary, self.environment('debian', '13'), \
                patch('platform.system', return_value='Linux'), patch.object(BUILDER.subprocess, 'run', side_effect=execute):
            destination = Path(temporary) / 'bundle'
            with self.assertRaises(subprocess.CalledProcessError):
                BUILDER.build(destination, Path(temporary) / 'mixed-binaries')
            self.assertFalse(destination.exists())
