"""Transient local peak monitoring. No PCM recording, files, or network export."""
import ctypes as C
import json, math, os, sys, time
from concurrent.futures import ThreadPoolExecutor
from audio_targets import Snapshot, listing

class Spec(C.Structure):
    _fields_=[('format',C.c_int),('rate',C.c_uint32),('channels',C.c_uint8)]
class Attr(C.Structure):
    _fields_=[(name,C.c_uint32) for name in ('maxlength','tlength','prebuf','minreq','fragsize')]

class Pulse:
    def __init__(self):
        self.lib=C.CDLL('libpulse.so.0')
        def bind(name,result,*args):
            f=getattr(self.lib,name);f.restype=result;f.argtypes=list(args);return f
        ptr=C.c_void_p; integer=C.c_int
        self.new=bind('pa_mainloop_new',ptr)
        self.api=bind('pa_mainloop_get_api',ptr,ptr)
        self.iterate=bind('pa_mainloop_iterate',integer,ptr,integer,C.POINTER(integer))
        self.ctxnew=bind('pa_context_new',ptr,ptr,C.c_char_p)
        self.connect=bind('pa_context_connect',integer,ptr,C.c_char_p,integer,ptr)
        self.state=bind('pa_context_get_state',integer,ptr)
        self.snew=bind('pa_stream_new',ptr,ptr,C.c_char_p,C.POINTER(Spec),ptr)
        self.monitor=bind('pa_stream_set_monitor_stream',integer,ptr,C.c_uint32)
        self.record=bind('pa_stream_connect_record',integer,ptr,C.c_char_p,C.POINTER(Attr),integer)
        self.callback_type=C.CFUNCTYPE(None,ptr,C.c_size_t,ptr)
        self.set_read=bind('pa_stream_set_read_callback',None,ptr,self.callback_type,ptr)
        self.peaks={}
        # Retain the C callback for the entire context lifetime.
        self.read_callback=self.callback_type(self.capture)
        self.sstate=bind('pa_stream_get_state',integer,ptr)
        self.source=bind('pa_stream_get_device_name',C.c_char_p,ptr)
        self.suspended=bind('pa_stream_is_suspended',integer,ptr)
        self.readable=bind('pa_stream_readable_size',C.c_size_t,ptr)
        self.peek=bind('pa_stream_peek',integer,ptr,C.POINTER(ptr),C.POINTER(C.c_size_t))
        self.drop=bind('pa_stream_drop',integer,ptr)
        self.disconnect=bind('pa_stream_disconnect',integer,ptr)
        self.unref=bind('pa_stream_unref',None,ptr)
        self.loop=self.new();self.ctx=self.ctxnew(self.api(self.loop),b'Decksmith peak levels')
        if not self.ctx or self.connect(self.ctx,("unix:"+os.environ.get("XDG_RUNTIME_DIR",f"/run/user/{os.getuid()}")+"/pulse/native").encode(),0,None)<0:raise RuntimeError('Audio unavailable')
    def tick(self):
        for _ in range(8):self.iterate(self.loop,0,None)
        if self.state(self.ctx) in (5,6):raise RuntimeError('Audio disconnected')
    def stream(self,key):
        source,index=key
        stream=self.snew(self.ctx,b'Decksmith level only',C.byref(Spec(5,20,1)),None) # FLOAT32LE peaks, 20 Hz
        if not stream:return None
        flags=0x0800 # peak detection only; leave hardware latency unchanged
        # DONT_INHIBIT_AUTO_SUSPEND can leave physical input streams stuck creating
        # on PipeWire (observed on Wave 3); monitoring needs an active source.
        # Check the actual source before accepting peaks, including after a move.
        attr=Attr(0xffffffff,0xffffffff,0xffffffff,0xffffffff,4)
        if (index is not None and self.monitor(stream,index)<0) or self.record(stream,source.encode(),C.byref(attr),flags)<0:
            self.unref(stream);return None
        self.set_read(stream,self.read_callback,None)
        return stream
    def capture(self,stream,_size,_userdata):
        value=self.read_peak(stream)
        if value is not None:self.peaks[stream]=max(value,self.peaks.get(stream,0.))
    def level(self,stream):
        if not stream or self.sstate(stream)!=2:
            self.peaks.pop(stream,None);return None
        if self.suspended(stream)>0:
            self.peaks.pop(stream,None);return 0.
        return self.peaks.pop(stream,None)
    def read_peak(self,stream):
        if not stream or self.sstate(stream)!=2:return None
        if self.suspended(stream)>0:return 0.0
        peak=None
        for _ in range(16):
            size=self.readable(stream)
            if size==0 or size==C.c_size_t(-1).value:break
            data=C.c_void_p();length=C.c_size_t()
            if self.peek(stream,C.byref(data),C.byref(length))<0:break
            if data.value:
                floats=C.cast(data,C.POINTER(C.c_float))
                for i in range(min(length.value//4,256)):
                    v=float(floats[i])
                    if math.isfinite(v):peak=max(peak or 0.,min(1.,abs(v)))
            elif length.value:peak=max(peak or 0.,0.)
            self.drop(stream)
        return peak
    def close(self,stream):
        if stream:
            self.set_read(stream,self.callback_type(),None)
            self.disconnect(stream);self.unref(stream);self.peaks.pop(stream,None)

def resolve(targets):
    snap=Snapshot();result={}
    for target in targets:
        try:
            kind,nodes=snap.nodes(target)
            if not nodes:result[target]=[];continue
            if kind=='sink-inputs':
                if len(nodes)>16:result[target]=[];continue
                if 'sinks' not in snap.cache:snap.cache['sinks']=listing('sinks')
                sinks={n['index']:n for n in snap.cache['sinks']}
                result[target]=[(sinks[n['sink']]['monitor_source'],n['index']) for n in nodes]
            else:
                result[target]=[(nodes[0]['monitor_source'] if kind=='sinks' else nodes[0]['name'],None)]
        except Exception:result[target]=[]
    return result

def display_level(peak):
    # -60..0 dBFS mapped to 0..100; zero is measured silence, None is unavailable.
    if peak is None:return None
    if peak<=.001:return 0
    return round(max(0.,min(100.,(20*math.log10(peak)+60)/60*100)))

def aggregate(keys,last,now):
    values=[last.get(key) for key in keys]
    # PipeWire app monitor delivery can pause for >500 ms while still READY.
    # Decay the last measured peak during a short gap, then expire it outright.
    # Failed/moved/removed streams clear their measurement before aggregation.
    if not keys or not all(v is not None and 0<=now-v[0]<1. for v in values):return None
    return display_level(max(v[1]*10**(-30*max(0.,now-v[0]-.15)/20) for v in values))

def needs_restart(state,created,now):
    # Inventory checks bound retries to once per two seconds; allow normal setup time.
    return state in (None,3,4) or (state in (0,1) and now-created>=3.)

def correct_source(pulse,stream,key):
    return bool(stream and pulse.sstate(stream)==2 and pulse.source(stream)==key[0].encode())

def run(targets):
    if len(targets)>4:raise ValueError('At most four dial targets')
    os.environ["PULSE_SERVER"]="unix:"+os.environ.get("XDG_RUNTIME_DIR",f"/run/user/{os.getuid()}")+"/pulse/native"
    pulse=Pulse();streams={};mapping={};last={};smooth={};created={}
    pool=ThreadPoolExecutor(max_workers=1);future=None;refresh=0.;publish=0.
    while True:
        now=time.monotonic();pulse.tick()
        if future is not None and future.done():
            mapping=future.result();future=None
            wanted={key for keys in mapping.values() for key in keys}
            for key in list(streams):
                if key not in wanted:pulse.close(streams.pop(key));last.pop(key,None);created.pop(key,None)
            for key in wanted:
                stream=streams.get(key);state=pulse.sstate(stream) if stream else None
                if needs_restart(state,created.get(key,now),now) or (state==2 and not correct_source(pulse,stream,key)):
                    pulse.close(streams.get(key));streams[key]=pulse.stream(key);last.pop(key,None);created[key]=now
        if now>=refresh and future is None and pulse.state(pulse.ctx)==4:
            future=pool.submit(resolve,targets);refresh=now+2.
        for key,stream in streams.items():
            if not correct_source(pulse,stream,key):
                last.pop(key,None);continue
            value=pulse.level(stream)
            if value is not None:last[key]=(now,value)
        if now>=publish:
            output={}
            for target in targets:
                keys=mapping.get(target,[])
                level=aggregate(keys,last,now)
                # Fast attack, ~0.4 second release; never smooth through unavailable states.
                previous=smooth.get(target)
                if level is not None and previous is not None:level=max(level,previous-12.5)
                smooth[target]=level;output[target]=level
            print(json.dumps(output),flush=True);publish=now+.05
        time.sleep(.01)

if __name__=='__main__':
    try:run(json.loads(sys.argv[1]))
    except (BrokenPipeError,KeyboardInterrupt):pass
