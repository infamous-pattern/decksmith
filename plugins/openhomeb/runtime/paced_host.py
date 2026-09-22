"""Opt-in lab policy for the inspected OpenHomeB 2.0.2 actions with explicit completion.
Not a generic OpenAction acknowledgement protocol or production adapter.
"""
import asyncio
import os
import signal
from host import Host

BRIGHTNESS = 'com.infamous-pattern.openhomeb.brightness'

SWITCH = 'com.infamous-pattern.openhomeb.switch'
SET = 'com.infamous-pattern.openhomeb.set'
CONTRACTS = {
    (BRIGHTNESS, 'Encoder'): {'dialRotate', 'dialDown'},
    (BRIGHTNESS, 'Keypad'): {'keyDown'},
    (SWITCH, 'Keypad'): {'keyDown'},
    (SET, 'Keypad'): {'keyUp'},
}

class PacedOpenHomeHost(Host):
    contracts = CONTRACTS
    accepted_actions = {SET}
    def __init__(self, *args):
        super().__init__(*args)
        self.write_gate = asyncio.Lock()
        self.pending = None
        self.completion_timeout = 3
        self.outcome_unknown = False
        self.dispatched = 0
        self.completions = {}

    async def start(self, command):
        if self.pending or self.write_gate.locked():
            raise RuntimeError('previous dispatch has not finished')
        if self.process or self.server:
            raise ValueError('already running')
        self.outcome_unknown = False
        self.completions.clear()
        self.logs.clear()
        await super().start(command)
        try:
            await self.wait_settings_ready()
        except BaseException:
            await self.stop()
            raise

    async def wait_settings_ready(self):
        # Compatibility for the unmodified release; the patched adapter overrides this.
        async with asyncio.timeout(5):
            while b'Homebridge settings loaded' not in self.logs:
                if self.failed.is_set(): raise RuntimeError('plugin exited before settings loaded')
                await asyncio.sleep(.01)

    async def appear(self, context, action, settings, controller='Keypad'):
        effective = self.saved['instances'].get(context, settings)
        self.validate_policy(action, controller, effective)
        await super().appear(context, action, settings, controller)

    def validate_policy(self, action, controller, settings):
        if (action, controller) not in self.contracts:
            raise ValueError('action/controller has no validated completion contract')
        if action in (BRIGHTNESS, SWITCH) and settings.get('showConfirmation', True) is not True:
            raise ValueError('completion policy requires confirmation enabled')

    async def input(self, context, event, epoch, **extra):
        instance = self.instances.get(context)
        if not instance or event not in self.contracts[(instance['action'], instance['controller'])]:
            return False
        # Key presses never accumulate as waiting coroutines behind a slow write.
        if event != 'dialRotate' and (self.write_gate.locked() or self.rotation_tasks):
            self.diagnostics.append('plugin busy; input rejected')
            return False
        try:
            return await super().input(context, event, epoch, **extra)
        except (RuntimeError, asyncio.TimeoutError):
            return False

    def abandon_pending(self, context=None):
        if self.pending and (context is None or self.instances.get(context, {}).get('wire') == self.pending[0]):
            for context_id, instance in self.instances.items():
                if instance['wire'] == self.pending[0]:
                    self.completions[context_id] = 'unknown'
            self.quarantine()
            if not self.pending[1].done():
                self.pending[1].set_exception(RuntimeError('action abandoned; outcome unknown'))

    async def disappear(self, context):
        self.abandon_pending(context)
        await super().disappear(context)
        self.completions.pop(context, None)

    async def stop(self):
        self.abandon_pending()
        await super().stop()

    def quarantine(self):
        if self.outcome_unknown: return
        self.outcome_unknown = True
        self.ready.clear(); self.failed.set(); self.epoch += 1
        self.diagnostics.append('action outcome unknown; plugin stopped, no automatic retry')
        if self.process:
            try: os.killpg(self.process.pid, signal.SIGTERM)
            except ProcessLookupError: pass
            self.tasks.append(asyncio.create_task(self.reap_quarantined(self.process)))
        # No cancellation of the dispatch task from itself.

    async def reap_quarantined(self, child):
        try:
            await asyncio.wait_for(child.wait(), 1)
        except asyncio.TimeoutError:
            try: os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            await child.wait()

    async def emit(self, context, event, **extra):
        instance = self.instances[context]
        if event not in self.contracts[(instance['action'], instance['controller'])]:
            return await super().emit(context, event, **extra)
        epoch = self.epoch
        async with self.write_gate:
            if epoch != self.epoch or not self.ready.is_set() or self.locked:
                raise RuntimeError('stale input')
            future = asyncio.get_running_loop().create_future()
            self.pending = (self.instances[context]['wire'], future)
            try:
                await super().emit(context, event, **extra)
                self.dispatched += 1
                self.completions[context] = 'pending'
                success = await asyncio.wait_for(future, self.completion_timeout)
                self.completions[context] = ('accepted' if instance['action'] in self.accepted_actions else 'confirmed') if success else 'failed'
                if not success:
                    raise RuntimeError('plugin reported failure; discard remaining batch')
            except (asyncio.TimeoutError, asyncio.CancelledError):
                self.completions[context] = 'unknown'
                self.quarantine()
                raise
            finally:
                self.pending = None

    async def handle(self, message):
        if message.get('event') == 'setSettings' and message.get('context') in self.wires:
            instance = self.instances[self.wires[message['context']]]
            self.validate_policy(instance['action'], instance['controller'], message.get('payload', {}))
        await super().handle(message)
        if self.pending and message.get('context') == self.pending[0]:
            event = message.get('event')
            if event in ('showOk', 'showAlert') and not self.pending[1].done():
                self.pending[1].set_result(event == 'showOk')

# Compatibility name for the earlier brightness-only test suite.
PacedBrightnessHost = PacedOpenHomeHost
