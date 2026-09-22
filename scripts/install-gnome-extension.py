#!/usr/bin/env python3
"""Install Decksmith's optional GNOME menu for this user; never restart the desktop."""
import argparse,subprocess,tempfile,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
UUID='decksmith@senecal.cc'
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--enable',action='store_true',help='Ask GNOME to enable the installed extension')
    args=parser.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        archive=Path(tmp)/(UUID+'.shell-extension.zip')
        with zipfile.ZipFile(archive,'w') as z:
            for name in ('metadata.json','extension.js','foreground.js','decksmith-symbolic.svg'):z.write(ROOT/'extensions/decksmith-gnome'/name,name)
        subprocess.run(['gnome-extensions','install','--force',str(archive)],check=True,timeout=15)
    print('Decksmith GNOME menu installed. It does not start or stop background controls.')
    if args.enable:
        result=subprocess.run(['gnome-extensions','enable',UUID],capture_output=True,text=True,timeout=10)
        if result.returncode:
            print('GNOME needs a new login before it can discover this extension. After logging in, enable Decksmith in Extensions.')
            return 2
        print('Enable request sent. Check Decksmith in Extensions for its current state.')
    else:print('Enable Decksmith in Extensions; a new login may be needed for discovery.')
    return 0
if __name__=='__main__':raise SystemExit(main())
