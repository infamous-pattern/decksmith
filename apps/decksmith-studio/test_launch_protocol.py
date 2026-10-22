"""Real GIO child launches against disposable desktop entries, never user apps."""
import json
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile
import time
import unittest


STUDIO = Path(__file__).resolve().parent
HARNESS = r'''
import os, sys, time
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, sys.argv[1])
import launch
import shared_builtin
from system_controls import Controls
app = launch.Gio.DesktopAppInfo.new('org.decksmith.ProtocolFixture.desktop')
assert app is not None
# Simulate parent/library output without invoking any real system control.
original_init = Controls.__init__
def initialize(self):
    print('PARENT_STARTUP_NOISE', flush=True)
    os.write(1, b'PARENT_STARTUP_FD_NOISE\n')
    original_init(self)
def control(self, command):
    self.authorize()
    print('PARENT_PRINT_NOISE', flush=True)
    os.write(1, b'PARENT_FD_NOISE\n')
with patch.object(launch.Gio.AppInfo, 'get_default_for_uri_scheme', return_value=app), \
     patch.object(Controls, '__init__', initialize), \
     patch.object(Controls, 'execute', control):
    shared_builtin.main()
'''
CHILD = r'''
import json, os, sys, time
from pathlib import Path
root = Path(__file__).parent
print('CHILD_PRINT_NOISE', flush=True)
os.write(1, b'{"status":"ready"}\n')
os.write(2, b'CHILD_STDERR_NOISE\n')
details = {'pid': os.getpid(), 'sid': os.getsid(0), 'pgid': os.getpgrp(),
           'argv': sys.argv[1:],
           'fds': [os.readlink('/proc/self/fd/' + str(fd)) for fd in (0, 1, 2)],
           'extra_pipes': [str(p) for p in Path('/proc/self/fd').iterdir()
                           if int(p.name) > 2 and p.exists()
                           and os.readlink(p).startswith('pipe:')]}
(root / 'started.tmp').write_text(json.dumps(details))
(root / 'started.tmp').replace(root / 'started.json')
# Parent test releases this child only after the helper process group is gone.
deadline = time.monotonic() + 10
while not (root / 'release').exists() and time.monotonic() < deadline:
    time.sleep(.01)
(root / 'finished').write_text('finished')
'''


class LaunchProtocolTests(unittest.TestCase):
    def read_reply(self, process):
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            self.assertTrue(selector.select(3), 'helper reply timed out')
        return json.loads(process.stdout.readline())

    def wait_file(self, path):
        deadline = time.monotonic() + 3
        while not path.exists() and time.monotonic() < deadline:
            time.sleep(.01)
        self.assertTrue(path.exists(), 'fixture did not reach ' + path.name)

    def exercise(self, kind):
        with tempfile.TemporaryDirectory(prefix='decksmith-launch-') as directory:
            root = Path(directory)
            apps = root / 'applications'
            apps.mkdir()
            script = root / 'child.py'
            script.write_text(CHILD)
            (apps / 'org.decksmith.ProtocolFixture.desktop').write_text(
                '[Desktop Entry]\nType=Application\nName=Protocol fixture\n'
                f'Exec={sys.executable} "{script}" %u\nTerminal=false\n')
            env = dict(os.environ, XDG_DATA_HOME=directory, XDG_DATA_DIRS=directory,
                       XDG_CONFIG_HOME=directory, PYTHONDONTWRITEBYTECODE='1')
            process = subprocess.Popen([sys.executable, '-B', '-u', '-c', HARNESS, str(STUDIO)],
                                       env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True, bufsize=1,
                                       start_new_session=True)
            child_pid = None
            try:
                action = ({'type': 'open_application', 'desktop_id': 'org.decksmith.ProtocolFixture.desktop'}
                          if kind == 'application' else
                          {'type': 'open_website', 'url': 'https://example.invalid/?a=1&b=2'})
                process.stdin.write(json.dumps(action) + '\n'); process.stdin.flush()
                self.assertEqual(self.read_reply(process), {'status': 'ready'})
                process.stdin.write('{"decision":"apply"}\n'); process.stdin.flush()
                self.assertEqual(self.read_reply(process), {'status': 'ok'})
                self.wait_file(root / 'started.json')
                details = json.loads((root / 'started.json').read_text())
                child_pid = details['pid']
                self.assertEqual(details['sid'], child_pid)
                self.assertEqual(details['pgid'], child_pid)
                self.assertEqual(details['fds'], ['/dev/null'] * 3)
                self.assertEqual(details['extra_pipes'], [])
                self.assertEqual(details['argv'], [] if kind == 'application' else [action['url']])
                # A later ordinary action still gets its own grant and reply;
                # neither child output nor parent library output can masquerade as one.
                process.stdin.write('{"type":"system","command":"dnd"}\n'); process.stdin.flush()
                self.assertEqual(self.read_reply(process), {'status': 'ready'})
                process.stdin.write('{"decision":"apply"}\n'); process.stdin.flush()
                self.assertEqual(self.read_reply(process), {'status': 'ok'})
                # Match Connection::drop: killing the helper group must not kill
                # the launched user app or leave its stdout holding our pipe open.
                os.killpg(process.pid, 9)
                process.wait(timeout=3)
                with selectors.DefaultSelector() as selector:
                    selector.register(process.stdout, selectors.EVENT_READ)
                    self.assertTrue(selector.select(3), 'launched child kept helper pipe open')
                remaining = process.stdout.readline()
                self.assertEqual(remaining, '')
                self.assertFalse((root / 'finished').exists())
                (root / 'release').touch()
                self.wait_file(root / 'finished')
            finally:
                (root / 'release').touch()
                if process.poll() is None:
                    os.killpg(process.pid, 9)
                process.wait(timeout=3)
                for stream in (process.stdin, process.stdout, process.stderr):
                    stream.close()
                if child_pid is not None:
                    self.wait_file(root / 'finished')

    def test_application_output_cannot_poison_later_builtin_and_child_outlives_helper(self):
        self.exercise('application')

    def test_website_output_cannot_poison_later_builtin_and_child_outlives_helper(self):
        self.exercise('website')


if __name__ == '__main__':
    unittest.main()
