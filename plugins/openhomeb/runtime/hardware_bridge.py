"""Single-accessory Unix IPC trial; EOF cancels pending plugin work."""
import asyncio
import json
import os
from pathlib import Path
import socket
import struct
from live_on_off_trial import TARGET


class HardwareBridge:
    def __init__(self, panel):
        self.panel=panel
        self.path=Path(os.environ['XDG_RUNTIME_DIR'])/'decksmith-plugin-lab.sock'
        self.state_path=self.path.with_name('decksmith-plugin-state.json')
        self.server=None;self.publisher=None;self.connections=set()

    async def start(self):
        if self.path.exists():raise RuntimeError('A plugin bridge socket already exists')
        self.server=await asyncio.start_unix_server(self.handle,path=self.path,limit=8192)
        self.path.chmod(0o600)
        self.publisher=asyncio.create_task(self.publish())

    async def publish(self):
        while True:
            state=self.panel.snapshot()
            data={'target':TARGET,'ready':state['ready'],'actual':state['actual'],'accessories':self.panel.assigned_states,'catalog_ready':self.panel.assigned_ready(),'enabled':self.panel.enabled,'manager_ready':True}
            temporary=self.state_path.with_suffix('.tmp')
            fd=os.open(temporary,os.O_WRONLY|os.O_CREAT|os.O_TRUNC|os.O_NOFOLLOW,0o600)
            with os.fdopen(fd,'w') as output:json.dump(data,output)
            os.replace(temporary,self.state_path)
            await asyncio.sleep(.5)

    @staticmethod
    def decode(data):
        if not isinstance(data,dict) or set(data)!={'binding','ticks'}:raise ValueError()
        b=data['binding'];ticks=data['ticks']
        if not isinstance(b,dict) or set(b)!={'provider','action','schema','settings'} or b['provider']!='com.infamous-pattern.openhomeb' or type(b['schema']) is not int or b['schema']!=1:raise ValueError()
        s=b['settings']
        if not isinstance(s,dict) or s.get('accessoryId')!=TARGET or type(ticks) is not int:raise ValueError()
        if b['action']=='com.infamous-pattern.openhomeb.set' and ticks==0 and s.get('characteristicType')=='On' and type(s.get('targetValue')) is bool:
            return ('on' if s['targetValue'] else 'off'),None
        if b['action']=='com.infamous-pattern.openhomeb.brightness' and 0<abs(ticks)<=20 and s.get('characteristicType')=='Brightness' and s.get('turnOnWhenAdjusting') is False:
            return ('up' if ticks>0 else 'down'),ticks
        raise ValueError()

    async def handle(self, reader, writer):
        task=asyncio.current_task();self.connections.add(task)
        operation=None;closed=None
        try:
            peer=writer.get_extra_info('socket').getsockopt(socket.SOL_SOCKET,socket.SO_PEERCRED,12)
            if struct.unpack('3i',peer)[1]!=os.getuid():raise ValueError()
            raw=await asyncio.wait_for(reader.readline(),1)
            if len(raw)>8192 or not raw.endswith(b'\n'):raise ValueError()
            data=json.loads(raw)
            generic=isinstance(data,dict) and isinstance(data.get('binding'),dict) and data['binding'].get('schema')==2
            if generic:
                from accessory_actions import decode,perform
                if set(data)!={'binding','ticks'}:raise ValueError()
                decode(data['binding'],data['ticks'])
            else:
                name,ticks=self.decode(data)
            if self.panel.busy or not (self.panel.assigned_ready() if generic else self.panel.snapshot()['ready']):raise ValueError()
            self.panel.busy=True
            operation=asyncio.create_task(perform(self.panel,data['binding'],data['ticks']) if generic else self.panel.perform(name,ticks))
            closed=asyncio.create_task(reader.read(1))
            done,_=await asyncio.wait([operation,closed],timeout=4.5,return_when=asyncio.FIRST_COMPLETED)
            if operation not in done or closed in done:
                operation.cancel();await asyncio.gather(operation,return_exceptions=True)
                await self.panel.host.stop()
                self.panel.recovery_required=True
                self.panel.manual_recovery=True
                self.panel.status='Hardware input cancelled. Reconnect; no input will be replayed.'
                return
            completed = await operation
            success = completed is True and (not self.panel.manual_recovery if generic else not self.panel.recovery_required)
            writer.write(json.dumps({'ok':success}).encode()+b'\n')
            await writer.drain()
        except (ValueError,KeyError,TypeError,asyncio.TimeoutError,ConnectionError):
            writer.write(b'{"ok":false}\n')
            try:await writer.drain()
            except ConnectionError:pass
        finally:
            if closed:closed.cancel()
            if operation and not operation.done():
                operation.cancel();await asyncio.gather(operation,return_exceptions=True)
                await self.panel.host.stop();self.panel.recovery_required=True
                self.panel.manual_recovery=True
            writer.close()
            self.connections.discard(task)

    async def close(self):
        if self.server:self.server.close();await self.server.wait_closed()
        for task in list(self.connections):task.cancel()
        await asyncio.gather(*list(self.connections),return_exceptions=True)
        if self.publisher:self.publisher.cancel();await asyncio.gather(self.publisher,return_exceptions=True)
        if self.server:
            self.path.unlink(missing_ok=True);self.state_path.unlink(missing_ok=True)
