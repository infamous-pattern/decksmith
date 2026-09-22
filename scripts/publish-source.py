#!/usr/bin/env python3
"""Export the canonical source tree to a separate public Git history."""
import argparse
import io
from pathlib import Path
import subprocess
import tarfile
import tempfile


def git(*args, cwd=None):
    return subprocess.check_output(['git', *args], cwd=cwd, text=True, timeout=60).strip()


def publish(push=False, github_only=False):
    root = Path(git('rev-parse', '--show-toplevel'))
    if git('status', '--porcelain', '--untracked-files=no'):
        raise RuntimeError('Commit tracked changes before exporting.')
    source = git('rev-parse', 'HEAD')
    tree = git('rev-parse', 'HEAD^{tree}')
    public_url = git('remote', 'get-url', 'github')
    if public_url not in ('https://github.com/infamous-pattern/decksmith.git',
                          'git@github.com:infamous-pattern/decksmith.git'):
        raise RuntimeError('Unexpected public destination.')
    marker = subprocess.run(['git', 'config', '--get', 'decksmith.publicTree'],
                            capture_output=True, text=True).stdout.strip()
    archive = subprocess.check_output(['git', 'archive', source])
    with tempfile.TemporaryDirectory(prefix='decksmith-public-') as temporary:
        checkout = Path(temporary) / 'source'
        subprocess.run(['git', 'clone', '--no-checkout', public_url, str(checkout)], check=True, timeout=60)
        existing = subprocess.run(['git', 'rev-parse', '--verify', 'HEAD'], cwd=checkout,
                                  capture_output=True, text=True).returncode == 0
        if existing:
            public_tree = git('rev-parse', 'HEAD^{tree}', cwd=checkout)
            if public_tree not in (marker, tree):
                raise RuntimeError('GitHub has unreviewed changes; reconcile them first.')
            git('read-tree', 'HEAD', cwd=checkout)
        else:
            git('symbolic-ref', 'HEAD', 'refs/heads/main', cwd=checkout)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(checkout, filter='data')
        git('add', '--all', cwd=checkout)
        if git('write-tree', cwd=checkout) != tree:
            raise RuntimeError('Exported source does not match the canonical tree.')
        for field in ('name', 'email'):
            git('config', 'user.' + field, git('config', 'user.' + field), cwd=checkout)
        if not existing or git('rev-parse', 'HEAD^{tree}', cwd=checkout) != tree:
            git('commit', '-m', git('log', '-1', '--format=%s'), cwd=checkout)
        public_commit = git('rev-parse', 'HEAD', cwd=checkout)
        print(f'Validated source tree {tree}; public commit {public_commit}', flush=True)
        if not push:
            print('Dry run: neither repository was pushed.')
            return
        if not github_only:
            subprocess.run(['git', 'push', 'origin', source + ':refs/heads/main'], cwd=root, check=True, timeout=60)
        subprocess.run(['git', 'push', 'origin', 'HEAD:refs/heads/main'], cwd=checkout, check=True, timeout=60)
        for remote, expected in ([('github', public_commit)] if github_only else [('origin', source), ('github', public_commit)]):
            actual = git('ls-remote', remote, 'refs/heads/main').split()[0]
            if actual != expected:
                raise RuntimeError(f'{remote} changed during publication; inspect before retrying.')
        git('config', 'decksmith.publicTree', tree)
        print('GitHub verified; Gitea synchronization is still pending.' if github_only else 'Both hosts verified: identical source trees, separate histories.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--publish', action='store_true')
    parser.add_argument('--github-only', action='store_true', help='Publish while Gitea is unavailable; synchronization remains pending')
    args = parser.parse_args()
    if args.github_only and not args.publish:parser.error('--github-only requires --publish')
    publish(args.publish, args.github_only)
