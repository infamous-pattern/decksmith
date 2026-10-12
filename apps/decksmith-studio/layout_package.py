"""Versioned ZIP layout packages with separated behavior, appearance and assets."""
from copy import deepcopy
from hashlib import sha256
from io import BytesIO
import json
from urllib.parse import urlsplit,urlunsplit
from zipfile import ZipFile,ZIP_DEFLATED

LIMIT=2*1024*1024
KEY_STYLE=('label','artwork','follow_page_name','label_position','label_color','label_background','background_color','appearance','icon_source','icon_tint')
DIAL_STYLE=('label','appearance','target_icon_png','display_style')
MODELS={'plus':{'keys':8,'dials':4,'touch':True},'mk2':{'keys':15,'dials':0,'touch':False}}
LEGACY_ACTION_FIELDS={
    **{name:{} for name in ('none','next_page','previous_page','volume_up','volume_down','mute_toggle','media_play_pause','media_next','media_previous')},
    'system':{'command':str},'go_to_page':{'page':int},'open_application':{'desktop_id':str},'open_website':{'url':str},
    'audio_adjust':{'target':str,'percent':int},'audio_mute':{'target':str},'audio_select':{'target':str},'push_to_talk':{'target':str},
    'volume_adjust':{'percent':int},'brightness_adjust':{'percent':int}}

def _legacy_shapes(layout):
    """Do not downgrade a future action schema; semantic validation is native."""
    actions=[key['action'] for page in layout['pages'] for key in page['keys']]
    for dials in [layout.get('dials')]+[page.get('dial_overrides') for page in layout['pages']]:
        for dial in dials or []:
            if dial is not None:
                if not isinstance(dial,dict) or set(dial)-set(DIAL_STYLE)-{'rotation','step','press','audio_target','media_player','plugin_rotation','plugin_press'}:
                    raise ValueError('Dial settings cannot be represented in a legacy layout.')
                actions.append(dial['press'])
    for action in actions:
        if not isinstance(action,dict) or not isinstance(action.get('type'),str) or action['type'] not in LEGACY_ACTION_FIELDS:raise ValueError('Action cannot be represented in a legacy layout.')
        fields=LEGACY_ACTION_FIELDS[action['type']]
        if set(action)!={'type'}|set(fields) or any(type(action[key]) is not kind for key,kind in fields.items()):
            raise ValueError('Action settings cannot be represented in a legacy layout.')

def _model(layout):
    """Structural compatibility only; the Rust parser remains authoritative."""
    if not isinstance(layout,dict) or set(layout)-{'version','model','theme','audio_dial','dials','pages'}:
        raise ValueError('Unsupported layout fields.')
    version=layout.get('version')
    if type(version) is not int or version not in (1,2):raise ValueError('Unsupported layout version.')
    if version==1:
        if 'model' in layout:raise ValueError('Legacy layouts must not specify a device model.')
        model='plus'
    else:
        model=layout.get('model')
        if not isinstance(model,str) or model not in MODELS:raise ValueError('Unsupported device model.')
    requirements=MODELS[model]
    pages=layout.get('pages')
    if not isinstance(pages,list) or not 1<=len(pages)<=16:raise ValueError('Keep 1–16 pages.')
    key_fields=set(KEY_STYLE)|{'action','plugin','media_player','icon_png'}
    for page in pages:
        if not isinstance(page,dict) or set(page)-{'name','background','keys','default','application','dial_overrides'}:
            raise ValueError('Unsupported page fields.')
        keys=page.get('keys')
        if not isinstance(keys,list) or len(keys)!=requirements['keys']:raise ValueError('Key count does not match the device model.')
        if any(not isinstance(key,dict) or set(key)-key_fields for key in keys):raise ValueError('Unsupported key fields.')
        overrides=page.get('dial_overrides')
        if overrides is not None and (requirements['dials']==0 or not isinstance(overrides,list) or len(overrides)!=requirements['dials']):
            raise ValueError('Dial overrides do not match the device model.')
    if 'audio_dial' in layout and type(layout['audio_dial']) is not bool:raise ValueError('Invalid audio dial setting.')
    dials=layout.get('dials')
    if model=='mk2' and (dials is not None or layout.get('audio_dial',False)):raise ValueError('MK.2 has no dials.')
    if dials is not None and (not isinstance(dials,list) or len(dials)!=requirements['dials']):raise ValueError('Dial count does not match the device model.')
    if len(json.dumps(layout,ensure_ascii=False,allow_nan=False,separators=(',',':')).encode())>1024*1024:
        raise ValueError('Layout exceeds 1 MB.')
    return model

def encode(layout, *, legacy=False):
    model=_model(layout)
    behavior=deepcopy(layout);appearance=[];files={};redacted=0
    if legacy:
        if model!='plus':raise ValueError('MK.2 cannot be exported as an eight-key legacy layout.')
        _legacy_shapes(behavior)
        behavior.pop('model',None);behavior['version']=1
    theme=behavior.pop('theme',None)
    if theme is not None:files['theme.json']=json.dumps(theme).encode()
    for page in behavior['pages']:
        visual={'background':page.pop('background'),'keys':[]}
        for key in page['keys']:
            style={name:key.pop(name) for name in KEY_STYLE if name in key}
            data=key.pop('icon_png',None)
            if data is not None:
                data=bytes(data);name='assets/'+sha256(data).hexdigest()+'.png'
                files[name]=data;style['image']=name
            visual['keys'].append(style)
            action=key['action']
            if action['type']=='open_website':
                url=urlsplit(action['url'])
                host=url.netloc.rsplit('@',1)[-1]
                clean=urlunsplit((url.scheme,host,url.path,'',''))
                redacted+=clean!=action['url'];action['url']=clean
        if page.get('dial_overrides') is not None:
            visual['dial_overrides']=[{k:d.pop(k) for k in DIAL_STYLE if k in d} if d is not None else None for d in page['dial_overrides']]
        appearance.append(visual)
    if behavior.get('dials') is not None:
        files['dial-appearance.json']=json.dumps([{k:dial.pop(k) for k in DIAL_STYLE if k in dial} for dial in behavior['dials']]).encode()
    notices=[]
    if any(k.get('icon_source','').startswith('tabler:') for p in layout['pages'] for k in p['keys']):
        from icon_library import ROOT
        notices.append(b'Tabler Icons v3.46.0 - https://github.com/tabler/tabler-icons\n'+(ROOT/'LICENSE').read_bytes())
    if any(k.get('icon_source','').startswith('rune:') for p in layout['pages'] for k in p['keys']):
        from icon_library import RUNE_ROOT
        notices.append((RUNE_ROOT/'NOTICE').read_bytes()+b'\n'+(RUNE_ROOT/'LICENSE').read_bytes())
    if notices:files['asset-notices.txt']=b'\n\n'.join(notices)
    files['behavior.json']=json.dumps(behavior,separators=(',',':')).encode()
    files['appearance.json']=json.dumps(appearance,separators=(',',':')).encode()
    package_version=behavior['version']
    manifest={'format':'decksmith-layout','version':package_version,'checksums':{name:sha256(data).hexdigest() for name,data in files.items()}}
    if package_version==2:manifest.update(model=model,requirements=MODELS[model])
    files['manifest.json']=json.dumps(manifest).encode()
    if len(files)>(160 if package_version==1 else 256):raise ValueError('Too many package entries.')
    output=BytesIO()
    with ZipFile(output,'w',ZIP_DEFLATED) as archive:
        for name,data in sorted(files.items()):archive.writestr(name,data)
    if len(output.getvalue())>LIMIT:raise ValueError('Layout package exceeds 2 MB.')
    return output.getvalue(),redacted

def decode(data):
    if len(data)>LIMIT:raise ValueError('Choose a layout package smaller than 2 MB.')
    with ZipFile(BytesIO(data)) as archive:
        entries=archive.infolist()
        if len(entries)>256 or len({entry.filename for entry in entries})!=len(entries) or sum(entry.file_size for entry in entries)>LIMIT:
            raise ValueError('Invalid or oversized package.')
        files={entry.filename:archive.read(entry) for entry in entries}
    manifest=json.loads(files.pop('manifest.json'))
    version=manifest.get('version')
    if manifest.get('format')!='decksmith-layout' or type(version) is not int or version not in (1,2):raise ValueError('Unsupported layout package version.')
    allowed={'format','version','checksums'}|({'model','requirements'} if version==2 else set())
    if set(manifest)!=allowed or not isinstance(manifest['checksums'],dict):raise ValueError('Invalid package manifest.')
    if version==1 and len(entries)>160:raise ValueError('Too many legacy package entries.')
    if set(files)!=set(manifest['checksums']):raise ValueError('Package contents do not match its manifest.')
    for name,data in files.items():
        if name not in ('behavior.json','appearance.json','dial-appearance.json','theme.json','asset-notices.txt') and name!='assets/'+sha256(data).hexdigest()+'.png':raise ValueError('Invalid asset name.')
        if sha256(data).hexdigest()!=manifest['checksums'][name]:raise ValueError('Package checksum failed.')
    layout=json.loads(files['behavior.json'])
    if 'theme.json' in files:layout['theme']=json.loads(files['theme.json'])
    appearance=json.loads(files['appearance.json'])
    if len(layout['pages'])!=len(appearance):raise ValueError('Page appearance mismatch.')
    for page,visual in zip(layout['pages'],appearance):
        page['background']=visual['background']
        if len(page['keys'])!=len(visual['keys']):raise ValueError('Key appearance mismatch.')
        for key,style in zip(page['keys'],visual['keys']):
            style=dict(style);asset=style.pop('image',None)
            if set(style)-set(KEY_STYLE):raise ValueError('Unsupported appearance field.')
            key.update(style)
            if asset is not None:key['icon_png']=list(files[asset])
        if 'dial_overrides' in visual:
            styles=visual['dial_overrides'];dials=page.get('dial_overrides',[])
            if len(styles)!=len(dials):raise ValueError('Dial override appearance mismatch.')
            for dial,style in zip(dials,styles):
                if (dial is None)!=(style is None):raise ValueError('Dial override appearance mismatch.')
                if style is not None:
                    if set(style)-set(DIAL_STYLE):raise ValueError('Unsupported dial appearance field.')
                    dial.update(style)
    if 'dial-appearance.json' in files:
        styles=json.loads(files['dial-appearance.json'])
        if len(styles)!=len(layout.get('dials',[])):raise ValueError('Dial appearance mismatch.')
        for dial,style in zip(layout['dials'],styles):
            if 'label' not in style or set(style)-set(DIAL_STYLE):raise ValueError('Unsupported dial appearance field.')
            dial.update(style)
    model=_model(layout)
    if layout['version']!=version:raise ValueError('Package and layout versions do not match.')
    if version==2:
        requirements=manifest['requirements'];expected=MODELS[model]
        if manifest['model']!=model or not isinstance(requirements,dict) or set(requirements)!=set(expected) or any(type(requirements[k]) is not type(v) or requirements[k]!=v for k,v in expected.items()):
            raise ValueError('Package device requirements do not match the layout.')
    return layout
