"""Lease-bound microphone hold. EOF or missing heartbeats always attempts mute."""
import os,select,signal,sys,time
from audio_targets import Snapshot,command,validate

def heartbeat(stream,timeout=.75):
    ready,_,_=select.select([stream],[],[],timeout)
    return bool(ready and os.read(stream.fileno(),4096))

def hold(target,stream):
    validate(target)
    if not target.startswith('input:'):raise ValueError('Choose a named microphone')
    name=target.split(':',1)[1]
    # Resolve before opening the mic; never change the default or another source.
    if not Snapshot().nodes(target)[1]:raise ValueError('Microphone unavailable')
    try:
        if not heartbeat(stream):return
        # A release may already have closed the pipe while discovery was running.
        ready,_,_=select.select([stream],[],[],0)
        if ready and not os.read(stream.fileno(),4096):return
        command('set-source-mute',name,'0')
        while heartbeat(stream):pass
    finally:
        for attempt in range(3):
            try:
                command('set-source-mute',name,'1')
                break
            except Exception:
                if attempt==2:raise
                time.sleep(.1)

def stop(*_args):raise SystemExit(0)

if __name__=='__main__':
    signal.signal(signal.SIGTERM,stop)
    signal.signal(signal.SIGINT,stop)
    try:hold(sys.argv[1],sys.stdin.buffer)
    except Exception:sys.exit(1)
