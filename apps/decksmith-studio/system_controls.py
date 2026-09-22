"""Allowlisted GNOME system controls. Persistent JSON-lines worker; no shell execution."""
import json,sys,os
from pathlib import Path
from gi.repository import Gio,GLib
COMMANDS={'lock','suspend','reboot','shutdown','dnd','night_light','bluetooth','power_cycle','power_saver','power_balanced','power_performance'}
LABELS={'lock':'Lock Desktop','suspend':'Suspend','reboot':'Reboot','shutdown':'Shutdown','dnd':'Do Not Disturb','night_light':'Night Light','bluetooth':'Bluetooth','power_cycle':'Power Mode','power_saver':'Power Saver','power_balanced':'Balanced','power_performance':'Performance'}
class Controls:
    def __init__(self):self.buses={};self.settings={}
    def call(self,system,name,path,iface,method,args=None,timeout=1500):
        if system not in self.buses:self.buses[system]=Gio.bus_get_sync(Gio.BusType.SYSTEM if system else Gio.BusType.SESSION,None)
        bus=self.buses[system]
        return bus.call_sync(name,path,iface,method,args,None,Gio.DBusCallFlags.NO_AUTO_START,timeout,None).unpack()
    def prop(self,system,name,path,iface,key):
        return self.call(system,name,path,'org.freedesktop.DBus.Properties','Get',GLib.Variant('(ss)',(iface,key)))[0]
    def put(self,system,name,path,iface,key,value):
        self.call(system,name,path,'org.freedesktop.DBus.Properties','Set',GLib.Variant('(ssv)',(iface,key,value)))
    def setting(self,schema):
        if schema not in self.settings:self.settings[schema]=Gio.Settings.new(schema)
        return self.settings[schema]
    def color(self,key):return self.prop(False,'org.gnome.SettingsDaemon.Color','/org/gnome/SettingsDaemon/Color','org.gnome.SettingsDaemon.Color',key)
    def rfkill(self,key):return self.prop(False,'org.gnome.SettingsDaemon.Rfkill','/org/gnome/SettingsDaemon/Rfkill','org.gnome.SettingsDaemon.Rfkill',key)
    def profiles(self):
        name='net.hadess.PowerProfiles';path='/net/hadess/PowerProfiles'
        return self.prop(True,name,path,name,'Profiles'),self.prop(True,name,path,name,'ActiveProfile')
    def state(self,command):
        if command not in COMMANDS:raise ValueError('Unsupported system action')
        if command=='dnd':
            s=self.setting('org.gnome.desktop.notifications')
            if not s.is_writable('show-banners'):raise ValueError('Notification setting is managed')
            active=not s.get_boolean('show-banners');return dict(available=True,active=active,text='On' if active else 'Off')
        if command=='night_light':
            active=self.color('NightLightActive') and not self.color('DisabledUntilTomorrow')
            return dict(available=True,active=active,text='On' if active else 'Off')
        if command=='bluetooth':
            available=self.rfkill('BluetoothHasAirplaneMode') and not self.rfkill('BluetoothHardwareAirplaneMode')
            active=not self.rfkill('BluetoothAirplaneMode')
            return dict(available=available,active=active,text=('On' if active else 'Off') if available else 'Unavailable')
        if command.startswith('power_'):
            profiles,active=self.profiles();desired=command.removeprefix('power_').replace('_','-')
            if desired=='saver':desired='power-saver'
            available=command=='power_cycle' or desired in [p['Profile'] for p in profiles]
            return dict(available=available,active=active==desired or command=='power_cycle',text={'power-saver':'Saver','balanced':'Balanced','performance':'Performance'}.get(active,active))
        if command in ('suspend','reboot','shutdown'):
            cap={'suspend':'CanSuspend','reboot':'CanReboot','shutdown':'CanPowerOff'}[command]
            value=self.call(True,'org.freedesktop.login1','/org/freedesktop/login1','org.freedesktop.login1.Manager',cap)[0]
            return dict(available=value in ('yes','challenge'),active=False,text='Confirm' if command in ('reboot','shutdown') else 'Ready')
        self.call(False,'org.gnome.ScreenSaver','/org/gnome/ScreenSaver','org.gnome.ScreenSaver','GetActive')
        return dict(available=True,active=False,text='Ready')
    def execute(self,command):
        if command not in COMMANDS:raise ValueError('Unsupported system action')
        state=self.state(command)
        if not state['available']:raise ValueError('Control unavailable')
        if command=='lock':self.call(False,'org.gnome.ScreenSaver','/org/gnome/ScreenSaver','org.gnome.ScreenSaver','Lock')
        elif command=='suspend':self.call(True,'org.freedesktop.login1','/org/freedesktop/login1','org.freedesktop.login1.Manager','Suspend',GLib.Variant('(b)',(True,)))
        elif command in ('reboot','shutdown'):
            import subprocess
            subprocess.Popen(['/usr/bin/python3','-B',str(Path(__file__).with_name('system_confirm.py')),command],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        elif command=='dnd':
            if not self.setting('org.gnome.desktop.notifications').set_boolean('show-banners',state['active']):raise ValueError('Setting denied')
        elif command=='bluetooth':self.put(False,'org.gnome.SettingsDaemon.Rfkill','/org/gnome/SettingsDaemon/Rfkill','org.gnome.SettingsDaemon.Rfkill','BluetoothAirplaneMode',GLib.Variant('b',state['active']))
        elif command=='night_light':self.night_light(not state['active'])
        else:
            profiles,current=self.profiles();names=[p['Profile'] for p in profiles]
            desired=command.removeprefix('power_');desired='power-saver' if desired=='saver' else desired
            if desired=='cycle':desired=names[(names.index(current)+1)%len(names)]
            if desired not in names:raise ValueError('Profile unavailable')
            self.put(True,'net.hadess.PowerProfiles','/net/hadess/PowerProfiles','net.hadess.PowerProfiles','ActiveProfile',GLib.Variant('s',desired))
        Gio.Settings.sync()
    def night_light(self,on):
        s=self.setting('org.gnome.settings-daemon.plugins.color')
        keys=('night-light-schedule-automatic','night-light-schedule-from','night-light-schedule-to')
        path=Path(os.environ.get('XDG_CONFIG_HOME',Path.home()/'.config'))/'decksmith/night-light-schedule.json'
        if on:
            if not path.exists():
                path.parent.mkdir(parents=True,exist_ok=True);temp=path.with_suffix('.tmp')
                temp.write_text(json.dumps({k:s.get_value(k).unpack() for k in keys}));temp.replace(path)
            s.delay();s.set_boolean(keys[0],False);s.set_double(keys[1],0.0);s.set_double(keys[2],24.0);s.set_boolean('night-light-enabled',True);s.apply()
            self.put(False,'org.gnome.SettingsDaemon.Color','/org/gnome/SettingsDaemon/Color','org.gnome.SettingsDaemon.Color','DisabledUntilTomorrow',GLib.Variant('b',False))
        else:
            s.delay();s.set_boolean('night-light-enabled',False)
            if path.exists() and not s.get_boolean(keys[0]) and s.get_double(keys[1])==0.0 and s.get_double(keys[2])==24.0:
                # Do not overwrite schedule edits made in GNOME while the override was on.
                data=json.loads(path.read_text())
                for key in keys:s.set_value(key,GLib.Variant('b' if key==keys[0] else 'd',data[key]))
            s.apply()
            if path.exists():path.unlink()

def main():
    controls=Controls()
    for line in sys.stdin:
        context=GLib.MainContext.default()
        while context.pending():context.iteration(False)
        try:
            request=json.loads(line)
            if 'execute' in request:controls.execute(request['execute']);reply={'ok':True}
            else:
                result={}
                for command in request.get('read',[]):
                    try:result[command]=controls.state(command)
                    except Exception:result[command]=dict(available=False,active=False,text='Unavailable')
                reply={'ok':True,'states':result}
        except Exception:reply={'ok':False,'error':'system_action_failed'}
        print(json.dumps(reply),flush=True)
if __name__=='__main__':main()
