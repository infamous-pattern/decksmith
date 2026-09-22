"""Local gettext catalogs; loaded once, with English as the source fallback.

Only presentation text belongs here. Never translate action IDs, D-Bus values,
configuration keys, error codes, or labels authored by the user.
"""
import gettext as _gettext
import struct
from pathlib import Path

DOMAIN = 'decksmith'
LOCALE_DIR = Path(__file__).resolve().parents[2] / 'locale'

def load(localedir=LOCALE_DIR, languages=None):
    """Use the session language when unspecified; restart to change language."""
    try:
        return _gettext.translation(DOMAIN, localedir=localedir,
                                    languages=languages, fallback=True)
    except (OSError, ValueError, UnicodeError, EOFError, struct.error):
        # A damaged optional catalog must not prevent controls from opening.
        return _gettext.NullTranslations()

_translation = load()
gettext = _translation.gettext
ngettext = _translation.ngettext
pgettext = _translation.pgettext
npgettext = _translation.npgettext
