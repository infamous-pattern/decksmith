"""Explicit device-addressed lab client. Never falls back to Control1 or files."""
import json
from threading import RLock
BUS='cc.senecal.Decksmith.ProfileLab'
PATH='/cc/senecal/Decksmith/ProfileLab'
IFACE='cc.senecal.Decksmith.ProfileLab2'

class DeviceClient:
    device_scoped=True
    def __init__(self,device,transport):
        self.id=device['id'];self.device_model=device['model'];self.name=device['name'];self.brightness=device['brightness']
        self.transport=transport;self.token=None;self.lock=RLock()
    def accept(self,response):
        value=json.loads(response[0])
        if value.get('id')!=self.id:raise ValueError('Wrong device reply')
        self.token=value['token'];return value
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
                return (json.dumps(dict(status,layout='profile:'+self.id,active_page=status['page'],display_ready=status['connected'],running=True,api_version=2)),)
            if method in ('PreviewKeys','PreviewTouch','ValidateLayout'):
                raw=values[0];page=values[1] if len(values)>1 else 0
                reply=self.transport('Preview',GLib.Variant('(ssyb)',(self.id,raw,page,method=='PreviewTouch')))
                if method=='ValidateLayout':return (raw,)
                return (reply,'virtual-lab') if method=='PreviewTouch' else reply
            if method in ('SaveLayout','ShowPage'):
                if self.token is None:raise ValueError('Reload this device before saving')
                token=json.dumps(self.token,separators=(',',':'))
                if method=='SaveLayout':
                    # Editor's next ShowPage selects the draft page. Persist it
                    # in the same save using the explicit current-page getter.
                    page=self.page() if hasattr(self,'page') else 0
                    reply=self.transport('SaveProfile',GLib.Variant('(sssysy)',(self.id,token,values[0],page,self.name,self.brightness)))
                else:reply=self.transport('ShowPage',GLib.Variant('(ssy)',(self.id,token,values[0])))
                self.accept(reply);return ()
            raise ValueError('This control is unavailable in the isolated profile lab.')

def transport(method,args=None):
    from gi.repository import Gio
    connection=Gio.bus_get_sync(Gio.BusType.SESSION,None)
    reply=connection.call_sync(BUS,PATH,IFACE,method,args,None,Gio.DBusCallFlags.NO_AUTO_START,7000,None)
    if method=='Preview':return reply.get_child_value(0).get_data_as_bytes().get_data()
    return reply.unpack()
