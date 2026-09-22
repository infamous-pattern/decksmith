"""Private durable preference; contains no credentials or device assignments."""
import json
import os
from pathlib import Path
import stat
import tempfile

def read(path):
    if path is None:return {'enabled':True}
    try:
        fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW)
    except FileNotFoundError:return {'enabled':True}
    with os.fdopen(fd) as stream:
        info=os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_uid!=os.getuid() or info.st_mode&0o077:raise ValueError('Unsafe preferences')
        data=json.loads(stream.read(4097))
    if not isinstance(data,dict) or set(data)-{'enabled','server','username','credential_profile','removed'} or type(data.get('enabled')) is not bool:raise ValueError('Invalid preferences')
    if 'removed' in data and (type(data['removed']) is not bool or (data['removed'] and (data['enabled'] or data.get('server','') or data.get('username','')))):raise ValueError('Invalid removed connection')
    return data

def load(path):return read(path)['enabled']

def write(path,data):
    if path is None:return
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True,mode=0o700)
    if path.is_symlink():raise ValueError('Unsafe preferences')
    fd,name=tempfile.mkstemp(prefix='.openhomeb-',dir=path.parent)
    try:
        with os.fdopen(fd,'w') as stream:
            json.dump(data,stream);stream.flush();os.fsync(stream.fileno())
        os.replace(name,path)
        directory=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY)
        try:os.fsync(directory)
        finally:os.close(directory)
    finally:
        if os.path.exists(name):os.unlink(name)


def save(path,enabled):
    if path is not None and Path(path).is_symlink():raise ValueError('Unsafe preferences')
    data=read(path);data['enabled']=enabled;write(path,data)
