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


def prepare(archive, output, expected_source):
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
            manifest.get('tested_distribution') != 'Fedora 44'):
        raise ValueError('Candidate must target Fedora 44 Linux x86_64.')
    companions = {'decksmith-install.py': 'scripts/decksmith-install.py',
                  'package_io.py': 'scripts/package_io.py',
                  'INSTALL.md': 'docs/installation.md'}
    downloads = {SUBJECTS[0]: archive_bytes}
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
            digest(downloads[name]) + '  ' + name + '\n' for name in SUBJECTS).encode())
        atomic(staged / 'candidate.json', json.dumps({
            'format': 'decksmith-release-candidate', 'version': 1,
            'bundle': manifest['id'], 'source_commit': expected_source,
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
    args = parser.parse_args()
    try:
        print(prepare(args.archive, args.output, args.expected_source))
    except (ValueError, OSError) as error:
        parser.exit(1, str(error) + '\n')
