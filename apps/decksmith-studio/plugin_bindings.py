"""Opaque, bounded saved assignments. Never execute a saved payload here."""
import json
import re
from copy import deepcopy


def validate(binding):
    if not isinstance(binding, dict) or set(binding) != {'provider','action','schema','settings'}:
        raise ValueError('Invalid plugin assignment.')
    for key in ('provider','action'):
        if not isinstance(binding[key], str) or not re.fullmatch(r'[A-Za-z0-9_.-]{1,128}', binding[key]):
            raise ValueError('Invalid plugin identity.')
    if type(binding['schema']) is not int or not 1 <= binding['schema'] <= 4294967295:
        raise ValueError('Invalid plugin settings version.')
    def bounded(value, depth=0):
        if depth > 8:return False
        if isinstance(value, dict):return len(value)<=64 and all(isinstance(k,str) and len(k.encode())<=128 and bounded(v,depth+1) for k,v in value.items())
        if isinstance(value, list):return len(value)<=64 and all(bounded(v,depth+1) for v in value)
        return value is None or type(value) in (bool,int,float,str)
    settings=binding['settings']
    if not isinstance(settings,dict) or not bounded(settings) or len(json.dumps(settings,ensure_ascii=False,allow_nan=False,separators=(',',':')).encode())>4096:
        raise ValueError('Plugin settings exceed supported bounds.')


def test_page(layout):
    """Stage a page; never save, install, or touch the user's actual device."""
    if len(layout['pages'])>=16:raise ValueError('The layout already has 16 pages.')
    if any(p['name']=='HB TEST' for p in layout['pages']):raise ValueError('HB TEST already exists.')
    def binding(action, settings):
        return {'provider':'com.infamous-pattern.openhomeb','action':action,'schema':1,'settings':settings}
    target='b3d109c968d17f5cc965ddfa89a1fa695a2d60832200b819ad08127ee15634b4'
    page={'name':'HB TEST','background':[30,34,39],'keys':[
        {'label':'EMPTY','label_position':'hidden','action':{'type':'none'}} for _ in range(8)]}
    for index, (label,value) in enumerate((('Main On',True),('Main Off',False))):
        page['keys'][index]={'label':label,'action':{'type':'none'},'plugin':binding('com.infamous-pattern.openhomeb.set',
            {'accessoryId':target,'characteristicType':'On','targetValue':value})}
    page['keys'][7]={'label':'Home','action':{'type':'go_to_page','page':next((i for i,p in enumerate(layout['pages']) if p.get('default')),0)}}
    page['dial_overrides']=[{'label':'Main LEDs','rotation':'none','step':1,'press':{'type':'none'},
        'plugin_rotation':binding('com.infamous-pattern.openhomeb.brightness',{'accessoryId':target,'characteristicType':'Brightness','turnOnWhenAdjusting':False})},None,None,None]
    result=deepcopy(layout);result['pages'].append(page)
    return result
