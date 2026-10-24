"""Opt-in shared microphone guardian; legacy V1 ptt_guard is unchanged.

One private pin survives helper restarts. Only a mute retry may revalidate the
same USB hardware serial after re-enumeration, once the host retired its writer.
Writes use the same libpulse context
that checked source index/serial, never a named/default-device fallback. Manual
unmute needs a separate host commit; EOF before that commit returns to Muted.
"""
import ctypes as C
import json
import os
from pathlib import Path
import select
import signal
import sys
import time

from audio_targets import validate
from system_sounds import Client as PulseClient, ChannelMap, Volume


class SampleSpec(C.Structure):
    _fields_ = [('format', C.c_int), ('rate', C.c_uint32), ('channels', C.c_uint8)]


# Stable prefix of pa_source_info through proplist (pulse/introspect.h).
class SourceInfo(C.Structure):
    _fields_ = [('name', C.c_char_p), ('index', C.c_uint32),
                ('description', C.c_char_p), ('sample_spec', SampleSpec),
                ('channel_map', ChannelMap), ('owner_module', C.c_uint32),
                ('volume', Volume), ('mute', C.c_int),
                ('monitor_of_sink', C.c_uint32), ('monitor_of_sink_name', C.c_char_p),
                ('latency', C.c_uint64), ('driver', C.c_char_p),
                ('flags', C.c_int), ('proplist', C.c_void_p)]


class Client(PulseClient):
    def __init__(self):
        super().__init__()
        ptr = C.c_void_p
        def bind(name, result, *args):
            function = getattr(self.lib, name)
            function.restype, function.argtypes = result, list(args)
            return function
        self.source_cb = C.CFUNCTYPE(None, ptr, C.POINTER(SourceInfo), C.c_int, ptr)
        self.source_name = bind('pa_context_get_source_info_by_name', ptr, ptr,
                                C.c_char_p, self.source_cb, ptr)
        self.source_index = bind('pa_context_get_source_info_by_index', ptr, ptr,
                                 C.c_uint32, self.source_cb, ptr)
        self.property = bind('pa_proplist_gets', C.c_char_p, ptr, C.c_char_p)
        self.source_mute = bind('pa_context_set_source_mute_by_index', ptr, ptr,
                                C.c_uint32, C.c_int, self.success_cb, ptr)

    def read_source(self, selector):
        # A helper connects only once. Never reconnect after loss of its context.
        if not self.context or self.ctxstate(self.context) != 4:
            raise ValueError('Microphone unavailable')
        found, completed = [], []
        @self.source_cb
        def callback(_context, pointer, eol, _data):
            if eol:
                completed.append(eol > 0)
            elif pointer:
                info = pointer.contents
                def prop(key):
                    value = self.property(info.proplist, key.encode('ascii')) if info.proplist else None
                    return value.decode('utf-8') if value else ''
                serial = self.property(info.proplist, b'object.serial') if info.proplist else None
                if info.name and serial and info.monitor_of_sink == 2**32 - 1:
                    try:
                        found.append({'name': info.name.decode('utf-8'),
                                      'index': int(info.index), 'serial': serial.decode('utf-8'),
                                      'muted': bool(info.mute),
                                      'hardware': usb_identity({key: prop(key) for key in
                                          ('device.bus', 'device.serial', 'device.vendor.id', 'device.product.id')})})
                    except UnicodeError:
                        completed.append(False)
        operation = self.source_index(self.context, selector, callback, None) if type(selector) is int else self.source_name(self.context, selector.encode('utf-8'), callback, None)
        self.operation(operation)
        if completed != [True] or len(found) != 1:
            raise ValueError('Microphone unavailable')
        return found[0]

    def set_mute(self, index, muted):
        if not self.context or self.ctxstate(self.context) != 4:
            raise ValueError('Microphone unavailable')
        success = []
        @self.success_cb
        def callback(_context, ok, _data):
            success.append(bool(ok))
        self.operation(self.source_mute(self.context, index, int(muted), callback, None))
        if success != [True]:
            raise ValueError('Microphone write failed')


def socket_identity():
    path = Path(os.environ.get('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}')) / 'pulse/native'
    info = path.stat()
    return [info.st_dev, info.st_ino]


def usb_identity(properties):
    keys = ('device.bus', 'device.serial', 'device.vendor.id', 'device.product.id')
    values = {key: properties.get(key, '') for key in keys}
    if values['device.bus'] != 'usb' or any(
            not isinstance(value, str) or not 0 < len(value) <= 256
            or any(ord(char) < 32 for char in value) for value in values.values()):
        return None
    if values['device.serial'].lower() in ('0', 'none', 'unknown'):
        return None
    return values


def pin_source(client, target, path, socket=socket_identity, *, recover=False):
    validate(target)
    if not target.startswith('input:'):
        raise ValueError('Choose a named microphone')
    name = target.split(':', 1)[1]
    if name.startswith('@') or name.isdigit():
        raise ValueError('Choose a named microphone')
    server = socket()
    if path.exists():
        if path.stat().st_size > 4096:
            raise ValueError('Invalid microphone pin')
        pin = json.loads(path.read_text())
        if not isinstance(pin, dict) or set(pin) not in ({'server', 'name', 'index', 'serial'}, {'server', 'name', 'index', 'serial', 'hardware'}):
            raise ValueError('Invalid microphone pin')
        if pin['name'] != name or pin['server'] != server or type(pin['index']) is not int or not 0 <= pin['index'] < 2**32 or not isinstance(pin['serial'], str) or not 0 < len(pin['serial']) <= 128:
            raise ValueError('Microphone identity changed')
    else:
        # The host supplies a new 0700 directory; one owner per target, no
        # replacement on retry. Reserve first: a failed initial lookup cannot
        # later mute a newly appearing source under the same name.
        with path.open('x') as output:
            os.chmod(path, 0o600)
            node = client.read_source(name)
            if node['name'] != name or not node['serial'] or len(node['serial']) > 128 or socket() != server:
                raise ValueError('Microphone identity changed')
            pin = dict(server=server, name=name, index=node['index'], serial=node['serial'], hardware=node.get('hardware'))
            json.dump(pin, output)
    try:
        check_source(client, pin, socket)
    except ValueError:
        # This path is available only to a mute-only helper, after the host has
        # confirmed that the old writer exited. Never reuse an old Live/toggle.
        hardware = pin.get('hardware')
        if not recover or not isinstance(hardware, dict) or usb_identity(hardware) != hardware:
            raise
        node = client.read_source(name)
        if node['name'] != name or node.get('hardware') != hardware or socket() != server:
            raise ValueError('Microphone hardware changed')
        candidate = dict(server=server, name=name, index=node['index'], serial=node['serial'], hardware=hardware)
        check_source(client, candidate, socket)
        temporary = path.with_suffix('.reconnect')
        try:
            with temporary.open('x') as output:
                os.chmod(temporary, 0o600)
                json.dump(candidate, output)
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)
        pin = candidate
    return pin


def check_source(client, pin, socket=socket_identity):
    if socket() != pin['server']:
        raise ValueError('Microphone server changed')
    node = client.read_source(pin['index'])
    if any(node[key] != pin[key] for key in ('index', 'name', 'serial')) or socket() != pin['server']:
        raise ValueError('Microphone identity changed')
    return node


def decision(stream, timeout=.75):
    if not select.select([stream], [], [], timeout)[0]:
        return None
    data = os.read(stream.fileno(), 4096)
    if not data or len(data) > 4096 or not data.endswith(b'\n'):
        return None
    lines = data.splitlines()
    if any(line not in (b'hold', b'commit') for line in lines):
        return None
    # A pre-write commit is rejected by allowed(). A commit may legitimately
    # follow hold pulses that were already in flight when the host read Live.
    return 'commit' if lines[-1:] == [b'commit'] and lines.count(b'commit') == 1 else 'hold' if all(line == b'hold' for line in lines) else None


def allowed(stream):
    if decision(stream) != 'hold':
        return False
    # Drain already queued EOF/cancellation after potentially slow discovery.
    if select.select([stream], [], [], 0)[0]:
        return decision(stream, 0) == 'hold'
    return True


def still_allowed(stream):
    for _ in range(4):
        if not select.select([stream], [], [], 0)[0]:
            return True
        if decision(stream, 0) != 'hold':
            return False
    return False


def execute(client, pin, mode, stream, report, socket=socket_identity):
    last_report = None
    def write(muted, authorize=False):
        nonlocal last_report
        check_source(client, pin, socket)
        if authorize and not still_allowed(stream):
            return False
        client.set_mute(pin['index'], muted)
        node = check_source(client, pin, socket)
        if node['muted'] != muted:
            raise ValueError('Microphone state not confirmed')
        state = 'muted' if muted else 'live'
        if state != last_report:
            report(state)
            last_report = state
        return True
    if mode == 'mute':
        write(True)
        return
    committed = False
    try:
        node = check_source(client, pin, socket)
        if not allowed(stream):
            return
        # No reconnect or target discovery between this lease check and write.
        if not write(False if mode == 'hold' else not node['muted'], authorize=True):
            return
        if mode == 'toggle':
            # Ordinary explicit unmute persists only after a fresh host commit.
            # Ignore only the already queued hold pulses while awaiting commit.
            for _ in range(128):
                value = decision(stream)
                if value == 'commit':
                    report('committed')
                    committed = True
                    return
                if value != 'hold':
                    return
            raise ValueError('Invalid microphone commit')
        next_check = 0.0
        while decision(stream) == 'hold':
            now = time.monotonic()
            if now >= next_check:
                # Heartbeats are lease authority, not evidence that this source
                # still exists. Detect USB loss even while both Decks stay held.
                check_source(client, pin, socket)
                next_check = now + .25
    finally:
        if not committed:
            write(True)


def main():
    if len(sys.argv) != 4 or sys.argv[2] not in ('hold', 'mute', 'toggle'):
        raise ValueError('Invalid microphone mode')
    client = Client()
    try:
        client.ready()
        pin = pin_source(client, sys.argv[1], Path(sys.argv[3]), recover=sys.argv[2] == 'mute')
        def report(value):
            sys.stdout.write(value + '\n')
            sys.stdout.flush()
        execute(client, pin, sys.argv[2], sys.stdin.buffer, report)
    finally:
        client.close()


if __name__ == '__main__':
    def stop(*_args):
        raise SystemExit(0)
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try:
        main()
    except Exception:
        sys.exit(1)
