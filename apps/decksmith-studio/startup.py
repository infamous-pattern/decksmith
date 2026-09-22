"""Installed user-service login preference; never starts or stops a running service."""
import subprocess

UNIT='decksmith.service'

def command(*args):
    return subprocess.run(['systemctl','--user',*args],capture_output=True,text=True,timeout=5)

def read():
    result=command('show',UNIT,'--property=LoadState,UnitFileState')
    if result.returncode:raise RuntimeError('Could not read login settings')
    props=dict(line.split('=',1) for line in result.stdout.splitlines() if '=' in line)
    state=props.get('UnitFileState','')
    if props.get('LoadState')!='loaded' or state not in ('enabled','disabled'):
        return {'available':False,'enabled':False,'detail':'Install the Decksmith runtime to manage login startup.'}
    return {'available':True,'enabled':state=='enabled','detail':'Background controls only. The window stays closed.'}

def set_enabled(enabled):
    if not isinstance(enabled,bool):raise ValueError('Expected a boolean')
    if not read()['available']:raise RuntimeError('Login startup is unavailable for this service')
    result=command('enable' if enabled else 'disable',UNIT)
    if result.returncode:raise RuntimeError('Could not save login settings')
    actual=read()
    if not actual['available'] or actual['enabled']!=enabled:raise RuntimeError('Login setting was not confirmed')
    return actual
