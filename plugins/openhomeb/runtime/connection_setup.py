"""Read-only candidate testing and private account configuration."""
import asyncio
import secrets
import time
from urllib.parse import urlsplit,urlunsplit
from accessory_catalog import Catalog,AuthenticationError
from credential_store import SecretToolStore,CredentialStoreError
from connection_preferences import write

def server_url(value):
 if not isinstance(value,str) or len(value)>512:raise ValueError('Enter a valid server address.')
 p=urlsplit(value.strip())
 if p.scheme not in ('http','https') or not p.hostname or p.username or p.password or p.query or p.fragment or p.path not in ('','/') or any(c.isspace() for c in value):raise ValueError('Use a complete http:// or https:// server address without credentials or a path.')
 if p.port is not None and not 1<=p.port<=65535:raise ValueError('Invalid port.')
 return urlunsplit((p.scheme,p.netloc.lower(),'','',''))

class Setup:
 def __init__(self,panel):self.panel=panel;self.store=SecretToolStore();self.candidate=None;self.result={'state':'idle','message':''}
 def expire(self):
  if self.candidate and time.monotonic()>self.candidate['expires']:
   self.candidate=None;self.result={'state':'expired','message':'Connection test expired. Test again before saving.'}
 async def test(self,data):
  self.candidate=None
  try:
   if set(data)!={'operation','server','username','password','otp'}:raise ValueError('Invalid connection fields.')
   server=server_url(data['server']);username=data['username'];password=data['password'];otp=data['otp']
   if not isinstance(username,str) or len(username)>128 or any(c in username for c in '\r\n\x00'):raise ValueError('Invalid username.')
   if not isinstance(password,str) or len(password)>4096 or any(c in password for c in '\r\n\x00'):raise ValueError('Invalid password.')
   if not isinstance(otp,str) or (otp and (not otp.isascii() or not otp.isdigit() or not 6<=len(otp)<=8)):raise ValueError('Enter the current two-factor code.')
   config=self.panel.connection_config
   if username and not password and server==config.get('server') and username==config.get('username') and config.get('credential_profile'):
    password=await self.store.lookup(config['credential_profile']) or ''
   catalog=Catalog(server,username,password,otp or None)
   await asyncio.to_thread(catalog.refresh)
   if catalog.auth_method=='none':catalog.username='';catalog.password=''
   identity=secrets.token_hex(16)
   self.candidate={'id':identity,'catalog':catalog,'expires':time.monotonic()+120}
   self.result={'state':'tested','message':('Connection verified. Save Connection to use it.' if catalog.auth_method=='password' else 'Connection verified; this server does not require sign-in. Save Connection to use it.'),'candidate':identity,'server':server,'username':catalog.username,'functions':len(catalog.items)}
  except AuthenticationError as e:
   self.result={'state':e.state,'message':{'required':'Enter your Homebridge username and password.','rejected':'Homebridge rejected the sign-in. Check your username and password.','two_factor':'Enter your current two-factor code, then test again.'}[e.state]}
  except CredentialStoreError:self.result={'state':'keyring','message':'Unlock GNOME Keyring and try again. No password was saved to a file.'}
  except ValueError:self.result={'state':'invalid','message':'Check the server address, username and code. Use an http:// or https:// address without a path.'}
  except Exception:self.result={'state':'offline','message':'Could not verify Homebridge. Check its address, network connection and certificate.'}
 async def apply(self,data):
  self.expire();candidate=self.candidate
  if set(data)!={'operation','candidate'} or not candidate or data['candidate']!=candidate['id']:
   self.result={'state':'expired','message':'Test the connection again before saving.'};return
  panel=self.panel;catalog=candidate['catalog'];profile=None
  old_enabled=panel.enabled;old_config=dict(panel.connection_config);old_catalog=panel.catalog;write_attempted=False;switched=False
  try:
   if catalog.username and catalog.password:
    profile=secrets.token_hex(16);await self.store.store(profile,catalog.password)
   config={'enabled':True if old_config.get('removed') else panel.enabled,'server':catalog.base,'username':catalog.username}
   if profile:config['credential_profile']=profile
   # Keep the prior config and credential until the new connection is established.
   switched=True
   await panel.host.stop();panel.catalog=catalog;panel.connection_config=config;panel.enabled=config['enabled']
   if panel.enabled:await panel.connect(use_session=True)
   write_attempted=True
   write(panel.preference_path,config)
   panel.assigned_states=catalog.states();panel.last_error=None
   self.result={'state':'saved','message':'Connection saved. Existing assignments are preserved.'};self.candidate=None
   # Retain the previous keyring entry so configuration backups remain usable.
  except Exception:
   if switched:
    await panel.host.stop();panel.catalog=old_catalog;panel.connection_config=old_config;panel.enabled=old_enabled
    if panel.enabled:
     try:await panel.connect(use_session=True)
     except Exception:panel.recovery_required=True
   if profile:
    try:await self.store.delete(profile)
    except Exception:pass
   restored=True
   if write_attempted:
    try:write(panel.preference_path,old_config)
    except Exception:restored=False
   self.result={'state':'failed','message':('Connection was not saved. The previous configuration was retained. Check the connection and GNOME Keyring.' if restored else 'Saving and restoring the configuration failed. Review Connection setup before restarting.')}
 async def remove(self,data):
  if set(data)!={'operation','confirm','delete_password'} or data['confirm'] is not True or type(data['delete_password']) is not bool:
   self.result={'state':'failed','message':'Confirm removal before continuing.'};return
  panel=self.panel;old=dict(panel.connection_config)
  config={'enabled':False,'removed':True,'server':'','username':''}
  profile=old.get('credential_profile')
  # Retain the opaque reference until optional deletion succeeds, so it can be retried.
  if profile:config['credential_profile']=profile
  try:write(panel.preference_path,config)
  except Exception:
   self.result={'state':'failed','message':'Connection could not be removed. The current connection was retained.'};return
  panel.enabled=False;panel.connection_config=config;self.candidate=None
  panel.assigned_states={};panel.actual=None;panel.updated=0;panel.read=None
  panel.catalog=Catalog('');panel.last_error=None
  try:await panel.host.stop()
  except Exception:
   self.result={'state':'failed','message':'Connection disabled and removed from settings, but its process did not stop. Restart Background controls.'};return
  panel.host.saved['global']={};panel.host.access_session=None
  panel.manual_recovery=False;panel.recovery_required=False
  if data['delete_password'] and profile:
   try:
    await self.store.delete(profile)
    config=dict(config);config.pop('credential_profile',None)
    write(panel.preference_path,config);panel.connection_config=config
   except Exception:
    self.result={'state':'removed','message':'Connection removed. Saved password cleanup could not be confirmed; unlock GNOME Keyring and retry Remove Connection with password deletion selected.'};return
  self.result={'state':'removed','message':('Connection removed. Assignments are preserved. Saved password deleted.' if data['delete_password'] and profile else 'Connection removed. Assignments are preserved.' + (' Saved password retained in GNOME Keyring.' if profile else ''))}
 async def perform(self,data):
  try:
   if data.get('operation')=='test':await self.test(data)
   elif data.get('operation')=='apply':await self.apply(data)
   elif data.get('operation')=='remove':await self.remove(data)
  finally:self.panel.busy=False
