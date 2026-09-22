import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('runner',ROOT/'plugins/openhomeb/runtime/managed_runner.py')
runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner)

class Runtime(unittest.TestCase):
    def test_stale_socket_cleanup_and_live_socket_protection(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);path=root/'decksmith-plugin-lab.sock'
            with socket.socket(socket.AF_UNIX) as server:
                server.bind(str(path));path.chmod(0o600);server.listen()
                with self.assertRaises(RuntimeError):runner.clean_stale(root)
                self.assertTrue(path.exists())
            runner.clean_stale(root);self.assertFalse(path.exists())
    def test_cleanup_refuses_symlink(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);target=root/'keep';target.write_text('keep')
            (root/'decksmith-plugin-state.json').symlink_to(target)
            with self.assertRaises(RuntimeError):runner.clean_stale(root)
            self.assertEqual(target.read_text(),'keep')
    def test_packaged_python_dependency_imports_without_venv(self):
        import subprocess
        release=json.loads((ROOT/'local/plugin-runtime-stage.json').read_text())['release']
        vendor=ROOT/'dist/plugin-runtime'/release/'vendor'
        subprocess.run(['/usr/bin/python3','-I','-c',f'import sys;sys.path.insert(0,{str(vendor)!r});from websockets.asyncio.server import serve'],check=True)

if __name__=='__main__':unittest.main()

class Installation(unittest.TestCase):
    def test_failed_activation_restores_previous_integration(self):
        from unittest.mock import patch
        spec=importlib.util.spec_from_file_location('installer',ROOT/'scripts/install-plugin-runtime.py')
        installer=importlib.util.module_from_spec(spec);spec.loader.exec_module(installer)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);home=root/'home'
            def release(name):
                path=root/name;path.mkdir();(path/'runtime.json').write_text(json.dumps({'id':name,'files':{}}));return path
            installer.install(release('old'),home,False)
            unit=home/'.config/systemd/user/decksmith-openhomeb.service';prior=unit.read_text()
            with patch.object(installer.subprocess,'run'), patch('time.sleep'), patch.dict(os.environ,{'XDG_RUNTIME_DIR':str(root/'runtime')}):
                with self.assertRaises(RuntimeError):installer.install(release('new'),home,True)
            self.assertEqual(unit.read_text(),prior)
            self.assertEqual((home/'.local/share/decksmith/plugin-runtime/current').resolve().name,'old')
