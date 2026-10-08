#!/usr/bin/env python3
"""Build a versioned native Linux runtime; never installs or changes services."""
import argparse,json,os,shlex,shutil,subprocess,tempfile
from pathlib import Path
from package_io import digest,tree_files,write_archive,atomic
from localization import compiled_catalogs
from project_version import application_version
from build_platform import bundle_platform
ROOT=Path(__file__).resolve().parents[1]
def build(output,binaries=None):
    version=application_version(ROOT)
    environment=bundle_platform()
    if binaries is None:
        env=dict(os.environ)
        flags=env['CARGO_ENCODED_RUSTFLAGS'].split('\x1f') if env.get('CARGO_ENCODED_RUSTFLAGS') else shlex.split(env.get('RUSTFLAGS',''))
        flags.extend([f'--remap-path-prefix={Path.home()}=/build/home',f'--remap-path-prefix={ROOT}=/build/decksmith'])
        env['CARGO_ENCODED_RUSTFLAGS']='\x1f'.join(flags)
        subprocess.run(['cargo','build','--release','--locked','-p','decksmithd','-p','decksmithctl','--features','decksmithd/hardware,decksmithctl/hardware'],cwd=ROOT,check=True,env=env)
        target=Path(env.get('CARGO_TARGET_DIR',ROOT/'target'))
        binaries=(target if target.is_absolute() else ROOT/target)/'release'
    # Reject binaries that cannot run on the named build environment, including
    # Fedora binaries copied to a Debian bundle. These checks do not open USB.
    for name,args in (('decksmithd',['--self-check']),('decksmithd',['--virtual-once']),('decksmithctl',['--help'])):
        subprocess.run([str(Path(binaries)/name),*args],check=True,capture_output=True,text=True,timeout=15)
    files=compiled_catalogs(ROOT)
    with tempfile.TemporaryDirectory() as temporary:
        for name in ('decksmithd','decksmithctl'):
            path=Path(temporary)/name;shutil.copy2(Path(binaries)/name,path);subprocess.run(['strip',str(path)],check=True)
            data=path.read_bytes()
            if str(Path.home()).encode() in data:raise ValueError('Binary contains the build user home path; rebuild with path remapping.')
            files['bin/'+name]=data
    for folder in ('apps/decksmith-studio','scripts','assets','brand','config','packaging','docs','extensions','plugins'):
        for name,data in tree_files(ROOT/folder).items():
            p=Path(name)
            if '__pycache__' in p.parts or p.name.startswith(('test_','check_')) or p.suffix=='.pyc':continue
            files[folder+'/'+name]=data
    for name in ('README.md','LICENSE'):files[name]=(ROOT/name).read_bytes()
    source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip())
    hashes={name:digest(data) for name,data in files.items()}
    identity=version+'-'+digest(json.dumps({'files':hashes,'source':source,'application_version':version,**environment},sort_keys=True).encode())[:12]
    manifest={'format':'decksmith-release','version':1,'id':identity,'application_version':version,'source_commit':source,'source_dirty':dirty,**environment,'files':hashes}
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
