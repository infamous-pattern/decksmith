#!/usr/bin/env python3
"""Extract marked UI text or validate/compile explicitly enabled gettext catalogs."""
import argparse
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def compiled_catalogs(root=ROOT):
    """Return bundle-relative paths and bytes; never write into the source tree."""
    root = Path(root)
    languages = []
    for line in (root / 'po/LINGUAS').read_text().splitlines():
        languages.extend(line.split('#', 1)[0].split())
    if len(languages) != len(set(languages)):
        raise ValueError('Duplicate language in po/LINGUAS')
    files = {}
    with tempfile.TemporaryDirectory() as directory:
        for language in languages:
            if not re.fullmatch(r'[a-z]{2,3}(?:_[A-Z]{2})?(?:@[a-z]+)?', language):
                raise ValueError('Invalid language in po/LINGUAS')
            source = root / 'po' / (language + '.po')
            if not source.is_file():
                raise ValueError('Missing translation catalog: ' + language)
            destination = Path(directory) / (language + '.mo')
            subprocess.run(['msgfmt', '--check', '--check-format', '-o', str(destination), str(source)], check=True)
            files[f'locale/{language}/LC_MESSAGES/decksmith.mo'] = destination.read_bytes()
    return files

def extract(root=ROOT):
    subprocess.run(['xgettext', '--language=Python', '--from-code=UTF-8',
                    '--keyword=tr', '--keyword=ngettext:1,2', '--keyword=pgettext:1c,2',
                    '--keyword=npgettext:1c,2,3', '--add-comments=Translators:',
                    '--package-name=Decksmith', '--package-version=0.1.0',
                    '--copyright-holder=Decksmith contributors',
                    '--files-from=po/POTFILES.in', '--output=po/decksmith.pot'],
                   cwd=root, check=True)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['extract', 'check'])
    args = parser.parse_args()
    if args.operation == 'extract': extract()
    else: print(f'Validated {len(compiled_catalogs())} enabled catalogs')
