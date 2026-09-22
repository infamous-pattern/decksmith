"""Read-only saved-control availability; never launches or changes audio."""
import json,sys
from audio_targets import Snapshot,state
from gi.repository import Gio,GLib
import media

def clean(value):return ''.join(c for c in str(value) if c.isprintable())[:100]
def fallback(target):
    if target=='system':return 'System sounds'
    if target=='microphone':return 'Default microphone'
    if target=='automatic':return 'Automatic media player'
    return clean(target.split('=',1)[-1] if target.startswith('app:') else target.removeprefix('org.mpris.MediaPlayer2.').split(':',1)[-1])
def notice(check,name,status,detail,hint,short,next_step):
    return {'check':check,'target_name':clean(name),'status':status,'detail':detail,'hint':hint,'short':short,'next_step':next_step}
def probe(checks,snapshot=None,bus=None):
    if len(checks)>24:raise ValueError('Too many controls')
    snapshot=snapshot or Snapshot();result=[];candidates=None;bus_failed=False
    for check in checks:
        target=check['target'];kind=check['kind'];name=check['label'] if target.startswith(('input:','output:')) else fallback(target)
        try:
            if kind=='audio':
                if target=='system':
                    import system_sounds
                    system_sounds.read()
                    result.append(notice(check,name,'available','Target available','', '', ''));continue
                nodes=snapshot.nodes(target)[1]
                if not nodes:
                    app=target.startswith('app:');mic=target=='microphone' or target.startswith('input:')
                    result.append(notice(check,name,'idle' if app else 'missing','No playback stream detected' if app else 'Audio device unavailable','Start playback in this app; Decksmith will reconnect automatically.' if app else 'Reconnect the device, or choose another audio target.','No audio' if app else 'Missing','Start app' if app else 'Connect mic' if mic else 'Connect output'));continue
                props=nodes[0].get('properties',{})
                name=props.get('application.name',name) if target.startswith('app:') else nodes[0].get('description') or name
                if check.get('command') not in ('select','mute') and state(nodes) is None:
                    result.append(notice(check,name,'unsupported','Volume information unavailable','Choose a target that exposes volume controls.','No volume','Edit target'));continue
            elif kind=='media':
                if candidates is None and not bus_failed:
                    try:
                        bus=bus or Gio.bus_get_sync(Gio.BusType.SESSION,None);candidates=media.players(bus)
                    except GLib.Error:bus_failed=True
                if bus_failed:raise RuntimeError('Media connection unavailable')
                available=[p for p in candidates if target=='automatic' or media.family(p[0])==target or p[0]==target]
                preferred=media.preferred_player(bus,available) if target=='automatic' else None
                if not available:
                    result.append(notice(check,name,'missing','No matching media player is running','Open this player and load media; enable its MPRIS integration if needed.','No player','Open player'));continue
                ranked=sorted(available,key=lambda item:(item[0]!=preferred,{'Playing':0,'Paused':1,'Stopped':2}.get(item[1].get('PlaybackStatus'),3),item[0]))
                selected=ranked[0][0];name=fallback(media.family(selected))
                try:name=bus.call_sync(selected,media.PATH,'org.freedesktop.DBus.Properties','Get',GLib.Variant('(ss)',('org.mpris.MediaPlayer2','Identity')),None,Gio.DBusCallFlags.NO_AUTO_START,100,None).unpack()[0]
                except (GLib.Error,AttributeError,TypeError):pass
                try:media.choose(available,check['command'],preferred)
                except ValueError:
                    result.append(notice(check,name,'unsupported','This player cannot perform '+check['command'].replace('_',' '),'Load a playlist or choose another player. Decksmith will not switch players to hide this failure.','Unsupported','Check player'));continue
            elif kind=='desktop':
                from system_controls import Controls,LABELS
                current=Controls().state(target);name=LABELS.get(target,target)
                if not current['available']:
                    result.append(notice(check,name,'unsupported','System control unavailable','Check GNOME settings and available hardware.','Unavailable','Check system'));continue
            elif kind=='application':
                app=Gio.DesktopAppInfo.new(target)
                if app is None or not app.should_show():
                    result.append(notice(check,name,'missing','Application is not available','Install the app or choose another application in Edit layout.','App missing','Edit app'));continue
                name=app.get_display_name()
            else:continue
            result.append(notice(check,name,'available','Target available','', '', ''))
        except Exception:
            result.append(notice(check,name,'unknown','Target status could not be checked','Check desktop audio or media settings. Decksmith will retry automatically.','Check target','Retrying'))
    return result
def serve(source,output):
    """Fresh, bounded status checks over a reusable process; no action execution."""
    while True:
        line=source.readline(65537)
        if not line:return
        if len(line)>65536 or not line.endswith('\n'):return
        try:
            checks=json.loads(line)
            if not isinstance(checks,list) or len(checks)>24:raise ValueError('Invalid checks')
            result=probe(checks)
            raw=json.dumps(result)
            if len(raw.encode())>65535:raise ValueError('Response too large')
        except Exception:raw='null'
        output.write(raw+'\n');output.flush()

if __name__=='__main__':
    try:
        if sys.argv[1]=='serve':serve(sys.stdin,sys.stdout)
        else:print(json.dumps(probe(json.loads(sys.argv[1]))))
    except Exception:sys.exit(1)
