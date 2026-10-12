#!/usr/bin/env python3
"""Select, download and authenticate a published Decksmith package before installation."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import tarfile
import time
import urllib.parse
import urllib.request

sys.dont_write_bytecode = True
# Imported only from reviewed source, or the private directory populated by the
# generated bootstrap. Never import a verifier from the downloaded release.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from runtime_dependencies import runtime_packages

RELEASES = {
    ('fedora', '44'): {
        'target': 'fedora44', 'label': 'Fedora Workstation 44 GNOME/Wayland',
        'version': 'v1.0.0', 'source': '68c42421ceaa65bb033993b56f0babdde2a14a6e',
        'manager': 'dnf', 'experimental': False,
    },
    ('debian', '13'): {
        'target': 'debian13', 'label': 'Debian 13 GNOME/Wayland',
        'version': 'v1.0.0-debian-preview.1', 'source': 'dee2f2c285041dc19388ed579b90886e4b58341a',
        'manager': 'apt-get', 'experimental': True,
    },
    ('ubuntu', '26.04'): {
        'target': 'ubuntu2604', 'label': 'Ubuntu 26.04 GNOME/Wayland',
        'version': 'v1.0.0-ubuntu-preview.1', 'source': 'e627ea4f7f660a6ff6ad4772a013989b36b717c4',
        'manager': 'apt-get', 'experimental': True,
    },
    ('pop', '24.04'): {
        'target': 'popos2404', 'label': 'Pop!_OS 24.04 COSMIC',
        'version': 'v1.0.0-popos-preview.1', 'source': 'be238905bb3f6aa793dcda2df063fa5a2f206cbc',
        'manager': 'apt-get', 'experimental': True,
    },
}

# Upstream's immutable v2.102.0 release fixes workflow identity verification.
# Digest taken from cli/cli's official gh_2.102.0_checksums.txt, reviewed Oct 11.
GH_VERSION = '2.102.0'
GH_ARCHIVE = 'gh_' + GH_VERSION + '_linux_amd64.tar.gz'
GH_SHA256 = 'bb766f710eef8ede859c18578c72c327597cd4c8a85b06001b1f3843c6019386'


def supports_attestation(command):
    try:
        version = subprocess.run([command, '--version'], capture_output=True, text=True, timeout=15)
        match = re.search(r'^gh version (\d+)\.(\d+)\.(\d+)\b', version.stdout or '', re.MULTILINE)
        if version.returncode or not match or tuple(map(int, match.groups())) < (2, 102, 0):
            return False
        return subprocess.run([command, 'attestation', 'verify', '--help'],
                              capture_output=True, text=True, timeout=15).returncode == 0
    except OSError:
        return False


def verification_cli(directory, *, allow_portable=False):
    if supports_attestation('gh'):
        return 'gh'
    message = ('GitHub CLI 2.102.0+ is required for attestation verification. '
               'Use its official package source, or allow a pinned temporary upstream copy.')
    if not allow_portable:
        try:
            # A controlling terminal is not seekable: buffered r+ creates a
            # BufferedRandom stream and fails before the consent prompt appears.
            with open('/dev/tty', 'r', encoding='utf-8') as terminal, \
                 open('/dev/tty', 'w', encoding='utf-8') as display:
                display.write(message + '\nDownload GitHub CLI ' + GH_VERSION +
                              ' temporarily for this installation? [y/N] ')
                display.flush()
                allow_portable = terminal.readline().strip().lower() in ('y', 'yes')
        except OSError as error:
            raise ValueError(message + ' Retry with --allow-portable-gh to opt in.') from error
    if not allow_portable:
        raise ValueError(message + ' Installation stopped; no signatures were bypassed.')
    archive = directory / GH_ARCHIVE
    print('Downloading pinned GitHub CLI ' + GH_VERSION + ' for temporary verification.', flush=True)
    download('https://github.com/cli/cli/releases/download/v' + GH_VERSION + '/' + GH_ARCHIVE,
             archive, 64 * 1024 * 1024)
    with archive.open('rb') as incoming:
        if hashlib.file_digest(incoming, 'sha256').hexdigest() != GH_SHA256:
            raise ValueError('GitHub CLI checksum differs from the reviewed pin; refusing execution.')
    # Read just the pinned archive's regular executable, never extract its paths.
    with tarfile.open(archive) as package:
        member = package.getmember('gh_' + GH_VERSION + '_linux_amd64/bin/gh')
        if not member.isfile() or member.size > 64 * 1024 * 1024:
            raise ValueError('Invalid GitHub CLI executable')
        with package.extractfile(member) as incoming:
            executable = directory / 'gh'
            with executable.open('xb') as outgoing:
                shutil.copyfileobj(incoming, outgoing)
    executable.chmod(0o700)
    if not supports_attestation(str(executable)):
        raise ValueError('Pinned GitHub CLI is unavailable; refusing installation.')
    return str(executable)


def select_release(release, *, experimental=False):
    key = (release.get('ID'), release.get('VERSION_ID'))
    if key not in RELEASES:
        raise ValueError('No verified download is configured for this OS/version. '
                         'See the support table; Fedora 45 and openSUSE are test-only environments.')
    selected = RELEASES[key]
    if selected['experimental'] and not experimental:
        raise ValueError(selected['label'] + ' is experimental. Read its preview limits, '
                         'then use --experimental to opt in.')
    return selected


def verifier_module():
    spec = importlib.util.spec_from_file_location(
        'reviewed_verifier', Path(__file__).with_name('verify-install.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def missing_packages(packages, manager):
    missing = []
    for package in packages:
        command = (['rpm', '-q', package] if manager == 'dnf' else
                   ['dpkg-query', '-W', '-f=${Status}', package])
        result = subprocess.run(command, capture_output=True, text=True, timeout=15)
        if result.returncode or (manager != 'dnf' and result.stdout.strip() != 'install ok installed'):
            missing.append(package)
    return missing


def install_dependencies(packages, selected, *, no_deps=False):
    missing = missing_packages(packages, selected['manager'])
    if not missing:
        return
    command = ['sudo', selected['manager'], 'install', *missing]
    print('Missing dependencies: ' + ', '.join(missing), flush=True)
    if no_deps:
        raise ValueError('Install these packages from your OS repositories, then retry: '
                         + ' '.join(command))
    if not shutil.which('sudo'):
        raise ValueError('sudo is unavailable. Ask an administrator to install: ' + ', '.join(missing))
    try:
        terminal = open('/dev/tty', 'r', encoding='utf-8')
    except OSError as error:
        raise ValueError('Dependency installation needs a terminal. Run: ' + ' '.join(command)) from error
    # Trusted OS package sources, normal package-manager confirmation; never -y.
    with terminal:
        subprocess.run(command, stdin=terminal, check=True)
    if missing_packages(packages, selected['manager']):
        raise ValueError('Required packages are still missing; installation stopped.')


class HTTPSOnlyRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, newurl):
        if urllib.parse.urlsplit(newurl).scheme != 'https':
            raise ValueError('Refusing a download redirect outside HTTPS')
        return super().redirect_request(request, file, code, message, headers, newurl)


def download(url, destination, limit):
    if urllib.parse.urlsplit(url).scheme != 'https':
        raise ValueError('Downloads require HTTPS')
    deadline = time.monotonic() + 300
    opener = urllib.request.build_opener(HTTPSOnlyRedirect())
    request = urllib.request.Request(url, headers={'User-Agent': 'Decksmith-verified-installer'})
    with opener.open(request, timeout=30) as incoming, destination.open('xb') as outgoing:
        if urllib.parse.urlsplit(incoming.geturl()).scheme != 'https':
            raise ValueError('Refusing non-HTTPS download')
        size = 0
        while block := incoming.read(1024 * 1024):
            if time.monotonic() > deadline:
                raise ValueError('Download timed out: ' + destination.name)
            size += len(block)
            if size > limit:
                raise ValueError('Download exceeds its size limit: ' + destination.name)
            outgoing.write(block)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--experimental', action='store_true', help='Opt into the matching Debian/Ubuntu/COSMIC preview')
    parser.add_argument('--check', action='store_true', help='Show OS, release and dependency status; no network or changes')
    parser.add_argument('--allow-portable-gh', action='store_true', help='Allow the pinned temporary upstream GitHub CLI when the system CLI is absent or too old')
    parser.add_argument('--no-deps', action='store_true', help='Stop if dependencies are missing instead of offering to install them')
    parser.add_argument('--verify-only', action='store_true', help='Download and verify without installing Decksmith')
    parser.add_argument('--stage-root', type=Path, help='Install into an isolated test directory; never manage services')
    args = parser.parse_args(argv)
    if os.geteuid() == 0:
        raise ValueError('Run as your regular desktop user, not with sudo.')
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise ValueError('Published downloads require Linux x86_64.')
    if sys.version_info < (3, 11):
        raise ValueError('Python 3.11 or newer is required for verification.')
    release = platform.freedesktop_os_release()
    selected = select_release(release, experimental=args.experimental)
    if args.verify_only and args.stage_root:
        raise ValueError('--stage-root cannot be combined with --verify-only')
    verifier = verifier_module()
    config = verifier.target_config(selected['target'])
    packages = runtime_packages(release)
    report = {'platform': selected['label'], 'experimental': selected['experimental'],
              'release': selected['version'], 'source_commit': selected['source'],
              'archive': config[2][0], 'dependencies': list(packages),
              'missing_dependencies': missing_packages(packages, selected['manager']),
              'verification_requirement': 'GitHub CLI 2.102.0+ or optional pinned temporary copy',
              'startup': 'unchanged; controls not started'}
    if args.check:
        print(json.dumps(report, indent=2))
        return report
    print('Decksmith: ' + selected['version'] + ' for ' + selected['label'], flush=True)
    if selected['experimental']:
        print('Experimental preview: physical hardware/audio and lifecycle support are not fully qualified.', flush=True)
    print('Saved data and startup preferences are preserved; background controls will not restart.', flush=True)
    install_dependencies(packages, selected, no_deps=args.no_deps)
    with tempfile.TemporaryDirectory(prefix='decksmith-downloads-') as temporary:
        directory = Path(temporary)
        os.chmod(directory, 0o700)
        gh_command = verification_cli(directory, allow_portable=args.allow_portable_gh)
        base = 'https://github.com/infamous-pattern/decksmith/releases/download/' + selected['version'] + '/'
        for name in (*config[3], 'attestation.json'):
            print('Downloading ' + name, flush=True)
            download(base + name, directory / name, config[4].get(name, 8 * 1024 * 1024))
        result = verifier.verify_install(directory, selected['source'],
                                        bundle=directory / 'attestation.json',
                                        target=selected['target'], install=not args.verify_only,
                                        stage_root=args.stage_root, gh_command=gh_command)
    print(json.dumps(result, indent=2))
    if result['installed']:
        print('Open Decksmith from your application menu. Close other Stream Deck controllers, '
              'then start Background controls when ready. For USB permissions or the optional '
              'panel icon, see the installation guide.')
    return result


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print('Installation stopped: ' + str(error), file=sys.stderr)
        sys.exit(1)
