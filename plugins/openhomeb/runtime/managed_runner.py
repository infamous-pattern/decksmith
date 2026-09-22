"""Single-instance supervised entry point; startup and recovery never replay input."""
import asyncio
import fcntl
import json
import os
from pathlib import Path
import socket
import stat
import sys
import urllib.request
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'vendor'))


def private(path, kind):
    info = path.lstat()
    if info.st_uid != os.getuid() or info.st_mode & 0o077 or not kind(info.st_mode):
        raise RuntimeError('Unsafe existing runtime artifact: ' + path.name)


def clean_stale(runtime):
    sock = runtime / 'decksmith-plugin-lab.sock'
    if sock.exists():
        private(sock, stat.S_ISSOCK)
        with socket.socket(socket.AF_UNIX) as probe:
            probe.settimeout(.5)
            try:
                probe.connect(str(sock))
            except (ConnectionRefusedError, FileNotFoundError):
                pass
            else:
                raise RuntimeError('Another plugin host is running')
    descriptor = runtime / 'decksmith-plugin-lab.json'
    if descriptor.exists():
        private(descriptor, stat.S_ISREG)
        if descriptor.stat().st_size > 4096:
            raise RuntimeError('Unexpected connection descriptor')
        data = json.loads(descriptor.read_text())
        url = urlsplit(data.get('url', ''))
        if (data.get('provider') != 'OpenHomeB' or url.scheme != 'http'
                or url.hostname != '127.0.0.1' or not url.port):
            raise RuntimeError('Unexpected connection descriptor')
        with socket.socket() as probe:
            probe.settimeout(.5)
            if probe.connect_ex(('127.0.0.1', url.port)) == 0:
                raise RuntimeError('Another plugin panel is running')
    paths = [sock, descriptor, runtime / 'decksmith-plugin-state.json',
             runtime / 'decksmith-plugin-state.tmp']
    for path in paths:
        if path.is_symlink():
            raise RuntimeError('Refusing a runtime symlink')
        if path.exists():
            private(path, stat.S_ISSOCK if path == sock else stat.S_ISREG)
    for path in paths:
        path.unlink(missing_ok=True)


def run():
    os.umask(0o077)
    runtime = Path(os.environ['XDG_RUNTIME_DIR'])
    fd = os.open(runtime / 'decksmith-openhomeb.lock', os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'w') as lock:
        private(runtime / 'decksmith-openhomeb.lock', stat.S_ISREG)
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        clean_stale(runtime)
        from live_panel import main
        asyncio.run(main(runtime / 'decksmith-plugin-lab.json', hardware=True, managed=True))


if __name__ == '__main__':
    run()
