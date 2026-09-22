"""Desktop launch helper, called with an action and one literal argument."""
import sys
from urllib.parse import urlsplit
import gi
gi.require_version('Gio', '2.0')
from gi.repository import Gio

def launch(kind, value, player=None):
    if kind == 'media':
        from media import execute
        execute(value,target=player)
    elif kind == 'application':
        if '/' in value or not value.endswith('.desktop'):
            raise ValueError('Invalid application ID')
        app = Gio.DesktopAppInfo.new(value)
        if app is None or not app.should_show():
            raise ValueError('Application unavailable')
        if not app.launch([], None):raise RuntimeError('Launch request failed')
    elif kind == 'website':
        url = urlsplit(value)
        if url.scheme not in ('http','https') or not url.hostname or any(c.isspace() for c in value):
            raise ValueError('Enter an http or https URL')
        if not Gio.AppInfo.launch_default_for_uri(value, None):raise RuntimeError('Launch request failed')
    else:
        raise ValueError('Unknown launch action')

if __name__ == '__main__':
    try:
        launch(*sys.argv[1:])
    except Exception as error:
        sys.exit({'No running media player':2,'The active player does not support this action':3,'Application unavailable':4}.get(str(error),1))
