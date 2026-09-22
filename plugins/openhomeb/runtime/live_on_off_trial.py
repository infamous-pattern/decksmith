"""Explicit, single-accessory live trial. Run only with the user's approval."""
import argparse
import asyncio
import json
import tempfile
import urllib.request
from pathlib import Path

from adjustment_host import BINARY, PacedAdjustmentTestHost
from paced_host import SET

ROOT = Path(__file__).resolve().parent
BASE = 'http://homebridge.local:8581'
TARGET = 'b3d109c968d17f5cc965ddfa89a1fa695a2d60832200b819ad08127ee15634b4'


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def reader():
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    with opener.open(urllib.request.Request(BASE + '/api/auth/noauth', data=b'', method='POST'), timeout=10) as response:
        token = json.load(response)['access_token']

    def read():
        request = urllib.request.Request(BASE + '/api/accessories/' + TARGET,
                                         headers={'Authorization': 'Bearer ' + token})
        with opener.open(request, timeout=10) as response:
            service = json.load(response)
        assert service['uniqueId'] == TARGET
        return {c['type']: c['value'] for c in service['serviceCharacteristics']
                if c['type'] in ('On', 'Brightness', 'Hue', 'Saturation', 'ColorTemperature')}
    return read


async def main():
    read = await asyncio.to_thread(reader)
    report = {'target': 'Main_LED’s', 'server': BASE, 'steps': []}
    report['before'] = await asyncio.to_thread(read)
    if bool(report['before']['On']):
        raise RuntimeError('Target is already on; stop because the agreed starting state changed.')
    manifest = json.loads((ROOT.parent / 'openhomeb-2.0.2/source/assets/manifest.json').read_text())
    with tempfile.TemporaryDirectory(prefix='decksmith-live-trial-') as tmp:
        host = PacedAdjustmentTestHost(manifest, Path(tmp) / 'settings.json')
        host.saved['global'] = {'homebridgeUrl': BASE, 'updateInterval': 5, 'catalogCacheSeconds': 30}
        attempted = False

        async def setup():
            await host.start([str(BINARY)])
            for name, value in [('on', True), ('off', False)]:
                await host.appear(name, SET, {'accessoryId': TARGET,
                                            'characteristicType': 'On', 'targetValue': value})

        async def send(name):
            accepted = await host.input(name, 'keyUp', host.epoch)
            report['steps'].append({'action': name, 'dispatched': accepted,
                                    'completion': host.completions.get(name)})
            return accepted

        async def verify(power):
            async with asyncio.timeout(15):
                while True:
                    state = await asyncio.to_thread(read)
                    if bool(state['On']) == power:
                        return state
                    await asyncio.sleep(.5)

        try:
            await setup()
            attempted = True
            if not await send('on'):
                raise RuntimeError('On result failed or uncertain; do not replay it.')
            report['on_readback'] = await verify(True)
            print('Homebridge confirms Main_LED’s is ON. Returning it to OFF in five seconds.', flush=True)
            await asyncio.sleep(5)
        finally:
            try:
                if attempted:
                    # A fresh explicit OFF is recovery, never a replay of an uncertain ON.
                    if host.outcome_unknown or not host.ready.is_set():
                        await host.stop()
                        await setup()
                    await send('off')
                    report['after'] = await verify(False)
                    report['other_values_preserved'] = all(
                        report['after'].get(k) == v for k, v in report['before'].items() if k != 'On')
                    print('Homebridge confirms Main_LED’s is OFF.', flush=True)
            finally:
                await host.stop()
                (ROOT / 'live-on-off-results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--approved-on-off', action='store_true', required=True)
    parser.parse_args()
    asyncio.run(main())
