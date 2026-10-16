"""Device-addressed clients. Endpoint selection is explicit; no Control1/file fallback."""
import json
from threading import RLock
BUS='cc.senecal.Decksmith.ProfileLab'
PATH='/cc/senecal/Decksmith/ProfileLab'
IFACE='cc.senecal.Decksmith.ProfileLab2'
WORKSPACE_BUS='cc.senecal.Decksmith.DeviceWorkspace'
WORKSPACE_PATH='/cc/senecal/Decksmith/DeviceWorkspace'
WORKSPACE_IFACE='cc.senecal.Decksmith.DeviceWorkspace2'
CONNECTED_BUS='cc.senecal.Decksmith.ConnectedWorkspace'
CONNECTED_PATH='/cc/senecal/Decksmith/ConnectedWorkspace'

class DeviceClient:
    device_scoped=True
    actions_available=False
    def __init__(self,device,transport):
        self.id=device['id'];self.device_model=device['model'];self.name=device['name'];self.brightness=device['brightness']
        self.transport=transport;self.token=None;self.lock=RLock();self.workspace_only=getattr(transport,'workspace_only',False);self.connected_workspace=getattr(transport,'connected_workspace',False);self.permanent=getattr(transport,'permanent',False);self.actions_available=self.permanent
    def accept(self,response):
        value=json.loads(response[0])
        if value.get('id')!=self.id:raise ValueError('Wrong device reply')
        self.token=value['token'];self.name=value.get('name',self.name);self.brightness=value.get('brightness',self.brightness);return value
    def devices(self):return json.loads(self.transport('ListDevices')[0])
    def __call__(self,method,args=None):
        from gi.repository import GLib
        values=args.unpack() if args is not None else ()
        with self.lock:
            if method=='GetLayout':
                value=self.accept(self.transport('GetProfile',GLib.Variant('(s)',(self.id,))))
                return (json.dumps(value['layout'],ensure_ascii=False),)
            if method=='GetStatus':
                catalog=self.devices();status=next(s for s in catalog['sessions'] if s['id']==self.id)
                status=dict(status,layout='profile:'+self.id,active_page=status['page'],display_ready=status.get('display_ready',status.get('input_ready',status['connected'])),running=not self.workspace_only,workspace_only=self.workspace_only,detected=any(c.get('association')=={'state':'assigned','value':self.id} for c in catalog.get('discovery',{}).get('candidates',[])),api_version=2)
                if self.permanent:
                    status['mode']='permanent-service'
                    shared=json.loads(control_transport('GetStatus')[0])
                    for key in ('auto_lock','automatic_pages'):status[key]=shared.get(key,{})
                return (json.dumps(status),)
            if method in ('PreviewKeys','PreviewTouch','ValidateLayout'):
                raw=values[0];page=values[1] if len(values)>1 else 0
                reply=self.transport('Preview',GLib.Variant('(ssyb)',(self.id,raw,page,method=='PreviewTouch')))
                if method=='ValidateLayout':return (raw,)
                return (reply,'saved-workspace' if self.workspace_only else 'virtual-lab') if method=='PreviewTouch' else reply
            if method=='GetPreviousLayout':return self.transport('GetPreviousProfile',GLib.Variant('(s)',(self.id,)))
            if method=='TestControl':
                if not self.permanent:raise ValueError('Live test actions are unavailable')
                if self.token is None:raise ValueError('Reload this device before testing')
                token=json.dumps(self.token,separators=(',',':'))
                self.transport('TestControl',GLib.Variant('(sssyy)',(self.id,token,values[0],values[1],values[2])));return ()
            if method in ('SaveLayout','ShowPage'):
                if self.token is None:raise ValueError('Reload this device before saving')
                token=json.dumps(self.token,separators=(',',':'))
                if method=='SaveLayout':
                    # Editor's next ShowPage selects the draft page. Persist it
                    # in the same save using the explicit current-page getter.
                    page=self.page() if hasattr(self,'page') else 0
                    reply=self.transport('SaveProfile',GLib.Variant('(sssysy)',(self.id,token,values[0],page,self.name,self.brightness)))
                else:
                    reply=self.transport('ShowPage',GLib.Variant('(ssy)',(self.id,token,values[0])))
                self.accept(reply);return ()
            raise ValueError('This control is unavailable for this device profile.')

def _transport(bus,path,interface,method,args=None):
    from gi.repository import Gio
    connection=Gio.bus_get_sync(Gio.BusType.SESSION,None)
    reply=connection.call_sync(bus,path,interface,method,args,None,Gio.DBusCallFlags.NO_AUTO_START,7000,None)
    if method=='Preview':return reply.get_child_value(0).get_data_as_bytes().get_data()
    return reply.unpack()


def transport(method,args=None):return _transport(BUS,PATH,IFACE,method,args)
def workspace_transport(method,args=None):return _transport(WORKSPACE_BUS,WORKSPACE_PATH,WORKSPACE_IFACE,method,args)
workspace_transport.workspace_only=True
def connected_transport(method,args=None):return _transport(CONNECTED_BUS,CONNECTED_PATH,WORKSPACE_IFACE,method,args)
connected_transport.connected_workspace=True


def permanent_transport(method,args=None):return _transport(CONNECTED_BUS,CONNECTED_PATH,WORKSPACE_IFACE,method,args)
permanent_transport.connected_workspace=True
permanent_transport.permanent=True

def control_transport(method,args=None):return _transport('cc.senecal.Decksmith','/cc/senecal/Decksmith','cc.senecal.Decksmith.Control1',method,args)

def permanent_available():
    """Read-only discovery; never adopts a lab or activates another service."""
    try:
        value=json.loads(permanent_transport('ListDevices')[0])
        return value.get('mode')=='permanent-service' and value.get('api_version')==2
    except Exception:return False
