use super::*;
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    sync::atomic::{AtomicU64, Ordering},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "decksmith-model-store-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        mkdir(&path).unwrap();
        Self(path)
    }
    fn source(&self) -> PathBuf {
        let source = self.0.join("legacy");
        mkdir(&source).unwrap();
        write(
            &source.join("layout.json"),
            include_bytes!("../../../config/audio.json"),
        )
        .unwrap();
        write(
            &source.join("control-panel.json"),
            b"{\"layout\":\"custom\",\"brightness\":63}\n",
        )
        .unwrap();
        write(&source.join("auto-lock.json"), b"true\n").unwrap();
        mkdir(&source.join("artwork")).unwrap();
        write(&source.join("artwork/original.bin"), &[0, 31, 255]).unwrap();
        source
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn migration_and_restore_keep_all_original_files_bytes_and_preferences() {
    let temp = Temp::new();
    let source = temp.source();
    let target = temp.0.join("store");
    let original = Snapshot::capture(&source).unwrap();
    assert_eq!(prepare(&source, &target).unwrap(), 1);
    let store = Store::open(&target).unwrap();
    assert_eq!(store.pointer().unwrap(), Some(1));
    assert_eq!(io(fs::metadata(&target)).unwrap().mode() & 0o777, 0o700);
    let plus =
        Document::parse(&read(&store.generation(1).join("plus.json"), LIMIT).unwrap()).unwrap();
    let mk2 =
        Document::parse(&read(&store.generation(1).join("mk2.json"), LIMIT).unwrap()).unwrap();
    assert_eq!(plus.model(), DeviceModel::Plus);
    assert_eq!(mk2.model(), DeviceModel::Mk2);
    let restore_dir = temp.0.join("restored");
    restore(&target, &restore_dir).unwrap();
    for file in original.files {
        assert_eq!(
            read(&source.join(&file.name), 2 * LIMIT).unwrap(),
            file.bytes
        );
        assert_eq!(
            read(&restore_dir.join(&file.name), 2 * LIMIT).unwrap(),
            file.bytes
        );
    }
    assert!(
        crate::pages::Pages::parse(&read(&restore_dir.join("layout.json"), LIMIT).unwrap()).is_ok()
    );
}

#[test]
fn interrupted_first_migration_never_publishes_partial_state() {
    for stage in 1..=3 {
        let temp = Temp::new();
        let source = temp.source();
        let snapshot = Snapshot::capture(&source).unwrap();
        let store = Store::create(&temp.0.join("store")).unwrap();
        assert_eq!(
            store.commit(&snapshot, None, Some(stage)),
            Err("migration_interrupted")
        );
        assert_eq!(store.pointer().unwrap(), None);
        assert_eq!(
            restore(&store.root, &temp.0.join("restore")),
            Err("layout_store_incomplete")
        );
        assert!(!temp.0.join("restore").exists());
        assert_eq!(store.commit(&snapshot, None, None).unwrap(), 2);
        assert_eq!(store.pointer().unwrap(), Some(2));
    }
}

#[test]
fn interrupted_revision_preserves_previous_generation_and_rejects_stale_writers() {
    let temp = Temp::new();
    let source = temp.source();
    let snapshot = Snapshot::capture(&source).unwrap();
    let store = Store::create(&temp.0.join("store")).unwrap();
    store.commit(&snapshot, None, None).unwrap();
    for stage in 1..=3 {
        assert_eq!(
            store.commit(&snapshot, Some(1), Some(stage)),
            Err("migration_interrupted")
        );
        assert_eq!(store.pointer().unwrap(), Some(1));
        store.load(1).unwrap();
    }
    assert_eq!(
        store.commit(&snapshot, None, None),
        Err("stale_layout_generation")
    );
    assert_eq!(store.commit(&snapshot, Some(1), None).unwrap(), 5);
}

#[test]
fn tampered_backup_or_profile_is_rejected_before_restore_or_activation() {
    for file in ["plus.json", "mk2.json", "legacy/control-panel.json"] {
        let temp = Temp::new();
        let source = temp.source();
        let target = temp.0.join("store");
        prepare(&source, &target).unwrap();
        let store = Store::open(&target).unwrap();
        fs::write(store.generation(1).join(file), b"{}").unwrap();
        assert_eq!(
            restore(&target, &temp.0.join("restore")),
            Err("layout_checksum_failed")
        );
        assert!(!temp.0.join("restore").exists());
        assert_eq!(
            store.commit(&Snapshot::capture(&source).unwrap(), Some(1), None),
            Err("layout_checksum_failed")
        );
    }
}

#[test]
fn malicious_paths_symlinks_and_nonprivate_stores_are_rejected() {
    let temp = Temp::new();
    let source = temp.source();
    assert!(prepare(&source, &source.join("inside")).is_err());
    let outside = temp.0.join("outside");
    write(&outside, b"preserve").unwrap();
    symlink(&outside, source.join("linked")).unwrap();
    assert!(prepare(&source, &temp.0.join("store")).is_err());
    assert!(!temp.0.join("store").exists());
    fs::remove_file(source.join("linked")).unwrap();
    symlink(&source, temp.0.join("alias")).unwrap();
    assert!(prepare(&temp.0.join("alias"), &temp.0.join("store")).is_err());
    let target = temp.0.join("store");
    prepare(&source, &target).unwrap();
    let store = Store::open(&target).unwrap();
    let path = store.generation(1).join("manifest.json");
    let mut manifest: Manifest = serde_json::from_slice(&read(&path, 512 * 1024).unwrap()).unwrap();
    manifest.legacy[0].path = "../../outside".into();
    fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert_eq!(store.load(1).err(), Some("invalid_manifest"));
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Store::open(&target).is_err());
    assert_eq!(read(&outside, 100).unwrap(), b"preserve");
}

#[test]
fn destinations_and_foreign_files_are_never_overwritten() {
    let temp = Temp::new();
    let source = temp.source();
    let target = temp.0.join("store");
    mkdir(&target).unwrap();
    write(&target.join("existing"), b"preserve").unwrap();
    assert!(prepare(&source, &target).is_err());
    assert_eq!(read(&target.join("existing"), 100).unwrap(), b"preserve");
    let good = temp.0.join("good");
    prepare(&source, &good).unwrap();
    assert!(restore(&good, &source).is_err());
    assert!(restore(&good, &good.join("restored")).is_err());
    assert_eq!(
        read(&source.join("control-panel.json"), 100).unwrap(),
        b"{\"layout\":\"custom\",\"brightness\":63}\n"
    );
}

#[test]
fn store_lock_excludes_other_writer_and_unknown_manifest_version_preserves_state() {
    let temp = Temp::new();
    let source = temp.source();
    let snapshot = Snapshot::capture(&source).unwrap();
    let store = Store::create(&temp.0.join("store")).unwrap();
    store.commit(&snapshot, None, None).unwrap();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(store.root.join("store.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert_eq!(
        store.commit(&snapshot, Some(1), None),
        Err("layout_store_busy")
    );
    drop(lock);
    let manifest = store.generation(1).join("manifest.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&read(&manifest, 512 * 1024).unwrap()).unwrap();
    value["version"] = 99.into();
    fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        store.commit(&snapshot, Some(1), None),
        Err("invalid_manifest")
    );
    assert_eq!(store.pointer().unwrap(), Some(1));
}

#[test]
fn explicit_tools_do_not_select_hardware_or_expose_saved_settings() {
    let temp = Temp::new();
    let source = temp.source();
    let args = vec![
        "--layout-info".into(),
        source.join("layout.json").to_string_lossy().into_owned(),
    ];
    let report: serde_json::Value =
        serde_json::from_str(&request(&args).unwrap().unwrap()).unwrap();
    assert_eq!(report["model"], "plus");
    assert!(report.get("settings").is_none());
    assert!(request(&["--hardware".into(), "--exclusive".into()]).is_none());
    assert!(request(&["--prepare-layout-store".into()]).is_none());
}

#[test]
fn bounded_generation_and_snapshot_entry_limits_preserve_active_state() {
    let temp = Temp::new();
    let source = temp.source();
    let snapshot = Snapshot::capture(&source).unwrap();
    let store = Store::create(&temp.0.join("store")).unwrap();
    store.commit(&snapshot, None, None).unwrap();
    for number in 2..=MAX_GENERATIONS as u64 {
        mkdir(&store.generation(number)).unwrap();
    }
    assert_eq!(
        store.commit(&snapshot, Some(1), None),
        Err("layout_generation_limit")
    );
    assert_eq!(store.pointer().unwrap(), Some(1));
    for number in 0..513 {
        mkdir(&source.join(format!("empty-{number}"))).unwrap();
    }
    assert_eq!(
        prepare(&source, &temp.0.join("too-many")).err(),
        Some("layout_snapshot_too_many_entries")
    );
    assert!(!temp.0.join("too-many").exists());
}
