"""Opt-in AI-server microphone faults on private PipeWire; no host audio/USB.

Run with the source path and evidence directory. The control subcommand is used
only by the ignored Rust integration test inside this script's private server.
"""
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile
import time


def control(action):
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    assert runtime.startswith('/tmp/ds-mic-')
    assert os.environ.get('PULSE_SERVER') == f'unix:{runtime}/pulse/native'
    assert os.environ.get('DECKSMITH_PRIVATE_MIC_TEST') == '1'
    module_path = Path(runtime)/'fixture-module.json'
    if module_path.exists():
        subprocess.run(['pactl', 'unload-module', module_path.read_text().strip()], check=True, timeout=3)
        module_path.unlink()
    if action != 'remove':
        assert action in ('same', 'different')
        serial = 'Other_unique_456' if action == 'different' else 'Fixture_unique_123'
        properties = f'device.bus=usb device.serial={serial} device.vendor.id=0fd9 device.product.id=0069'
        module = subprocess.check_output(['pactl', 'load-module', 'module-remap-source',
            'master=fixture-base.monitor', 'source_name=fixture-mic', "source_properties='"+properties+"'"], text=True, timeout=3).strip()
        module_path.write_text(module)


def run(source, evidence):
    evidence.mkdir(parents=True, exist_ok=True)
    temporary = tempfile.TemporaryDirectory(prefix='ds-mic-')
    runtime = Path(temporary.name)
    env = dict(os.environ, XDG_RUNTIME_DIR=str(runtime), PIPEWIRE_RUNTIME_DIR=str(runtime),
               PULSE_SERVER='unix:'+str(runtime/'pulse/native'), PYTHONDONTWRITEBYTECODE='1',
               DECKSMITH_PRIVATE_MIC_TEST='1', DECKSMITH_RESOURCE_ROOT=str(source))
    config = evidence/'private.conf'
    config.write_text('''context.properties = { core.daemon = true core.name = pipewire-0 support.dbus = false }
context.spa-libs = { audio.convert.* = audioconvert/libspa-audioconvert support.* = support/libspa-support }
context.modules = [
 { name = libpipewire-module-protocol-native }
 { name = libpipewire-module-spa-node-factory }
 { name = libpipewire-module-client-node }
 { name = libpipewire-module-adapter }
 { name = libpipewire-module-metadata }
 { name = libpipewire-module-link-factory }
 { name = libpipewire-module-access args = { access.socket = { pipewire-0 = "unrestricted" } } }
]
context.objects = [
 { factory = metadata args = { metadata.name = default } }
 { factory = spa-node-factory args = { factory.name = support.node.driver node.name = Dummy-Driver priority.driver = 200000 } }
 { factory = adapter args = { factory.name = support.null-audio-sink node.name = sentinel-mic media.class = Audio/Source/Virtual audio.position = [ MONO ] } }
]
''')
    pulse_config = evidence/'pulse.conf'
    pulse_config.write_text('''context.properties = { support.dbus = false }
context.spa-libs = { audio.convert.* = audioconvert/libspa-audioconvert support.* = support/libspa-support }
context.modules = [ { name = libpipewire-module-protocol-native } { name = libpipewire-module-client-node } { name = libpipewire-module-adapter } { name = libpipewire-module-metadata } { name = libpipewire-module-protocol-pulse } ]
pulse.properties = { server.address = [ "unix:native" ] }
''')
    processes, files, checks = [], [], []
    def start(args, **kwargs):
        child = subprocess.Popen(args, env=env, start_new_session=True, **kwargs)
        processes.append(child)
        return child
    def server(name, args):
        log = (evidence/(name+'.log')).open('w'); files.append(log)
        return start(args, stdout=log, stderr=subprocess.STDOUT)
    def pactl(*args):
        return subprocess.check_output(['pactl', *args], env=env, text=True, timeout=3, stderr=subprocess.PIPE)
    def node(name='fixture-mic'):
        return next(n for n in json.loads(pactl('--format=json','list','sources')) if n['name'] == name)
    def wait_node():
        deadline = time.monotonic()+6
        while True:
            try: return node()
            except (StopIteration, subprocess.SubprocessError):
                if time.monotonic() >= deadline: raise
                time.sleep(.05)
    def guard(mode, name='pin.json'):
        return start(['python3', '-B', str(source/'apps/decksmith-studio/shared_microphone_native.py'),
                      'input:fixture-mic', mode, str(evidence/name)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    def line(child, expected):
        assert select.select([child.stdout], [], [], 4)[0], ('timeout', expected)
        actual = child.stdout.readline().decode().strip()
        assert actual == expected, (expected, actual, child.poll())
    def change(action):
        subprocess.run(['python3', '-B', str(Path(__file__).resolve()), 'control', action], env=env, check=True, timeout=10)
        if action != 'remove': return wait_node()
    def stop(child):
        if child.poll() is None:
            os.killpg(child.pid, signal.SIGTERM)
            try: child.wait(timeout=3)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL); child.wait(timeout=3)
    def unchanged_sentinel():
        assert node('sentinel-mic')['mute'] == sentinel['mute']
        assert node('sentinel-mic')['volume'] == sentinel['volume']
    try:
        server('pipewire', ['pipewire','-c',str(config)])
        pulse = server('pulse', ['pipewire-pulse','-c',str(pulse_config)])
        deadline = time.monotonic()+6
        while True:
            try: pactl('info'); break
            except subprocess.SubprocessError:
                if time.monotonic() >= deadline: raise
                time.sleep(.05)
        pactl('load-module', 'module-null-sink', 'sink_name=fixture-base')
        change('same')
        before = wait_node(); (evidence/'initial-source.json').write_text(json.dumps(before, indent=2)+'\n')
        assert before['properties']['device.serial'] == 'Fixture_unique_123'
        pactl('set-default-source', 'sentinel-mic')
        sentinel = node('sentinel-mic')
        pactl('set-source-mute','fixture-mic','1')
        child = guard('hold'); child.stdin.write(b'hold\n'); child.stdin.flush(); line(child,'live')
        change('remove'); child.stdin.close(); assert child.wait(timeout=4) != 0
        replacement = change('same'); assert replacement['properties']['object.serial'] != before['properties']['object.serial']
        pactl('set-source-mute','fixture-mic','0')
        for mode in ('hold','toggle'):
            child = guard(mode); child.stdin.write(b'hold\n'); child.stdin.flush(); assert child.wait(timeout=4) != 0
            assert not node()['mute']
        child = guard('mute'); child.stdin.close(); line(child,'muted'); assert child.wait(timeout=4) == 0
        assert node()['mute']; unchanged_sentinel()
        checks.append('removed live source: old hold/toggle refused; same USB identity recovered by mute only')
        for _ in range(10):
            change('same'); pactl('set-source-mute','fixture-mic','0')
            child = guard('mute'); child.stdin.close(); line(child,'muted'); assert child.wait(timeout=4) == 0
            assert node()['mute']; unchanged_sentinel()
        checks.append('10 native source re-enumerations recovered muted; unrelated microphone mute and volumes unchanged')
        retained = (evidence/'pin.json').read_bytes()
        change('different'); pactl('set-source-mute','fixture-mic','0')
        child = guard('mute'); child.stdin.close(); assert child.wait(timeout=4) != 0
        assert not node()['mute']; assert (evidence/'pin.json').read_bytes() == retained; unchanged_sentinel()
        checks.append('same name with different USB serial refused without mute/write')
        change('same')
        test_state = Path(os.environ.get('DECKSMITH_TEST_STATE', Path.home()/'.local/share/decksmith-test-host'))
        rust_env = dict(env, CARGO_TARGET_DIR=str(test_state/'build'),
                        CARGO_HOME=str(test_state/'cargo'),
                        RUSTUP_HOME=str(test_state/'rustup'), RUSTUP_TOOLCHAIN='1.97.1')
        rust_env['PATH'] = rust_env['CARGO_HOME']+'/bin:'+rust_env['PATH']
        for test in ('private_native_microphone_pair_acceptance', 'private_native_microphone_reconnection_acceptance'):
            with (evidence/(test+'.log')).open('w') as log:
                result = subprocess.run(['cargo','test','-p','decksmithd','--all-features','--locked','--offline',test,'--','--ignored','--nocapture'], cwd=source, env=rust_env, stdout=log, stderr=subprocess.STDOUT, timeout=120)
            assert result.returncode == 0, test
            checks.append(test); unchanged_sentinel()
        # Keep the old socket inode alive so replacement cannot reuse it.
        (runtime/'pulse/native').rename(runtime/'pulse/retired-native')
        stop(pulse); server('pulse-restarted',['pipewire-pulse','-c',str(pulse_config)])
        deadline = time.monotonic()+6
        while True:
            try: pactl('info'); break
            except subprocess.SubprocessError:
                if time.monotonic() >= deadline: raise
                time.sleep(.05)
        (runtime/'fixture-module.json').unlink(missing_ok=True)
        pactl('load-module', 'module-null-sink', 'sink_name=fixture-base')
        change('same')
        wait_node(); pactl('set-source-mute','fixture-mic','0')
        child = guard('mute'); child.stdin.close(); assert child.wait(timeout=4) != 0
        assert not node()['mute']; assert (evidence/'pin.json').read_bytes() == retained
        checks.append('different audio-server socket fails closed, retained physical pin unchanged')
        result = {'status':'passed','checks':checks}
    except BaseException as error:
        result = {'status':'failed','checks':checks,'error':repr(error), 'traceback':__import__('traceback').format_exc()}
    finally:
        for child in reversed(processes): stop(child)
        for log in files: log.close()
        result.update(real_hardware_accessed=False, owned_processes_reaped=all(p.poll() is not None for p in processes))
        temporary.cleanup()
        (evidence/'result.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps(result), flush=True)
    return 0 if result['status']=='passed' else 1


if __name__ == '__main__':
    if len(sys.argv) == 3 and sys.argv[1] == 'control': control(sys.argv[2])
    else: raise SystemExit(run(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()))
