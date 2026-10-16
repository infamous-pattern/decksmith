"""Verify the editor recovery command reaches only the installed normal unit."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class ServiceScriptTests(unittest.TestCase):
    def command(self, packaged, action):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'scripts').mkdir()
            script = root / 'scripts/decksmith-service.sh'
            script.write_bytes((ROOT / 'scripts/decksmith-service.sh').read_bytes())
            if packaged:
                (root / 'release.json').write_text('{}')
            executable = root / 'fake-bin'
            executable.mkdir()
            log = root / 'calls'
            fake = executable / 'systemctl'
            fake.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$SERVICE_TEST_LOG"\n')
            fake.chmod(0o700)
            env = dict(os.environ, PATH=str(executable)+os.pathsep+os.environ['PATH'], SERVICE_TEST_LOG=str(log))
            result = subprocess.run(['/bin/bash', str(script), action], env=env, capture_output=True, text=True, timeout=5)
            return result, log.read_text().splitlines() if log.exists() else []

    def test_packaged_restart_addresses_normal_service(self):
        result, calls = self.command(True, 'restart')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, ['--user', 'restart', 'decksmith.service'])

    def test_development_restart_does_not_spawn_another_daemon(self):
        result, calls = self.command(False, 'restart')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, ['--user', 'restart', 'decksmith.service'])

    def test_unknown_action_has_no_service_effect(self):
        for packaged in (True, False):
            result, calls = self.command(packaged, 'unknown')
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(calls, [])

if __name__ == '__main__':
    unittest.main()
