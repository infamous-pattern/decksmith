"""Key artwork rendering: use original pixels, preserve proportions, resize once."""
from io import BytesIO
from PIL import Image, ImageOps

MAX_BYTES=8*1024*1024

def source_image(data):
    if len(data)>MAX_BYTES: raise ValueError('Choose an image smaller than 8 MB.')
    source=Image.open(BytesIO(data))
    if source.format=='ICO':
        source=source.ico.getimage(max(source.ico.sizes(),key=lambda size:size[0]*size[1]))
    if source.width*source.height>16_000_000: raise ValueError('Choose an image below 16 megapixels.')
    source=ImageOps.exif_transpose(source)
    return source.convert('RGBA')

def dimensions(data):
    source=source_image(data)
    return source.size

def render(data, size=104, crop=False, background='#1e2227'):
    source=source_image(data)
    size=max(48,min(120,int(size)))
    image=ImageOps.fit(source,(size,size),Image.Resampling.LANCZOS) if crop else ImageOps.contain(source,(size,size),Image.Resampling.LANCZOS)
    canvas=Image.new('RGBA',(120,120),background)
    canvas.alpha_composite(image,((120-image.width)//2,(120-image.height)//2))
    output=BytesIO()
    canvas.convert('RGB').save(output,format='PNG',optimize=True)
    return list(output.getvalue())
