import base64,importlib.util,io,json,sys,tarfile,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from package_io import digest,write_archive,read_archive,release_files,atomic
spec=importlib.util.spec_from_file_location('installer',Path(__file__).with_name('decksmith-install.py'));manager=importlib.util.module_from_spec(spec);sys.modules[spec.name]=manager;spec.loader.exec_module(manager)
ROOT=Path(__file__).resolve().parents[1]
class InstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='Decksmith staged % ');self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name);self.paths=manager.Paths.create(self.root/'home')
        self.guard=patch.object(manager,'validate_install',side_effect=lambda p,r:manager.integrations(p,r));self.guard.start();self.addCleanup(self.guard.stop)
    def bundle(self,identity):
        files={'bin/decksmithd':b'#!/bin/sh\nexit 0\n','bin/decksmithctl':b'#!/bin/sh\nexit 0\n','apps/decksmith-studio/panel.py':b'# fixture','scripts/decksmith-install.py':b'# fixture','scripts/package_io.py':b'# fixture','config/audio.json':b'{}'}
        for name in ('decksmith.service.in','cc.senecal.Decksmith.Studio.desktop.in'):files['packaging/'+name]=(ROOT/'packaging'/name).read_bytes()
        manifest={'format':'decksmith-release','version':1,'id':identity,'architecture':__import__('platform').machine(),'files':{n:digest(d) for n,d in files.items()}}
        files['release.json']=json.dumps(manifest).encode();path=self.root/(identity+'.tar.gz');write_archive(path,files);return path
    def test_install_update_rollback_and_removal_preserve_data_and_do_not_manage_host_service(self):
        config=self.paths.config/'decksmith/layout.json';atomic(config,b'{"saved":"user"}')
        with patch.object(manager,'run',side_effect=AssertionError('Host command forbidden')):
            with patch.object(manager,'validate_saved'):
                first=manager.install(self.paths,self.bundle('one'));self.assertTrue(Path(first['backup']).is_file());self.assertEqual(config.read_bytes(),b'{"saved":"user"}')
                manager.install(self.paths,self.bundle('two'));self.assertEqual(manager.current(self.paths),'two')
                manager.rollback(self.paths);self.assertEqual(manager.current(self.paths),'one')
                manager.uninstall(self.paths);self.assertTrue(config.exists());self.assertTrue((self.paths.app/'releases/two').is_dir());self.assertFalse(self.paths.integrations()['desktop'].exists())
    def test_original_integration_restored_exactly(self):
        desktop=self.paths.integrations()['desktop'];original=b'[Desktop Entry]\nName=Decksmith\nExec=/old/decksmith\n';atomic(desktop,original)
        manager.install(self.paths,self.bundle('one'));manager.rollback(self.paths,True);self.assertEqual(desktop.read_bytes(),original);self.assertFalse(self.paths.integrations()['unit'].exists())
    def test_tampering_and_customized_integration_are_not_overwritten(self):
        bundle=self.bundle('one');files=read_archive(bundle);files['bin/decksmithd']=b'tampered';write_archive(bundle,files)
        with self.assertRaises(ValueError):manager.install(self.paths,bundle)
        manager.install(self.paths,self.bundle('two'));launcher=self.paths.integrations()['launcher'];launcher.write_text('custom change')
        with self.assertRaises(ValueError):manager.install(self.paths,self.bundle('three'))
        self.assertEqual(launcher.read_text(),'custom change');self.assertEqual(manager.current(self.paths),'two')
    def test_archives_reject_traversal_and_links(self):
        for name,kind in [('../escape',tarfile.REGTYPE),('escape',tarfile.SYMTYPE)]:
            path=self.root/'bad.tar.gz'
            with tarfile.open(path,'w:gz') as tar:
                info=tarfile.TarInfo(name);info.type=kind;info.linkname='/tmp/escape';tar.addfile(info,io.BytesIO())
            with self.assertRaises(ValueError):read_archive(path)
    def test_backup_restore_merges_without_deleting_newer_files(self):
        config=self.paths.config/'decksmith';icons=self.paths.data/'decksmith/icons';atomic(config/'a.json',b'first');atomic(icons/'icon.png',b'image')
        backup=manager.backup(self.paths);atomic(config/'a.json',b'changed');atomic(config/'new.json',b'keep')
        manager.restore(self.paths,backup);self.assertEqual((config/'a.json').read_bytes(),b'first');self.assertEqual((config/'new.json').read_bytes(),b'keep');self.assertEqual((icons/'icon.png').read_bytes(),b'image')
    def test_failed_restore_restores_previous_files(self):
        config=self.paths.config/'decksmith';atomic(config/'a.json',b'old');atomic(config/'b.json',b'old');archive=manager.backup(self.paths)
        atomic(config/'a.json',b'current');atomic(config/'b.json',b'current');failed=False
        def fail_once(path,data,*args):
            nonlocal failed
            if Path(path)==config/'b.json' and data==b'old' and not failed:failed=True;raise OSError('simulated disk error')
            return atomic(path,data,*args)
        with patch.object(manager,'atomic',side_effect=fail_once):
            with self.assertRaises(OSError):manager.restore(self.paths,archive)
        self.assertEqual((config/'a.json').read_bytes(),b'current');self.assertEqual((config/'b.json').read_bytes(),b'current')
    def test_corrupt_backup_and_invalid_saved_layout_do_not_write_configuration(self):
        manager.install(self.paths,self.bundle('one'));config=self.paths.config/'decksmith/layout.json';atomic(config,b'current');archive=manager.backup(self.paths)
        with patch.object(manager,'run',side_effect=__import__('subprocess').CalledProcessError(1,['validate'])):
            with self.assertRaises(__import__('subprocess').CalledProcessError):manager.restore(self.paths,archive)
        self.assertEqual(config.read_bytes(),b'current')
        files=read_archive(archive);files['config/layout.json']=b'corrupt';write_archive(archive,files)
        with self.assertRaises(ValueError):manager.restore(self.paths,archive)
        self.assertEqual(config.read_bytes(),b'current')

    def test_release_rejects_parent_identity_and_missing_empty_file(self):
        archive=self.bundle('safe');files=read_archive(archive);manifest=json.loads(files['release.json'])
        manifest['id']='..';files['release.json']=json.dumps(manifest).encode();write_archive(archive,files)
        with self.assertRaises(ValueError):release_files(archive)
        manifest['id']='safe';manifest['files']['missing']=digest(b'');files['release.json']=json.dumps(manifest).encode();write_archive(archive,files)
        with self.assertRaises(ValueError):release_files(archive)
    def test_failed_activation_stops_new_service_before_restoring_old(self):
        manager.install(self.paths,self.bundle('old'))
        with patch.object(manager,'active',return_value=True), patch.object(manager,'wait_ready',side_effect=ValueError('Not ready')), patch.object(manager,'service') as service:
            with self.assertRaises(ValueError):manager.install(self.paths,self.bundle('new'),True)
            actions=[call.args[1] for call in service.call_args_list]
            self.assertEqual(actions,['stop','reset-failed','start','stop','start'])
        self.assertEqual(manager.current(self.paths),'old')
        self.assertEqual(manager.load_record(self.paths)['current'],'old')
