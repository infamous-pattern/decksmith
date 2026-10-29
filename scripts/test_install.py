import base64,importlib.util,io,json,os,stat,sys,tarfile,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from package_io import digest,write_archive,read_archive,release_files,atomic,tree_files
spec=importlib.util.spec_from_file_location('installer',Path(__file__).with_name('decksmith-install.py'));manager=importlib.util.module_from_spec(spec);sys.modules[spec.name]=manager;spec.loader.exec_module(manager)
ROOT=Path(__file__).resolve().parents[1]
class InstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='Decksmith staged % ');self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name);self.paths=manager.Paths.create(self.root/'home')
        self.guard=patch.object(manager,'validate_install',side_effect=lambda p,r:manager.integrations(p,r));self.guard.start();self.addCleanup(self.guard.stop)
    def bundle(self,identity,include_custom_icon=True):
        files={'bin/decksmithd':b'#!/bin/sh\nexit 0\n','bin/decksmithctl':b'#!/bin/sh\nexit 0\n','apps/decksmith-studio/panel.py':b'# fixture','scripts/decksmith-install.py':b'# fixture','scripts/package_io.py':b'# fixture','config/audio.json':b'{}','brand/decksmith-app.svg':b'<svg/>'}
        if include_custom_icon:files['brand/custom-icon.svg']=b'<svg/>'
        for name in ('decksmith.service.in','cc.senecal.Decksmith.Studio.desktop.in'):files['packaging/'+name]=(ROOT/'packaging'/name).read_bytes()
        manifest={'format':'decksmith-release','version':1,'id':identity,'architecture':__import__('platform').machine(),'files':{n:digest(d) for n,d in files.items()}}
        files['release.json']=json.dumps(manifest).encode();path=self.root/(identity+'.tar.gz');write_archive(path,files);return path
    def test_directory_snapshots_enforce_total_limit_before_archive_creation(self):
        folder=self.root/'snapshot';folder.mkdir()
        (folder/'a').write_bytes(b'abcd');(folder/'b').write_bytes(b'ef')
        with patch('package_io.LIMIT',6):
            self.assertEqual(tree_files(folder),{'a':b'abcd','b':b'ef'})
            (folder/'b').write_bytes(b'efg')
            with self.assertRaisesRegex(ValueError,'too large'):tree_files(folder)
            self.assertEqual((folder/'a').read_bytes(),b'abcd')
            self.assertEqual((folder/'b').read_bytes(),b'efg')
    def test_directory_snapshot_uses_archive_file_count_limit(self):
        folder=self.root/'snapshot';folder.mkdir()
        for name in ('a','b','c'):(folder/name).write_bytes(b'')
        with patch('package_io.MAX_FILES',2):
            with self.assertRaisesRegex(ValueError,'too large'):tree_files(folder)
    def test_directory_snapshot_rejects_symlink_at_root(self):
        folder=self.root/'real';folder.mkdir();(folder/'data').write_bytes(b'preserve')
        link=self.root/'linked';link.symlink_to(folder, target_is_directory=True)
        with self.assertRaisesRegex(ValueError,'Links are not supported'):tree_files(link)
        self.assertEqual((folder/'data').read_bytes(),b'preserve')
        link.unlink();link.symlink_to(self.root/'missing', target_is_directory=True)
        with self.assertRaisesRegex(ValueError,'Links are not supported'):tree_files(link)
    def test_archive_writer_never_replaces_existing_archive_when_file_limit_is_exceeded(self):
        path=self.root/'backup.tar.gz';path.write_bytes(b'previous backup')
        with patch('package_io.MAX_FILES',2):
            with self.assertRaisesRegex(ValueError,'file count'):
                write_archive(path,{'a':b'','b':b'','c':b''})
            self.assertEqual(path.read_bytes(),b'previous backup')
            write_archive(path,{'a':b'','b':b''})
            self.assertEqual(read_archive(path),{'a':b'','b':b''})
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
            with self.assertRaisesRegex(ValueError,'backup layout.json could not be validated.*No settings were changed'):manager.restore(self.paths,archive)
        self.assertEqual(config.read_bytes(),b'current')
        files=read_archive(archive);files['config/layout.json']=b'corrupt';write_archive(archive,files)
        with self.assertRaises(ValueError):manager.restore(self.paths,archive)
        self.assertEqual(config.read_bytes(),b'current')

    def test_invalid_saved_layout_gives_recovery_guidance_and_preserves_data(self):
        config=self.paths.config/'decksmith/layout.json';atomic(config,b'invalid but preserved')
        with patch.object(manager,'run',side_effect=__import__('subprocess').CalledProcessError(1,['validate'])):
            with self.assertRaisesRegex(ValueError,'saved layout could not be validated.*No settings were changed.*verified compatible backup'):
                manager.validate_saved(self.paths,self.root/'release')
        self.assertEqual(config.read_bytes(),b'invalid but preserved')

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
    def test_failed_runtime_validation_does_not_persist_or_switch_release(self):
        def reject(_paths,_release):raise ValueError('runtime unavailable')
        with patch.object(manager,'validate_install',side_effect=reject):
            with self.assertRaisesRegex(ValueError,'runtime unavailable'):manager.install(self.paths,self.bundle('fresh'))
        self.assertFalse((self.paths.app/'releases/fresh').exists())
        self.assertIsNone(manager.current(self.paths))
        self.assertFalse(self.paths.record.exists())

        manager.install(self.paths,self.bundle('working'))
        with patch.object(manager,'validate_install',side_effect=reject):
            with self.assertRaisesRegex(ValueError,'runtime unavailable'):manager.install(self.paths,self.bundle('incompatible'))
        self.assertFalse((self.paths.app/'releases/incompatible').exists())
        self.assertEqual(manager.current(self.paths),'working')
        self.assertEqual(manager.load_record(self.paths)['current'],'working')
    def test_safe_custom_desktop_is_preserved_across_update_and_rollback(self):
        manager.install(self.paths,self.bundle('one'))
        desktop=self.paths.integrations()['desktop']
        current=str(self.paths.app/'current')
        custom=desktop.read_text().replace(current+'/brand/decksmith-app.svg',current+'/brand/custom-icon.svg').replace('Name=Decksmith','Name=Decksmith\nX-Decksmith-Preference=custom')
        desktop.write_text(custom)

        manager.install(self.paths,self.bundle('two',include_custom_icon=False))
        normalized=custom.replace(current+'/brand/custom-icon.svg',current+'/brand/decksmith-app.svg')
        self.assertEqual(desktop.read_text(),normalized)
        self.assertTrue(manager.load_record(self.paths)['preserved']['desktop'])
        self.assertEqual(manager.load_record(self.paths)['managed']['desktop'],digest(normalized.encode()))

        manager.rollback(self.paths)
        self.assertEqual(manager.current(self.paths),'one')
        self.assertEqual(desktop.read_text(),normalized)
    def test_custom_desktop_with_changed_exec_is_not_preserved_or_overwritten(self):
        manager.install(self.paths,self.bundle('one'))
        desktop=self.paths.integrations()['desktop'];desktop.write_text(desktop.read_text().replace('/usr/bin/python3','/bin/sh'))
        with self.assertRaisesRegex(ValueError,'Integration was customized'):
            manager.install(self.paths,self.bundle('two'))
        self.assertEqual(manager.current(self.paths),'one')
        self.assertEqual(desktop.read_text().split('Exec=',1)[1].splitlines()[0].split()[0],'/bin/sh')

    def test_new_starter_is_selected_and_existing_saved_layout_is_preserved(self):
        archive=self.bundle('new-defaults');files=read_archive(archive)
        files['config/plus-default.json']=(ROOT/'config/plus-default.json').read_bytes()
        manifest=json.loads(files['release.json']);manifest['files']['config/plus-default.json']=digest(files['config/plus-default.json'])
        files['release.json']=json.dumps(manifest).encode();write_archive(archive,files)
        saved=self.paths.config/'decksmith/layout.json';atomic(saved,b'{"custom":"preserve exactly"}')
        with patch.object(manager,'validate_saved'):
            manager.install(self.paths,archive)
        unit=self.paths.integrations()['unit'].read_text()
        self.assertIn('config/plus-default.json',unit)
        self.assertEqual(saved.read_bytes(),b'{"custom":"preserve exactly"}')
        defaults=json.loads((self.paths.app/'current/config/plus-default.json').read_text())
        self.assertEqual(defaults['theme']['appearance']['icon_size'],85)
        self.assertEqual(defaults['pages'][0]['keys'][4]['action']['type'],'open_website')

    def preview_settings(self):
        self.paths.config.mkdir(parents=True,exist_ok=True);self.paths.config.chmod(0o700)
        config=self.paths.config/'decksmith';config.mkdir(mode=0o775);config.chmod(0o775)
        path=config/'control-panel.json';path.write_bytes(b'{"layout":"audio","brightness":63}');path.chmod(0o664)
        return config,path
    def test_preview_upgrade_preserves_bytes_and_original_modes_in_retained_copy(self):
        manager.install(self.paths,self.bundle('1.0.0-preview'));config,path=self.preview_settings();data=path.read_bytes()
        result=manager.install(self.paths,self.bundle('1.5.2-test'));previous=Path(result['preview_config_backup'])
        self.assertEqual(path.read_bytes(),data);self.assertEqual(stat.S_IMODE(config.stat().st_mode),0o700);self.assertEqual(stat.S_IMODE(path.stat().st_mode),0o600)
        self.assertEqual((previous/path.name).read_bytes(),data);self.assertEqual(stat.S_IMODE(previous.stat().st_mode),0o775);self.assertEqual(stat.S_IMODE((previous/path.name).stat().st_mode),0o664)
        again=manager.install(self.paths,self.bundle('1.5.2-test'));self.assertNotIn('preview_config_backup',again);self.assertEqual(manager.load_record(self.paths)['preview_config_backup'],str(previous))
    def test_failed_preview_activation_restores_original_configuration_and_modes(self):
        manager.install(self.paths,self.bundle('1.0.0-preview'));config,path=self.preview_settings();data=path.read_bytes()
        with patch.object(manager,'wait_ready',side_effect=ValueError('Not ready')):
            with self.assertRaisesRegex(ValueError,'Not ready'):manager.install(self.paths,self.bundle('1.5.2-test'),True)
        self.assertEqual(path.read_bytes(),data);self.assertEqual(stat.S_IMODE(config.stat().st_mode),0o775);self.assertEqual(stat.S_IMODE(path.stat().st_mode),0o664)
        self.assertEqual(manager.current(self.paths),'1.0.0-preview');self.assertFalse(list(self.paths.config.glob('.decksmith-preview-*')))
    def test_preview_migration_refuses_shared_parent_links_and_executable_files(self):
        for case in ['shared-parent','symlink','hardlink','executable']:
            with self.subTest(case=case),tempfile.TemporaryDirectory() as temporary:
                paths=manager.Paths.create(Path(temporary)/'home');paths.config.mkdir(parents=True,mode=0o700);paths.config.chmod(0o700)
                config=paths.config/'decksmith';config.mkdir(mode=0o775);config.chmod(0o775);path=config/'settings.json';path.write_bytes(b'unchanged');path.chmod(0o664)
                outside=Path(temporary)/'outside';outside.write_bytes(b'outside');outside.chmod(0o664)
                if case=='shared-parent':paths.config.chmod(0o755)
                elif case=='symlink':path.unlink();path.symlink_to(outside)
                elif case=='hardlink':path.unlink();os.link(outside,path)
                else:path.chmod(0o775)
                with self.assertRaises(ValueError):manager.migrate_private_preview(paths,'1.0.0-preview')
                self.assertEqual(outside.read_bytes(),b'outside');self.assertEqual(stat.S_IMODE(outside.stat().st_mode),0o664);self.assertEqual(stat.S_IMODE(config.stat().st_mode),0o775)
                self.assertFalse(list(paths.config.glob('.decksmith-private-*')));self.assertFalse(list(paths.config.glob('.decksmith-preview-*')))
    def test_preview_migration_copy_sync_failure_leaves_original_tree_intact(self):
        config,path=self.preview_settings();data=path.read_bytes()
        with patch.object(manager,'sync_directory',side_effect=OSError('disk error')):
            with self.assertRaises(OSError):manager.migrate_private_preview(self.paths,'1.0.0-preview')
        self.assertEqual(path.read_bytes(),data);self.assertEqual(stat.S_IMODE(config.stat().st_mode),0o775)
        self.assertFalse(list(self.paths.config.glob('.decksmith-private-*')))

    def test_preview_migration_parent_sync_failure_restores_original_after_swap(self):
        config,path=self.preview_settings();data=path.read_bytes();original_sync=manager.sync_directory
        def fail_parent(directory):
            if directory==self.paths.config:raise OSError('parent sync error')
            return original_sync(directory)
        with patch.object(manager,'sync_directory',side_effect=fail_parent):
            with self.assertRaises(OSError):manager.migrate_private_preview(self.paths,'1.0.0-preview')
        self.assertEqual(path.read_bytes(),data);self.assertEqual(stat.S_IMODE(config.stat().st_mode),0o775);self.assertEqual(stat.S_IMODE(path.stat().st_mode),0o664)
        self.assertFalse(list(self.paths.config.glob('.decksmith-private-*')));self.assertFalse(list(self.paths.config.glob('.decksmith-preview-*')))
    def test_preview_migration_does_not_add_owner_write_to_readonly_settings(self):
        config,_=self.preview_settings();readonly=config/'readonly.json';readonly.write_bytes(b'keep readonly');readonly.chmod(0o444)
        previous=manager.migrate_private_preview(self.paths,'1.0.0-preview')
        self.assertEqual(readonly.read_bytes(),b'keep readonly');self.assertEqual(stat.S_IMODE(readonly.stat().st_mode),0o400)
        self.assertEqual(stat.S_IMODE((previous/'readonly.json').stat().st_mode),0o444)

    def test_preview_migration_refuses_readonly_directory_without_changing_it(self):
        config,path=self.preview_settings();folder=config/'readonly';folder.mkdir();folder.chmod(0o500)
        try:
            with self.assertRaisesRegex(ValueError,'read-only directories'):manager.migrate_private_preview(self.paths,'1.0.0-preview')
            self.assertEqual(stat.S_IMODE(folder.stat().st_mode),0o500);self.assertEqual(stat.S_IMODE(config.stat().st_mode),0o775)
            self.assertEqual(path.read_bytes(),b'{"layout":"audio","brightness":63}')
        finally:folder.chmod(0o700)
