#!/usr/bin/env python3
"""Package the approved local OpenHomeB trial as a Decksmith companion service."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[1]
MODULES = ('host.py','paced_host.py','adjustment_host.py','live_on_off_trial.py',
           'connection_preferences.py','credential_store.py','connection_setup.py','accessory_catalog.py','accessory_actions.py','live_panel.py','hardware_bridge.py','live_panel.html','managed_runner.py')


def build(lab, destination):
    lab = Path(lab)
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=destination) as tmp:
        root = Path(tmp)
        host = root / 'host-prototype';host.mkdir()
        for name in MODULES:
            shutil.copy2(REPO/'plugins/openhomeb/runtime'/name, host/name)
        source = lab.parent/'openhomeb-adjust-completion'
        receipt = json.loads((source/'verification/build-receipt.json').read_text())
        binary = source/'binaries/openhomeb'
        if hashlib.sha256(binary.read_bytes()).hexdigest() != receipt['binary_sha256']:
            raise ValueError('Plugin checksum does not match reviewed receipt')
        if (source/'source/LICENSE').exists():
            shutil.copy2(source/'source/LICENSE',root/'OPENHOMEB-LICENSE')
        for relative in ('binaries/openhomeb','verification/build-receipt.json'):
            target=root/'openhomeb-adjust-completion'/relative;target.parent.mkdir(parents=True,exist_ok=True)
            shutil.copy2(source/relative,target)
        target=root/'openhomeb-2.0.2/source/assets';target.mkdir(parents=True)
        shutil.copy2(lab.parent/'openhomeb-2.0.2/source/assets/manifest.json',target/'manifest.json')
        sites=list((lab/'.venv/lib').glob('python*/site-packages'))
        if len(sites)!=1:raise ValueError('Expected one reviewed Python environment')
        vendor=root/'vendor';vendor.mkdir()
        shutil.copytree(sites[0]/'websockets',vendor/'websockets',ignore=shutil.ignore_patterns('__pycache__','*.pyc','*.so'))
        for info in sites[0].glob('websockets-*.dist-info'):
            shutil.copytree(info,vendor/info.name)
        files={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest()
               for p in sorted(root.rglob('*')) if p.is_file()}
        identity=hashlib.sha256(json.dumps(files,sort_keys=True).encode()).hexdigest()[:16]
        (root/'runtime.json').write_text(json.dumps({'id':identity,'files':files},indent=2)+'\n')
        release=destination/identity
        if not release.exists():shutil.copytree(root,release)
        return release


def quote(path):
    return '"'+str(path).replace('\\','\\\\').replace('"','\\"').replace('%','%%')+'"'


def install(release, home, activate):
    home=Path(home);base=home/'.local/share/decksmith/plugin-runtime';releases=base/'releases'
    releases.mkdir(parents=True,exist_ok=True)
    manifest=json.loads((release/'runtime.json').read_text())
    for name,digest in manifest['files'].items():
        if hashlib.sha256((release/name).read_bytes()).hexdigest()!=digest:raise ValueError('Runtime checksum mismatch')
    target=releases/manifest['id']
    if not target.exists():shutil.copytree(release,target)
    current=base/'current'
    previous=os.readlink(current) if current.is_symlink() else None
    unit_dir=home/'.config/systemd/user';unit_dir.mkdir(parents=True,exist_ok=True)
    unit=unit_dir/'decksmith-openhomeb.service';dropin=unit_dir/'decksmith.service.d/openhomeb.conf'
    backup=base/'backups'/__import__('datetime').datetime.now().strftime('%Y%m%d-%H%M%S-%f');backup.mkdir(parents=True)
    for name,path in [('unit',unit),('dropin',dropin)]:
        if path.exists():shutil.copy2(path,backup/name)
    (backup/'previous.json').write_text(json.dumps({'release':previous})+'\n')
    staged=base/'current.new';staged.unlink(missing_ok=True);staged.symlink_to(target);staged.replace(current)
    unit.write_text('[Unit]\nDescription=Decksmith OpenHomeB companion\nPartOf=decksmith.service\nBindsTo=decksmith.service\nAfter=decksmith.service\nStartLimitIntervalSec=0\n\n[Service]\nType=exec\nExecStart=/usr/bin/python3 -B '+quote(current/'host-prototype/managed_runner.py')+'\nEnvironment=PYTHONNOUSERSITE=1\nUMask=0077\nRestart=on-failure\nRestartSec=10\nTimeoutStopSec=8\nKillMode=control-group\nNoNewPrivileges=true\n')
    dropin.parent.mkdir(parents=True,exist_ok=True)
    dropin.write_text('[Unit]\nWants=decksmith-openhomeb.service\n')
    if activate:
        try:
            subprocess.run(['systemctl','--user','daemon-reload'],check=True)
            subprocess.run(['systemctl','--user','restart','decksmith-openhomeb.service'],check=True)
            import time
            runtime=Path(os.environ['XDG_RUNTIME_DIR'])
            for _ in range(60):
                state=runtime/'decksmith-plugin-state.json'
                try:
                    fresh=time.time()-state.stat().st_mtime<3
                    ready=json.loads(state.read_text()).get('manager_ready') is True
                except (OSError,ValueError):fresh=ready=False
                if fresh and ready:break
                time.sleep(.25)
            else:raise RuntimeError('Companion did not become ready; restoring previous integration')
        except BaseException:
            subprocess.run(['systemctl','--user','stop','decksmith-openhomeb.service'],check=False)
            for name,path in [('unit',unit),('dropin',dropin)]:
                if (backup/name).exists():shutil.copy2(backup/name,path)
                else:path.unlink(missing_ok=True)
            current.unlink(missing_ok=True)
            if previous:current.symlink_to(previous)
            subprocess.run(['systemctl','--user','daemon-reload'],check=False)
            if (backup/'unit').exists():subprocess.run(['systemctl','--user','start','decksmith-openhomeb.service'],check=False)
            raise
    return {'release':manifest['id'],'backup':str(backup),'activation_requested':activate}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--lab',type=Path,required=True)
    p.add_argument('--output',type=Path,default=REPO/'dist/plugin-runtime')
    p.add_argument('--install-home',type=Path);p.add_argument('--activate',action='store_true')
    a=p.parse_args();release=build(a.lab,a.output)
    print(json.dumps(install(release,a.install_home,a.activate) if a.install_home else {'package':str(release)}))
