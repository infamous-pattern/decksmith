"""Bounded local archives and release integrity, shared by build/install/backup."""
from pathlib import Path,PurePosixPath
from hashlib import sha256
import io,json,tarfile,os,tempfile
LIMIT=512*1024*1024

def digest(data):return sha256(data).hexdigest()
def safe_name(name):
    p=PurePosixPath(name)
    if not name or name == '.' or p.is_absolute() or '..' in p.parts or str(p)!=name or '\\' in name:raise ValueError('Unsafe package path')
    return name

def atomic(path,data,mode=0o644):
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True)
    fd,name=tempfile.mkstemp(prefix='.'+path.name+'-',dir=path.parent)
    try:
        with os.fdopen(fd,'wb') as f:f.write(data);f.flush();os.fsync(f.fileno())
        os.chmod(name,mode);os.replace(name,path)
    finally:
        if os.path.exists(name):os.unlink(name)

def read_archive(path):
    files={};total=0
    with tarfile.open(path,'r:gz') as archive:
        for member in archive:
            name=safe_name(member.name)
            if member.isdir():continue
            if not member.isfile() or name in files:raise ValueError('Package contains links, special files or duplicates')
            total+=member.size
            if total>LIMIT or len(files)>=10000:raise ValueError('Package is too large')
            files[name]=archive.extractfile(member).read()
    return files

def write_archive(path,files):
    out=io.BytesIO()
    if sum(map(len,files.values()))>LIMIT:raise ValueError('Backup or package exceeds 512 MB')
    with tarfile.open(fileobj=out,mode='w:gz') as archive:
        for name,data in sorted(files.items()):
            safe_name(name);info=tarfile.TarInfo(name);info.size=len(data);info.mode=0o600;archive.addfile(info,io.BytesIO(data))
    atomic(path,out.getvalue(),0o600)

def tree_files(root):
    root=Path(root);result={}
    if not root.exists():return result
    for path in sorted(root.rglob('*')):
        if path.is_symlink():raise ValueError(f'Links are not supported in managed data: {path.name}')
        if path.is_file():result[path.relative_to(root).as_posix()]=path.read_bytes()
    return result

def release_files(source):
    source=Path(source)
    files=tree_files(source) if source.is_dir() else read_archive(source)
    try:manifest=json.loads(files['release.json'])
    except (KeyError,ValueError):raise ValueError('Missing release manifest')
    if manifest.get('format')!='decksmith-release' or manifest.get('version')!=1:raise ValueError('Unsupported release format')
    expected=manifest.get('files')
    if not isinstance(expected,dict) or not expected:raise ValueError('Invalid release manifest')
    for name,value in expected.items():
        safe_name(name)
        if name not in files or digest(files[name])!=value:raise ValueError('Release integrity check failed: '+name)
    extra=set(files)-set(expected)-{'release.json'}
    if extra:raise ValueError('Release contains unlisted files')
    for name in ('bin/decksmithd','bin/decksmithctl','apps/decksmith-studio/panel.py','scripts/decksmith-install.py','scripts/package_io.py','config/audio.json'):
        if name not in expected:raise ValueError('Incomplete release: '+name)
    identity=manifest.get('id','')
    if not identity or identity in ('.','..') or len(identity)>100 or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._+-' for c in identity):raise ValueError('Invalid release identity')
    return manifest,{name:files[name] for name in expected}|{'release.json':files['release.json']}
