#!/usr/bin/env python3
"""Stage verified release downloads; never installs, signs or publishes them."""
import argparse
import json
from pathlib import Path
import re
import tempfile

from package_io import LIMIT, digest, release_files, atomic

SUBJECTS = ('decksmith-linux-x86_64.tar.gz', 'decksmith-install.py',
            'package_io.py', 'INSTALL.md')


TARGETS = {
    'fedora44': ('Fedora 44', SUBJECTS[0]),
    'debian13': ('Debian 13', 'decksmith-debian13-x86_64.tar.gz'),
    'ubuntu2604': ('Ubuntu 26.04', 'decksmith-ubuntu2604-x86_64.tar.gz'),
}


def prepare(archive, output, expected_source, *, target='fedora44'):
    if target not in TARGETS:
        raise ValueError('Unknown candidate target')
    distribution, archive_name = TARGETS[target]
    subjects = (archive_name,) + SUBJECTS[1:]
    if not re.fullmatch(r'[0-9a-f]{40}', expected_source):
        raise ValueError('Expected source must be a full Git commit SHA.')
    archive, output = Path(archive), Path(output)
    if output.exists():
        raise ValueError('Candidate destination already exists; use a new directory.')
    # Verify the same immutable bytes we will stage, even if the input path is
    # replaced during validation. The private snapshot is never executed.
    with archive.open('rb') as stream:
        archive_bytes = stream.read(LIMIT + 1)
    if len(archive_bytes) > LIMIT:
        raise ValueError('Candidate archive exceeds the package size limit.')
    with tempfile.TemporaryDirectory(prefix='decksmith-candidate-validation-') as temporary:
        snapshot = Path(temporary) / 'bundle.tar.gz'
        snapshot.write_bytes(archive_bytes)
        manifest, files = release_files(snapshot)
    if manifest.get('source_commit') != expected_source:
        raise ValueError('Bundle source does not match the expected commit.')
    if manifest.get('source_dirty') is not False:
        raise ValueError('Release candidates require a clean source tree.')
    if (manifest.get('platform') != 'Linux' or
            manifest.get('architecture') != 'x86_64' or
            manifest.get('tested_distribution') != distribution):
        raise ValueError('Candidate must target ' + distribution + ' Linux x86_64.')
    companions = {'decksmith-install.py': 'scripts/decksmith-install.py',
                  'package_io.py': 'scripts/package_io.py',
                  'INSTALL.md': 'docs/installation.md'}
    downloads = {archive_name: archive_bytes}
    for name, member in companions.items():
        path = archive.parent / name
        if path.is_symlink() or path.read_bytes() != files.get(member):
            raise ValueError('Bootstrap file differs from the verified bundle: ' + name)
        downloads[name] = files[member]
    # Build privately beside the destination. Validation failures never leave a
    # partly staged candidate; existing output is never replaced.
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.decksmith-candidate-', dir=output.parent) as temporary:
        staged = Path(temporary) / 'downloads'
        staged.mkdir()
        for name, data in downloads.items():
            atomic(staged / name, data)
        atomic(staged / 'SHA256SUMS', ''.join(
            digest(downloads[name]) + '  ' + name + '\n' for name in subjects).encode())
        atomic(staged / 'candidate.json', json.dumps({
            'format': 'decksmith-release-candidate', 'version': 1,
            'bundle': manifest['id'], 'target': target, 'source_commit': expected_source,
            'files': {name: digest(data) for name, data in downloads.items()},
            'qualification': 'pending; staging is not release acceptance',
        }, indent=2, sort_keys=True).encode())
        staged.rename(output)
    return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--expected-source', required=True)
    parser.add_argument('--target', choices=TARGETS, default='fedora44')
    args = parser.parse_args()
    try:
        print(prepare(args.archive, args.output, args.expected_source, target=args.target))
    except (ValueError, OSError) as error:
        parser.exit(1, str(error) + '\n')
