"""Capability-checked assignments; one serial write, independent readback, no retry."""
import asyncio
from accessory_catalog import caption, finite
from paced_host import SET

PROVIDER='com.infamous-pattern.openhomeb'

def decode(binding,ticks):
 if not isinstance(binding,dict) or set(binding)!={'provider','action','schema','settings'}:raise ValueError('Invalid binding')
 s=binding['settings'];op=binding['action'].removeprefix(PROVIDER+'.')
 if binding['provider']!=PROVIDER or type(binding['schema']) is not int or binding['schema']!=2 or binding['action']!=PROVIDER+'.'+op:raise ValueError('Unsupported binding')
 if op not in ('toggle','on','off','level','status') or not isinstance(s,dict) or set(s)!={'accessoryId'}:raise ValueError('Unsupported action')
 identity=s['accessoryId']
 if not isinstance(identity,str) or len(identity)!=64 or any(c not in '0123456789abcdef' for c in identity):raise ValueError('Invalid identity')
 if type(ticks) is not int or (not 0<abs(ticks)<=20 if op=='level' else ticks!=0):raise ValueError('Invalid input')
 return identity,op

async def perform(panel,binding,ticks):
 identity,op=decode(binding,ticks);context='assigned';dispatched=False
 try:
  item,raw=await asyncio.to_thread(panel.catalog.one,identity)
  panel.last_error=None
  if op not in item['operations']:raise ValueError('Capability unavailable')
  if op=='status':
   panel.assigned_states[identity]={'text':caption(item),'on':True};return True
  chars={c['type']:c for c in raw['serviceCharacteristics']}
  characteristic=item['level'] if op=='level' else item['power']
  value=chars[characteristic].get('value')
  if op=='level':
   if not finite(value) or not 0<=value<=100:raise ValueError('Invalid level')
   power=item['power']
   if power and chars[power].get('value') in (False,0):return True
   step=chars[characteristic].get('minStep',1)
   if not finite(step) or step<=0 or step>100:step=1
   expected=max(0,min(100,round((value+ticks*step)/step)*step))
  else:
   if value not in (False,True,0,1):raise ValueError('Invalid power')
   expected=not bool(value) if op=='toggle' else op=='on'
   if chars[characteristic].get('format')!='bool':expected=int(expected)
  await panel.host.disappear(context)
  panel.host.saved['instances'].pop(context,None)
  await panel.host.appear(context,SET,{'accessoryId':identity,'characteristicType':characteristic,'targetValue':expected})
  dispatched=True
  if not await panel.host.input(context,'keyUp',panel.host.epoch) or panel.host.completions.get(context)!='accepted':raise RuntimeError('Write not acknowledged')
  async with asyncio.timeout(3):
   while True:
    item,raw=await asyncio.to_thread(panel.catalog.one,identity)
    observed=next(c.get('value') for c in raw['serviceCharacteristics'] if c['type']==characteristic)
    if (abs(observed-expected)<0.0001 if op=='level' and finite(observed) else observed==expected):break
    await asyncio.sleep(.2)
  state={'text':caption(item),'on':bool(expected) if op!='level' else True}
  if op=='level':state['level']=observed;state['text']=f'{observed:g}%'
  panel.assigned_states[identity]=state
  panel.status='Homebridge confirmed the assignment.'
  return True
 except asyncio.CancelledError:
  if dispatched:panel.manual_recovery=True;panel.recovery_required=True
  raise
 except Exception:
  panel.last_error='unconfirmed' if dispatched else 'device_unavailable'
  if dispatched:panel.manual_recovery=True;panel.recovery_required=True
  panel.status='Accessory unavailable or command unconfirmed. Nothing was replayed.'
  return False
 finally:
  panel.busy=False
