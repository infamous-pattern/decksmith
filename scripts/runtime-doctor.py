#!/usr/bin/env python3
"""Read-only runtime/dependency checks; no hardware writes or service changes."""
import argparse,ctypes,json,os,platform,shutil,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
def check(root=ROOT,verify=True):
    root=Path(root);results=[]
    def result(name,ok,detail):results.append({'check':name,'ok':bool(ok),'detail':detail})
    result('Linux',sys.platform=='linux',platform.platform())
    for executable,package in [('python3','python3'),('systemctl','systemd'),('gdbus','glib2'),('pactl','pulseaudio-utils'),('wpctl','wireplumber')]:result(executable,shutil.which(executable),package)
    try:
        import gi
        gi.require_version('Gtk','4.0');gi.require_version('Adw','1');gi.require_version('Rsvg','2.0')
        from gi.repository import Gtk,Adw,Gio,Rsvg
        import cairo
        from PIL import Image
        result('GTK/Python artwork libraries',hasattr(Gtk,'FileDialog') and hasattr(Adw,'Dialog'),'gtk4, libadwaita, python3-gobject, python3-pillow, python3-cairo, librsvg2')
    except (ImportError,ValueError):result('GTK/Python artwork libraries',False,'Install gtk4 libadwaita python3-gobject python3-pillow python3-cairo librsvg2')
    for library,package in [('libpulse.so.0','pulseaudio-libs'),('libudev.so.1','systemd-libs')]:
        try:ctypes.CDLL(library);result(library,True,package)
        except OSError:result(library,False,'Install '+package)
    if verify:
        try:
            from package_io import release_files
            manifest,_=release_files(root);result('Release integrity',True,manifest['id']);result('Architecture',manifest['architecture']==platform.machine(),manifest['architecture'])
        except (OSError,ValueError) as e:result('Release integrity',False,str(e))
    for args,name in [(['--self-check'],'Runtime resources'),(['--virtual-once'],'VirtualDeck')]:
        try:
            done=subprocess.run([str(root/'bin/decksmithd'),*args],capture_output=True,text=True,timeout=10)
            result(name,done.returncode==0,done.stdout.strip() if done.returncode==0 else done.stderr.strip()[-500:])
        except (OSError,subprocess.SubprocessError) as e:result(name,False,str(e))
    devices=[]
    try:devices=json.loads(subprocess.check_output([str(root/'bin/decksmithctl'),'devices'],text=True,timeout=5))
    except (OSError,ValueError,subprocess.SubprocessError):pass
    return {'ready':all(r['ok'] for r in results),'checks':results,'devices':devices,'device_note':'No connected device is required to install. For access problems, see the bundled Plus udev rule; no rules were changed.'}
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--root',type=Path,default=ROOT);args=p.parse_args();report=check(args.root);print(json.dumps(report,indent=2));sys.exit(0 if report['ready'] else 1)
