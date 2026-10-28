import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock

from import_file import read_limited

class ImportFileTests(unittest.TestCase):
    def test_size_limit_includes_exact_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'import'
            path.write_bytes(b'1234')
            self.assertEqual(read_limited(path,4,'too large'),b'1234')
            path.write_bytes(b'12345')
            with self.assertRaisesRegex(ValueError,'too large'):
                read_limited(path,4,'too large')

    def test_read_itself_is_bounded(self):
        path=MagicMock()
        source=path.open.return_value.__enter__.return_value
        source.read.return_value=b'12345'
        with self.assertRaisesRegex(ValueError,'too large'):
            read_limited(path,4,'too large')
        source.read.assert_called_once_with(5)
