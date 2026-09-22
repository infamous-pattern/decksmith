"""Appearance-only defaults and overrides; never alter a control's behavior."""
from copy import deepcopy
import json
PRESETS={'makers_mark':("Maker’s Mark",[107,58,26],[30,34,39],[240,230,215],[90,170,255]),
         'dark':('Dark',[30,34,39],[22,25,30],[245,245,245],[90,170,255]),
         'light':('Light',[236,240,245],[242,244,248],[24,30,38],[25,90,175]),
         'high_contrast':('High Contrast',[0,0,0],[0,0,0],[255,255,255],[255,225,60])}
FONTS={'sans': 'Sans', 'serif': 'Serif', 'mono': 'Monospace', 'roboto': 'Roboto', 'open_sans': 'Open Sans', 'lato': 'Lato', 'montserrat': 'Montserrat', 'oswald': 'Oswald', 'raleway': 'Raleway', 'poppins': 'Poppins', 'nunito': 'Nunito', 'merriweather': 'Merriweather', 'source_sans3': 'Source Sans 3', 'viking_runes': 'Viking Runes (decorative)'}
SIZES={'small':'Small','normal':'Normal','large':'Large'}
COLORS={'default':'#f0e6d7','white':'#ffffff','black':'#000000','red':'#ff4646','orange':'#ff9b3c','yellow':'#ffe13c','green':'#5ae678','blue':'#5aaaff','purple':'#c382ff'}
STYLE_FIELDS={'icon_size':range(10,101),'font':FONTS,'size':SIZES,'label_color':COLORS,'background_color':COLORS,
              'label_position':dict.fromkeys(('hidden','top','middle','bottom')),
              'label_background':dict.fromkeys(('dark','transparent'))}
LEGACY=('label_position','label_color','background_color','label_background')

def validate_style(style):
    if not isinstance(style,dict) or set(style)-set(STYLE_FIELDS):raise ValueError('Unsupported appearance settings.')
    for k,v in style.items():
        if k=='icon_size':
            if type(v) is not int or not 10<=v<=100 or v%5:raise ValueError('Icon size must be 10–100% in steps of 5.')
        elif not isinstance(v,str) or v not in STYLE_FIELDS[k]:raise ValueError('Choose a supported appearance setting.')

def validate(layout):
    theme=layout.get('theme')
    if theme is not None:
        if not isinstance(theme,dict) or set(theme)-{'preset','appearance'} or theme.get('preset') not in PRESETS:raise ValueError('Choose a supported theme.')
        validate_style(theme.get('appearance',{}))
    for page in layout['pages']:
        for control in page['keys']+list(filter(None,page.get('dial_overrides') or [])):validate_style(control.get('appearance',{}))
    for dial in layout.get('dials') or []:validate_style(dial.get('appearance',{}))

def effective(layout,control,page_background=None,dial=False):
    theme=layout.get('theme');preset=PRESETS[(theme or {}).get('preset','makers_mark')]
    system=not dial and control.get('action',{}).get('type')=='system'
    style={'icon_size':75 if system else 100,'font':'sans','size':'normal','label_position':'bottom' if theme or system else ('hidden' if control.get('artwork') else 'middle'),
           'label_background':'transparent' if theme else 'dark','label_color':'default','background_color':'default'}
    for layer in ((theme or {}).get('appearance',{}),{k:control[k] for k in LEGACY if k in control},control.get('appearance',{})):
        style.update({k:v for k,v in layer.items() if not (k in ('label_color','background_color') and v=='default')})
    bg=preset[2 if dial else 1] if theme else ([30,34,39] if dial else (page_background or preset[1]))
    style['background']=bg if style['background_color']=='default' else rgb(style['background_color'])
    style['color']=preset[3] if style['label_color']=='default' else rgb(style['label_color'])
    style['accent']=preset[4]
    return style

def rgb(name):return list(bytes.fromhex(COLORS[name].lstrip('#')))
def reset(control):
    control.pop('appearance',None)
    for k in LEGACY:control.pop(k,None)
def set_theme(layout,preset):
    if preset is None:layout.pop('theme',None)
    else:layout['theme']={'preset':preset,'appearance':deepcopy(layout.get('theme',{}).get('appearance',{}))}


def encode_theme(theme):
    validate({'theme':theme,'pages':[]})
    if theme is None:raise ValueError('Choose a preset first.')
    return json.dumps({'format':'decksmith-theme','version':1,'theme':theme},indent=2).encode()

def decode_theme(raw):
    if len(raw)>65536:raise ValueError('Theme file is too large.')
    data=json.loads(raw)
    if not isinstance(data,dict) or set(data)!={'format','version','theme'} or data['format']!='decksmith-theme' or type(data['version']) is not int or data['version']!=1 or not isinstance(data['theme'],dict):raise ValueError('Unsupported theme file.')
    validate({'theme':data['theme'],'pages':[]})
    return data['theme']
