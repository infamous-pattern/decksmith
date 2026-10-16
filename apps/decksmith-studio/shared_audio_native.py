"""Opt-in shared app/event audio writes with a per-write host authorization.

The reader/legacy executor never use this protocol. Discovery and identity reads
finish before `ready`; the Rust owner then rechecks its device fence and permits
one write. A lost acknowledgement is never replayed.
"""
import json
import os
from pathlib import Path
import subprocess

import audio_targets as audio


def message(source):
    line = source.readline(8193)
    if not line or len(line) > 8192 or not line.endswith('\n'):
        raise ValueError('Invalid shared request')
    return json.loads(line)


def send(output, value):
    output.write(json.dumps(value) + '\n')
    output.flush()


def identity(node):
    index = node.get('index')
    serial = node.get('properties', {}).get('object.serial')
    if type(index) is not int or not 0 <= index < 2**32 or not isinstance(serial, str) or not serial or len(serial) > 128:
        raise ValueError('Volume unavailable')
    return index, serial


def socket_identity():
    path = Path(os.environ.get('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}')) / 'pulse/native'
    info = path.stat()
    return info.st_dev, info.st_ino


def authorize(source, output):
    send(output, {'status': 'ready'})
    decision = message(source)
    if decision == {'decision': 'abort'}:
        raise ValueError('Stale action scope')
    if decision != {'decision': 'apply'}:
        raise ValueError('Invalid shared request')


def execute(action, source, output, *, snapshot=None, command=None,
            socket=None, event=None):
    snapshot = snapshot or audio.Snapshot
    command = command or audio.command
    socket = socket or socket_identity
    target = action.get('target')
    audio.validate(target)
    if action.get('type') == 'audio_select':
        if set(action) != {'type','target'} or not target.startswith(('input:','output:')):
            raise ValueError('Unsupported audio action')
        server=socket()
        kind,nodes=snapshot().nodes(target)
        if len(nodes)!=1:raise ValueError('Audio target unavailable')
        pin=identity(nodes[0]);name=nodes[0].get('name')
        if not isinstance(name,str) or name != target.split(':',1)[1]:
            raise ValueError('Audio target unavailable')
        current=snapshot().nodes(target)[1]
        if socket()!=server or len(current)!=1 or identity(current[0])!=pin:
            raise ValueError('Audio target unavailable')
        authorize(source,output)
        try:command('set-default-sink' if kind=='sinks' else 'set-default-source',name)
        except Exception as error:raise RuntimeError('Shared effect uncertain') from error
        return
    fields = {'type', 'target', 'percent'} if action.get('type') == 'audio_adjust' else {'type', 'target'}
    if set(action) != fields or action.get('type') not in ('audio_adjust', 'audio_mute'):
        raise ValueError('Unsupported audio action')
    if target not in ('system', 'default_output') and not target.startswith('app:'):
        raise ValueError('Unsupported audio action')
    adjust = action['type'] == 'audio_adjust'
    if adjust and (type(action['percent']) is not int or not 0 < abs(action['percent']) <= 100):
        raise ValueError('Invalid volume step')
    server = socket()
    if target == 'system':
        import system_sounds
        connection = event if event is not None else system_sounds.client()
        info = connection.read()
        if adjust:
            left = action['percent']
            while left:
                step = max(-20, min(20, left))
                info = system_sounds.adjusted(info, dict(action, percent=step))
                left -= step
        else:
            info = system_sounds.adjusted(info, action)
        value = system_sounds.state(info)
        # Read/ready before authorization. write() reuses the already ready
        # libpulse connection; its subsequent wait only awaits this write.
        connection.ready()
        kind, nodes = snapshot().nodes(target)
        if len(nodes) > 16:
            raise ValueError('Too many application streams')
        for node in nodes:
            identity(node)
        if socket() != server:
            raise ValueError('Audio target unavailable')
        authorize(source, output)
        try:
            connection.write(info, prepared_only=True)
        except Exception as error:
            raise RuntimeError('Shared effect uncertain') from error
    else:
        kind, nodes = snapshot().nodes(target)
        value = audio.state(nodes)
        if value is None:
            raise ValueError('Audio target unavailable')
    if len(nodes) > 16:
        raise ValueError('Too many application streams')
    pins = [(identity(node), node) for node in nodes]
    suffix = 'sink' if kind == 'sinks' else 'sink-input'
    muted = '0' if all(node.get('mute', False) for node in nodes) else '1'
    for pin, node in pins:
        # Never authorize a replacement stream that reused an index. Fresh
        # selector membership and server identity checks precede the handshake.
        current = snapshot().nodes(target)[1]
        if socket() != server or not any(identity(item) == pin for item in current):
            raise ValueError('Audio target unavailable')
        if target == 'system':
            amount = str(value['percent']) + '%' if adjust else '1' if value['muted'] else '0'
        else:
            state = audio.state([node])
            if state is None:
                raise ValueError('Volume unavailable')
            amount = str(max(0, min(100, state['percent'] + action['percent']))) + '%' if adjust else muted
        authorize(source, output)
        try:
            command('set-'+suffix+'-volume' if adjust else 'set-'+suffix+'-mute', str(pin[0]), amount)
        except Exception as error:
            raise RuntimeError('Shared effect uncertain') from error


def serve(source, output):
    while True:
        try:
            action = message(source)
        except (ValueError, EOFError):
            return
        try:
            if not isinstance(action, dict):
                raise ValueError('Invalid shared request')
            execute(action, source, output)
            result = {'status': 'ok'}
        except Exception as error:
            code = 'uncertain' if isinstance(error, RuntimeError) else 'stale' if str(error) == 'Stale action scope' else audio.action_error(error)
            result = {'status': 'error', 'code': code}
        send(output, result)
