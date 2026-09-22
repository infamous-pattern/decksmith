"""Portable, supersampled media symbols with room for a two-line caption."""
from functools import lru_cache
from io import BytesIO
from PIL import Image, ImageDraw

LABELS={'media_play_pause':'Play Pause','media_previous':'Previous Track','media_next':'Next Track'}

@lru_cache(maxsize=16)
def icon(action, background='#1e2227'):
    scale=4
    image=Image.new('RGB',(120*scale,120*scale),background)
    draw=ImageDraw.Draw(image)
    def polygon(points):draw.polygon([(x*scale,y*scale) for x,y in points],fill='#5aaaff')
    def rectangle(box):draw.rectangle(tuple(n*scale for n in box),fill='#5aaaff')
    if action=='media_play_pause':
        polygon([(26,14),(26,54),(55,34)])
        rectangle((66,14,75,54));rectangle((84,14,93,54))
    elif action in ('media_previous','media_next'):
        points=[(40,14),(40,54),(76,34)]
        if action=='media_previous':points=[(120-x,y) for x,y in points]
        polygon(points)
        rectangle((29,14,37,54) if action=='media_previous' else (83,14,91,54))
    else:raise ValueError('Not a media action')
    image=image.resize((120,120),Image.Resampling.LANCZOS)
    out=BytesIO();image.save(out,format='PNG')
    return out.getvalue()

def populate(key, background='#1e2227', preserve_label=False):
    action=key['action']['type']
    key.pop('icon_source',None);key.pop('icon_tint',None)
    key.update(label=key.get('label',LABELS[action]) if preserve_label else LABELS[action],artwork='application_icon',icon_png=list(icon(action,background)),
               label_position='bottom',label_color='white',label_background='transparent',follow_page_name=False)
