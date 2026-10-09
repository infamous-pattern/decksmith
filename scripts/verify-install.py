#!/usr/bin/env python3
"""Verify accepted Decksmith downloads before optionally installing them per user."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
REPOSITORY = 'infamous-pattern/decksmith'
WORKFLOW = REPOSITORY + '/.github/workflows/release-candidate.yml'
DOWNLOADS = ('decksmith-linux-x86_64.tar.gz', 'decksmith-install.py',
             'package_io.py', 'INSTALL.md')
SUBJECTS = DOWNLOADS + ('SHA256SUMS', 'candidate.json')
LIMITS = {name: 1024 * 1024 for name in SUBJECTS}
LIMITS.update({'decksmith-linux-x86_64.tar.gz': 512 * 1024 * 1024,
               'SHA256SUMS': 4096, 'candidate.json': 16384})

TARGETS = {
    'fedora44': ('Fedora 44', DOWNLOADS[0], WORKFLOW),
    'debian13': ('Debian 13', 'decksmith-debian13-x86_64.tar.gz',
                 REPOSITORY + '/.github/workflows/debian-candidate.yml'),
    'ubuntu2604': ('Ubuntu 26.04', 'decksmith-ubuntu2604-x86_64.tar.gz',
                   REPOSITORY + '/.github/workflows/ubuntu-candidate.yml'),
    'popos2404': ('Ubuntu 24.04', 'decksmith-popos2404-x86_64.tar.gz',
                  REPOSITORY + '/.github/workflows/popos-candidate.yml'),
}


def target_config(target):
    if target not in TARGETS:
        raise ValueError('Unknown installation target')
    distribution, archive, workflow = TARGETS[target]
    downloads = (archive,) + DOWNLOADS[1:]
    subjects = downloads + ('SHA256SUMS', 'candidate.json')
    limits = {name: LIMITS.get(name, 512 * 1024 * 1024) for name in subjects}
    return distribution, workflow, downloads, subjects, limits


def snapshot_file(source, destination, limit):
    """Only copy bounded regular files; later verification uses these exact bytes."""
    descriptor = os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(descriptor, 'rb') as incoming:
        metadata = os.fstat(incoming.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > limit:
            raise ValueError('Not a bounded regular file: ' + source.name)
        size = 0
        with destination.open('xb') as outgoing:
            os.chmod(destination, 0o600)
            while block := incoming.read(1024 * 1024):
                size += len(block)
                if size > limit:
                    raise ValueError('Download exceeds its size limit: ' + source.name)
                outgoing.write(block)


def checksums(directory, target='fedora44'):
    downloads = target_config(target)[2]
    expected = {}
    for line in (directory / 'SHA256SUMS').read_text().splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9_.-]+)', line)
        if not match:
            raise ValueError('Invalid checksum manifest')
        checksum, name = match.groups()
        if name not in downloads or name in expected:
            raise ValueError('Unexpected or duplicate checksum entry')
        with (directory / name).open('rb') as stream:
            if hashlib.file_digest(stream, 'sha256').hexdigest() != checksum:
                raise ValueError('Checksum mismatch: ' + name)
        expected[name] = checksum
    if set(expected) != set(downloads):
        raise ValueError('Incomplete checksum manifest')
    return expected


def verified_metadata(directory, source, target='fedora44'):
    distribution, _, downloads, _, _ = target_config(target)
    hashes = checksums(directory, target)
    candidate = json.loads((directory / 'candidate.json').read_text())
    if (not isinstance(candidate, dict)
            or candidate.get('format') != 'decksmith-release-candidate'
            or type(candidate.get('version')) is not int or candidate['version'] != 1
            or candidate.get('source_commit') != source
            or candidate.get('files') != hashes
            or candidate.get('target', 'fedora44') != target):
        raise ValueError('Candidate metadata does not match the accepted source/downloads')
    # Import only after all six subjects have passed signature verification.
    spec = importlib.util.spec_from_file_location(
        'verified_package_io', directory / 'package_io.py')
    package = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(package)
    manifest, files = package.release_files(directory / downloads[0])
    if (manifest.get('source_commit') != source
            or manifest.get('source_dirty') is not False
            or manifest.get('platform') != 'Linux'
            or manifest.get('architecture') != 'x86_64'
            or manifest.get('tested_distribution') != distribution
            or manifest.get('id') != candidate.get('bundle')):
        raise ValueError('Archive metadata does not match the accepted candidate')
    for name, member in [('decksmith-install.py', 'scripts/decksmith-install.py'),
                         ('package_io.py', 'scripts/package_io.py'),
                         ('INSTALL.md', 'docs/installation.md')]:
        if (directory / name).read_bytes() != files[member]:
            raise ValueError('Standalone download differs from archive: ' + name)
    return {'verified': manifest['id'], 'source_commit': source,
            'target': target, 'archive_sha256': hashes[downloads[0]],
            'manifest_files': len(manifest['files'])}


def verify_install(directory, source, *, bundle=None, install=False, stage_root=None, target='fedora44'):
    _, workflow, downloads, subjects, limits = target_config(target)
    if not re.fullmatch(r'[0-9a-f]{40}', source):
        raise ValueError('Supply the full accepted public source commit, not a branch/tag')
    if stage_root is not None and not install:
        raise ValueError('--stage-root requires --install')
    if install and stage_root is None and os.geteuid() == 0:
        raise ValueError('Install as your regular desktop user, not root')
    with tempfile.TemporaryDirectory(prefix='decksmith-verified-') as temporary:
        snapshot = Path(temporary)
        for name in subjects:
            snapshot_file(Path(directory) / name, snapshot / name, limits[name])
        signing_bundle = None
        if bundle is not None:
            signing_bundle = snapshot / 'attestation.json'
            snapshot_file(Path(bundle), signing_bundle, 8 * 1024 * 1024)
        for name in subjects:
            command = ['gh', 'attestation', 'verify', str(snapshot / name),
                       '--repo', REPOSITORY, '--signer-workflow', workflow,
                       '--source-ref', 'refs/heads/main', '--source-digest', source,
                       '--deny-self-hosted-runners']
            if signing_bundle is not None:
                command += ['--bundle', str(signing_bundle)]
            subprocess.run(command, check=True, timeout=90)
        result = verified_metadata(snapshot, source, target)
        result['installed'] = False
        if install:
            # Ignore PYTHONPATH/user modules; only the signed snapshot supplies
            # the installer's companion module. Never forward --activate.
            command = [sys.executable, '-I', '-B', '-c',
                       'import runpy,sys; sys.path.insert(0,sys.argv[1]); '
                       'sys.argv=sys.argv[2:]; '
                       'runpy.run_path(sys.argv[0],run_name="__main__")',
                       str(snapshot), str(snapshot / 'decksmith-install.py')]
            if stage_root is not None:
                command += ['--stage-root', str(Path(stage_root).absolute())]
            command += ['install', str(snapshot / downloads[0])]
            subprocess.run(command, check=True, timeout=120)
            result.update(installed=True, staged=stage_root is not None,
                          startup='unchanged; controls not activated')
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path, help='Directory of accepted signed downloads')
    parser.add_argument('--target', choices=TARGETS, default='fedora44')
    parser.add_argument('--source-sha', required=True,
                        help='Full expected public source commit from release evidence')
    parser.add_argument('--bundle', type=Path, help='Retained signing bundle; trust roots may need network')
    parser.add_argument('--install', action='store_true', help='Install after verification; never activate controls')
    parser.add_argument('--stage-root', type=Path, help='Isolated install; never manage host services')
    args = parser.parse_args()
    result = verify_install(args.directory, args.source_sha, bundle=args.bundle,
                            install=args.install, stage_root=args.stage_root, target=args.target)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print('Verification/installation stopped: ' + str(error), file=sys.stderr)
        sys.exit(1)
