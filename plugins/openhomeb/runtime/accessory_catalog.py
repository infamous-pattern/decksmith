"""Bounded, read-only Homebridge discovery and explicit capability policy."""
import json
import math
import time
import urllib.request
import urllib.error
from urllib.parse import quote
BASE='http://homebridge.local:8581'
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs):return None

TYPES = {
 'Lightbulb': ('Light','lightbulb-symbolic'), 'Outlet': ('Socket','power-plug-symbolic'),
 'Switch': ('Switch','switch-on-symbolic'), 'Fan': ('Fan','weather-windy-symbolic'),
 'Fanv2': ('Fan','weather-windy-symbolic'), 'Speaker': ('Speaker','audio-speakers-symbolic'),
 'Microphone': ('Microphone','audio-input-microphone-symbolic'),
 'CameraOperatingMode': ('Camera','camera-video-symbolic'),
 'MotionSensor': ('Motion sensor','motion-sensor-symbolic'),
 'ContactSensor': ('Contact sensor','door-symbolic'),
 'TemperatureSensor': ('Temperature sensor','temperature-symbolic'),
 'HumiditySensor': ('Humidity sensor','weather-showers-symbolic'),
 'LightSensor': ('Light sensor','display-brightness-symbolic'),
 'OccupancySensor': ('Occupancy sensor','system-users-symbolic'),
 'SmokeSensor': ('Smoke sensor','dialog-warning-symbolic'),
 'LeakSensor': ('Leak sensor','water-symbolic'),
 'CarbonMonoxideSensor': ('CO sensor','dialog-warning-symbolic'),
 'CarbonDioxideSensor': ('CO2 sensor','dialog-warning-symbolic'),
 'AirQualitySensor': ('Air quality sensor','weather-fog-symbolic'),
 'Battery': ('Battery','battery-symbolic'), 'Doorbell': ('Doorbell','alarm-symbolic'),
 'SecuritySystem': ('Security system','security-high-symbolic'),
}
READINGS = ('MotionDetected','ContactSensorState','CurrentTemperature','CurrentRelativeHumidity',
 'CurrentAmbientLightLevel','OccupancyDetected','SmokeDetected','LeakDetected','CarbonMonoxideDetected',
 'CarbonDioxideDetected','AirQuality','BatteryLevel','SecuritySystemCurrentState',
 'HomeKitCameraActive','Mute','Volume','On','Active','ProgrammableSwitchEvent','Brightness','RotationSpeed')


def finite(value):
 return type(value) in (int,float) and math.isfinite(value)


def describe(service):
 kind=service.get('type');identity=service.get('uniqueId')
 if kind not in TYPES or not isinstance(identity,str) or not 1<=len(identity)<=128:return None
 name=service.get('serviceName') or TYPES[kind][0]
 if not isinstance(name,str):return None
 chars={c.get('type'):c for c in service.get('serviceCharacteristics',[]) if isinstance(c,dict)}
 def readable(k):return k in chars and ('pr' in chars[k].get('perms',[]) or ('perms' not in chars[k] and chars[k].get('canRead') is True))
 def writable(k):return readable(k) and ('pw' in chars[k].get('perms',[]) or ('perms' not in chars[k] and chars[k].get('canWrite') is True))
 power='On' if kind in ('Lightbulb','Outlet','Switch','Fan') else 'Active' if kind=='Fanv2' else None
 if power and not writable(power):power=None
 if power and chars[power].get('format') not in ('bool','uint8'):power=None
 level='Brightness' if kind=='Lightbulb' else 'RotationSpeed' if kind in ('Fan','Fanv2') else None
 if level and not (power and writable(level) and chars[level].get('format') in ('int','uint8','float')
  and finite(chars[level].get('minValue')) and finite(chars[level].get('maxValue'))
  and chars[level]['minValue']==0 and chars[level]['maxValue']==100):level=None
 readings=[]
 for k in READINGS:
  if readable(k):
   v=chars[k].get('value')
   if type(v) is bool or finite(v):readings.append({'type':k,'value':v,'unit':str(chars[k].get('unit') or '')[:20]})
 operations=[]
 if power:operations += ['toggle','on','off']
 if level:operations.append('level')
 if readings:operations.append('status')
 if not operations:return None
 return {'id':identity,'name':name[:128],'kind':TYPES[kind][0],'icon':TYPES[kind][1],
  'power':power,'level':level,'operations':operations,'readings':readings,
  'default':'toggle' if power else 'status'}


def caption(item):
 readings=item['readings']
 if not readings:return 'Unavailable'
 r=readings[0];k=r['type'];v=r['value']
 pairs={'MotionDetected':('Clear','Motion'),'ContactSensorState':('Closed','Open'),
  'OccupancyDetected':('Clear','Occupied'),'SmokeDetected':('Clear','Smoke'),
  'LeakDetected':('Dry','Leak'),'On':('Off','On'),'Active':('Off','On'),
  'HomeKitCameraActive':('Inactive','Active'),'Mute':('Live','Muted')}
 if k in pairs and v in (0,1):return pairs[k][int(v)]
 if k in ('CarbonMonoxideDetected','CarbonDioxideDetected'):return 'Clear' if v==0 else 'Detected' if v==1 else 'Unknown'
 if k=='AirQuality':return {0:'Unknown',1:'Excellent',2:'Good',3:'Fair',4:'Inferior',5:'Poor'}.get(v,'Unknown')
 if k=='CurrentTemperature':return f'{v:g} C'
 if k in ('CurrentRelativeHumidity','BatteryLevel','Volume'):return f'{v:g}%'
 if k=='CurrentAmbientLightLevel':return f'{v:g} lux'
 if k=='SecuritySystemCurrentState':return {0:'Stay armed',1:'Away armed',2:'Night armed',3:'Disarmed',4:'Alarm'}.get(v,'Unknown')
 if k=='ProgrammableSwitchEvent':return 'Connected' # Last event is not current ringing state.
 return str(v)


class AuthenticationError(RuntimeError):
 def __init__(self,state):super().__init__(state);self.state=state

class Catalog:
 def __init__(self,base=BASE,username='',password='',otp=None):
  self.items=[];self.updated=0;self.token=None;self.base=base
  self.username=username;self.password=password;self.otp=otp;self.expires=0;self.auth_error=None;self.auth_method='none'
 def authenticate(self,opener):
  if self.auth_error:raise AuthenticationError(self.auth_error)
  def post(path,data):
   with opener.open(urllib.request.Request(self.base+path,data=json.dumps(data).encode(),headers={'Content-Type':'application/json'},method='POST'),timeout=5) as r:
    raw=r.read(16385)
   if len(raw)>16384:raise ValueError('Invalid authentication response')
   return json.loads(raw)
  try:
   try:
    result=post('/api/auth/noauth',{});self.auth_method='none'
   except urllib.error.HTTPError as e:
    e.close()
    if e.code not in (401,403):raise
    if not self.username or not self.password:raise AuthenticationError('required') from None
    payload={'username':self.username,'password':self.password}
    if self.otp:payload['otp']=self.otp
    try:
     result=post('/api/auth/login',payload);self.auth_method='password'
    except urllib.error.HTTPError as e:
     e.close()
     if e.code==412:raise AuthenticationError('two_factor') from None
     if e.code in (401,403):raise AuthenticationError('rejected') from None
     raise
   token=result.get('access_token');seconds=result.get('expires_in',3600)
   if not isinstance(token,str) or not 1<=len(token)<=8192 or any(c.isspace() for c in token):raise ValueError('Invalid session')
   if type(seconds) not in (int,float) or not 1<=seconds<=31536000:raise ValueError('Invalid session lifetime')
   self.token=token;self.expires=time.monotonic()+seconds
  except AuthenticationError as e:self.auth_error=e.state;raise
  finally:self.otp=None
 def request(self,path):
  opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),NoRedirect())
  if self.token is None or time.monotonic()>=self.expires:self.authenticate(opener)
  try:
   with opener.open(urllib.request.Request(self.base+path,headers={'Authorization':'Bearer '+self.token}),timeout=5) as r:
    raw=r.read(2_000_001)
   if len(raw)>2_000_000:raise ValueError('Homebridge response exceeds limit')
   return json.loads(raw)
  except urllib.error.HTTPError as e:
   e.close()
   if e.code in (401,403):
    self.token=None;self.auth_error='required';raise AuthenticationError('required') from None
   raise
 def refresh(self):
  try:
   raw=self.request('/api/accessories')
   if not isinstance(raw,list) or len(raw)>2048:raise ValueError('Invalid catalogue')
   items=[v for s in raw if isinstance(s,dict) and (v:=describe(s))]
   self.items=sorted(items,key=lambda i:(i['kind'],i['name'].casefold(),i['id']))[:256]
   self.updated=time.monotonic()
  except Exception:self.updated=0;raise
 def one(self,identity):
  raw=self.request('/api/accessories/'+quote(identity,safe=''))
  if raw.get('uniqueId')!=identity:raise ValueError('Accessory identity changed')
  item=describe(raw)
  if not item:raise ValueError('Accessory no longer supported')
  return item,raw
 def snapshot(self):
  return {'available':bool(self.updated and time.monotonic()-self.updated<12),'items':self.items}
 def states(self):
  if not self.snapshot()['available']:return {}
  return {i['id']:{'text':caption(i),'on':next((bool(r['value']) for r in i['readings'] if r['type'] in ('On','Active')),True), 'level':next((r['value'] for r in i['readings'] if r['type']==i['level']),None)} for i in self.items}
