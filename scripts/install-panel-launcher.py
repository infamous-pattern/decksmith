#!/usr/bin/env python3
"""Install the panel under its GTK application ID and migrate our legacy shortcut."""
import os
import subprocess
from pathlib import Path
from gi.repository import Gio

ROOT = Path(__file__).resolve().parents[1]
APP_ID = 'cc.senecal.Decksmith.Studio'

def main():
    base = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share')))
    apps = base / 'applications'
    apps.mkdir(parents=True, exist_ok=True)
    target = apps / f'{APP_ID}.desktop'
    panel = str(ROOT / 'apps/decksmith-studio/panel.py')
    escaped = panel.replace('\\', '\\\\').replace('"', '\\"').replace('%', '%%')
    text = (ROOT / 'packaging/cc.senecal.Decksmith.Studio.desktop.in').read_text()
    text = text.replace('@PANEL@', escaped).replace('@ICON@',str(ROOT/'brand/decksmith-app.svg'))
    temporary = ROOT / 'local/cc.senecal.Decksmith.Studio.desktop'
    temporary.parent.mkdir(exist_ok=True)
    temporary.write_text(text)
    subprocess.run(['desktop-file-validate', str(temporary)], check=True)
    target.write_text(text)
    target.chmod(0o644)
    legacy = apps / 'decksmith.desktop'
    # Only retire the shortcut created for this checkout, never an unrelated file.
    if legacy.exists() and str(ROOT) in legacy.read_text():
        legacy.unlink()
    settings = Gio.Settings.new('org.gnome.shell')
    favorites = list(settings.get_strv('favorite-apps'))
    updated = [f'{APP_ID}.desktop' if item == 'decksmith.desktop' else item for item in favorites]
    if updated != favorites:
        settings.set_strv('favorite-apps', list(dict.fromkeys(updated)))
        Gio.Settings.sync()
    subprocess.run(['update-desktop-database', str(apps)], check=True)
    print('Installed Decksmith panel launcher.')

if __name__ == '__main__':
    main()
