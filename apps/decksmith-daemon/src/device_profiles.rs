//! Explicit private profile generations and cached device-addressed controls.
//! Inert until a host supplies a store, discovery and adapters; never migrates
//! production paths, opens HID, starts helpers or connects a public bus endpoint.
use crate::{
    layout_document::{Document, LIMIT},
    layout_store as disk,
    pages::{Action, Config, ModelControls},
};
use decksmith_core::{
    device_context::{DeviceContext, DeviceId},
    device_model::DeviceModel,
};
use decksmith_device::{
    manager::{DeviceManager, ReceivedInput, SessionLayout},
    registry::{Discovery, MAX_DEVICES, REGISTRY_BYTES, Registry},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::Arc,
};
type Result<T> = std::result::Result<T, &'static str>;
const GENERATIONS: usize = 16;
const PREFS_BYTES: usize = 65_536;

pub struct Enrollment<'a> {
    pub id: DeviceId,
    pub name: &'a str,
    pub brightness: u8,
    pub legacy_plus: Option<&'a [u8]>,
}
impl<'a> Enrollment<'a> {
    pub fn new(id: DeviceId, name: &'a str, brightness: u8, legacy_plus: Option<&'a [u8]>) -> Self {
        Self {
            id,
            name,
            brightness,
            legacy_plus,
        }
    }
}
pub struct Profile {
    pub id: DeviceId,
    pub revision: u64,
    pub saved_page: u8,
    document: Arc<Document>,
    legacy: Option<Arc<[u8]>>,
}
impl Profile {
    pub fn document(&self) -> &Document {
        &self.document
    }
}
pub struct Catalog {
    pub generation: u64,
    registry: Registry,
    profiles: BTreeMap<DeviceId, Profile>,
    preferences: Arc<[u8]>,
    pub selected: Option<DeviceId>,
}
impl Catalog {
    fn empty() -> Self {
        Self {
            generation: 0,
            registry: Registry::default(),
            profiles: BTreeMap::new(),
            preferences: Arc::from(b"{}".as_slice()),
            selected: None,
        }
    }
    pub fn profile(&self, id: DeviceId) -> Result<&Profile> {
        self.profiles.get(&id).ok_or("device_profile_missing")
    }
    pub fn summaries(&self) -> Vec<decksmith_device::registry::Summary<'_>> {
        self.registry.summaries()
    }
    /// Private raw preferences retained without interpreting global service/login
    /// settings. Never print/export these with public status or layout packages.
    pub fn preferences_private(&self) -> &[u8] {
        &self.preferences
    }
}
fn private(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path).map_err(|_| "profile_store_io")?;
    if !m.is_file()
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o077 != 0
        || m.nlink() != 1
    {
        return Err("profile_file_not_private");
    }
    Ok(())
}
fn read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    private(path)?;
    disk::read(path, limit)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn preferences(bytes: &[u8]) -> Result<()> {
    if bytes.len() > PREFS_BYTES
        || serde_json::from_slice::<BTreeMap<String, Box<serde_json::value::RawValue>>>(bytes)
            .is_err()
    {
        return Err("invalid_profile_preferences");
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    bytes: usize,
    sha256: String,
}
fn file(bytes: &[u8]) -> File {
    File {
        bytes: bytes.len(),
        sha256: digest(bytes),
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: DeviceId,
    revision: u64,
    saved_page: u8,
    layout: File,
    legacy: Option<File>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    generation: u64,
    selected: Option<DeviceId>,
    registry: File,
    preferences: File,
    profiles: Vec<Row>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    version: u8,
    generation: u64,
}
pub struct Store {
    root: PathBuf,
}
impl Store {
    /// Exclusive, explicit destination only. Normal startup never calls this.
    pub fn create(root: &Path) -> Result<Self> {
        disk::mkdir(root)?;
        disk::mkdir(&root.join("generations"))?;
        disk::sync(root)?;
        disk::sync(root.parent().ok_or("invalid_profile_path")?)?;
        Self::open(root)
    }
    pub fn open(root: &Path) -> Result<Self> {
        disk::directory(root)?;
        if fs::metadata(root).map_err(|_| "profile_store_io")?.mode() & 0o077 != 0 {
            return Err("profile_store_not_private");
        }
        disk::directory(&root.join("generations"))?;
        Ok(Self { root: root.into() })
    }
    fn pointer(&self) -> Result<u64> {
        let path = self.root.join("current.json");
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(_) => return Err("profile_store_io"),
            Ok(_) => {}
        }
        let p: Pointer =
            serde_json::from_slice(&read(&path, 4096)?).map_err(|_| "invalid_profile_pointer")?;
        if p.version != 1 || p.generation == 0 {
            return Err("invalid_profile_pointer");
        }
        Ok(p.generation)
    }
    fn dir(&self, generation: u64) -> PathBuf {
        self.root
            .join("generations")
            .join(format!("g{generation:016x}"))
    }
    fn next(&self) -> Result<u64> {
        let mut max = 0;
        let mut count = 0;
        for (n, e) in fs::read_dir(self.root.join("generations"))
            .map_err(|_| "profile_store_io")?
            .enumerate()
        {
            if n >= GENERATIONS {
                return Err("profile_generation_limit");
            }
            let e = e.map_err(|_| "profile_store_io")?;
            let name = e.file_name();
            let name = name.to_str().ok_or("invalid_profile_generation")?;
            if name.len() != 17
                || !name.starts_with('g')
                || !name[1..]
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("invalid_profile_generation");
            }
            disk::directory(&e.path())?;
            count += 1;
            max = max.max(
                u64::from_str_radix(&name[1..], 16).map_err(|_| "invalid_profile_generation")?,
            );
        }
        if count >= GENERATIONS {
            return Err("profile_generation_limit");
        }
        max.checked_add(1).ok_or("profile_generation_exhausted")
    }
    fn checked(&self, dir: &Path, name: &str, file: &File, limit: usize) -> Result<Vec<u8>> {
        if file.bytes > limit {
            return Err("profile_file_too_large");
        }
        let bytes = read(&dir.join(name), limit)?;
        if bytes.len() != file.bytes || digest(&bytes) != file.sha256 {
            return Err("profile_checksum_failed");
        }
        Ok(bytes)
    }
    fn load_generation(&self, generation: u64) -> Result<Catalog> {
        let dir = self.dir(generation);
        disk::directory(&dir)?;
        let m: Manifest =
            serde_json::from_slice(&read(&dir.join("manifest.json"), REGISTRY_BYTES)?)
                .map_err(|_| "invalid_profile_manifest")?;
        if m.version != 1 || m.generation != generation || m.profiles.len() > MAX_DEVICES {
            return Err("invalid_profile_manifest");
        }
        let registry =
            Registry::decode(&self.checked(&dir, "registry.json", &m.registry, REGISTRY_BYTES)?)?;
        let prefs = self.checked(&dir, "preferences.json", &m.preferences, PREFS_BYTES)?;
        preferences(&prefs)?;
        let mut profiles = BTreeMap::new();
        for row in m.profiles {
            let record = registry.get(row.id).ok_or("profile_identity_missing")?;
            if row.revision == 0 || profiles.contains_key(&row.id) {
                return Err("invalid_profile_manifest");
            }
            let doc = Document::for_model(
                &self.checked(&dir, &format!("{}.json", row.id), &row.layout, LIMIT)?,
                record.model(),
            )?;
            if doc.version() != 2 || row.saved_page >= doc.topology().0 {
                return Err("invalid_profile_page");
            }
            let legacy = if let Some(entry) = row.legacy {
                let bytes =
                    self.checked(&dir, &format!("{}-legacy.json", row.id), &entry, LIMIT)?;
                if record.model() != DeviceModel::Plus || Document::parse(&bytes)?.version() != 1 {
                    return Err("invalid_profile_legacy");
                }
                Some(Arc::from(bytes))
            } else {
                None
            };
            profiles.insert(
                row.id,
                Profile {
                    id: row.id,
                    revision: row.revision,
                    saved_page: row.saved_page,
                    document: Arc::new(doc),
                    legacy,
                },
            );
        }
        if profiles.len() != registry.records().len()
            || m.selected.is_some_and(|id| !profiles.contains_key(&id))
        {
            return Err("profile_identity_missing");
        }
        Ok(Catalog {
            generation,
            registry,
            profiles,
            preferences: Arc::from(prefs),
            selected: m.selected,
        })
    }
    pub fn load(&self) -> Result<Catalog> {
        let generation = self.pointer()?;
        let catalog = if generation == 0 {
            if fs::read_dir(self.root.join("generations"))
                .map_err(|_| "profile_store_io")?
                .next()
                .is_some()
            {
                return Err("profile_store_incomplete");
            }
            Catalog::empty()
        } else {
            self.load_generation(generation)?
        };
        if self.pointer() != Ok(generation) {
            return Err("profile_read_changed");
        }
        Ok(catalog)
    }
    fn lock(&self) -> Result<disk::WriterLock> {
        let path = self.root.join("store.lock");
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
            .map_err(|_| "profile_store_io")?;
        private(&path)?;
        disk::WriterLock::acquire(f)
    }
    fn commit(&self, catalog: &Catalog, stop: Option<u8>) -> Result<u64> {
        let generation = self.next()?;
        let dir = self.dir(generation);
        disk::mkdir(&dir)?;
        let registry = catalog.registry.encode_private()?;
        disk::write(&dir.join("registry.json"), &registry)?;
        disk::write(&dir.join("preferences.json"), &catalog.preferences)?;
        let mut rows = Vec::new();
        for p in catalog.profiles.values() {
            disk::write(&dir.join(format!("{}.json", p.id)), p.document.bytes())?;
            if let Some(bytes) = &p.legacy {
                disk::write(&dir.join(format!("{}-legacy.json", p.id)), bytes)?;
            }
            rows.push(Row {
                id: p.id,
                revision: p.revision,
                saved_page: p.saved_page,
                layout: file(p.document.bytes()),
                legacy: p.legacy.as_ref().map(|bytes| file(bytes)),
            });
        }
        if stop == Some(1) {
            return Err("profile_commit_interrupted");
        }
        let manifest = Manifest {
            version: 1,
            generation,
            selected: catalog.selected,
            registry: file(&registry),
            preferences: file(&catalog.preferences),
            profiles: rows,
        };
        disk::write(
            &dir.join("manifest.json"),
            &serde_json::to_vec(&manifest).map_err(|_| "invalid_profile_manifest")?,
        )?;
        disk::sync(&dir)?;
        disk::sync(&self.root.join("generations"))?;
        self.load_generation(generation)?;
        if stop == Some(2) {
            return Err("profile_commit_interrupted");
        }
        if self.pointer()? != catalog.generation {
            return Err("stale_profile_generation");
        }
        let temp = self.root.join(format!("current-{generation:016x}.tmp"));
        disk::write(
            &temp,
            &serde_json::to_vec(&Pointer {
                version: 1,
                generation,
            })
            .map_err(|_| "invalid_profile_pointer")?,
        )?;
        fs::rename(temp, self.root.join("current.json")).map_err(|_| "profile_store_io")?;
        disk::sync(&self.root).map_err(|_| "profile_commit_uncertain")?;
        Ok(generation)
    }
    /// Explicit enrollment binds the supplied snapshot/slot to a fresh opaque ID.
    /// Only Plus may import a V1 document; MK.2 always starts with its own defaults.
    pub fn enroll(
        &self,
        expected: u64,
        discovery: &Discovery,
        slot: u8,
        enrollment: Enrollment<'_>,
    ) -> Result<u64> {
        let Enrollment {
            id,
            name,
            brightness,
            legacy_plus,
        } = enrollment;
        let _lock = self.lock()?;
        let mut catalog = self.load()?;
        if catalog.generation != expected {
            return Err("stale_profile_generation");
        }
        let model = discovery
            .candidates()
            .iter()
            .find(|c| c.slot() == slot)
            .ok_or("unknown_candidate")?
            .model();
        if model == DeviceModel::Mk2 && legacy_plus.is_some() {
            return Err("mk2_requires_own_defaults");
        }
        let (document, legacy) = match (model, legacy_plus) {
            (DeviceModel::Plus, Some(bytes)) => {
                let original = Document::for_model(bytes, model)?;
                if original.version() != 1 {
                    return Err("profile_import_requires_legacy_plus");
                }
                (original.upgraded()?, Some(Arc::from(bytes)))
            }
            (DeviceModel::Plus, None) => (
                Document::parse(include_bytes!("../../../config/navigation.json"))?.upgraded()?,
                None,
            ),
            (DeviceModel::Mk2, None) => (Document::mk2_defaults()?, None),
            _ => return Err("invalid_profile_model"),
        };
        catalog.registry.associate(
            catalog.registry.revision(),
            id,
            name,
            brightness,
            discovery,
            slot,
        )?;
        let saved_page = document.topology().1;
        catalog.profiles.insert(
            id,
            Profile {
                id,
                revision: 1,
                saved_page,
                document: Arc::new(document),
                legacy,
            },
        );
        if catalog.selected.is_none() {
            catalog.selected = Some(id);
        }
        self.commit(&catalog, None)
    }
    /// Merge against the latest catalog under the writer lock. Edits to another
    /// device do not invalidate this device's own expected revision.
    pub fn save_profile(
        &self,
        id: DeviceId,
        expected: u64,
        bytes: &[u8],
        saved_page: u8,
        name: &str,
        brightness: u8,
    ) -> Result<u64> {
        let _lock = self.lock()?;
        let mut catalog = self.load()?;
        let old = catalog.profile(id)?;
        if old.revision != expected {
            return Err("stale_device_profile");
        }
        let revision = old
            .revision
            .checked_add(1)
            .ok_or("profile_revision_exhausted")?;
        let doc = Document::for_model(bytes, old.document.model())?;
        if doc.version() != 2 || saved_page >= doc.topology().0 {
            return Err("invalid_profile_page");
        }
        let legacy = old.legacy.clone();
        catalog
            .registry
            .update(catalog.registry.revision(), id, name, brightness)?;
        catalog.profiles.insert(
            id,
            Profile {
                id,
                revision,
                saved_page,
                document: Arc::new(doc),
                legacy,
            },
        );
        self.commit(&catalog, None)
    }
    pub fn select(&self, expected: u64, id: DeviceId) -> Result<u64> {
        let _lock = self.lock()?;
        let mut catalog = self.load()?;
        if catalog.generation != expected {
            return Err("stale_profile_generation");
        }
        catalog.profile(id)?;
        catalog.selected = Some(id);
        self.commit(&catalog, None)
    }
    /// Explicit recovery to a new destination; never rewrites the current store
    /// or adopts unpublished files automatically. IDs/preferences remain intact.
    pub fn restore_generation(&self, generation: u64, destination: &Path) -> Result<u64> {
        let source = self.root.canonicalize().map_err(|_| "profile_store_io")?;
        let parent = destination
            .parent()
            .ok_or("invalid_profile_path")?
            .canonicalize()
            .map_err(|_| "profile_store_io")?;
        if parent.starts_with(source) {
            return Err("restore_inside_profile_store");
        }
        let mut catalog = self.load_generation(generation)?;
        catalog.generation = 0;
        let restored = Self::create(destination)?;
        let _lock = restored.lock()?;
        restored.commit(&catalog, None)
    }
    pub fn save_preferences_private(&self, expected: u64, bytes: &[u8]) -> Result<u64> {
        preferences(bytes)?;
        let _lock = self.lock()?;
        let mut catalog = self.load()?;
        if catalog.generation != expected {
            return Err("stale_profile_generation");
        }
        catalog.preferences = Arc::from(bytes);
        self.commit(&catalog, None)
    }
}
/// Explicit host enrollment helper; never derives identity from order or serial.
pub fn random_device_id() -> Result<DeviceId> {
    use std::io::Read;
    let mut file = std::fs::File::open("/dev/urandom").map_err(|_| "device_id_random_failed")?;
    for _ in 0..4 {
        let mut bits = [0; 16];
        file.read_exact(&mut bits)
            .map_err(|_| "device_id_random_failed")?;
        if let Ok(id) = DeviceId::from_bytes(bits) {
            return Ok(id);
        }
    }
    Err("device_id_random_failed")
}

/// Active action tables are immutable and device-addressed. Loading/applying a
/// saved catalog is explicit; it never replaces the production worker implicitly.
pub struct Controls {
    manager: DeviceManager,
    tables: BTreeMap<DeviceId, ModelControls>,
}
impl Controls {
    pub fn from_catalog(catalog: &Catalog) -> Result<Self> {
        let registry = Registry::decode(&catalog.registry.encode_private()?)?;
        let mut manager = DeviceManager::new(registry)?;
        let mut tables = BTreeMap::new();
        for p in catalog.profiles.values() {
            let table = Config::parse(p.document.bytes())?.model_controls();
            let (count, default) = table.topology();
            let revision = manager.configure(
                p.id,
                0,
                SessionLayout::new(p.document.model(), count, default)?,
            )?;
            manager.page(p.id, revision, p.saved_page)?;
            tables.insert(p.id, table);
        }
        Ok(Self { manager, tables })
    }
    pub fn manager(&self) -> &DeviceManager {
        &self.manager
    }
    pub fn discover(&mut self, discovery: Discovery) -> Result<u64> {
        self.manager.reconcile(discovery)
    }
    pub fn attach(
        &mut self,
        revision: u64,
        slot: u8,
        device: Box<dyn decksmith_device::DeckDevice>,
    ) -> Result<DeviceId> {
        self.manager.attach(revision, slot, device)
    }
    pub fn disconnect(&mut self, id: DeviceId) -> Result<()> {
        self.manager.disconnect(id)
    }
    pub fn gate(&mut self, locked: bool, suspended: bool) -> Result<()> {
        self.manager.gate(locked, suspended)
    }
    pub fn poll_round(&mut self) -> Vec<ReceivedInput> {
        self.manager.poll_round()
    }
    pub fn brightness(&mut self, context: DeviceContext, percent: u8) -> Result<()> {
        if !self.manager.current(&context) {
            return Err("stale_device_context");
        }
        self.manager
            .brightness(context.device, context.binding, percent)
    }
    pub fn paint(
        &mut self,
        context: DeviceContext,
        keys: &[u8],
        touch: Option<&[u8]>,
    ) -> Result<()> {
        self.manager.paint(context, keys, touch)
    }
    pub fn stop(&mut self) -> Vec<DeviceId> {
        self.manager.stop()
    }
    pub fn resolve(&self, event: &ReceivedInput) -> Result<Option<Action>> {
        if event.context.control != event.input.event.control()
            || !self.manager.current(&event.context)
        {
            return Err("stale_device_context");
        }
        let page = self.manager.active_page(event.context.device)?;
        Ok(self
            .tables
            .get(&event.context.device)
            .ok_or("device_profile_missing")?
            .resolve(page, &event.input.event))
    }
    pub fn apply_saved_profile(
        &mut self,
        catalog: &Catalog,
        id: DeviceId,
        expected_binding: u64,
    ) -> Result<u64> {
        let p = catalog.profile(id)?;
        let table = Config::parse(p.document.bytes())?.model_controls();
        let (count, default) = table.topology();
        let revision = self.manager.configure(
            id,
            expected_binding,
            SessionLayout::new(p.document.model(), count, default)?,
        )?;
        let revision = self.manager.page(id, revision, p.saved_page)?;
        self.tables.insert(id, table);
        let percent = catalog
            .registry
            .get(id)
            .ok_or("profile_identity_missing")?
            .brightness();
        self.manager.brightness(id, revision, percent)?;
        Ok(revision)
    }
    pub fn navigate(&mut self, context: DeviceContext, page: u8) -> Result<u64> {
        if !self.manager.current(&context) {
            return Err("stale_device_context");
        }
        self.manager.page(context.device, context.binding, page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_core::{
        InputEvent, RawEvent, TouchGesture,
        device_model::{ControlAddress, DeviceDescriptor},
    };
    use decksmith_device::{DeckDevice, DeviceError, VirtualDeck, registry::Candidate};
    use std::{
        cell::RefCell,
        os::unix::fs::{PermissionsExt, symlink},
        rc::Rc,
    };
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "decksmith-profiles-{}",
                random_device_id().unwrap()
            ));
            disk::mkdir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn id(n: u8) -> DeviceId {
        DeviceId::from_bytes([n; 16]).unwrap()
    }
    fn discovery(reverse: bool) -> Discovery {
        Discovery::new(if reverse {
            vec![
                Candidate::new(0, DeviceModel::Mk2, Some("mk2-private")),
                Candidate::new(1, DeviceModel::Plus, Some("plus-private")),
            ]
        } else {
            vec![
                Candidate::new(0, DeviceModel::Plus, Some("plus-private")),
                Candidate::new(1, DeviceModel::Mk2, Some("mk2-private")),
            ]
        })
        .unwrap()
    }
    fn pair() -> (Temp, Store) {
        let temp = Temp::new();
        let store = Store::create(&temp.0.join("store")).unwrap();
        let g = store
            .enroll(
                0,
                &discovery(false),
                0,
                Enrollment::new(
                    id(1),
                    "Desk 🎛",
                    39,
                    Some(include_bytes!("../../../config/audio.json")),
                ),
            )
            .unwrap();
        store
            .enroll(
                g,
                &discovery(false),
                1,
                Enrollment::new(id(2), "MK.2", 78, None),
            )
            .unwrap();
        (temp, store)
    }
    #[test]
    fn enrollment_preserves_legacy_bytes_independent_defaults_identity_and_preferences() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        assert_eq!(
            &**c.profile(id(1)).unwrap().legacy.as_ref().unwrap(),
            include_bytes!("../../../config/audio.json")
        );
        assert_eq!(
            c.profile(id(2)).unwrap().document.bytes(),
            Document::mk2_defaults().unwrap().bytes()
        );
        assert_eq!(c.selected, Some(id(1)));
        let prefs = br#"{ "future":184467440737095516160001, "start_at_login":true }"#;
        s.save_preferences_private(c.generation, prefs).unwrap();
        let c = s.load().unwrap();
        assert_eq!(c.preferences_private(), prefs);
        let status = serde_json::to_string(&c.summaries()).unwrap();
        assert!(!status.contains("plus-private"));
        assert!(!status.contains("mk2-private"));
        assert_eq!(
            c.registry
                .resolve(&discovery(true), &discovery(true).candidates()[1]),
            decksmith_device::registry::Association::Assigned(id(1))
        );
    }
    #[test]
    fn independent_edit_revisions_merge_and_same_device_stale_save_is_rejected() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        s.save_profile(
            id(1),
            1,
            c.profile(id(1)).unwrap().document.bytes(),
            1,
            "New 🎉",
            25,
        )
        .unwrap();
        s.save_profile(
            id(2),
            1,
            c.profile(id(2)).unwrap().document.bytes(),
            2,
            "Other",
            80,
        )
        .unwrap();
        let current = s.load().unwrap();
        assert_eq!(current.profile(id(1)).unwrap().saved_page, 1);
        assert_eq!(current.profile(id(2)).unwrap().saved_page, 2);
        assert_eq!(current.registry.get(id(1)).unwrap().brightness(), 25);
        assert_eq!(
            s.save_profile(
                id(1),
                1,
                c.profile(id(1)).unwrap().document.bytes(),
                0,
                "Stale",
                50
            ),
            Err("stale_device_profile")
        );
        assert_eq!(
            s.select(c.generation, id(2)),
            Err("stale_profile_generation")
        );
        assert_eq!(s.load().unwrap().generation, current.generation);
    }
    #[test]
    fn wrong_model_page_duplicate_and_ambiguous_associations_are_rejected() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        assert!(
            s.save_profile(
                id(2),
                1,
                c.profile(id(1)).unwrap().document.bytes(),
                0,
                "Wrong",
                50
            )
            .is_err()
        );
        assert!(
            s.save_profile(
                id(2),
                1,
                c.profile(id(2)).unwrap().document.bytes(),
                15,
                "Wrong",
                50
            )
            .is_err()
        );
        assert_eq!(
            s.enroll(
                c.generation,
                &discovery(false),
                1,
                Enrollment::new(
                    id(3),
                    "Copy",
                    50,
                    Some(include_bytes!("../../../config/audio.json"))
                )
            ),
            Err("mk2_requires_own_defaults")
        );
        let d = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Mk2, Some("duplicated")),
            Candidate::new(1, DeviceModel::Mk2, Some("duplicated")),
        ])
        .unwrap();
        assert!(
            s.enroll(
                c.generation,
                &d,
                0,
                Enrollment::new(id(3), "Ambiguous", 50, None)
            )
            .is_err()
        );
        assert_eq!(s.load().unwrap().generation, c.generation);
    }
    #[test]
    fn corruption_is_rejected_and_explicit_restore_preserves_the_previous_generation() {
        let (t, s) = pair();
        let c = s.load().unwrap();
        s.select(c.generation, id(2)).unwrap();
        let current = s.load().unwrap().generation;
        fs::write(s.dir(current).join(format!("{}.json", id(2))), b"corrupted").unwrap();
        assert!(s.load().is_err());
        assert_eq!(
            s.restore_generation(c.generation, &s.root.join("nested")),
            Err("restore_inside_profile_store")
        );
        s.restore_generation(c.generation, &t.0.join("recovered"))
            .unwrap();
        let r = Store::open(&t.0.join("recovered")).unwrap().load().unwrap();
        assert_eq!(r.selected, Some(id(1)));
        assert_eq!(
            r.profile(id(1)).unwrap().document.bytes(),
            c.profile(id(1)).unwrap().document.bytes()
        );
    }
    #[test]
    fn interrupted_generation_never_replaces_the_previous_pointer() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        {
            let _lock = s.lock().unwrap();
            assert_eq!(s.commit(&c, Some(1)), Err("profile_commit_interrupted"));
        }
        assert_eq!(s.load().unwrap().generation, c.generation);
        {
            let _lock = s.lock().unwrap();
            assert_eq!(s.commit(&c, Some(2)), Err("profile_commit_interrupted"));
        }
        assert_eq!(s.load().unwrap().generation, c.generation);
    }
    #[test]
    fn symlink_hardlink_permissions_missing_pointer_and_writer_conflict_fail_closed() {
        let (t, s) = pair();
        let pointer = s.root.join("current.json");
        let bytes = fs::read(&pointer).unwrap();
        fs::set_permissions(&pointer, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(s.load().is_err());
        fs::set_permissions(&pointer, fs::Permissions::from_mode(0o600)).unwrap();
        let link = t.0.join("link");
        fs::hard_link(&pointer, &link).unwrap();
        assert!(s.load().is_err());
        fs::remove_file(link).unwrap();
        fs::remove_file(&pointer).unwrap();
        symlink("missing", &pointer).unwrap();
        assert!(s.load().is_err());
        fs::remove_file(&pointer).unwrap();
        assert!(s.load().is_err());
        disk::write(&pointer, &bytes).unwrap();
        let _lock = s.lock().unwrap();
        assert!(s.select(s.load().unwrap().generation, id(2)).is_err());
    }
    #[test]
    fn generation_budget_stops_without_deleting_backups_or_replaying_a_save() {
        let (_t, s) = pair();
        for _ in 2..GENERATIONS {
            let c = s.load().unwrap();
            s.select(c.generation, id(1)).unwrap();
        }
        let c = s.load().unwrap();
        assert_eq!(
            s.select(c.generation, id(2)),
            Err("profile_generation_limit")
        );
        assert_eq!(s.load().unwrap().generation, c.generation);
        assert_eq!(
            fs::read_dir(s.root.join("generations")).unwrap().count(),
            GENERATIONS
        );
    }
    #[derive(Clone)]
    struct Probe(Rc<RefCell<VirtualDeck>>);
    impl DeckDevice for Probe {
        fn descriptor(&self) -> DeviceDescriptor {
            self.0.borrow().descriptor()
        }
        fn poll_event(&mut self) -> std::result::Result<Option<InputEvent>, DeviceError> {
            self.0.borrow_mut().poll_event()
        }
        fn poll_event_nonblocking(
            &mut self,
        ) -> std::result::Result<Option<InputEvent>, DeviceError> {
            self.0.borrow_mut().poll_event_nonblocking()
        }
        fn set_key_image(&mut self, key: u8, bytes: &[u8]) -> std::result::Result<(), DeviceError> {
            self.0.borrow_mut().set_key_image(key, bytes)
        }
        fn set_touch_image(&mut self, bytes: &[u8]) -> std::result::Result<(), DeviceError> {
            self.0.borrow_mut().set_touch_image(bytes)
        }
        fn set_brightness(&mut self, p: u8) -> std::result::Result<(), DeviceError> {
            self.0.borrow_mut().set_brightness(p)
        }
        fn blank(&mut self) -> std::result::Result<(), DeviceError> {
            self.0.borrow_mut().blank()
        }
    }
    fn connect(controls: &mut Controls, reverse: bool) -> [Rc<RefCell<VirtualDeck>>; 2] {
        let revision = controls.discover(discovery(reverse)).unwrap();
        let plus = Rc::new(RefCell::new(VirtualDeck::default()));
        let mk2 = Rc::new(RefCell::new(VirtualDeck::new(DeviceModel::Mk2)));
        controls
            .attach(
                revision,
                if reverse { 1 } else { 0 },
                Box::new(Probe(plus.clone())),
            )
            .unwrap();
        controls
            .attach(
                revision,
                if reverse { 0 } else { 1 },
                Box::new(Probe(mk2.clone())),
            )
            .unwrap();
        controls.poll_round();
        [plus, mk2]
    }
    fn edge(
        controls: &mut Controls,
        probe: &Rc<RefCell<VirtualDeck>>,
        key: u8,
        pressed: bool,
        timestamp: u64,
    ) -> ReceivedInput {
        probe
            .borrow_mut()
            .inject(InputEvent {
                timestamp_ms: timestamp,
                event: RawEvent::Key {
                    index: key,
                    pressed,
                },
            })
            .unwrap();
        controls.poll_round().pop().unwrap()
    }
    #[test]
    fn saved_profiles_route_all_fifteen_mk2_keys_and_keep_pages_and_brightness_independent() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        let mut controls = Controls::from_catalog(&c).unwrap();
        let probes = connect(&mut controls, false);
        let down = edge(&mut controls, &probes[1], 14, true, 0);
        assert!(controls.resolve(&down).unwrap().is_none());
        let up = edge(&mut controls, &probes[1], 14, false, 1);
        assert_eq!(
            controls.resolve(&up).unwrap(),
            Some(Action::GoToPage { page: 1 })
        );
        controls.navigate(up.context, 1).unwrap();
        controls.poll_round();
        assert!(controls.resolve(&up).is_err());
        assert_eq!(controls.manager().active_page(id(1)), Ok(0));
        assert_eq!(controls.manager().active_page(id(2)), Ok(1));
        for key in 0..15 {
            let down = edge(&mut controls, &probes[1], key, true, 2 + u64::from(key) * 2);
            assert_eq!(down.context.device, id(2));
            assert!(controls.resolve(&down).is_ok());
            let up = edge(
                &mut controls,
                &probes[1],
                key,
                false,
                3 + u64::from(key) * 2,
            );
            assert_eq!(up.context.device, id(2));
            assert!(controls.resolve(&up).is_ok());
        }
        let context = controls
            .manager()
            .context(id(1), ControlAddress::Key(0))
            .unwrap();
        controls.brightness(context, 23).unwrap();
        assert_eq!(probes[0].borrow().brightness, 23);
        assert_eq!(probes[1].borrow().brightness, 78);
        assert!(
            controls
                .manager()
                .context(id(2), ControlAddress::DialRotation(0))
                .is_err()
        );
        controls.stop();
    }
    #[test]
    fn reversed_reconnect_reuses_saved_id_and_pages_and_rejects_old_context() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        let mut controls = Controls::from_catalog(&c).unwrap();
        let _probes = connect(&mut controls, false);
        let old = controls
            .manager()
            .context(id(2), ControlAddress::Key(14))
            .unwrap();
        controls.navigate(old, 2).unwrap();
        controls.poll_round();
        controls.disconnect(id(1)).unwrap();
        controls.disconnect(id(2)).unwrap();
        let probes = connect(&mut controls, true);
        assert_eq!(controls.manager().active_page(id(2)), Ok(2));
        assert!(controls.navigate(old, 1).is_err());
        assert_eq!(probes[0].borrow().brightness, 39);
        assert_eq!(probes[1].borrow().brightness, 78);
        controls.stop();
    }
    #[test]
    fn applying_one_saved_profile_fences_only_that_device_and_retains_other_ready_context() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        let mut controls = Controls::from_catalog(&c).unwrap();
        let probes = connect(&mut controls, false);
        let a = controls
            .manager()
            .context(id(1), ControlAddress::Key(0))
            .unwrap();
        let b = controls
            .manager()
            .context(id(2), ControlAddress::Key(14))
            .unwrap();
        s.save_profile(
            id(1),
            1,
            c.profile(id(1)).unwrap().document.bytes(),
            1,
            "Changed",
            15,
        )
        .unwrap();
        let updated = s.load().unwrap();
        controls
            .apply_saved_profile(&updated, id(1), a.binding)
            .unwrap();
        assert!(controls.manager().current(&b));
        assert!(!controls.manager().current(&a));
        controls.poll_round();
        assert_eq!(controls.manager().active_page(id(1)), Ok(1));
        assert_eq!(probes[0].borrow().brightness, 15);
        assert_eq!(probes[1].borrow().brightness, 78);
        controls.gate(true, false).unwrap();
        assert!(
            controls
                .manager()
                .context(id(1), ControlAddress::Key(0))
                .is_err()
        );
        assert!(
            controls
                .manager()
                .context(id(2), ControlAddress::Key(0))
                .is_err()
        );
        controls.stop();
    }
    #[test]
    fn cached_input_routing_does_not_read_or_decode_saved_files() {
        let (_t, s) = pair();
        let c = s.load().unwrap();
        let mut controls = Controls::from_catalog(&c).unwrap();
        let probes = connect(&mut controls, false);
        fs::write(s.root.join("current.json"), b"broken").unwrap();
        assert!(s.load().is_err());
        let _ = edge(&mut controls, &probes[1], 14, true, 0);
        let up = edge(&mut controls, &probes[1], 14, false, 1);
        assert_eq!(
            controls.resolve(&up).unwrap(),
            Some(Action::GoToPage { page: 1 })
        );
        controls.stop();
    }
    #[test]
    fn cached_plus_controls_match_legacy_keys_dials_and_inherited_page_overrides() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value.as_object_mut().unwrap().remove("dials");
        value["pages"][1]["dial_overrides"] = serde_json::json!([null,null,null,{"label":"Custom","rotation":"brightness","step":2,"press":{"type":"none"}}]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let config = Config::parse(&bytes).unwrap();
        let table = config.model_controls();
        let mut legacy = crate::pages::Pages::parse(&bytes).unwrap();
        for page in 0..2 {
            legacy.index = page;
            for key in 0..8 {
                let down = RawEvent::Key {
                    index: key,
                    pressed: true,
                };
                let up = RawEvent::Key {
                    index: key,
                    pressed: false,
                };
                assert_eq!(table.resolve(page, &down), legacy.target(&down));
                assert_eq!(table.resolve(page, &up), legacy.target(&up));
            }
            for n in 0..4 {
                let rotate = RawEvent::DialRotate { index: n, ticks: 2 };
                assert_eq!(table.resolve(page, &rotate), legacy.target(&rotate));
                let down = RawEvent::DialPush {
                    index: n,
                    pressed: true,
                };
                let up = RawEvent::DialPush {
                    index: n,
                    pressed: false,
                };
                assert_eq!(table.resolve(page, &down), legacy.target(&down));
                assert_eq!(table.resolve(page, &up), legacy.target(&up));
                let touch = RawEvent::Touch {
                    x: u16::from(n) * 200 + 10,
                    y: 10,
                    gesture: TouchGesture::Tap,
                };
                assert_eq!(table.resolve(page, &touch), legacy.target(&touch));
            }
        }
        assert!(
            table
                .resolve(
                    0,
                    &RawEvent::Touch {
                        x: 900,
                        y: 0,
                        gesture: TouchGesture::FlickLeft
                    }
                )
                .is_none()
        );
    }
}
