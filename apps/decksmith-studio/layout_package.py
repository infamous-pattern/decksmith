"""Versioned ZIP layout packages with separated behavior, appearance and assets."""
from copy import deepcopy
from hashlib import sha256
from io import BytesIO
import json
from urllib.parse import urlsplit,urlunsplit
from zipfile import ZipFile,ZIP_DEFLATED

LIMIT=2*1024*1024
KEY_STYLE=('label','artwork','follow_page_name','label_position','label_color','label_background','background_color','appearance','icon_source','icon_tint')
DIAL_STYLE=('label','appearance','target_icon_png')

def encode(layout):
    behavior=deepcopy(layout);appearance=[];files={};redacted=0
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
    if any(k.get('icon_source','').startswith('tabler:') for p in layout['pages'] for k in p['keys']):
        from icon_library import ROOT
        files['asset-notices.txt']=b'Tabler Icons v3.46.0 - https://github.com/tabler/tabler-icons\n'+(ROOT/'LICENSE').read_bytes()
    files['behavior.json']=json.dumps(behavior,separators=(',',':')).encode()
    files['appearance.json']=json.dumps(appearance,separators=(',',':')).encode()
    manifest={'format':'decksmith-layout','version':1,'checksums':{name:sha256(data).hexdigest() for name,data in files.items()}}
    files['manifest.json']=json.dumps(manifest).encode()
    output=BytesIO()
    with ZipFile(output,'w',ZIP_DEFLATED) as archive:
        for name,data in sorted(files.items()):archive.writestr(name,data)
    if len(output.getvalue())>LIMIT:raise ValueError('Layout package exceeds 2 MB.')
    return output.getvalue(),redacted

def decode(data):
    if len(data)>LIMIT:raise ValueError('Choose a layout package smaller than 2 MB.')
    with ZipFile(BytesIO(data)) as archive:
        entries=archive.infolist()
        if len(entries)>160 or len({entry.filename for entry in entries})!=len(entries) or sum(entry.file_size for entry in entries)>LIMIT:
            raise ValueError('Invalid or oversized package.')
        files={entry.filename:archive.read(entry) for entry in entries}
    manifest=json.loads(files.pop('manifest.json'))
    if manifest.get('format')!='decksmith-layout' or manifest.get('version')!=1:raise ValueError('Unsupported layout package version.')
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
            if len(styles)!=4 or len(dials)!=4:raise ValueError('Dial override appearance mismatch.')
            for dial,style in zip(dials,styles):
                if (dial is None)!=(style is None):raise ValueError('Dial override appearance mismatch.')
                if style is not None:
                    if set(style)-set(DIAL_STYLE):raise ValueError('Unsupported dial appearance field.')
                    dial.update(style)
    if 'dial-appearance.json' in files:
        styles=json.loads(files['dial-appearance.json'])
        if len(styles)!=4 or len(layout.get('dials',[]))!=4:raise ValueError('Dial appearance mismatch.')
        for dial,style in zip(layout['dials'],styles):
            if 'label' not in style or set(style)-set(DIAL_STYLE):raise ValueError('Unsupported dial appearance field.')
            dial.update(style)
    return layout
