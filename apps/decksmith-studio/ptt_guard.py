"""Lease-bound microphone hold. EOF or missing heartbeats always attempts mute."""
import os,select,signal,sys,time
from audio_targets import Snapshot,command,validate

def heartbeat(stream,timeout=.75):
    ready,_,_=select.select([stream],[],[],timeout)
    return bool(ready and os.read(stream.fileno(),4096))

def named(target):
    validate(target)
    if not target.startswith('input:'):raise ValueError('Choose a named microphone')
    return target.split(':',1)[1]

def mute(target,report=lambda state:None):
    name=named(target)
    for attempt in range(3):
        try:
            command('set-source-mute',name,'1')
            report('muted')
            return
        except Exception:
            if attempt==2:raise
            time.sleep(.1)

def hold(target,stream,report=lambda state:None):
    name=named(target)
    try:
        # Resolve before opening; even discovery failure attempts named mute.
        if not Snapshot().nodes(target)[1]:raise ValueError('Microphone unavailable')
        if not heartbeat(stream):return
        # A release may already have closed the pipe while discovery was running.
        ready,_,_=select.select([stream],[],[],0)
        if ready and not os.read(stream.fileno(),4096):return
        command('set-source-mute',name,'0')
        report('live')
        while heartbeat(stream):pass
    finally:
        mute(target,report)

def stop(*_args):raise SystemExit(0)

if __name__=='__main__':
    signal.signal(signal.SIGTERM,stop)
    signal.signal(signal.SIGINT,stop)
    try:
        if len(sys.argv)==2:hold(sys.argv[1],sys.stdin.buffer)
        elif len(sys.argv)==3 and sys.argv[2] in ('--shared','--mute'):
            def report(state):
                sys.stdout.write(state+'\n');sys.stdout.flush()
            if sys.argv[2]=='--mute':mute(sys.argv[1],report)
            else:hold(sys.argv[1],sys.stdin.buffer,report)
        else:raise ValueError('Invalid guardian mode')
    except Exception:sys.exit(1)
