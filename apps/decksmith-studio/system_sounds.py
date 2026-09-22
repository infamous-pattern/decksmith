"""Persistent event-sound volume through libpulse's stream-restore extension.

Uses the same event-role record as GNOME Sound settings. Never touches a sink
or falls back to master volume. Connections are reused by the audio reader;
all operations have a short deadline and disconnect safely after failure.
"""
import ctypes as C
import os
import subprocess
import time

EVENT = b'sink-input-by-media-role:event'
PA_UPDATE_REPLACE = 2

class ChannelMap(C.Structure):
    _fields_ = [('channels', C.c_uint8), ('map', C.c_int * 32)]

class Volume(C.Structure):
    _fields_ = [('channels', C.c_uint8), ('values', C.c_uint32 * 32)]

class Info(C.Structure):
    _fields_ = [('name', C.c_char_p), ('channel_map', ChannelMap),
                ('volume', Volume), ('device', C.c_char_p), ('mute', C.c_int)]


def default_info():
    info = Info()
    info.name = EVENT
    info.channel_map.channels = info.volume.channels = 1
    info.channel_map.map[0] = 0  # PA_CHANNEL_POSITION_MONO
    info.volume.values[0] = 65536
    return info


def state(info):
    if not 1 <= info.volume.channels <= 32:
        raise ValueError('Volume unavailable')
    return {'percent': min(1000, round(max(info.volume.values[:info.volume.channels]) * 100 / 65536)),
            'muted': bool(info.mute), 'icon': 'speaker'}


def adjusted(info, action):
    """Keep channel balance, saved routing and mute while adjusting volume."""
    value = state(info)
    if action['type'] == 'audio_adjust':
        step = action['percent']
        if type(step) is not int or not 0 < abs(step) <= 20:
            raise ValueError('Invalid volume step')
        level = round(max(0, min(100, value['percent'] + step)) * 65536 / 100)
        previous = max(info.volume.values[:info.volume.channels])
        for index in range(info.volume.channels):
            info.volume.values[index] = round(info.volume.values[index] * level / previous) if previous else level
    elif action['type'] == 'audio_mute':
        info.mute = not info.mute
    else:
        raise ValueError('Unsupported audio action')
    return info


class Client:
    def __init__(self):
        self.loop = self.context = None
        try: self.lib = C.CDLL('libpulse.so.0')
        except OSError as error: raise ValueError('System sounds unavailable') from error
        ptr = C.c_void_p
        def bind(name, result, *args):
            f = getattr(self.lib, name); f.restype = result; f.argtypes = list(args)
            return f
        self.new = bind('pa_mainloop_new', ptr)
        self.api = bind('pa_mainloop_get_api', ptr, ptr)
        self.iterate = bind('pa_mainloop_iterate', C.c_int, ptr, C.c_int, ptr)
        self.free = bind('pa_mainloop_free', None, ptr)
        self.ctxnew = bind('pa_context_new', ptr, ptr, C.c_char_p)
        self.connect = bind('pa_context_connect', C.c_int, ptr, C.c_char_p, C.c_int, ptr)
        self.ctxstate = bind('pa_context_get_state', C.c_int, ptr)
        self.disconnect = bind('pa_context_disconnect', None, ptr)
        self.unref = bind('pa_context_unref', None, ptr)
        self.opstate = bind('pa_operation_get_state', C.c_int, ptr)
        self.opcancel = bind('pa_operation_cancel', None, ptr)
        self.opunref = bind('pa_operation_unref', None, ptr)
        self.read_cb = C.CFUNCTYPE(None, ptr, C.POINTER(Info), C.c_int, ptr)
        self.success_cb = C.CFUNCTYPE(None, ptr, C.c_int, ptr)
        self.read_op = bind('pa_ext_stream_restore_read', ptr, ptr, self.read_cb, ptr)
        self.write_op = bind('pa_ext_stream_restore_write', ptr, ptr, C.c_int,
                             C.POINTER(Info), C.c_uint, C.c_int, self.success_cb, ptr)

    def close(self):
        if self.context:
            self.disconnect(self.context); self.unref(self.context); self.context = None
        if self.loop:
            self.free(self.loop); self.loop = None

    def wait(self, done):
        deadline = time.monotonic() + .4
        while not done():
            if time.monotonic() >= deadline or self.ctxstate(self.context) in (5, 6):
                raise ValueError('System sounds unavailable')
            if self.iterate(self.loop, 0, None) < 0:
                raise ValueError('System sounds unavailable')
            if not done(): time.sleep(.001)

    def ready(self):
        if self.context and self.ctxstate(self.context) == 4: return
        self.close()
        self.loop = self.new()
        if not self.loop: raise ValueError('System sounds unavailable')
        self.context = self.ctxnew(self.api(self.loop), b'Decksmith system sounds')
        server = ('unix:' + os.environ.get('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}') + '/pulse/native').encode()
        if not self.context or self.connect(self.context, server, 0, None) < 0:
            self.close(); raise ValueError('System sounds unavailable')
        try: self.wait(lambda: self.ctxstate(self.context) == 4)
        except Exception: self.close(); raise

    def operation(self, op):
        if not op: raise ValueError('System sounds unavailable')
        try: self.wait(lambda: self.opstate(op) != 0)
        finally:
            self.opcancel(op); self.opunref(op)

    def read(self):
        self.ready()
        found = []; completed = []
        @self.read_cb
        def callback(_context, pointer, eol, _data):
            if eol:
                completed.append(eol > 0)
            elif pointer and pointer.contents.name == EVENT:
                original = pointer.contents
                info = Info.from_buffer_copy(original)
                # Callback strings belong to libpulse and expire on return.
                info.name = EVENT
                info.device = bytes(original.device) if original.device else None
                found.append(info)
        try:
            self.operation(self.read_op(self.context, callback, None))
            if completed != [True]: raise ValueError('System sounds unavailable')
            return found[-1] if found else default_info()
        except Exception: self.close(); raise

    def write(self, info):
        self.ready(); success = []
        @self.success_cb
        def callback(_context, ok, _data): success.append(bool(ok))
        try:
            # PA_UPDATE_REPLACE, this single event record only, instant apply.
            self.operation(self.write_op(self.context, PA_UPDATE_REPLACE, C.byref(info), 1, 1, callback, None))
            if success != [True]: raise ValueError('System sounds unavailable')
        except Exception: self.close(); raise


_client = None

def client():
    global _client
    if _client is None: _client = Client()
    return _client

def read(): return state(client().read())

def execute(action):
    connection = client()
    info = adjusted(connection.read(), action)
    connection.write(info)
    # PipeWire persists the event role but may leave already-running event
    # streams unchanged, even with apply_immediately. Update those only.
    from audio_targets import Snapshot, command
    nodes = Snapshot().nodes('system')[1]
    if len(nodes) > 16: raise ValueError('Too many application streams')
    value = state(info)
    for node in nodes:
        try:
            if action['type'] == 'audio_adjust':
                command('set-sink-input-volume', str(node['index']), str(value['percent']) + '%')
            else:
                command('set-sink-input-mute', str(node['index']), '1' if value['muted'] else '0')
        except subprocess.CalledProcessError:
            # A short alert can end between discovery and the command.
            if any(n['index'] == node['index'] for n in Snapshot().nodes('system')[1]): raise
