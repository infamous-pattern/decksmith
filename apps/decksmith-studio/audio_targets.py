"""PipeWire's PulseAudio interface: stable application and device audio targets."""
import json,subprocess,sys

PROPERTIES=('application.id','application.process.binary','application.name')

def command(*args):
    result=subprocess.run(['/usr/bin/pactl',*args],capture_output=True,text=True,timeout=.4,check=True)
    if len(result.stdout)>2*1024*1024:raise ValueError('Audio inventory too large')
    return result.stdout

def listing(kind):return json.loads(command('--format=json','list',kind))

def app_target(node):
    props=node.get('properties',{})
    for key in PROPERTIES:
        if props.get(key):return 'app:'+key+'='+props[key]
    return None

def validate(target):
    if not isinstance(target,str) or not target or len(target)>512 or any(ord(c)<32 for c in target):raise ValueError('Invalid audio target')
    if target in ('system','default_output','microphone'):return
    if target.startswith(('output:','input:')) and target.split(':',1)[1] and not target.split(':',1)[1].startswith('-'):return
    if target.startswith('app:') and '=' in target:
        key,value=target[4:].split('=',1)
        if key in PROPERTIES and value:return
    raise ValueError('Invalid audio target')

def inventory():
    targets=[{'id':'system','label':'System sounds'},{'id':'microphone','label':'Default microphone'}]
    for kind,prefix in [('sinks','output'),('sources','input')]:
        try:nodes=listing(kind)
        except (ValueError,subprocess.SubprocessError):nodes=[]
        for node in nodes:
            if prefix=='input' and (node['name'].endswith('.monitor') or node.get('properties',{}).get('device.class')=='monitor'):continue
            targets.append({'id':prefix+':'+node['name'],'label':('Output · ' if prefix=='output' else 'Input · ')+(node.get('description') if node.get('description') not in (None,'','(null)') else node['name'])})
    from audio_apps import refresh
    installed=refresh()
    try:streams=listing('sink-inputs')
    except (ValueError,subprocess.SubprocessError):streams=[]
    covered=set()
    for app in installed:
        binary=app['id'].split('=',1)[1]
        matching=[node for node in streams if node.get('properties',{}).get('application.process.binary')==binary]
        covered.update(app_target(node) for node in matching)
        targets.append({'id':app['id'],'name':app['name'],'label':'App · '+app['name']+(' · Audio detected' if matching else ' · Installed')})
    seen=set()
    for node in streams:
        identity=app_target(node)
        if identity and identity not in covered and identity not in seen:
            seen.add(identity)
            props=node.get('properties',{})
            name=props.get('application.name') or props.get('application.process.binary') or identity
            targets.append({'id':identity,'name':name,'label':'App · '+name+' · Audio detected'})
    return targets

class Snapshot:
    def __init__(self):self.cache={}
    def info(self):
        if 'info' not in self.cache:self.cache['info']=json.loads(command('--format=json','info'))
        return self.cache['info']
    def nodes(self,target):
        validate(target)
        kind='sink-inputs' if target=='system' or target.startswith('app:') else 'sources' if target=='microphone' or target.startswith('input:') else 'sinks'
        if kind not in self.cache:self.cache[kind]=listing(kind)
        nodes=self.cache[kind]
        if target=='system':
            return kind,[node for node in nodes if node.get('properties',{}).get('media.role')=='event']
        if target in ('default_output','microphone'):
            name=self.info()['default_sink_name' if target=='default_output' else 'default_source_name']
            return kind,[node for node in nodes if node['name']==name]
        if target.startswith('app:'):
            key,value=target[4:].split('=',1)
            return kind,[node for node in nodes if node.get('properties',{}).get(key)==value]
        return kind,[node for node in nodes if node['name']==target.split(':',1)[1]]

def state(nodes):
    if not nodes:return None
    levels=[channel['value']/65536*100 for node in nodes for channel in node.get('volume',{}).values()]
    if not levels:return None
    return {'percent':min(1000,round(max(levels))),'muted':all(node.get('mute',False) for node in nodes)}

def device_icon(target,nodes):
    """Use explicit port/device metadata; never guess from user-editable labels."""
    if target.startswith('app:'):return 'application'
    if not nodes:return 'unknown'
    props=nodes[0].get('properties',{})
    if props.get('device.class')=='monitor' or nodes[0].get('name','').endswith('.monitor'):return 'unknown'
    if target=='microphone' or target.startswith('input:'):return 'microphone'
    active=nodes[0].get('active_port')
    port=active if isinstance(active,dict) else next((p for p in nodes[0].get('ports',[]) if p.get('name')==active),{})
    values=[port.get('type',''),props.get('device.form_factor',''),props.get('device.icon_name','')]
    for value in values:
        value=str(value).lower().replace('_','-')
        if value in ('headphones','headphone','headset','audio-headphones','audio-headset','audio-headphones-bluetooth','audio-headset-bluetooth'):return 'headphones'
        if value in ('speaker','speakers','audio-speakers','audio-speakers-bluetooth'):return 'speaker'
    return 'unknown'

def read(targets):
    if len(targets)>13:raise ValueError('Too many targets')
    snapshot=Snapshot();result=[]
    for target in targets:
        try:
            if target=='system':
                import system_sounds
                if target not in snapshot.cache:snapshot.cache[target]=system_sounds.read()
                result.append(snapshot.cache[target]);continue
            nodes=snapshot.nodes(target)[1]
            value=state(nodes)
            if value is not None:value['icon']=device_icon(target,nodes)
            if value is not None and target.startswith(('output:','input:')):
                try:value['active']=nodes[0]['name']==snapshot.info()['default_sink_name' if target.startswith('output:') else 'default_source_name']
                except (ValueError,KeyError,subprocess.SubprocessError):pass
            result.append(value)
        except (ValueError,KeyError,subprocess.SubprocessError):result.append(None)
    return result

def execute(action):
    target=action['target'];validate(target)
    if action['type']=='audio_select':
        if not target.startswith(('output:','input:')):raise ValueError('Choose an output or input device')
        kind,nodes=Snapshot().nodes(target)
        if not nodes:raise ValueError('Device unavailable')
        command('set-default-sink' if kind=='sinks' else 'set-default-source',nodes[0]['name'])
        return
    if target=='system':
        import system_sounds
        system_sounds.execute(action);return
    kind,nodes=Snapshot().nodes(target)
    if not nodes:raise ValueError('Audio target unavailable')
    if len(nodes)>16:raise ValueError('Too many application streams')
    suffix={'sinks':'sink','sources':'source','sink-inputs':'sink-input'}[kind]
    if action['type']=='audio_adjust':
        percent=action['percent']
        if not isinstance(percent,int) or not 0<abs(percent)<=20:raise ValueError('Invalid volume step')
        for node in nodes:
            value=state([node])
            if value is None:raise ValueError('Volume unavailable')
            amount=max(0,min(100,value['percent']+percent))
            command('set-'+suffix+'-volume',str(node['index']),str(amount)+'%')
    elif action['type']=='audio_mute':
        # Mixed states become muted together; never invert each stream independently.
        value='0' if all(node.get('mute',False) for node in nodes) else '1'
        for node in nodes:command('set-'+suffix+'-mute',str(node['index']),value)
    else:raise ValueError('Unsupported audio action')

def serve_read(source, output):
    """Bounded read-only requests, with a fresh shared snapshot per request."""
    while True:
        line=source.readline(8193)
        if not line:return
        if len(line)>8192 or not line.endswith('\n'):return
        try:
            targets=json.loads(line)
            if not isinstance(targets,list) or len(targets)>13:raise ValueError('Invalid targets')
            for target in targets:validate(target)
            result=read(targets)
        except Exception:
            result=None
        output.write(json.dumps(result)+'\n');output.flush()

if __name__=='__main__':
    try:
        if sys.argv[1]=='serve-read':serve_read(sys.stdin,sys.stdout)
        elif sys.argv[1]=='read':print(json.dumps(read(json.loads(sys.argv[2]))))
        elif sys.argv[1]=='execute':execute(json.loads(sys.argv[2]))
        else:raise ValueError('Unsupported operation')
    except subprocess.TimeoutExpired:sys.exit(4)
    except Exception as error:sys.exit(2 if str(error) in ('Device unavailable','Audio target unavailable') else 3 if str(error) in ('Volume unavailable','Too many application streams','Unsupported audio action') else 1)
