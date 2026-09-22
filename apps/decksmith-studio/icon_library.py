"""Offline Tabler starter catalog and content-addressed local user imports."""
from pathlib import Path
from hashlib import sha256
from io import BytesIO
import json,os,re
import xml.etree.ElementTree as ET
from PIL import Image,ImageOps
from artwork import source_image,MAX_BYTES
ROOT=Path(__file__).resolve().parents[2]/'assets/icons/tabler'
FULL_SET_URL='https://github.com/tabler/tabler-icons/releases/latest'

class SafeSvgTree(ET.TreeBuilder):
    def doctype(self, name, pubid, system):
        raise ValueError('SVG document types and entities are not supported.')

def svg_png(raw,size=120,color='#ffffff'):
    if len(raw)>MAX_BYTES or b'<!DOCTYPE' in raw.upper() or b'<!ENTITY' in raw.upper():raise ValueError('Unsupported SVG document.')
    root=ET.fromstring(raw,parser=ET.XMLParser(target=SafeSvgTree()))
    allowed={'svg','g','path','rect','circle','ellipse','line','polyline','polygon','title','desc'}
    for e in root.iter():
        if e.tag.split('}')[-1] not in allowed:raise ValueError('Use a self-contained SVG made of simple shapes.')
        for k,v in e.attrib.items():
            if k.split('}')[-1].lower().startswith(('on','href')) or 'url(' in v.lower() or '@import' in v.lower():raise ValueError('External or active SVG content is not supported.')
    root.set('color',color)
    import gi,cairo
    gi.require_version('Rsvg','2.0')
    from gi.repository import Rsvg
    ET.register_namespace('', 'http://www.w3.org/2000/svg')
    handle=Rsvg.Handle.new_from_data(ET.tostring(root))
    surface=cairo.ImageSurface(cairo.FORMAT_ARGB32,size,size)
    viewport=Rsvg.Rectangle();viewport.x=0;viewport.y=0;viewport.width=size;viewport.height=size
    handle.render_document(cairo.Context(surface),viewport)
    out=BytesIO();surface.write_to_png(out);return out.getvalue()

def raster(raw,size=120):
    image=source_image(raw);image=ImageOps.contain(image,(round(size*.87),round(size*.87)),Image.Resampling.LANCZOS)
    canvas=Image.new('RGBA',(size,size));canvas.alpha_composite(image,((size-image.width)//2,(size-image.height)//2))
    out=BytesIO();canvas.save(out,format='PNG',optimize=True);return out.getvalue()

class Library:
    def __init__(self,root=None):
        self.root=Path(root) if root else Path(os.environ.get('XDG_DATA_HOME',Path.home()/'.local/share'))/'decksmith/icons'
        self.catalog=json.loads((ROOT/'catalog.json').read_text())
    def items(self,query='',category=None):
        items=[dict(item,source='Tabler') for item in self.catalog['items']]
        if self.root.exists():
            for path in sorted(self.root.glob('*.json')):
                try:
                    d=json.loads(path.read_text());digest=d['hash']
                    if re.fullmatch('[a-f0-9]{64}',digest) and path.stem==digest and (self.root/(digest+'.png')).is_file():
                        items.append(dict(d,id='local:'+digest,source='Imported',category='Imported',tags=d['name']))
                except (OSError,ValueError,KeyError):continue
        words=query.lower().split()
        return [i for i in items if (not category or i['category']==category) and all(w in (i['name']+' '+i.get('tags','')+' '+i['source']).lower() for w in words)]
    def image(self,item,size=120,color='#ffffff'):
        if item['id'].startswith('tabler:'):
            name=item['file']
            if Path(name).name!=name:raise ValueError('Invalid icon name.')
            raw=(ROOT/name).read_bytes()
            if sha256(raw).hexdigest()!=item['sha256']:raise ValueError('Bundled icon checksum mismatch.')
            return raster(svg_png(raw,max(size,256),color),size)
        digest=item['id'].removeprefix('local:')
        if not re.fullmatch('[a-f0-9]{64}',digest):raise ValueError('Invalid imported icon.')
        raw=(self.root/(digest+'.png')).read_bytes()
        if sha256(raw).hexdigest()!=digest:raise ValueError('Imported icon checksum mismatch.')
        return raster(raw,size)
    def import_file(self,path):
        path=Path(path)
        if not path.is_file():raise ValueError('Choose an image file.')
        if path.stat().st_size>MAX_BYTES:raise ValueError('Choose an icon smaller than 8 MB.')
        raw=path.read_bytes()
        if path.suffix.lower()=='.svg':raw=svg_png(raw,512)
        # Store a generous normalized source; the key render is made from this once.
        image=source_image(raw);image.thumbnail((1024,1024),Image.Resampling.LANCZOS)
        out=BytesIO();image.save(out,format='PNG',optimize=True);png=out.getvalue();digest=sha256(png).hexdigest()
        name=path.stem[:80];self.root.mkdir(parents=True,exist_ok=True)
        target=self.root/(digest+'.png');metadata=self.root/(digest+'.json')
        if not target.exists():
            temp=target.with_suffix('.tmp');temp.write_bytes(png);temp.replace(target)
        if not metadata.exists():
            data={'hash':digest,'name':name,'original_format':path.suffix.lower(),'provenance':'User imported local artwork'}
            temp=metadata.with_suffix('.tmp');temp.write_text(json.dumps(data));temp.replace(metadata)
        return next(i for i in self.items() if i['id']=='local:'+digest)
