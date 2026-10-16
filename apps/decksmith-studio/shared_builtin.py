"""Opt-in ordinary built-ins; no shell or plugin execution.

Each underlying adapter requests a grant after discovery and immediately before
an external effect. Legacy launch/system entry points do not use this protocol.
"""
import re
import sys
from shared_audio_native import authorize, message, send


def execute(action, source, output, *, launcher=None, controls=None, grant=None):
    grant=grant or (lambda:authorize(source,output))
    if not isinstance(action, dict):raise ValueError('Unsupported built-in')
    kind=action.get('type')
    if kind in ('open_application','open_website'):
        field='desktop_id' if kind=='open_application' else 'url'
        if set(action)!={'type',field}:raise ValueError('Unsupported built-in')
        value=action[field]
        if not isinstance(value,str):raise ValueError('Unsupported built-in')
        if kind=='open_application' and not (len(value)<=255 and re.fullmatch(r'[A-Za-z0-9._-]+\.desktop',value)):
            raise ValueError('Unsupported built-in')
        if kind=='open_website' and (len(value)>2048 or any(c.isspace() or ord(c)<32 for c in value)):
            raise ValueError('Unsupported built-in')
        if launcher is None:
            from launch import launch
            launcher=launch
        launcher('application' if kind=='open_application' else 'website',value,
                 authorize=grant)
    elif kind=='system' and set(action)=={'type','command'}:
        from system_controls import Controls, COMMANDS
        if action['command'] not in COMMANDS:raise ValueError('Unsupported built-in')
        if controls is None:controls=Controls()
        controls.authorize=grant
        controls.guarded=True
        controls.execute(action['command'])
    else:raise ValueError('Unsupported built-in')


def serve(source, output):
    # Sharing one system adapter avoids duplicated persistent GSettings caches.
    from system_controls import Controls,GLib
    controls=Controls()
    while True:
        try:action=message(source)
        except (ValueError,EOFError):return
        # Process pending GSettings notifications between requests, matching the
        # legacy persistent system adapter without an unbounded event drain.
        context=GLib.MainContext.default()
        for _ in range(32):
            if not context.pending():break
            context.iteration(False)
        grants=0
        def grant():
            nonlocal grants
            authorize(source,output)
            grants+=1
        try:
            # execute installs the callback on the adapter before discovery.
            execute(action,source,output,controls=controls,grant=grant)
            reply={'status':'ok'}
        except Exception as error:
            # Ordinary adapter exceptions may follow an issued effect. Only a
            # known authorization abort is definitely stale; never replay others.
            code='stale' if not grants and str(error)=='Stale action scope' else 'unsupported' if str(error)=='Unsupported built-in' else 'unavailable' if str(error)=='Application unavailable' else 'uncertain'
            reply={'status':'error','code':code}
        send(output,reply)
        if reply['status']=='error':return

if __name__=='__main__':serve(sys.stdin,sys.stdout)
