"""Read-only review of saved Homebridge assignments, including dial inheritance."""
from i18n import gettext as tr
PROVIDER='com.infamous-pattern.openhomeb'

def assignments(layout,items,available):
 names={item['id']:item['name'] for item in items}
 rows=[]
 def add(binding,location,label):
  if not isinstance(binding,dict) or binding.get('provider')!=PROVIDER:return
  settings=binding.get('settings',{});identity=settings.get('accessoryId','')
  operation=binding.get('action','').removeprefix(PROVIDER+'.')
  if operation=='set':operation='on' if settings.get('targetValue') else 'off'
  action={'on':tr('On'),'off':tr('Off'),'toggle':tr('Toggle power'),'level':tr('Adjust level'),'brightness':tr('Brightness'),'status':tr('Status only')}.get(operation,tr('Unsupported action'))
  name=names.get(identity)
  target=name or ((label+' · ') if label else '')+tr('Accessory unavailable')
  subtitle=target+' — '+action
  if not available:subtitle+=' · '+tr('Connection unavailable')
  rows.append((location,subtitle))
 shared=layout.get('dials') or []
 for page in layout.get('pages',[]):
  page_name=page.get('name','')
  for i,key in enumerate(page.get('keys',[])):
   add(key.get('plugin'),page_name+' · '+tr('Key')+f' {i+1}',key.get('label',''))
  overrides=page.get('dial_overrides') or []
  for i in range(4):
   override=overrides[i] if i<len(overrides) else None
   dial=override if override is not None else shared[i] if i<len(shared) else None
   if not dial:continue
   location=page_name+' · '+tr('Dial')+f' {i+1}'
   if override is None:location+=' · '+tr('Shared')
   add(dial.get('plugin_rotation'),location+' · '+tr('Turn'),dial.get('label',''))
   add(dial.get('plugin_press'),location+' · '+tr('Press'),dial.get('label',''))
 return rows

def read_review(client):
 import json
 from panel import call
 layout=json.loads(call('GetLayout')[0])
 try:catalog=client.catalog()
 except Exception:catalog={'items':[],'available':False}
 return assignments(layout,catalog['items'],catalog['available'])
