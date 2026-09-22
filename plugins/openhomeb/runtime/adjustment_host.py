"""Explicit opt-in for the separately built adjustment completion experiment."""
import asyncio
import copy
import hashlib
import json
import math
import time
from pathlib import Path
from paced_host import PacedOpenHomeHost, BRIGHTNESS

ADJUST = 'com.infamous-pattern.openhomeb.adjust'
BUILD = Path(__file__).resolve().parent.parent / 'openhomeb-adjust-completion'
BINARY = BUILD / 'binaries/openhomeb'

class PacedAdjustmentTestHost(PacedOpenHomeHost):
    contracts = {**PacedOpenHomeHost.contracts, (ADJUST, 'Encoder'): {'dialRotate'}}
    accepted_actions = PacedOpenHomeHost.accepted_actions | {ADJUST}

    def __init__(self, *args):
        super().__init__(*args)
        self.settings_ready = asyncio.Event()
        self.auth_state = 'connecting'
        self.pending_otp = None
        self.access_session = None
        self.brightness_results = {}

    async def wait_settings_ready(self):
        await asyncio.wait_for(self.settings_ready.wait(), 5)

    async def send(self, data):
        if data.get('event') == 'didReceiveGlobalSettings':
            data = copy.deepcopy(data)
            data['payload']['settings']['_decksmithSession'] = self.token
            if self.access_session:
                access_token,expires=self.access_session
                data['payload']['settings']['_decksmithAccessToken']=access_token
                data['payload']['settings']['_decksmithTokenSeconds']=max(1,int(expires-time.monotonic()))
            if self.pending_otp is not None:
                data['payload']['settings']['_decksmithOtp'] = self.pending_otp
                self.pending_otp = None
        await super().send(data)

    async def emit(self, context, event, **extra):
        if event == 'dialRotate':
            self.brightness_results.pop(context, None)
        await super().emit(context, event, **extra)

    async def handle(self, message):
        if message.get('event') == 'decksmithBrightnessConfirmed':
            wire=message.get('context')
            context=self.wires.get(wire)
            value=message.get('payload',{}).get('value')
            if (not self.pending or wire != self.pending[0] or self.instances.get(context,{}).get('action') != BRIGHTNESS
                or type(value) not in (int,float) or not math.isfinite(value) or not 0 <= value <= 100):
                raise ValueError('invalid brightness completion')
            self.brightness_results[context]=value
            return
        if message.get('event') == 'decksmithSettingsReady':
            if message.get('payload') != {'session': self.token}:
                raise ValueError('invalid settings acknowledgement')
            self.settings_ready.set()
            return
        if message.get('event') == 'decksmithAuth':
            state = message.get('payload', {}).get('state')
            if state not in ('connecting','authenticated','required','rejected','two_factor'):
                raise ValueError('invalid authentication status')
            self.auth_state = state
            return
        await super().handle(message)

    def save(self):
        # Credentials stay in memory for the child; never serialize them to JSON.
        original = self.saved
        def redact(value):
            if isinstance(value, dict):
                return {k:redact(v) for k,v in value.items() if k.lower() not in
                        {'password','otp','token','access_token','_decksmithsession','_decksmithotp','_decksmithaccesstoken'}}
            if isinstance(value, list): return [redact(v) for v in value]
            return value
        self.saved = redact(original)
        try: super().save()
        finally: self.saved = original

    async def load_password(self, store, profile):
        password = await store.lookup(profile)
        if password is None: raise RuntimeError('No saved Homebridge credential')
        self.saved['global']['password'] = password

    async def start(self, command):
        # Never apply this new completion assumption to the unpatched release.
        expected = json.loads((BUILD/'verification/build-receipt.json').read_text())['binary_sha256']
        if len(command) != 1 or Path(command[0]).resolve() != BINARY.resolve():
            raise ValueError('adjustment completion requires the isolated patched binary')
        if hashlib.sha256(BINARY.read_bytes()).hexdigest() != expected:
            raise ValueError('patched binary checksum mismatch')
        self.brightness_results.clear()
        self.settings_ready.clear()
        self.auth_state = 'connecting'
        try: await super().start(command)
        finally: self.pending_otp = None
