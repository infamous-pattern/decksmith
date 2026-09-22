"""Isolated OpenAction compatibility experiment. No physical-device/app access."""
import asyncio
import collections
import copy
import json
import os
from pathlib import Path
import secrets
import signal
import time
from websockets.asyncio.server import serve
from websockets.exceptions import ConnectionClosed

LIMIT = 65536

def encode(value):
    data = json.dumps(value, allow_nan=False)
    if len(data.encode()) > LIMIT: raise ValueError('message too large')
    return data

class Host:
    def __init__(self, manifest, settings_path):
        self.manifest = manifest
        self.actions = {a['UUID']: a for a in manifest['Actions']}
        if not 1 <= len(self.actions) <= 64: raise ValueError('action limit')
        self.path = Path(settings_path)
        self.saved = {'global': {}, 'instances': {}}
        if self.path.exists():
            if self.path.stat().st_size > LIMIT: raise ValueError('settings too large')
            self.saved = json.loads(self.path.read_text())
        self.instances = {}; self.display = {}; self.diagnostics = collections.deque(maxlen=32)
        self.logs = bytearray(); self.tasks = []; self.process = self.server = self.socket = None
        self.ready = asyncio.Event(); self.failed = asyncio.Event(); self.epoch = 0; self.locked = False
        self.token = ''; self.received = 0
        self.wires={}; self.retired=collections.deque(maxlen=256)
        self.rotations={}; self.rotation_tasks={}; self.rotation_events=0; self.rotation_rejections=0

    def save(self):
        data = encode(self.saved)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        temporary = self.path.with_suffix('.tmp')
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, 'w') as output: output.write(data)
        os.replace(temporary, self.path)

    async def start(self, command):
        if self.process or self.server: raise ValueError('already running')
        self.token = secrets.token_hex(24); self.ready.clear(); self.failed.clear()
        self.display.clear(); self.instances.clear(); self.wires.clear(); self.epoch += 1
        self.server = await serve(self.connection, '127.0.0.1', 0, origins=[None],
            compression=None, max_size=LIMIT, max_queue=8, write_limit=LIMIT,
            open_timeout=2, close_timeout=1, ping_interval=10, ping_timeout=5)
        port = self.server.sockets[0].getsockname()[1]
        info = {'devices':[{'id':'virtual-plus','name':'Decksmith test device',
                            'size':{'rows':2,'columns':4},'type':7}]}
        env = {key: os.environ[key] for key in ('PATH','LANG','LC_ALL') if key in os.environ}
        env['HOME'] = str(self.path.parent.resolve())
        try:
            self.process = await asyncio.create_subprocess_exec(*command,
                '-port',str(port),'-pluginUUID',self.token,'-registerEvent','registerPlugin',
                '-info',encode(info), cwd=self.path.parent, env=env, start_new_session=True,
                stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.STDOUT)
            self.tasks = [asyncio.create_task(self.drain()), asyncio.create_task(self.watch())]
            await asyncio.wait_for(self.ready.wait(), 5)
        except BaseException:
            await self.stop(); raise

    async def drain(self):
        while chunk := await self.process.stdout.read(4096):
            self.logs.extend(chunk); del self.logs[:-8192]

    async def watch(self):
        await self.process.wait(); self.failed.set(); self.ready.clear()
        if self.socket: await self.socket.close()

    async def stop(self):
        self.ready.clear(); self.epoch += 1
        await self.cancel_rotations()
        if self.process:
            # Also clean up descendants of a plugin that exited first.
            try: os.killpg(self.process.pid, signal.SIGTERM)
            except ProcessLookupError: pass
            try: await asyncio.wait_for(self.process.wait(), 1)
            except asyncio.TimeoutError:
                try: os.killpg(self.process.pid, signal.SIGKILL)
                except ProcessLookupError: pass
                await self.process.wait()
        if self.server:
            self.server.close(); await self.server.wait_closed()
        for task in self.tasks: task.cancel()
        await asyncio.gather(*self.tasks, return_exceptions=True)
        self.tasks=[]; self.process=self.server=self.socket=None

    async def set_locked(self, locked):
        self.locked=locked; self.epoch+=1
        if locked: await self.stop()  # Drop any pre-lock plugin queue.
        # Caller must explicitly restart and re-appear before accepting new input.

    async def send(self, data):
        if not self.socket: raise RuntimeError('plugin unavailable')
        await asyncio.wait_for(self.socket.send(encode(data)), 1)

    async def connection(self, socket):
        accepted=False
        try:
            hello=json.loads(await asyncio.wait_for(socket.recv(),2))
            if hello != {'event':'registerPlugin','uuid':self.token} or self.socket:
                await socket.close(1008,'registration rejected'); return
            self.socket=socket; accepted=True; self.ready.set()
            window=time.monotonic(); count=0
            async for raw in socket:
                now=time.monotonic()
                if now-window>=1: window=now; count=0
                count+=1
                if count>200: raise ValueError('message rate exceeded')
                message=json.loads(raw)
                if not isinstance(message,dict): raise ValueError('invalid message')
                self.received+=1
                await self.handle(message)
        except (ValueError, KeyError, TypeError, asyncio.TimeoutError, ConnectionClosed):
            self.diagnostics.append('connection closed or invalid protocol message')
            await socket.close(1008,'protocol failure')
        finally:
            if accepted:
                self.socket=None; self.ready.clear(); self.failed.set()

    async def handle(self, m):
        event=m['event']; context=m.get('context'); payload=m.get('payload',{})
        if event in ('getGlobalSettings','setGlobalSettings'):
            if context!=self.token: raise ValueError('foreign plugin')
            if event=='setGlobalSettings':
                if not isinstance(payload,dict): raise ValueError('invalid settings')
                candidate=copy.deepcopy(self.saved); candidate['global']=payload; encode(candidate)
                self.saved=candidate; self.save()
            await self.send({'event':'didReceiveGlobalSettings','payload':{'settings':self.saved['global']}})
            return
        if context in self.retired: return  # Late response from a previous page incarnation.
        if context not in self.wires: raise ValueError('unknown context')
        context=self.wires[context]
        if event in ('getSettings','setSettings'):
            if event=='setSettings':
                if not isinstance(payload,dict): raise ValueError('invalid settings')
                candidate=copy.deepcopy(self.saved); candidate['instances'][context]=payload; encode(candidate)
                self.saved=candidate; self.instances[context]['settings']=payload; self.save()
            await self.emit(context,'didReceiveSettings'); return
        if event=='setTitle':
            title=payload.get('title')
            if title is not None and (not isinstance(title,str) or len(title)>256): raise ValueError('title limit')
        elif event=='setState':
            state=payload['state']
            if type(state) is not int or not 0<=state<len(self.actions[self.instances[context]['action']].get('States',[])):
                raise ValueError('invalid state')
        elif event=='setFeedback':
            if not isinstance(payload,dict) or len(encode(payload))>4096: raise ValueError('feedback limit')
            if set(payload)-{'title','value','indicator'}: raise ValueError('feedback field unsupported')
        elif event not in ('showOk','showAlert'):
            # No URL launching, image loading or property inspector execution.
            self.diagnostics.append('unsupported event: '+str(event)[:60]); return
        self.display.setdefault(context,{})[event]=payload  # latest-only virtual display

    async def appear(self, context, action, settings, controller='Keypad'):
        if action not in self.actions or controller not in self.actions[action].get('Controllers',[]):
            raise ValueError('invalid action/controller')
        if not isinstance(context,str) or not 1<=len(context)<=128: raise ValueError('context limit')
        if not isinstance(settings,dict): raise ValueError('invalid settings')
        if context not in self.instances and len(self.instances)>=12: raise ValueError('instance limit')
        encode(settings)
        settings=self.saved['instances'].get(context,settings)
        if context in self.instances: await self.disappear(context)
        wire=secrets.token_hex(16)
        self.instances[context]={'action':action,'settings':settings,'controller':controller,'wire':wire}
        self.wires[wire]=context
        await self.emit(context,'willAppear')

    async def emit(self, context, event, **extra):
        instance=self.instances[context]
        payload={'settings':instance['settings'],'coordinates':{'row':0,'column':0},
                 'controller':instance['controller'],'state':self.display.get(context,{}).get('setState',{}).get('state',0),'isInMultiAction':False,**extra}
        await self.send({'event':event,'context':instance['wire'],'action':instance['action'],
                         'device':'virtual-plus','payload':payload})

    async def input(self, context, event, epoch, **extra):
        if self.locked or epoch!=self.epoch or not self.ready.is_set() or context not in self.instances: return False
        if event not in ('keyDown','keyUp','dialDown','dialUp','dialRotate','touchTap'): raise ValueError('invalid input')
        if event=='dialRotate' and (type(extra.get('ticks')) is not int or not -20<=extra['ticks']<=20): raise ValueError('tick limit')
        if event=='dialRotate':
            if self.instances[context]['controller']!='Encoder': raise ValueError('not a dial')
            return self.queue_rotation(context,extra['ticks'],extra.get('pressed',False))
        await self.emit(context,event,**extra); return True

    async def disappear(self, context):
        if context not in self.instances: return
        await self.cancel_rotations(context)
        instance=self.instances[context]
        wire=instance['wire']
        self.wires.pop(wire,None); self.retired.append(wire)
        try:
            if self.ready.is_set(): await self.emit(context,'willDisappear')
        finally:
            self.instances.pop(context,None); self.display.pop(context,None)

    async def change_page(self):
        self.epoch+=1
        for context in list(self.instances): await self.disappear(context)

    async def cancel_rotations(self, context=None):
        keys=list(self.rotation_tasks) if context is None else [context]
        tasks=[]
        for key in keys:
            task=self.rotation_tasks.pop(key,None)
            if task: task.cancel(); tasks.append(task)
            self.rotations.pop(key,None)
        await asyncio.gather(*tasks,return_exceptions=True)

    def queue_rotation(self, context, ticks, pressed):
        if type(pressed) is not bool: raise ValueError('invalid pressed state')
        if ticks==0: return True
        queue=self.rotations.setdefault(context,collections.deque())
        combine=bool(queue and queue[-1][0]*ticks>0 and queue[-1][1]==pressed)
        if sum(abs(n) for n,_ in queue)+abs(ticks)>200 or (not combine and len(queue)>=16):
            self.rotation_rejections+=1
            self.diagnostics.append('dial backlog full; input rejected')
            return False
        if combine: queue[-1][0]+=ticks
        else: queue.append([ticks,pressed])
        if context not in self.rotation_tasks:
            self.rotation_tasks[context]=asyncio.create_task(self.flush_rotation(context,self.epoch))
        return True

    async def flush_rotation(self, context, epoch):
        try:
            await asyncio.sleep(.05)  # Combine a short burst without losing direction changes.
            queue=self.rotations[context]
            while queue and epoch==self.epoch and self.ready.is_set() and not self.locked:
                ticks,pressed=queue[0]; amount=max(-20,min(20,ticks))
                queue[0][0]-=amount
                if queue[0][0]==0: queue.popleft()
                await self.emit(context,'dialRotate',ticks=amount,pressed=pressed)
                self.rotation_events+=1
                await asyncio.sleep(.05)
        except (ConnectionClosed,RuntimeError,asyncio.TimeoutError):
            self.diagnostics.append('dial dispatch failed; remaining input discarded')
        finally:
            self.rotations.pop(context,None); self.rotation_tasks.pop(context,None)
