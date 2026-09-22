#!/usr/bin/env python3
"""Build a versioned Fedora development runtime; never installs or changes services."""
import argparse,json,platform,shutil,subprocess,tempfile
from pathlib import Path
from package_io import digest,tree_files,write_archive,atomic
from localization import compiled_catalogs
ROOT=Path(__file__).resolve().parents[1]
def build(output,binaries=None):
    if binaries is None:
        subprocess.run(['cargo','build','--release','--locked','-p','decksmithd','-p','decksmithctl','--features','decksmithd/hardware,decksmithctl/hardware'],cwd=ROOT,check=True)
        binaries=ROOT/'target/release'
    files=compiled_catalogs(ROOT)
    with tempfile.TemporaryDirectory() as temporary:
        for name in ('decksmithd','decksmithctl'):
            path=Path(temporary)/name;shutil.copy2(Path(binaries)/name,path);subprocess.run(['strip',str(path)],check=True)
            files['bin/'+name]=path.read_bytes()
    for folder in ('apps/decksmith-studio','scripts','assets','brand','config','packaging','docs','extensions','plugins'):
        for name,data in tree_files(ROOT/folder).items():
            p=Path(name)
            if '__pycache__' in p.parts or p.name.startswith(('test_','check_')) or p.suffix=='.pyc':continue
            files[folder+'/'+name]=data
    for name in ('README.md','LICENSE'):files[name]=(ROOT/name).read_bytes()
    source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip())
    hashes={name:digest(data) for name,data in files.items()}
    identity='0.1.0-'+digest(json.dumps({'files':hashes,'source':source},sort_keys=True).encode())[:12]
    manifest={'format':'decksmith-release','version':1,'id':identity,'application_version':'0.1.0','source_commit':source,'source_dirty':dirty,'platform':'Linux','architecture':platform.machine(),'tested_distribution':'Fedora 44','files':hashes}
    files['release.json']=json.dumps(manifest,sort_keys=True,indent=2).encode()
    output=Path(output);output.mkdir(parents=True,exist_ok=True);destination=output/('decksmith-'+identity+'.tar.gz')
    write_archive(destination,files);atomic(destination.with_suffix(destination.suffix+'.sha256'),(digest(destination.read_bytes())+'  '+destination.name+'\n').encode())
    for name in ('decksmith-install.py','package_io.py'):
        atomic(output/name,(ROOT/'scripts'/name).read_bytes())
    atomic(output/'INSTALL.md',(ROOT/'docs/installation.md').read_bytes())
    print(destination);return destination
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',default=str(ROOT/'dist'));parser.add_argument('--binaries',type=Path,help='Use existing hardware-enabled binaries instead of building')
    args=parser.parse_args();build(args.output,args.binaries)
