"""Shared defaults and full per-dial page overrides, without I/O."""
from copy import deepcopy

def defaults(layout):
    if layout.get('dials') is not None:return deepcopy(layout['dials'])
    return [{'label':'Volume' if i==0 and layout.get('audio_dial') else f'Dial {i+1}',
             'rotation':'volume' if i==0 and layout.get('audio_dial') else 'none','step':1,
             'press':{'type':'mute_toggle' if i==0 and layout.get('audio_dial') else 'none'}} for i in range(4)]

def is_override(layout,page,index):
    return (layout['pages'][page].get('dial_overrides') or [None]*4)[index] is not None

def effective(layout,page):
    shared=defaults(layout);overrides=layout['pages'][page].get('dial_overrides') or [None]*4
    return [deepcopy(overrides[i]) if overrides[i] is not None else shared[i] for i in range(4)]

def customize(layout,page,index):
    dial=effective(layout,page)[index]
    target=layout['pages'][page]
    if target.get('dial_overrides') is None:target['dial_overrides']=[None]*4
    target['dial_overrides'][index]=dial

def revert(layout,page,index):
    target=layout['pages'][page];overrides=target.get('dial_overrides')
    if overrides is not None:
        overrides[index]=None
        if not any(d is not None for d in overrides):target.pop('dial_overrides')

def store(layout,page,index,dial):
    if is_override(layout,page,index):layout['pages'][page]['dial_overrides'][index]=deepcopy(dial)
    else:
        shared=defaults(layout);shared[index]=deepcopy(dial);layout['dials']=shared
