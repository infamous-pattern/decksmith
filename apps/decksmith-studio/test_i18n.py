"""Locale infrastructure tests use temporary catalogs, never shipping translations."""
import gettext
import importlib.util
import os
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from i18n import load

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('catalog_tools', ROOT / 'scripts/localization.py')
tools = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tools)

PO = '''msgid ""
msgstr ""
"Project-Id-Version: Decksmith test\\n"
"PO-Revision-Date: 2026-09-17 00:00+0000\\n"
"Last-Translator: Test\\n"
"Language-Team: Test\\n"
"Language: fr\\n"
"MIME-Version: 1.0\\n"
"Content-Type: text/plain; charset=UTF-8\\n"
"Content-Transfer-Encoding: 8bit\\n"
"Plural-Forms: nplurals=2; plural=(n > 1);\\n"

msgid "Home"
msgstr "Accueil"

msgid "Screen"
msgstr "Écran"

msgctxt "control"
msgid "Key"
msgstr "Touche"

msgid "One page"
msgid_plural "Many pages"
msgstr[0] "Une page"
msgstr[1] "Plusieurs pages"
'''

class LocalizationTests(unittest.TestCase):
    def test_missing_catalog_is_english_with_context_and_plurals(self):
        with tempfile.TemporaryDirectory() as root:
            t=load(root,['de'])
            self.assertEqual(t.gettext('Home'),'Home')
            self.assertEqual(t.pgettext('control','Key'),'Key')
            self.assertEqual(t.ngettext('One page','Many pages',2),'Many pages')

    def test_corrupt_catalog_falls_back_safely(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'de/LC_MESSAGES/decksmith.mo';p.parent.mkdir(parents=True)
            p.write_bytes(b'not a catalog')
            self.assertEqual(load(root,['de']).gettext('Home'),'Home')

    @unittest.skipUnless(shutil.which('msgfmt'),'gettext build tools not installed')
    def test_packaged_catalog_locale_fallback_context_plural_and_unknown_text(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root);(root/'po').mkdir()
            (root/'po/LINGUAS').write_text('fr\n')
            (root/'po/fr.po').write_text(PO)
            for path,data in tools.compiled_catalogs(root).items():
                p=root/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
            # Use a regional locale without its own catalog, from the session environment.
            with patch.dict(os.environ,{'LANGUAGE':'fr_FR','LC_ALL':'fr_FR.UTF-8'}):
                t=load(root/'locale')
            self.assertEqual(t.gettext('Home'),'Accueil')
            self.assertEqual(t.gettext('Screen'),'Écran')
            self.assertEqual(t.pgettext('control','Key'),'Touche')
            self.assertEqual(t.ngettext('One page','Many pages',2),'Plusieurs pages')
            self.assertEqual(t.gettext('Untranslated'),'Untranslated')

    def test_empty_shipping_catalog_set_needs_no_compiler(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root);(root/'po').mkdir();(root/'po/LINGUAS').write_text('# English only\n')
            with patch.object(tools.subprocess,'run',side_effect=AssertionError('No tool needed')):
                self.assertEqual(tools.compiled_catalogs(root),{})

    def test_language_paths_are_bounded(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root);(root/'po').mkdir();(root/'po/LINGUAS').write_text('../fr\n')
            with self.assertRaises(ValueError):tools.compiled_catalogs(root)
