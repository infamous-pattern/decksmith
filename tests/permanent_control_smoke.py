#!/usr/bin/env python3
"""Private-bus regression for the selected-device facade; never opens physical devices."""
import ast
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def call(bus, path, interface, method, *arguments):
    # A timeout must kill only this client, never a system or desktop service.
    result = subprocess.run(
        ['gdbus', 'call', '--session', '--dest', bus, '--object-path', path,
         '--method', interface + '.' + method, *arguments],
        text=True, capture_output=True, timeout=3, check=False,
    )
    if result.returncode:
        raise RuntimeError(method + ": " + result.stderr.strip())
    return ast.literal_eval(result.stdout)


def main(binary):
    assert os.environ.get('DECKSMITH_ISOLATED_TEST') == '1'
    assert os.environ.get('DBUS_SESSION_BUS_ADDRESS', '').startswith('unix:path=/tmp/dbus-')
    with tempfile.TemporaryDirectory(prefix='decksmith-permanent-bus-') as temporary:
        root = Path(temporary)
        # Container checkouts can belong to the outer runner UID. Give this
        # private fixture its own resources without weakening ownership checks.
        resources = root / 'resources'
        shutil.copytree(Path(__file__).resolve().parents[1] / 'config', resources / 'config')
        env = dict(os.environ, PULSE_SERVER='unix:' + str(root / 'no-audio'),
                   PIPEWIRE_REMOTE='no-pipewire', DECKSMITH_RESOURCE_ROOT=str(resources))
        for name in ('CONFIG', 'DATA', 'STATE', 'CACHE'):
            directory = root / name.lower()
            directory.mkdir()
            env['XDG_' + name + '_HOME'] = str(directory)
        workspace = ('cc.senecal.Decksmith.ConnectedWorkspace',
                     '/cc/senecal/Decksmith/ConnectedWorkspace',
                     'cc.senecal.Decksmith.DeviceWorkspace2')
        facade = ('cc.senecal.Decksmith', '/cc/senecal/Decksmith',
                  'cc.senecal.Decksmith.Control1')
        with (root / 'daemon.log').open('w+') as log:
            daemon = subprocess.Popen([str(binary), '--multi-device-virtual',
                                       str(root / 'profiles'), '--seconds', '30'],
                                      env=env, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 10
                while True:
                    try:
                        catalog = json.loads(call(*workspace, 'ListDevices')[0])
                        if len(catalog.get('sessions', [])) == 2 and all(
                            d['input_ready'] and d['display_ready']
                            for d in catalog['sessions']
                        ):
                            break
                    except (subprocess.SubprocessError, ValueError, RuntimeError):
                        pass
                    assert daemon.poll() is None and time.monotonic() < deadline, 'Virtual workspace not ready'
                    time.sleep(.05)
                expected = {d['id']: d['brightness'] for d in catalog['sessions']}
                models = set()
                for device in catalog['devices']:
                    current = json.loads(call(*workspace, 'ListDevices')[0])
                    call(*workspace, 'SelectDevice', device['id'], str(current['generation']))
                    # The facade itself calls the workspace on the same bus. This
                    # catches synchronous handlers stalling zbus's helper runtime.
                    status = json.loads(call(*facade, 'GetStatus')[0])
                    assert status['selected_device'] == device['id']
                    models.add(status['device_model'])
                    document = json.loads(call(*facade, 'GetLayout')[0])
                    assert len(document['pages'][0]['keys']) == (15 if status['device_model'] == 'mk2' else 8)
                    document['pages'][0]['keys'][0]['label'] = 'Bus regression ✅'
                    call(*facade, 'SaveLayout', json.dumps(document))
                    assert json.loads(call(*facade, 'GetLayout')[0])['pages'][0]['keys'][0]['label'] == 'Bus regression ✅'
                    end = time.monotonic() + 3
                    while True:
                        ready = json.loads(call(*facade, 'GetStatus')[0])
                        if ready['input_ready'] and ready['display_ready']:
                            break
                        assert time.monotonic() < end, 'Saved device did not settle'
                        time.sleep(.05)
                    value = 37 if status['device_model'] == 'mk2' else 63
                    call(*facade, 'SetBrightness', str(value))
                    expected[device['id']] = value
                    end = time.monotonic() + 3
                    while True:
                        final = json.loads(call(*workspace, 'ListDevices')[0])
                        actual = {d['id']: d['brightness'] for d in final['sessions']}
                        if actual == expected:
                            break
                        assert time.monotonic() < end, 'Brightness did not settle independently'
                        time.sleep(.05)
                assert models == {'mk2', 'plus'}
            except BaseException:
                log.flush()
                log.seek(0)
                print(log.read(), file=sys.stderr)
                raise
            finally:
                daemon.terminate()
                try:
                    daemon.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    daemon.kill()
                    daemon.wait()
                log.seek(0)
                detail = log.read()
            assert daemon.returncode == 0, detail
    print('PASS private paired facade: selected owner, saved labels, independent brightness and clean stop')


if __name__ == '__main__':
    main(Path(sys.argv[1]).resolve())
