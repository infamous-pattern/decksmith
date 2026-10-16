"""Guard distro selection, dependency handling and signed bootstrap execution."""
import importlib.util
import io
import os
import pty
import select
import signal
from pathlib import Path
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch

from runtime_dependencies import runtime_packages, package_hints


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


BOOT = load('bootstrap', 'bootstrap-install.py')
BUILD = load('bootstrap_build', 'build-bootstrap.py')


class BootstrapTests(unittest.TestCase):
    def terminal_child(self, program, marker, reply):
        """Use a real controlling terminal: regular-file mocks miss r+ failures."""
        import sys
        pid, descriptor = pty.fork()
        if pid == 0:
            os.execv(sys.executable, [sys.executable, '-B', '-c', program])
        output = b''; responded = False; ended = False
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                if select.select([descriptor], [], [], .1)[0]:
                    try:
                        output += os.read(descriptor, 65536)
                    except OSError:
                        pass
                if not responded and marker.encode() in output:
                    os.write(descriptor, reply.encode()); responded = True
                finished, status = os.waitpid(pid, os.WNOHANG)
                if finished:
                    ended = True
                    self.assertEqual(os.waitstatus_to_exitcode(status), 0, output.decode(errors='replace'))
                    self.assertTrue(responded, output.decode(errors='replace'))
                    return output.decode(errors='replace')
            self.fail('Controlling-terminal test timed out: ' + output.decode(errors='replace'))
        finally:
            if not ended:
                os.killpg(pid, signal.SIGKILL); os.waitpid(pid, 0)
            os.close(descriptor)

    def terminal_loader(self):
        return ('import sys; sys.path.insert(0, ' + repr(str(Path(__file__).parent.resolve())) + ')\n'
                'import importlib.util\n'
                's=importlib.util.spec_from_file_location("boot", ' + repr(str(Path(__file__).with_name('bootstrap-install.py').resolve())) + ')\n'
                'b=importlib.util.module_from_spec(s);s.loader.exec_module(b)\n')

    def test_real_tty_portable_consent_reaches_download(self):
        program = self.terminal_loader() + '''
from pathlib import Path
b.supports_attestation=lambda command: False
def download(*args):raise RuntimeError('CONSENT_ACCEPTED')
b.download=download
try:b.verification_cli(Path('/unused'))
except RuntimeError as e:assert str(e)=='CONSENT_ACCEPTED';print('PASS accepted',flush=True)
else:raise AssertionError('Expected intercepted download')
'''
        self.assertIn('PASS accepted', self.terminal_child(program, '[y/N] ', 'y\n'))

    def test_real_tty_decline_never_downloads(self):
        program = self.terminal_loader() + '''
from pathlib import Path
b.supports_attestation=lambda command: False
def download(*args):raise AssertionError('Declined download executed')
b.download=download
try:b.verification_cli(Path('/unused'))
except ValueError as e:assert 'no signatures were bypassed' in str(e);print('PASS declined',flush=True)
else:raise AssertionError('Expected safe refusal')
'''
        self.assertIn('PASS declined', self.terminal_child(program, '[y/N] ', 'n\n'))

    def test_real_tty_dependency_confirmation_input(self):
        program = self.terminal_loader() + '''
from unittest.mock import patch
checks=iter([['fixture-package'],[]])
def package_manager(command,stdin,check):
 assert command==['sudo','dnf','install','fixture-package'] and check
 print('DEPENDENCY_CONFIRMATION',flush=True)
 assert stdin.readline().strip()=='y'
with patch.object(b,'missing_packages',side_effect=lambda *args:next(checks)),patch.object(b.shutil,'which',return_value='/usr/bin/sudo'),patch.object(b.subprocess,'run',side_effect=package_manager):
 b.install_dependencies(('fixture-package',), b.RELEASES['fedora','44'])
print('PASS terminal input',flush=True)
'''
        self.assertIn('PASS terminal input', self.terminal_child(program, 'DEPENDENCY_CONFIRMATION', 'y\n'))

    def test_shell_contains_exact_reviewed_sources(self):
        self.assertEqual(Path(__file__).with_name('install.sh').read_text(), BUILD.render())
        for number, name in enumerate(BUILD.FILES):
            shell = BUILD.render()
            block = shell.split("<<'DECKSMITH_SOURCE_" + str(number) + "'\n", 1)[1].split(
                '\nDECKSMITH_SOURCE_' + str(number), 1)[0]
            self.assertEqual(block, Path(__file__).with_name(name).read_text())

    def test_explicit_preview_opt_in_and_exact_os_version(self):
        for (distribution, version), selected in BOOT.RELEASES.items():
            release = {'ID': distribution, 'VERSION_ID': version}
            self.assertEqual(BOOT.select_release(release, experimental=True), selected)
            if selected['experimental']:
                with self.assertRaisesRegex(ValueError, 'experimental'):
                    BOOT.select_release(release)
        for release in ({'ID': 'fedora', 'VERSION_ID': '45'},
                        {'ID': 'ubuntu', 'VERSION_ID': '24.04'},
                        {'ID': 'opensuse-tumbleweed'},
                        {'ID': 'unknown', 'ID_LIKE': 'ubuntu', 'VERSION_ID': '26.04'}):
            with self.assertRaises(ValueError):
                BOOT.select_release(release, experimental=True)

    def test_approved_release_archive_workflow_and_source_pins_match_guides(self):
        guides = {'fedora44': 'v1.5-verified-installation.md', 'debian13': 'debian-preview-installation.md',
                  'ubuntu2604': 'ubuntu-preview-installation.md', 'popos2404': 'popos-preview-installation.md'}
        verifier = BOOT.verifier_module()
        for selected in BOOT.RELEASES.values():
            guide = (Path(__file__).parents[1] / 'docs' / guides[selected['target']]).read_text()
            self.assertIn(selected['source'], guide)
            self.assertIn(selected['version'], guide)
            self.assertIn(verifier.target_config(selected['target'])[2][0], guide)

    def test_apt_requires_installed_status_not_just_package_record(self):
        results = [subprocess.CompletedProcess([], 0, 'install ok installed'),
                   subprocess.CompletedProcess([], 0, 'deinstall ok config-files'),
                   subprocess.CompletedProcess([], 1, '')]
        with patch.object(BOOT.subprocess, 'run', side_effect=results):
            self.assertEqual(BOOT.missing_packages(('one', 'two', 'three'), 'apt-get'), ['two', 'three'])

    def test_no_deps_never_escalates(self):
        with patch.object(BOOT, 'missing_packages', return_value=['gh']), patch.object(BOOT.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'sudo dnf install gh'):
                BOOT.install_dependencies(('gh',), BOOT.RELEASES['fedora', '44'], no_deps=True)
            run.assert_not_called()

    def test_package_installation_uses_tty_and_normal_confirmation(self):
        with tempfile.TemporaryFile(mode='w+') as terminal:
            with patch.object(BOOT, 'missing_packages', side_effect=[['gh'], []]), \
                 patch.object(BOOT.shutil, 'which', return_value='/usr/bin/sudo'), \
                 patch('builtins.open', return_value=terminal), \
                 patch.object(BOOT.subprocess, 'run') as run:
                BOOT.install_dependencies(('gh',), BOOT.RELEASES['debian', '13'])
                run.assert_called_once_with(['sudo', 'apt-get', 'install', 'gh'], stdin=terminal, check=True)

    def test_http_redirect_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'HTTPS'):
            BOOT.HTTPSOnlyRedirect().redirect_request(None, None, 302, '', {}, 'http://example.com/file')

    def test_download_size_cap_and_non_https_url(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / 'file'
            with self.assertRaises(ValueError):
                BOOT.download('http://github.com/file', destination, 10)
            response = io.BytesIO(b'payload-too-long')
            response.geturl = lambda: 'https://github.com/file'
            with patch.object(BOOT.urllib.request, 'build_opener') as opener:
                opener.return_value.open.return_value = response
                with self.assertRaisesRegex(ValueError, 'size limit'):
                    BOOT.download('https://github.com/file', destination, 10)

    def test_portable_cli_rejects_wrong_digest_before_import_or_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            def fake_download(url, destination, limit):
                destination.write_bytes(b'not the reviewed CLI archive')
            with patch.object(BOOT, 'supports_attestation', return_value=False), \
                 patch.object(BOOT, 'download', side_effect=fake_download), \
                 patch.object(BOOT.tarfile, 'open') as read_archive:
                with self.assertRaisesRegex(ValueError, 'reviewed pin'):
                    BOOT.verification_cli(Path(temporary), allow_portable=True)
                read_archive.assert_not_called()

    def test_old_cli_and_capability_failure_are_not_accepted(self):
        for text in ('gh version 2.46.0', 'gh version 2.97.0', 'unparseable'):
            with patch.object(BOOT.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, text)):
                self.assertFalse(BOOT.supports_attestation('gh'))

    def environment(self):
        stack = __import__('contextlib').ExitStack()
        stack.enter_context(patch.object(BOOT.os, 'geteuid', return_value=1000))
        stack.enter_context(patch.object(BOOT.platform, 'system', return_value='Linux'))
        stack.enter_context(patch.object(BOOT.platform, 'machine', return_value='x86_64'))
        stack.enter_context(patch.object(BOOT.platform, 'freedesktop_os_release', return_value={'ID': 'fedora', 'VERSION_ID': '44'}))
        return stack

    def test_check_mode_does_not_download_install_or_mutate(self):
        with self.environment(), patch.object(BOOT, 'missing_packages', return_value=['gh']), \
             patch.object(BOOT, 'download') as download, patch.object(BOOT, 'install_dependencies') as install, \
             patch.object(BOOT.subprocess, 'run') as run, patch('sys.stdout', new_callable=io.StringIO):
            report = BOOT.main(['--check'])
            self.assertEqual(report['release'], 'v1.5.1')
            self.assertIn('2.102.0', report['verification_requirement'])
            download.assert_not_called(); install.assert_not_called(); run.assert_not_called()

    def test_old_gh_stops_before_download(self):
        with self.environment(), patch.object(BOOT, 'missing_packages', return_value=[]), \
             patch.object(BOOT, 'download') as download, \
             patch.object(BOOT.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)), \
             patch('builtins.open', side_effect=OSError('no tty')), \
             patch('sys.stdout', new_callable=io.StringIO):
            with self.assertRaisesRegex(ValueError, 'attestation verification'):
                BOOT.main([])
            download.assert_not_called()

    def test_orchestrator_passes_exact_pin_target_and_never_activates(self):
        verifier = BOOT.verifier_module()
        with self.environment(), patch.object(BOOT, 'missing_packages', return_value=[]), \
             patch.object(BOOT, 'download') as download, patch.object(BOOT, 'verifier_module', return_value=verifier), \
             patch.object(verifier, 'verify_install', return_value={'installed': True}) as verify, \
             patch.object(BOOT.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'gh version 2.102.0')), \
             patch('sys.stdout', new_callable=io.StringIO):
            BOOT.main(['--stage-root', '/tmp/test-stage'])
            self.assertEqual(download.call_count, 7)
            args, kwargs = verify.call_args
            self.assertEqual(args[1], BOOT.RELEASES['fedora', '44']['source'])
            self.assertEqual(kwargs['target'], 'fedora44')
            self.assertTrue(kwargs['install'])
            self.assertNotIn('activate', kwargs)
            self.assertFalse(args[0].exists(), 'Temporary downloads were not cleaned up')

    def test_dependency_hints_are_shared_and_unknown_distro_not_fedora(self):
        self.assertIn('python3-gi', runtime_packages({'ID': 'debian'}))
        self.assertIn('python3-gobject', runtime_packages({'ID': 'fedora'}))
        self.assertEqual(runtime_packages({'ID': 'unknown'}), ())
        self.assertIn('distribution guide', package_hints({'ID': 'unknown'})['gdbus'])
        self.assertIn('distribution guide', package_hints({'ID': 'opensuse-tumbleweed'})['gdbus'])
