"""Exercise the exact verifier embedded in the downloadable shell bootstrap."""
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT=Path(__file__).with_name('install.sh').read_text()
VERIFY=SCRIPT.split("<<'PYVERIFY'\n",1)[1].split('\nPYVERIFY',1)[0]

class BootstrapIntegrity(unittest.TestCase):
    def test_complete_manifest_is_required_before_execution(self):
        names=('decksmith-linux-x86_64.tar.gz','decksmith-install.py','package_io.py','INSTALL.md')
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for name in names:(root/name).write_bytes(b'trusted fixture')
            # Isolated Python must not import a module from the download directory.
            (root/'hashlib.py').write_text('raise RuntimeError("untrusted import")')
            rows=[hashlib.sha256((root/n).read_bytes()).hexdigest()+'  '+n for n in names]
            cases=[(rows,True),(rows[:1],False),(rows+[rows[0]],False),([],False),
                   (rows[:-1]+['0'*64+'  INSTALL.md'],False),
                   (rows+['0'*64+'  ../escape'],False),(['x'*4097],False)]
            for lines,valid in cases:
                (root/'SHA256SUMS').write_text('\n'.join(lines)+'\n')
                result=subprocess.run([sys.executable,'-I','-c',VERIFY],cwd=root,capture_output=True,text=True)
                self.assertEqual(result.returncode==0,valid,result.stderr)
