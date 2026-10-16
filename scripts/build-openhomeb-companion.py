#!/usr/bin/env python3
"""Rebuild the pinned OpenHomeB adaptation in a new directory; never install it.

The input must be the reviewed adaptation, not the upstream release source.
All source, dependency, test and binary receipts remain in the output directory.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import tomllib

PATCHES = {'rustls': '0.23.46', 'quinn-proto': '0.11.18'}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inventory(root):
    return {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob('*'))
            if p.is_file() and not any(x in p.relative_to(root).parts
                                      for x in ('target', '.git', '__pycache__'))}


def build(source, output):
    source, output = source.resolve(), output.absolute()
    if output.exists() or output.is_relative_to(source):
        raise ValueError('Choose a new output directory outside the preserved source')
    before = inventory(source)
    output.mkdir(parents=True, mode=0o700)
    copied = output/'source'
    shutil.copytree(source, copied, ignore=shutil.ignore_patterns('target', '.git', '__pycache__'))
    if inventory(copied) != before:
        raise ValueError('Source copy verification failed')
    logs = output/'verification'; logs.mkdir()
    (logs/'input-source-sha256.json').write_text(json.dumps(before, indent=2)+'\n')
    status = {'status': 'running', 'started': time.time(), 'steps': [], 'installed': False}
    def save():
        (logs/'result.json').write_text(json.dumps(status, indent=2)+'\n')
    env = dict(os.environ)
    env['CARGO_TARGET_DIR'] = str(output/'target')
    env['CARGO_PROFILE_RELEASE_STRIP'] = 'symbols'
    env['CARGO_PROFILE_RELEASE_DEBUG'] = '0'
    env['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join([
        '--remap-path-prefix='+str(Path.home())+'=/build/home',
        '--remap-path-prefix='+str(output)+'=/build/openhomeb-build'])
    def run(name, args):
        status['phase'] = name; save()
        with (logs/(name+'.log')).open('w') as log:
            result = subprocess.run(args, cwd=copied, env=env, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=1200)
        status['steps'].append({'name': name, 'exit_status': result.returncode})
        save()
        if result.returncode:
            raise RuntimeError(name+' failed; inspect its retained log')
    try:
        for name, version in PATCHES.items():
            run('update-'+name, ['cargo', 'update', '-p', name, '--precise', version])
        lock = tomllib.loads((copied/'Cargo.lock').read_text())
        versions = {p['name']: p['version'] for p in lock['package']}
        for name, version in PATCHES.items():
            if versions.get(name) != version:
                raise ValueError('Patched lock version not selected: '+name)
        run('fmt', ['cargo', 'fmt', '--all', '--', '--check'])
        run('clippy', ['cargo', 'clippy', '--all-targets', '--locked', '--', '-D', 'warnings'])
        run('rust-tests', ['cargo', 'test', '--locked'])
        run('release-build', ['cargo', 'build', '--release', '--locked'])
        binary = output/'binaries/openhomeb'; binary.parent.mkdir()
        shutil.copy2(output/'target/release/openhomeb', binary)
        subprocess.run(['strip', '--strip-all', str(binary)], check=True)
        raw = binary.read_bytes()
        if str(Path.home()).encode() in raw or str(output).encode() in raw:
            raise ValueError('Private build path remains in executable')
        after = inventory(copied)
        changed = {p: h for p, h in after.items() if before.get(p) != h}
        if set(changed) != {'Cargo.lock'}:
            raise ValueError('Unexpected source change beyond Cargo.lock')
        if inventory(source) != before:
            raise ValueError('Preserved input source changed during build')
        receipt = {'version': lock['package'][next(i for i,p in enumerate(lock['package']) if p['name']=='openhomeb')]['version'],
                   'build_kind': 'dependency-hardening',
                   'binary_sha256': digest(binary),
                   'changed_files': changed,
                   'source_files': after,
                   'dependency_patches': PATCHES,
                   'adapter_contract': 'brightness-completion-value-v1+host-session-v1',
                   'path_remapped': True, 'stripped': True}
        (logs/'build-receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        status.update(status='passed', finished=time.time(), receipt='build-receipt.json')
        save()
        return receipt
    except BaseException as error:
        status.update(status='failed', error=str(error), finished=time.time())
        save()
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(build(args.source, args.output), indent=2))
