"""Discover installed audio-capable desktop entries; never launch their commands."""
from pathlib import PurePath
from gi.repository import Gio

CATEGORIES={'Audio','Video','AudioVideo','WebBrowser','InstantMessaging','VideoConference','Telephony'}
# Recognition rules describe runtime binaries, not records of installed apps.
# A candidate becomes connected only when an actual stream matches its identity.
BINARIES={
    'com.brave.Browser':'brave','brave-browser':'brave',
    'google-chrome':'chrome','google-chrome-stable':'chrome',
    'org.chromium.Chromium':'chromium','chromium':'chromium',
    'org.mozilla.firefox':'firefox','firefox':'firefox',
    'com.discordapp.Discord':'Discord','discord':'Discord',
    'us.zoom.Zoom':'zoom','Zoom':'zoom','zoom':'zoom',
    'org.mpz_player.mpz':'mpz','mpz':'mpz',
    'org.telegram.desktop':'telegram-desktop',
    'org.ferdium.Ferdium':'ferdium',
    'io.mpv.Mpv':'mpv','mpv':'mpv',
    'org.videolan.VLC':'vlc','vlc':'vlc',
    'com.spotify.Client':'spotify','spotify':'spotify',
    'com.github.IsmaelMartinez.teams_for_linux':'teams-for-linux',
    'teams-for-linux':'teams-for-linux',
}
WRAPPERS={'env','flatpak','snap','sh','bash','python','python3','java','electron','AppRun'}

def discover(apps=None):
    result=[];seen=set()
    for app in Gio.AppInfo.get_all() if apps is None else apps:
        if not app.should_show() or not app.get_id():continue
        categories=set((app.get_categories() or '').split(';')) if hasattr(app,'get_categories') else set()
        if not categories.intersection(CATEGORIES):continue
        desktop_id=app.get_id().removesuffix('.desktop')
        binary=BINARIES.get(desktop_id)
        if binary is None:
            # A native executable is a useful candidate; wrapper executables are not.
            executable=app.get_executable() or ''
            binary=PurePath(executable).name
            if binary in WRAPPERS or not binary or any(c.isspace() for c in binary):continue
        target='app:application.process.binary='+binary
        if target in seen:continue
        seen.add(target)
        result.append({'id':target,'name':app.get_display_name(),'desktop_id':app.get_id()})
    return sorted(result,key=lambda item:item['name'].casefold())

# Session-only preload: populated at application launch, never persisted to disk.
_catalog=[]
def refresh():
    global _catalog
    current=discover()
    _catalog=current
    return current

def cached():
    return [dict(item) for item in _catalog]
