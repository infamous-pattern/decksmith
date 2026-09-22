#!/usr/bin/env python3
"""Decksmith per-user install, update, backup and rollback. Startup is never enabled automatically."""
import argparse,base64,contextlib,fcntl,json,os,platform,shlex,shutil,subprocess,sys,tempfile,time
from dataclasses import dataclass
from pathlib import Path
sys.dont_write_bytecode=True
from package_io import atomic,digest,read_archive,release_files,safe_name,tree_files,write_archive
APP='cc.senecal.Decksmith.Studio'
ROOT=Path(__file__).resolve().parents[1]
@dataclass
class Paths:
    data:Path;config:Path;state:Path;bin:Path;staged:bool=False
    @classmethod
    def create(cls,stage=None):
        if stage:
            root=Path(stage).absolute();return cls(root/'data',root/'config',root/'state',root/'bin',True)
        home=Path.home()
        def xdg(key,fallback):
            path=Path(os.environ.get(key,str(fallback)))
            if not path.is_absolute():raise ValueError(key+' must be an absolute path')
            return path
        return cls(xdg('XDG_DATA_HOME',home/'.local/share'),xdg('XDG_CONFIG_HOME',home/'.config'),xdg('XDG_STATE_HOME',home/'.local/state'),home/'.local/bin')
    @property
    def app(self):return self.data/'decksmith/app'
    @property
    def backups(self):return self.state/'decksmith/backups'
    @property
    def record(self):return self.app/'install.json'
    def integrations(self):return {'desktop':self.data/'applications'/f'{APP}.desktop','unit':self.config/'systemd/user/decksmith.service','launcher':self.bin/'decksmith','manager':self.bin/'decksmith-manage','cli':self.bin/'decksmithctl'}

def run(*args,check=True):return subprocess.run(args,capture_output=True,text=True,timeout=15,check=check)
def service(paths,action,check=True):
    if paths.staged:return None
    return run('systemctl','--user',action,'decksmith.service',check=check)
def active(paths):return False if paths.staged else service(paths,'is-active',False).stdout.strip()=='active'
def reload(paths):
    if not paths.staged:run('systemctl','--user','daemon-reload')
@contextlib.contextmanager
def locked(paths):
    directory=paths.state/'decksmith';directory.mkdir(parents=True,exist_ok=True);lock=directory/'install.lock'
    with lock.open('a') as f:
        os.chmod(lock,0o600);fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB);yield

def load_record(paths):return json.loads(paths.record.read_text()) if paths.record.exists() else {}
def current(paths):
    link=paths.app/'current'
    if not link.is_symlink():
        if link.exists():raise ValueError('Current install path is not a managed symlink')
        return None
    target=os.readlink(link)
    if not target.startswith('releases/') or target.count('/')!=1:raise ValueError('Unrecognized current release link')
    return target.split('/')[1]
def point(paths,identity):
    link=paths.app/'current';temporary=paths.app/f'.current-{os.getpid()}'
    if temporary.exists() or temporary.is_symlink():raise ValueError('A previous install needs inspection')
    temporary.symlink_to('releases/'+identity);os.replace(temporary,link)
def save_record(paths,record):atomic(paths.record,json.dumps(record,indent=2).encode(),0o600)
def integration_snapshot(paths):
    result={}
    for name,path in paths.integrations().items():
        if path.is_symlink():raise ValueError('Integration is a symlink; preserve it before installing: '+str(path))
        result[name]=None if not path.exists() else {'bytes':base64.b64encode(path.read_bytes()).decode(),'mode':path.stat().st_mode&0o777}
    return result

def restore_integrations(paths,snapshot):
    for name,path in paths.integrations().items():
        item=snapshot.get(name)
        if item is None:
            if path.exists():path.unlink()
        else:atomic(path,base64.b64decode(item['bytes']),item['mode'])
def assert_owned(paths,record):
    for name,path in paths.integrations().items():
        if not path.exists():continue
        if path.is_symlink():raise ValueError('Refusing to overwrite a custom integration link: '+str(path))
        if name in record.get('managed',{}):
            if digest(path.read_bytes())!=record['managed'][name]:raise ValueError('Integration was customized; keep a copy before updating: '+str(path))
        elif name!='desktop' or 'Name=Decksmith' not in path.read_text():raise ValueError('Existing file is not managed by this installer: '+str(path))

def backup(paths):
    # Read twice to avoid taking a snapshot while the editor is saving.
    for _ in range(5):
        first={**{'config/'+n:d for n,d in tree_files(paths.config/'decksmith').items()},**{'icons/'+n:d for n,d in tree_files(paths.data/'decksmith/icons').items()}}
        second={**{'config/'+n:d for n,d in tree_files(paths.config/'decksmith').items()},**{'icons/'+n:d for n,d in tree_files(paths.data/'decksmith/icons').items()}}
        if first==second:break
        time.sleep(.1)
    else:raise ValueError('Configuration is changing; finish saving and retry')
    manifest={'format':'decksmith-backup','version':1,'files':{n:digest(d) for n,d in first.items()}}
    first['backup.json']=json.dumps(manifest).encode();paths.backups.mkdir(parents=True,exist_ok=True)
    destination=paths.backups/(time.strftime('%Y%m%d-%H%M%S')+'-'+str(time.time_ns())[-6:]+'.tar.gz')
    write_archive(destination,first);return destination

def systemd_quote(path):return '"'+str(path).replace('\\','\\\\').replace('"','\\"').replace('%','%%')+'"'
def integrations(paths,release):
    root=paths.app/'current'
    unit=(release/'packaging/decksmith.service.in').read_text()
    for name,path in [('DAEMON',root/'bin/decksmithd'),('CONFIG',root/'config/audio.json'),('ROOT',root)]:unit=unit.replace('@'+name+'@',systemd_quote(path))
    panel=str(root/'apps/decksmith-studio/panel.py').replace('\\','\\\\').replace('"','\\"').replace('%','%%')
    desktop=(release/'packaging/cc.senecal.Decksmith.Studio.desktop.in').read_text().replace('@PANEL@',panel).replace('@ICON@',str(root/'brand/decksmith-app.svg'))
    def wrapper(command):return ('#!/bin/sh\nexport PYTHONDONTWRITEBYTECODE=1\nexec '+command+' "$@"\n').encode()
    return {'unit':unit.encode(),'desktop':desktop.encode(),'launcher':wrapper('/usr/bin/python3 '+shlex.quote(str(root/'apps/decksmith-studio/panel.py'))),'manager':wrapper('/usr/bin/python3 '+shlex.quote(str(root/'scripts/decksmith-install.py'))),'cli':wrapper(shlex.quote(str(root/'bin/decksmithctl')))}

def validate_install(paths,release):
    env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
    done=subprocess.run(['/usr/bin/python3',str(release/'scripts/runtime-doctor.py'),'--root',str(release)],capture_output=True,text=True,timeout=30,env=env)
    report=json.loads(done.stdout) if done.stdout.strip() else {'ready':False,'error':done.stderr}
    if not report['ready']:raise ValueError('Runtime dependency check failed: '+json.dumps(report))
    desired=integrations(paths,release)
    with tempfile.TemporaryDirectory() as tmp:
        path=Path(tmp)/f'{APP}.desktop';path.write_bytes(desired['desktop'])
        if shutil.which('desktop-file-validate'):run('desktop-file-validate',str(path))
        if shutil.which('systemd-analyze'):
            unit=Path(tmp)/'decksmith.service'
            unit.write_text(desired['unit'].decode().replace(systemd_quote(paths.app/'current'/'bin/decksmithd'),systemd_quote(release/'bin/decksmithd')))
            run('systemd-analyze','--user','verify',str(unit))
    return desired

def validate_saved(paths,release):
    layout=paths.config/'decksmith/layout.json'
    if layout.exists():run(str(release/'bin/decksmithd'),'--validate-layout',str(layout))

def wait_ready(paths,release):
    if paths.staged:return
    from gi.repository import Gio
    bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
    deadline=time.monotonic()+8
    while time.monotonic()<deadline:
        try:
            answer=bus.call_sync('cc.senecal.Decksmith','/cc/senecal/Decksmith','cc.senecal.Decksmith.Control1','GetStatus',None,None,Gio.DBusCallFlags.NO_AUTO_START,250,None)
            if json.loads(answer.unpack()[0]).get('api_version')==1:return
        except Exception:pass
        time.sleep(.15)
    raise ValueError('Installed service did not become ready; restoring the previous integration')

def install(paths,source,activate=False):
    manifest,files=release_files(source)
    if manifest['architecture']!=platform.machine():raise ValueError('This bundle is for '+manifest['architecture'])
    record=load_record(paths);assert_owned(paths,record);old=current(paths);before=integration_snapshot(paths)
    paths.app.mkdir(parents=True,exist_ok=True);releases=paths.app/'releases';releases.mkdir(exist_ok=True);identity=manifest['id'];release=releases/identity
    if release.exists():
        existing,_=release_files(release)
        if existing!=manifest:raise ValueError('Release identity collision')
    else:
        temporary=Path(tempfile.mkdtemp(prefix='.install-',dir=releases))
        try:
            for name,data in files.items():atomic(temporary/name,data,0o755 if name.startswith('bin/') or name.endswith('.sh') else 0o644)
            os.replace(temporary,release)
        finally:
            if temporary.exists():shutil.rmtree(temporary)
    desired=validate_install(paths,release);validate_saved(paths,release);snapshot=backup(paths);was_active=active(paths)
    legacy_unit=None
    if was_active and before['unit'] is None and not paths.staged:
        fragment=run('systemctl','--user','show','decksmith.service','--property=FragmentPath','--value').stdout.strip()
        if fragment:legacy_unit=Path(fragment).read_bytes()
    if was_active and activate:service(paths,'stop')
    try:
        for name,path in paths.integrations().items():atomic(path,desired[name],0o755 if name in ('launcher','manager','cli') else 0o644)
        point(paths,identity);reload(paths)
        if activate:service(paths,'reset-failed',False);service(paths,'start');wait_ready(paths,release)
        updated={'format':'decksmith-install','version':1,'current':identity,'previous':old if old!=identity else record.get('previous'),'original':record.get('original',before),'managed':{n:digest(d) for n,d in desired.items()},'last_backup':str(snapshot)}
        save_record(paths,updated)
    except Exception:
        if activate:service(paths,'stop',False)
        restore_integrations(paths,before)
        if old:point(paths,old)
        elif (paths.app/'current').is_symlink():(paths.app/'current').unlink()
        if legacy_unit is not None:atomic(paths.integrations()['unit'],legacy_unit)
        reload(paths)
        if was_active and activate:service(paths,'start',False)
        raise
    return {'installed':identity,'previous':updated['previous'],'backup':str(snapshot),'activated':bool(activate and not paths.staged),'startup':'unchanged (never enabled by installer)','staged':paths.staged}

def require_stopped(paths):
    if active(paths):raise ValueError('Stop Background controls before restoring, rolling back or removing the app')
    if not paths.staged:
        check=run('gdbus','call','--session','--dest','org.freedesktop.DBus','--object-path','/org/freedesktop/DBus','--method','org.freedesktop.DBus.NameHasOwner',APP,check=False)
        if 'true' in check.stdout:raise ValueError('Close Decksmith after saving wanted edits before continuing')

def rollback(paths,original=False):
    require_stopped(paths);record=load_record(paths);assert_owned(paths,record)
    if original:
        if not record.get('original'):raise ValueError('No original integration snapshot')
        snapshot=backup(paths);restore_integrations(paths,record['original']);record.update(managed={},integrations_removed=True,last_backup=str(snapshot));save_record(paths,record);reload(paths)
        return {'restored':'pre-install launchers and service integration','preserved':'all configuration and release files','backup':str(snapshot),'service':'stopped; use the restored launcher when ready'}
    previous=record.get('previous')
    if not previous:raise ValueError('No previous installed release; use rollback --original for the pre-install integration')
    release=paths.app/'releases'/previous;release_files(release);desired=validate_install(paths,release);validate_saved(paths,release)
    snapshot=backup(paths);old=current(paths);before=integration_snapshot(paths)
    try:
        for name,path in paths.integrations().items():atomic(path,desired[name],0o755 if name in ('launcher','manager','cli') else 0o644)
        point(paths,previous);record.update(current=previous,previous=old,last_backup=str(snapshot),managed={n:digest(d) for n,d in desired.items()});save_record(paths,record);reload(paths)
    except Exception:
        restore_integrations(paths,before);point(paths,old);reload(paths);raise
    return {'current':previous,'previous':old,'backup':str(snapshot),'service':'stopped; start when ready'}

def restore(paths,archive):
    require_stopped(paths);files=read_archive(archive);manifest=json.loads(files.pop('backup.json'))
    if manifest.get('format')!='decksmith-backup' or manifest.get('version')!=1 or set(manifest.get('files',{}))!=set(files):raise ValueError('Invalid backup manifest')
    for name,data in files.items():
        safe_name(name)
        if not name.startswith(('config/','icons/')) or digest(data)!=manifest['files'][name]:raise ValueError('Backup integrity check failed')
    root=paths.app/'current'
    with tempfile.TemporaryDirectory() as tmp:
        for name in ('layout.json','layout.previous.json'):
            if 'config/'+name in files:
                file=Path(tmp)/name;file.write_bytes(files['config/'+name]);run(str(root/'bin/decksmithd'),'--validate-layout',str(file))
        if 'config/control-panel.json' in files:
            settings=json.loads(files['config/control-panel.json'])
            if set(settings)-{'layout','brightness'} or settings.get('layout') not in ('audio','navigation','custom') or (settings.get('brightness') is not None and (type(settings['brightness']) is not int or not 0<=settings['brightness']<=100)):raise ValueError('Invalid saved settings')
    snapshot=backup(paths);before={}
    try:
        for name,data in files.items():
            category,relative=name.split('/',1);target=(paths.config/'decksmith' if category=='config' else paths.data/'decksmith/icons')/relative
            if target.is_symlink():raise ValueError('Restore target is a link')
            before[target]=target.read_bytes() if target.exists() else None;atomic(target,data,0o600)
    except Exception:
        for target,data in before.items():
            if data is None:target.unlink(missing_ok=True)
            else:atomic(target,data,0o600)
        raise
    return {'restored':len(files),'backup_before_restore':str(snapshot),'extra_files':'preserved','service':'stopped; start when ready'}

def uninstall(paths):
    require_stopped(paths);record=load_record(paths);assert_owned(paths,record)
    if not record:raise ValueError('No managed installation')
    snapshot=backup(paths)
    if not paths.staged:service(paths,'disable',False)
    for name,path in paths.integrations().items():
        if path.exists() and digest(path.read_bytes())==record.get('managed',{}).get(name):path.unlink()
    reload(paths);record['integrations_removed']=True;save_record(paths,record)
    return {'removed':'managed launchers and service integration','preserved':'configuration, imported icons, release files and backups','backup':str(snapshot)}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--stage-root',type=Path,help='Isolated filesystem test; never call the host service manager')
    sub=p.add_subparsers(dest='command',required=True)
    install_args=sub.add_parser('install');install_args.add_argument('bundle',type=Path);install_args.add_argument('--activate',action='store_true')
    sub.add_parser('backup');roll=sub.add_parser('rollback');roll.add_argument('--original',action='store_true');sub.add_parser('uninstall');sub.add_parser('status')
    restore_args=sub.add_parser('restore');restore_args.add_argument('archive',type=Path)
    args=p.parse_args();paths=Paths.create(args.stage_root)
    with locked(paths):
        if args.command=='install':result=install(paths,args.bundle,args.activate)
        elif args.command=='backup':result={'backup':str(backup(paths))}
        elif args.command=='rollback':result=rollback(paths,args.original)
        elif args.command=='restore':result=restore(paths,args.archive)
        elif args.command=='uninstall':result=uninstall(paths)
        else:result=load_record(paths)
    print(json.dumps(result,indent=2))
if __name__=='__main__':
    try:main()
    except (OSError,ValueError,KeyError,subprocess.SubprocessError) as error:print(str(error),file=sys.stderr);sys.exit(1)
