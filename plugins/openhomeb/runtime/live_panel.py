"""Opt-in, single-accessory panel for the approved Main_LED's live trial."""
import argparse
import collections
import re
import asyncio
import json
import os
import secrets
import signal
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from adjustment_host import BINARY, PacedAdjustmentTestHost
from live_on_off_trial import BASE, ROOT, TARGET, reader
from paced_host import BRIGHTNESS, SET


class LivePanel:
    def __init__(self, managed=False):
        self.temp = tempfile.TemporaryDirectory(prefix='decksmith-live-panel-')
        manifest = json.loads((ROOT.parent / 'openhomeb-2.0.2/source/assets/manifest.json').read_text())
        self.host = PacedAdjustmentTestHost(manifest, Path(self.temp.name) / 'settings.json')
        self.host.saved['global'] = {'homebridgeUrl': BASE, 'updateInterval': 2, 'catalogCacheSeconds': 30}
        from accessory_catalog import Catalog
        self.catalog=Catalog(BASE)
        self.catalog_task=None
        self.assigned_states={}
        self.read = None
        self.actual = None
        self.updated = 0
        self.status = 'Connecting to Main_LED’s…'
        self.busy = False
        self.poll_task = None
        self.command_task = None
        self.recovery_task = None
        self.managed = managed
        from connection_preferences import load,read
        self.preference_path=(Path(os.environ.get('XDG_CONFIG_HOME',str(Path.home()/'.config')))/'decksmith/openhomeb.json') if managed else None
        self.enabled=load(self.preference_path)
        self.connection_config=read(self.preference_path)
        self.connection_config.setdefault('server','' if self.connection_config.get('removed') else BASE)
        self.connection_config.setdefault('username','')
        from connection_setup import Setup,server_url
        self.catalog=Catalog('' if self.connection_config.get('removed') else server_url(self.connection_config['server']))
        self.setup=Setup(self)
        self.last_error=None
        self.manual_recovery = False
        self.recovery_required = False
        self.trace = collections.deque(maxlen=16)

    async def connect(self,use_session=False):
        from accessory_catalog import Catalog
        from connection_setup import server_url
        from credential_store import SecretToolStore
        config=self.connection_config
        if config.get('removed'):raise ValueError('No saved connection')
        if not use_session:
            password=''
            if config.get('credential_profile'):
                try:password=await SecretToolStore().lookup(config['credential_profile']) or ''
                except Exception:
                    self.catalog.auth_error='required'
                    raise
            self.catalog=Catalog(server_url(config['server']),config.get('username',''),password)
        await asyncio.to_thread(self.catalog.refresh)
        self.host.saved['global']={'homebridgeUrl':self.catalog.base,'username':self.catalog.username,
            'password':self.catalog.password,'updateInterval':2,'catalogCacheSeconds':30}
        self.host.access_session=(self.catalog.token,self.catalog.expires)
        def read_target():
            _,raw=self.catalog.one(TARGET)
            return {c['type']:c['value'] for c in raw['serviceCharacteristics'] if c['type'] in ('On','Brightness')}
        self.read=read_target
        try:
            await self.refresh()
        except Exception:
            pass  # One unavailable accessory must not disable all assignments.
        await self.host.start([str(BINARY)])
        if any(item['id']==TARGET for item in self.catalog.items):
            for name, value in [('on', True), ('off', False)]:
                await self.host.appear(name, SET, {'accessoryId': TARGET, 'characteristicType': 'On', 'targetValue': value})
            await self.host.appear('brightness', BRIGHTNESS, {'accessoryId': TARGET,
                'characteristicType': 'Brightness', 'turnOnWhenAdjusting': False, 'increment': 1}, 'Encoder')
        async with asyncio.timeout(8):
            while self.host.auth_state != 'authenticated':
                if self.host.failed.is_set():
                    raise RuntimeError('plugin unavailable')
                await asyncio.sleep(.05)
        self.recovery_required = False
        self.manual_recovery = False

    async def start(self):
        if self.enabled:
            try:
                await self.connect()
            except Exception:
                await self.host.stop()
                self.recovery_required=True
                self.status='Homebridge connection unavailable.'
        else:
            self.status='Homebridge is disabled. Assignments are preserved.'
        self.poll_task = asyncio.create_task(self.poll())
        self.catalog_task=asyncio.create_task(self.poll_catalog())
        if self.managed:
            self.recovery_task = asyncio.create_task(self.recover())

    def assigned_ready(self):
        return bool(self.enabled and self.host.ready.is_set() and self.host.auth_state=='authenticated'
            and not self.host.outcome_unknown and not self.manual_recovery
            and self.catalog.snapshot()['available'])

    async def poll_catalog(self):
        while True:
            if self.enabled:
                # Status reads must not starve during sustained dial writes.
                # Ignore a completed read if setup/removal replaced its session.
                catalog = self.catalog
                try:
                    await asyncio.to_thread(catalog.refresh)
                    if self.enabled and self.catalog is catalog:
                        self.assigned_states=catalog.states()
                except Exception:
                    if self.enabled and self.catalog is catalog:
                        self.assigned_states={}
            await asyncio.sleep(5)

    async def refresh(self):
        try:
            state = await asyncio.to_thread(self.read)
            if type(state.get('On')) not in (int, bool) or state['On'] not in (0, 1):
                raise ValueError('invalid power state')
            level = state.get('Brightness')
            if type(level) not in (int, float) or not 0 <= level <= 100:
                raise ValueError('invalid brightness')
            self.actual = {'on': bool(state['On']), 'brightness': level}
            self.updated = time.monotonic()
        except Exception:
            self.updated = 0
            self.recovery_required = True
            raise

    async def poll(self):
        while True:
            await asyncio.sleep(2)
            if not self.enabled or self.busy or not any(item['id']==TARGET for item in self.catalog.items):
                continue
            try:
                await self.refresh()
            except Exception:
                self.status = 'Live status unavailable. Controls paused; reconnect to try again.'

    async def recover(self):
        delay = 5
        while True:
            await asyncio.sleep(delay)
            if not self.enabled or self.busy or self.manual_recovery or self.host.outcome_unknown or self.catalog.auth_error:
                continue
            # Discovery can recover before legacy controls clear their lockout.
            # Confirm a fresh read without restarting a healthy plugin. Unknown
            # command outcomes are excluded by the manual/unknown gates above.
            if self.assigned_ready() and self.recovery_required:
                try:
                    await self.refresh()
                except Exception:
                    delay = min(delay * 2, 60)
                    continue
                self.recovery_required = False
            if self.snapshot()['ready'] or self.assigned_ready():
                delay = 5
                continue
            self.busy = True
            succeeded = await self.perform('reconnect')
            delay = 5 if succeeded else min(delay * 2, 60)

    def snapshot(self):
        self.setup.expire()
        fresh = bool(self.updated and time.monotonic() - self.updated < 6)
        if self.updated and not fresh:
            self.recovery_required = True
        ready = self.enabled and fresh and self.host.ready.is_set() and not self.host.outcome_unknown
        ready = ready and self.host.auth_state == 'authenticated' and not self.recovery_required
        display = self.host.display.get('brightness', {}).get('setFeedback', {})
        message = self.status
        if not self.busy and not ready:
            if self.host.outcome_unknown:
                message = 'Command outcome uncertain. Check the light, then reconnect. Nothing will be replayed.'
            elif self.recovery_required:
                message = 'Connection interrupted. Controls paused until you reconnect.'
            else:
                message = 'Plugin unavailable. Reconnect to resume controls.'
        return {'enabled':self.enabled,'server':self.connection_config['server'],'username':self.connection_config.get('username',''),'setup':self.setup.result,'has_saved_password':bool(self.connection_config.get('credential_profile')),'connection':self.connection_status(),
                'device_count':len(self.catalog.items) if self.catalog.snapshot()['available'] and self.enabled else 0,
                'target': 'Main_LED’s', 'actual': self.actual, 'fresh': fresh,
                'ready': ready, 'busy': self.busy, 'message': message,
                'feedback_fresh': bool(ready),
                'unknown': self.host.outcome_unknown, 'feedback': display.get('value'),
                'last_result': dict(self.host.completions), 'trace': list(self.trace)}

    def connection_status(self):
        if self.last_error=='settings_error':return 'settings_error'
        if self.connection_config.get('removed'):return 'not_configured'
        if not self.enabled:return 'disabled'
        if self.manual_recovery or self.host.outcome_unknown:return 'unconfirmed'
        if self.busy:return 'connecting'
        if self.catalog.auth_error or self.host.auth_state in ('required','rejected','two_factor'):return 'authentication_required'
        if self.assigned_ready():return self.last_error or 'connected'
        return 'disconnected'

    async def setup_command(self,data):
        if self.busy or not isinstance(data,dict) or data.get('operation') not in ('test','apply','remove'):return {'ok':False}
        self.busy=True
        self.command_task=asyncio.create_task(self.setup.perform(data))
        return {'ok':True}

    async def command(self, data):
        if not isinstance(data, dict) or set(data) != {'command'}:
            raise ValueError('fixed commands only')
        name = data['command']
        if name not in ('on', 'off', 'up', 'down', 'reconnect','enable','disable'):
            raise ValueError('unsupported command')
        if self.connection_config.get('removed'):return {'ok':False}
        if self.busy or (not self.enabled and name != 'enable') or (name not in ('reconnect','enable','disable') and not self.snapshot()['ready']):
            return {'ok': False}
        self.busy = True
        self.status = 'Reconnecting…' if name == 'reconnect' else 'Sending command…'
        self.command_task = asyncio.create_task(self.perform(name))
        return {'ok': True}

    async def perform(self, name, hardware_ticks=None):
        if self.connection_config.get('removed'):return False
        started=time.monotonic()
        trace={'command':name,'ticks':hardware_ticks}
        self.trace.append(trace)
        try:
            if name == 'disable':
                from connection_preferences import save
                save(self.preference_path,False)
                self.enabled=False
                self.connection_config['enabled']=False
                self.last_error=None
                self.assigned_states={}
                self.catalog.updated=0
                await self.host.stop()
                self.status='Homebridge disabled. Assignments preserved.'
                return True
            if name == 'enable':
                from connection_preferences import save
                save(self.preference_path,True)
                self.enabled=True
                self.connection_config['enabled']=True
            if name in ('reconnect','enable'):
                await self.host.stop()
                reuse=bool(self.catalog.token and self.catalog.expires>time.monotonic() and not self.catalog.auth_error and self.catalog.base==self.connection_config['server'])
                await self.connect(use_session=reuse)
                await asyncio.to_thread(self.catalog.refresh)
                self.assigned_states=self.catalog.states()
                self.last_error=None
                self.status = 'Reconnected. No previous command was replayed.'
                return True
            # Read immediately before a relative change; never compute from stale UI state.
            await self.refresh()
            trace['before']=dict(self.actual)
            if name in ('on', 'off'):
                expected = name == 'on'
                if not await self.host.input(name, 'keyUp', self.host.epoch):
                    raise RuntimeError('action failed')
                context = name
            else:
                if not self.actual['on']:
                    self.status = 'Light is off. Use the On key before adjusting brightness.'
                    trace['skipped'] = 'light_off'
                    return True
                ticks = hardware_ticks if hardware_ticks is not None else (5 if name == 'up' else -5)
                expected = max(0, min(100, self.actual['brightness'] + ticks))
                trace['expected']=expected
                context = 'brightness'
                if not await self.host.input(context, 'dialRotate', self.host.epoch, ticks=ticks):
                    raise RuntimeError('input rejected')
                async with asyncio.timeout(8):
                    while self.host.rotation_tasks:
                        await asyncio.sleep(.05)
                if self.host.completions.get(context) != 'confirmed':
                    raise RuntimeError('unconfirmed brightness')
                expected = self.host.brightness_results.get(context)
                if expected is None:
                    raise RuntimeError('missing brightness completion value')
                trace['confirmed_value']=expected
            async with asyncio.timeout(10):
                while True:
                    await self.refresh()
                    observed = self.actual['on' if name in ('on', 'off') else 'brightness']
                    trace['observed']=observed
                    # The pinned brightness action confirms within one percentage
                    # point (this target advertises a 1-point step). Some lights
                    # report 1 after a request for 0. Require the plugin's confirmed
                    # completion above, then independently check the same tolerance.
                    matched = (observed == expected if name in ('on', 'off')
                               else abs(observed - expected) <= 1)
                    if matched:
                        expected = observed  # Display the actual returned level.
                        break
                    await asyncio.sleep(.3)
            self.status = 'Homebridge confirms ' + (('on.' if expected else 'off.') if name in ('on', 'off') else f'{expected:g}% brightness.')
            return True
        except Exception:
            if name in ('enable','disable') and self.enabled != (name=='enable'):
                self.last_error='settings_error'
                self.status='Preference could not be saved; previous setting retained.'
                return False
            # A terminal negative acknowledgement ends this operation. A new user
            # input may proceed only after an independent read confirms availability.
            # Never retry the rejected command, or recover uncertain/lost replies here.
            context = name if name in ('on', 'off') else 'brightness'
            known_failure = (name != 'reconnect'
                and self.host.completions.get(context) == 'failed'
                and not self.host.outcome_unknown
                and self.host.ready.is_set()
                and self.host.auth_state == 'authenticated')
            self.recovery_required = True
            if name not in ('reconnect','enable','disable') and not known_failure:
                self.manual_recovery = True
            if known_failure:
                try:
                    await self.refresh()
                except Exception:
                    pass
                else:
                    self.recovery_required = False
            self.status = ('The last adjustment failed. Current light status refreshed; try a new input. Nothing was replayed.'
                if not self.recovery_required else
                'Command failed or could not be confirmed. Check the light; no automatic retry.')
            return False
        finally:
            trace['seconds']=round(time.monotonic()-started,3)
            trace['completion']=self.host.completions.get('brightness' if name in ('up','down') else name)
            # Store only parsed numeric diagnostics, never raw plugin logs/tokens.
            calculations=re.findall(rb'current=([0-9.]+), requested=([0-9.]+)',bytes(self.host.logs))
            if calculations:trace['plugin_calculation']=[float(x) for x in calculations[-1]]
            self.busy = False

    async def close(self):
        if self.catalog_task:
            self.catalog_task.cancel()
            await asyncio.gather(self.catalog_task,return_exceptions=True)
        if self.recovery_task:
            self.recovery_task.cancel()
            await asyncio.gather(self.recovery_task, return_exceptions=True)
        if self.poll_task:
            self.poll_task.cancel()
            await asyncio.gather(self.poll_task, return_exceptions=True)
        if self.command_task:
            await asyncio.gather(self.command_task, return_exceptions=True)
        await self.host.stop()
        self.temp.cleanup()


async def main(connection_file=None, hardware=False, managed=False):
    panel = LivePanel(managed=managed)
    loop = asyncio.get_running_loop()
    stop = asyncio.Event()
    token = secrets.token_urlsafe(32)
    bridge = None

    class Web(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def respond(self, code, body, content_type='application/json'):
            raw = body.encode()
            self.send_response(code)
            self.send_header('Content-Type', content_type)
            self.send_header('Content-Length', str(len(raw)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.send_header('Referrer-Policy', 'no-referrer')
            self.send_header('Content-Security-Policy', "default-src 'self'; script-src 'nonce-" + token + "'; style-src 'unsafe-inline'; frame-ancestors 'none'; base-uri 'none'")
            self.end_headers()
            self.wfile.write(raw)

        def valid_host(self):
            return self.headers.get('Host') == f'127.0.0.1:{self.server.server_port}'

        def do_GET(self):
            if not self.valid_host():
                self.respond(403, '{}'); return
            if self.path.startswith('/?token=') and secrets.compare_digest(self.path, '/?token=' + token):
                self.respond(200, (ROOT / 'live_panel.html').read_text().replace('__TOKEN__', token), 'text/html; charset=utf-8')
            elif self.path == '/catalog':
                if self.headers.get('X-Lab-Token') != token:
                    self.respond(403, '{}'); return
                async def catalog():
                    result=panel.catalog.snapshot()
                    result['available']=result['available'] and panel.enabled
                    return result
                self.respond(200, json.dumps(asyncio.run_coroutine_threadsafe(catalog(), loop).result(3)))
            elif self.path == '/state':
                if self.headers.get('X-Lab-Token') != token:
                    self.respond(403, '{}'); return
                async def state():
                    return panel.snapshot()
                self.respond(200, json.dumps(asyncio.run_coroutine_threadsafe(state(), loop).result(3)))
            else:
                self.respond(404, '{}')

        def do_POST(self):
            if not self.valid_host() or self.headers.get('X-Lab-Token') != token:
                self.respond(403, '{}'); return
            origin = self.headers.get('Origin')
            if origin and origin != f'http://127.0.0.1:{self.server.server_port}':
                self.respond(403, '{}'); return
            try:
                length = int(self.headers.get('Content-Length', '0'))
                if self.path not in ('/command','/setup') or not 0 < length <= (8192 if self.path=='/setup' else 128):
                    raise ValueError()
                data = json.loads(self.rfile.read(length))
                result = asyncio.run_coroutine_threadsafe(panel.setup_command(data) if self.path=='/setup' else panel.command(data), loop).result(3)
                self.respond(200, json.dumps(result))
            except (ValueError, TimeoutError):
                self.respond(400, '{}')

    web = ThreadingHTTPServer(('127.0.0.1', 0), Web)
    web.daemon_threads = True
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, stop.set)
    try:
        await panel.start()
        if hardware:
            from hardware_bridge import HardwareBridge
            bridge=HardwareBridge(panel)
            await bridge.start()
        threading.Thread(target=web.serve_forever, daemon=True).start()
        if connection_file is not None:
            descriptor = {'version': 1, 'url': f'http://127.0.0.1:{web.server_port}', 'token': token,
                          'provider': 'OpenHomeB', 'target': 'Main_LED’s'}
            fd = os.open(connection_file, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(fd, 'w') as output:
                json.dump(descriptor, output)
        if not managed:print(f'http://127.0.0.1:{web.server_port}/?token={token}', flush=True)
        await stop.wait()
    finally:
        # shutdown only after serve_forever has started.
        if panel.poll_task:
            await asyncio.to_thread(web.shutdown)
        web.server_close()
        if bridge:await bridge.close()
        await panel.close()
        if connection_file is not None:
            try:
                current = json.loads(Path(connection_file).read_text())
                if current.get('token') == token:
                    Path(connection_file).unlink()
            except (OSError, ValueError):
                pass


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--live-main-led', action='store_true', required=True)
    parser.add_argument('--connection-file', type=Path)
    parser.add_argument('--hardware', action='store_true')
    args = parser.parse_args()
    asyncio.run(main(args.connection_file,args.hardware))
