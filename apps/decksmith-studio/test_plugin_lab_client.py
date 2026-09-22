import json
import tempfile
import unittest
from pathlib import Path
from plugin_lab_client import LabClient


class ConnectionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.path = Path(self.temp.name) / 'connection.json'
        self.data = {'version': 1, 'url': 'http://127.0.0.1:43210', 'token': 'a' * 43,
                     'provider': 'OpenHomeB', 'target': 'Main_LED’s'}

    def tearDown(self):
        self.temp.cleanup()

    def write(self):
        self.path.write_text(json.dumps(self.data));self.path.chmod(0o600)

    def test_private_local_descriptor(self):
        self.write()
        self.assertEqual(LabClient(self.path).url, self.data['url'])

    def test_foreign_destinations_and_credentials_rejected(self):
        for url in ('http://192.0.2.19:8581', 'http://localhost:1234',
                    'http://secret@127.0.0.1:1234', 'http://127.0.0.1:1234/redirect',
                    'http://127.0.0.1:1234?token=abc'):
            self.data['url'] = url;self.write()
            with self.assertRaises(ValueError):LabClient(self.path)

    def test_shared_file_and_symlink_rejected(self):
        self.write();self.path.chmod(0o644)
        with self.assertRaises(ValueError):LabClient(self.path)
        self.path.chmod(0o600)
        link = self.path.with_name('link');link.symlink_to(self.path)
        with self.assertRaises(OSError):LabClient(link)

    def test_unapproved_command_rejected_before_network(self):
        self.write()
        with self.assertRaises(ValueError):LabClient(self.path).request('toggle-all')

    def test_existing_client_uses_rotated_endpoint_and_token(self):
        import io
        from unittest.mock import patch
        self.write();client=LabClient(self.path)
        self.data['url']='http://127.0.0.1:5678';self.data['token']='B'*40;self.write()
        with patch.object(client.opener,'open',return_value=io.BytesIO(b'{"ok":true}')) as opened:
            self.assertTrue(client.request('off')['ok'])
        request=opened.call_args.args[0]
        self.assertEqual(request.full_url,'http://127.0.0.1:5678/command')
        self.assertEqual(request.get_header('X-lab-token'),'B'*40)
