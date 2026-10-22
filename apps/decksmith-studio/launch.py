"""Desktop launch helper, called with an action and one literal argument."""
import sys,os
from urllib.parse import urlsplit
import gi
gi.require_version('Gio', '2.0')
from gi.repository import Gio,GLib

def independent_child(*_):
    try:
        os.setsid()
        # A launched application outlives this helper, but must never read its
        # requests or write into its JSON reply channel (including via stderr).
        null_fd = os.open(os.devnull, os.O_RDWR)
        for fd in (0, 1, 2):
            os.dup2(null_fd, fd)
        if null_fd > 2:
            os.close(null_fd)
    except OSError:os._exit(126)  # Do not exec in our helper's group if isolation fails.


def detached_launch(app,uris):
    # A user-launched application must outlive our bounded helper. GIO still
    # expands the desktop entry; only the child's session differs from legacy.
    if not hasattr(app,'launch_uris_as_manager'):raise ValueError('Application unavailable')
    return app.launch_uris_as_manager(uris,None,GLib.SpawnFlags.SEARCH_PATH,
                                     independent_child,(),None,None)


def launch(kind, value, player=None, *, authorize=None):
    if kind == 'media':
        from media import execute
        execute(value,target=player)
    elif kind == 'application':
        if '/' in value or not value.endswith('.desktop'):
            raise ValueError('Invalid application ID')
        app = Gio.DesktopAppInfo.new(value)
        if app is None or not app.should_show():
            raise ValueError('Application unavailable')
        if authorize is None:
            success=app.launch([],None)
        else:
            if not hasattr(app,'launch_uris_as_manager'):raise ValueError('Application unavailable')
            authorize()
            success=detached_launch(app,[])
        if not success:raise RuntimeError('Launch request failed')
    elif kind == 'website':
        url = urlsplit(value)
        if url.scheme not in ('http','https') or not url.hostname or any(c.isspace() for c in value):
            raise ValueError('Enter an http or https URL')
        # Resolve the handler before granting the URI launch; no discovery after
        # the grant. Keep the normal path's default-handler behavior.
        if authorize is None:
            if not Gio.AppInfo.launch_default_for_uri(value, None):raise RuntimeError('Launch request failed')
        else:
            app = Gio.AppInfo.get_default_for_uri_scheme(url.scheme)
            if app is None or not hasattr(app,'launch_uris_as_manager'):raise ValueError('Application unavailable')
            authorize()
            if not detached_launch(app,[value]):raise RuntimeError('Launch request failed')
    else:
        raise ValueError('Unknown launch action')

if __name__ == '__main__':
    try:
        launch(*sys.argv[1:])
    except Exception as error:
        sys.exit({'No running media player':2,'The active player does not support this action':3,'Application unavailable':4}.get(str(error),1))
