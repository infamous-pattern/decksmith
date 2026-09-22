"""Bounded MPRIS control of one running player; never launches a player."""
from gi.repository import Gio, GLib
import json,os,re
from pathlib import Path

METHODS={'play_pause':'PlayPause','next':'Next','previous':'Previous'}
PATH='/org/mpris/MediaPlayer2'
IFACE='org.mpris.MediaPlayer2.Player'

def valid_player(value):
    return isinstance(value,str) and len(value)<=255 and re.fullmatch(r'org\.mpris\.MediaPlayer2\.[A-Za-z_][A-Za-z0-9_-]*(?:\.[A-Za-z_][A-Za-z0-9_-]*)*',value) is not None

def family(name):
    return re.sub(r'\.instance[0-9]+$','',name)

def choices():
    bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
    result={}
    for name,_props in players(bus):
        target=family(name)
        try:label=bus.call_sync(name,PATH,'org.freedesktop.DBus.Properties','Get',GLib.Variant('(ss)',('org.mpris.MediaPlayer2','Identity')),None,Gio.DBusCallFlags.NO_AUTO_START,100,None).unpack()[0]
        except GLib.Error:label=target.removeprefix('org.mpris.MediaPlayer2.')
        if valid_player(target):result[target]={'id':target,'label':str(label)}
    return sorted(result.values(),key=lambda item:item['label'].casefold())

def players(bus,target=None):
    names=bus.call_sync('org.freedesktop.DBus','/org/freedesktop/DBus','org.freedesktop.DBus','ListNames',None,None,Gio.DBusCallFlags.NONE,200,None).unpack()[0]
    result=[]
    for name in sorted(n for n in names if n.startswith('org.mpris.MediaPlayer2.') and (target is None or family(n)==target or n==target))[:8]:
        try:
            props=bus.call_sync(name,PATH,'org.freedesktop.DBus.Properties','GetAll',GLib.Variant('(s)',(IFACE,)),None,Gio.DBusCallFlags.NO_AUTO_START,100,None).unpack()[0]
            result.append((name,props))
        except GLib.Error:continue
    return result

def choose(candidates, action, preferred=None):
    if action not in METHODS:raise ValueError('Unsupported media action')
    ranked=sorted(candidates,key=lambda item:(item[0]!=preferred,{'Playing':0,'Paused':1,'Stopped':2}.get(item[1].get('PlaybackStatus'),3),item[0]))
    if not ranked:raise ValueError('No running media player')
    name,props=ranked[0]
    capability={'next':'CanGoNext','previous':'CanGoPrevious','play_pause':'CanPause' if props.get('PlaybackStatus')=='Playing' else 'CanPlay'}[action]
    if not props.get('CanControl',False) or not props.get(capability,False):raise ValueError('The active player does not support this action')
    return name

def owner(bus,name):
    return bus.call_sync('org.freedesktop.DBus','/org/freedesktop/DBus','org.freedesktop.DBus','GetNameOwner',GLib.Variant('(s)',(name,)),None,Gio.DBusCallFlags.NONE,100,None).unpack()[0]

def preferred_player(bus,candidates):
    if not os.environ.get('XDG_RUNTIME_DIR'):return None
    try:
        previous=json.loads((Path(os.environ['XDG_RUNTIME_DIR'])/'decksmith-media-player.json').read_text())
        if any(name==previous['name'] for name,_ in candidates) and owner(bus,previous['name'])==previous['owner']:return previous['name']
    except (OSError,ValueError,KeyError,GLib.Error):pass
    return None

def execute(action, bus=None, target=None):
    if action not in METHODS:raise ValueError('Unsupported media action')
    if target is not None and not valid_player(target):raise ValueError('Invalid media player')
    real_bus=bus is None
    if real_bus:bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
    candidates=players(bus,target)
    preferred=None
    cache=Path(os.environ['XDG_RUNTIME_DIR'])/'decksmith-media-player.json' if real_bus and target is None and os.environ.get('XDG_RUNTIME_DIR') else None
    if cache is not None:preferred=preferred_player(bus,candidates)
    name=choose(candidates,action,preferred)
    selected_owner=owner(bus,name) if real_bus or target is not None else name
    bus.call_sync(selected_owner,PATH,IFACE,METHODS[action],None,None,Gio.DBusCallFlags.NO_AUTO_START,400,None)
    if cache is not None:
        try:
            temporary=cache.with_suffix('.tmp')
            temporary.write_text(json.dumps({'name':name,'owner':selected_owner}))
            temporary.replace(cache)
        except OSError:
            pass  # Playback already changed; never report it as a retryable failure.
