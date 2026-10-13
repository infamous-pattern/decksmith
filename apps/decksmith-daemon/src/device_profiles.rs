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
    pub(crate) fn same_associations(&self, other: &Self) -> bool {
        self.registry.same_associations(&other.registry)
    }
    pub fn profile(&self, id: DeviceId) -> Result<&Profile> {
        self.profiles.get(&id).ok_or("device_profile_missing")
    }
    pub fn summaries(&self) -> Vec<decksmith_device::registry::Summary<'_>> {
        self.registry.summaries()
    }
    /// Redacted discovery associations: new/ambiguous/missing identities remain
    /// unassigned. No raw serials or USB paths enter this result.
    pub fn associations(
        &self,
        discovery: &Discovery,
    ) -> Vec<(u8, decksmith_device::registry::Association)> {
        discovery
            .candidates()
            .iter()
            .map(|c| (c.slot(), self.registry.resolve(discovery, c)))
            .collect()
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
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retention {
    version: u8,
    generation: u64,
    source: PathBuf,
    archive: PathBuf,
    retired: Vec<u64>,
}
#[derive(Serialize, Debug)]
pub struct ArchiveReport {
    pub archived: usize,
    pub retained: usize,
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
    pub fn history_status(&self) -> Result<(usize, bool)> {
        Ok((
            self.inventory()?.len(),
            fs::symlink_metadata(self.root.join("retention.json")).is_ok(),
        ))
    }
    pub fn archive_sibling(&self, expected: u64, keep: usize) -> Result<ArchiveReport> {
        match fs::symlink_metadata(self.root.join("retention.json")) {
            Ok(_) => {
                let journal: Retention = serde_json::from_slice(&read(
                    &self.root.join("retention.json"),
                    REGISTRY_BYTES,
                )?)
                .map_err(|_| "invalid_profile_retention")?;
                self.resume_archive(expected, &journal.archive)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let destination = self
                    .root
                    .parent()
                    .ok_or("invalid_profile_path")?
                    .join(format!("profile-history-{}", random_device_id()?));
                self.archive_history(expected, keep, &destination)
            }
            Err(_) => Err("profile_store_io"),
        }
    }
    fn no_retention(&self) -> Result<()> {
        match fs::symlink_metadata(self.root.join("retention.json")) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            _ => Err("profile_retention_pending"),
        }
    }
    fn entries(&self, generation: u64) -> Result<Vec<(String, usize)>> {
        let dir = self.dir(generation);
        let manifest: Manifest =
            serde_json::from_slice(&read(&dir.join("manifest.json"), REGISTRY_BYTES)?)
                .map_err(|_| "invalid_profile_manifest")?;
        let mut entries = vec![
            ("manifest.json".into(), REGISTRY_BYTES),
            ("registry.json".into(), REGISTRY_BYTES),
            ("preferences.json".into(), PREFS_BYTES),
        ];
        for row in manifest.profiles {
            entries.push((format!("{}.json", row.id), LIMIT));
            if row.legacy.is_some() {
                entries.push((format!("{}-legacy.json", row.id), LIMIT));
            }
        }
        Ok(entries)
    }
    fn inventory(&self) -> Result<Vec<u64>> {
        let mut generations = Vec::new();
        for entry in fs::read_dir(self.root.join("generations")).map_err(|_| "profile_store_io")? {
            if generations.len() >= GENERATIONS {
                return Err("profile_generation_limit");
            }
            let entry = entry.map_err(|_| "profile_store_io")?;
            let name = entry.file_name();
            let name = name.to_str().ok_or("invalid_profile_generation")?;
            if name.len() != 17
                || !name.starts_with('g')
                || !name[1..]
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("invalid_profile_generation");
            }
            disk::directory(&entry.path())?;
            generations.push(
                u64::from_str_radix(&name[1..], 16).map_err(|_| "invalid_profile_generation")?,
            );
        }
        generations.sort_unstable();
        Ok(generations)
    }
    /// Explicit archive-and-prune only. Never deletes an unverified backup or
    /// adopts interrupted generations. A durable journal blocks saves during
    /// interrupted pruning; explicit resume verifies its exact destination.
    pub fn archive_history(
        &self,
        expected: u64,
        keep: usize,
        destination: &Path,
    ) -> Result<ArchiveReport> {
        self.archive_history_at(expected, keep, destination, None)
    }
    fn archive_history_at(
        &self,
        expected: u64,
        keep: usize,
        destination: &Path,
        stop: Option<u8>,
    ) -> Result<ArchiveReport> {
        let _lock = self.lock()?;
        self.no_retention()?;
        if !(2..=8).contains(&keep) {
            return Err("invalid_profile_retention");
        }
        let catalog = self.load()?;
        if catalog.generation != expected {
            return Err("stale_profile_generation");
        }
        let generations = self.inventory()?;
        if generations.len() <= keep {
            return Err("profile_archive_not_needed");
        }
        if generations.last() != Some(&expected) {
            return Err("profile_unpublished_generation");
        }
        let source = self.root.canonicalize().map_err(|_| "profile_store_io")?;
        let parent = destination
            .parent()
            .ok_or("invalid_profile_path")?
            .canonicalize()
            .map_err(|_| "profile_store_io")?;
        if parent.starts_with(&source) {
            return Err("archive_inside_profile_store");
        }
        // Validate ALL source generations and reject extra files before creating
        // the archive. An incomplete/corrupt generation needs explicit recovery.
        for g in &generations {
            self.load_generation(*g)?;
            let names = self.entries(*g)?;
            for entry in fs::read_dir(self.dir(*g)).map_err(|_| "profile_store_io")? {
                let entry = entry.map_err(|_| "profile_store_io")?;
                if !names
                    .iter()
                    .any(|(name, _)| entry.file_name() == std::ffi::OsStr::new(name))
                {
                    return Err("unexpected_profile_file");
                }
            }
        }
        let retired = generations[..generations.len() - keep].to_vec();
        let archived = Self::create(destination)?;
        for g in &retired {
            let dir = archived.dir(*g);
            disk::mkdir(&dir)?;
            for (name, limit) in self.entries(*g)? {
                disk::write(&dir.join(&name), &read(&self.dir(*g).join(name), limit)?)?;
            }
            disk::sync(&dir)?;
            archived.load_generation(*g)?;
        }
        disk::sync(&archived.root.join("generations"))?;
        disk::write(
            &archived.root.join("current.json"),
            &serde_json::to_vec(&Pointer {
                version: 1,
                generation: *retired.last().ok_or("profile_archive_not_needed")?,
            })
            .map_err(|_| "invalid_profile_pointer")?,
        )?;
        let journal = Retention {
            version: 1,
            generation: expected,
            source,
            archive: archived
                .root
                .canonicalize()
                .map_err(|_| "profile_store_io")?,
            retired,
        };
        let bytes = serde_json::to_vec(&journal).map_err(|_| "invalid_profile_retention")?;
        disk::write(&archived.root.join("retention-source.json"), &bytes)?;
        disk::sync(&archived.root)?;
        archived.load()?;
        // Archive is complete and durable before the first source unlink.
        if stop == Some(1) {
            return Err("profile_archive_prepared_not_pruned");
        }
        disk::write(&self.root.join("retention.json"), &bytes)?;
        disk::sync(&self.root).map_err(|_| "profile_retention_prepare_uncertain")?;
        self.finish_retention(&archived, &journal, stop)
    }
    pub fn resume_archive(&self, expected: u64, destination: &Path) -> Result<ArchiveReport> {
        let _lock = self.lock()?;
        let journal: Retention =
            serde_json::from_slice(&read(&self.root.join("retention.json"), REGISTRY_BYTES)?)
                .map_err(|_| "invalid_profile_retention")?;
        if journal.generation != expected {
            return Err("stale_profile_generation");
        }
        let archived = Self::open(destination)?;
        self.finish_retention(&archived, &journal, None)
    }
    fn finish_retention(
        &self,
        archived: &Self,
        journal: &Retention,
        stop: Option<u8>,
    ) -> Result<ArchiveReport> {
        if journal.version != 1
            || journal.retired.is_empty()
            || journal.retired.len() >= GENERATIONS
            || journal.source != self.root.canonicalize().map_err(|_| "profile_store_io")?
            || journal.archive
                != archived
                    .root
                    .canonicalize()
                    .map_err(|_| "profile_store_io")?
            || journal.archive.starts_with(&journal.source)
            || journal.retired.windows(2).any(|w| w[0] >= w[1])
            || journal
                .retired
                .iter()
                .any(|g| *g == 0 || *g >= journal.generation)
            || self.pointer()? != journal.generation
        {
            return Err("invalid_profile_retention");
        }
        let expected = serde_json::to_vec(journal).map_err(|_| "invalid_profile_retention")?;
        if read(&self.root.join("retention.json"), REGISTRY_BYTES)? != expected
            || read(&archived.root.join("retention-source.json"), REGISTRY_BYTES)? != expected
        {
            return Err("profile_archive_mismatch");
        }
        self.load()?;
        archived.load()?;
        if archived.pointer()? != *journal.retired.last().ok_or("invalid_profile_retention")? {
            return Err("profile_archive_mismatch");
        }
        // Validate the complete archive and every remaining source file first.
        // Missing files are allowed only here, to resume our journaled unlink.
        for g in &journal.retired {
            archived.load_generation(*g)?;
            let names = archived.entries(*g)?;
            match fs::symlink_metadata(self.dir(*g)) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err("profile_store_io"),
                Ok(_) => disk::directory(&self.dir(*g))?,
            }
            for entry in fs::read_dir(self.dir(*g)).map_err(|_| "profile_store_io")? {
                let entry = entry.map_err(|_| "profile_store_io")?;
                let (name, limit) = names
                    .iter()
                    .find(|(name, _)| entry.file_name() == std::ffi::OsStr::new(name))
                    .ok_or("unexpected_profile_file")?;
                if read(&entry.path(), *limit)? != read(&archived.dir(*g).join(name), *limit)? {
                    return Err("profile_archive_mismatch");
                }
            }
        }
        let mut unlinked = 0;
        for g in &journal.retired {
            if !self.dir(*g).try_exists().map_err(|_| "profile_store_io")? {
                continue;
            }
            for (name, _) in archived.entries(*g)? {
                match fs::remove_file(self.dir(*g).join(name)) {
                    Ok(()) => unlinked += 1,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => return Err("profile_archive_complete_prune_incomplete"),
                }
                if stop == Some(2) && unlinked == 1 {
                    return Err("profile_retention_interrupted");
                }
            }
            fs::remove_dir(self.dir(*g))
                .map_err(|_| "profile_archive_complete_prune_incomplete")?;
            disk::sync(&self.root.join("generations"))?;
        }
        fs::remove_file(self.root.join("retention.json"))
            .map_err(|_| "profile_retention_completion_uncertain")?;
        disk::sync(&self.root).map_err(|_| "profile_retention_completion_uncertain")?;
        Ok(ArchiveReport {
            archived: journal.retired.len(),
            retained: self.inventory()?.len(),
        })
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
        self.no_retention()?;
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
        if catalog.selected == Some(id) {
            return Ok(catalog.generation);
        }
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
    /// Explicit enrollment sync, called after a verified private store commit.
    /// Only the new device's settings are compiled; other input/render contexts
    /// and adapters remain valid.
    pub fn enroll_saved_profile(&mut self, catalog: &Catalog, id: DeviceId) -> Result<()> {
        let p = catalog.profile(id)?;
        let table = Config::parse(p.document.bytes())?.model_controls();
        let (count, default) = table.topology();
        let registry = Registry::decode(&catalog.registry.encode_private()?)?;
        self.manager.enroll_saved(
            registry,
            id,
            SessionLayout::new(p.document.model(), count, default)?,
            p.saved_page,
        )?;
        self.tables.insert(id, table);
        Ok(())
    }
    #[cfg(feature = "hardware")]
    pub fn discover_hardware(
        &mut self,
        inventory: &decksmith_device::hardware::Inventory,
    ) -> Result<u64> {
        self.discover(inventory.snapshot())
    }
    /// Sole physical opener path for the new supervisor. Prove saved association
    /// and exact snapshot before touching HID; attach still owns its brightness
    /// write and fences only this connection. Production does not call this yet.
    #[cfg(feature = "hardware")]
    pub fn attach_physical(
        &mut self,
        revision: u64,
        inventory: &decksmith_device::hardware::Inventory,
        slot: u8,
        owner: &mut decksmith_device::hardware::HardwareOwner,
    ) -> Result<DeviceId> {
        self.manager
            .attachment_target(revision, inventory.discovery(), slot)?;
        let device = owner
            .open(inventory, slot)
            .map_err(|_| "physical_device_unavailable")?;
        self.attach(revision, slot, Box::new(device))
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
    pub fn paint_touch(&mut self, context: DeviceContext, frame: &[u8]) -> Result<()> {
        self.manager.write_touch(&context, frame)
    }
    pub fn paint_key(&mut self, context: DeviceContext, frame: &[u8]) -> Result<()> {
        self.manager.write_key(&context, frame)
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
    /// Offline workspace browsing must never reroute a connected device.
    pub fn browse_saved_page(&mut self, id: DeviceId, expected: u64, page: u8) -> Result<u64> {
        if self
            .manager
            .statuses()
            .iter()
            .any(|s| s.id == id && s.connected)
        {
            return Err("workspace_device_is_live");
        }
        self.manager.page(id, expected, page)
    }
    pub fn fence_input(&mut self, id: DeviceId, binding: u64) -> Result<u64> {
        self.manager.invalidate_binding(id, binding)
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
    fn live_enrollment_keeps_existing_held_input_context_frame_and_page() {
        let temp = Temp::new();
        let store = Store::create(&temp.0.join("store")).unwrap();
        let g = store
            .enroll(
                0,
                &discovery(false),
                0,
                Enrollment::new(
                    id(1),
                    "Desk",
                    39,
                    Some(include_bytes!("../../../config/audio.json")),
                ),
            )
            .unwrap();
        let c = store.load().unwrap();
        let mut controls = Controls::from_catalog(&c).unwrap();
        let old_revision = controls.discover(discovery(false)).unwrap();
        let plus = Rc::new(RefCell::new(VirtualDeck::default()));
        controls
            .attach(old_revision, 0, Box::new(Probe(plus.clone())))
            .unwrap();
        controls.poll_round();
        let held = edge(&mut controls, &plus, 0, true, 1);
        let before = controls.manager.statuses();
        let frame = vec![21; 120 * 120 * 3 * 8];
        let strip = vec![22; 800 * 100 * 3];
        controls.paint(held.context, &frame, Some(&strip)).unwrap();
        store
            .enroll(
                g,
                &discovery(false),
                1,
                Enrollment::new(id(2), "MK.2", 68, None),
            )
            .unwrap();
        let c = store.load().unwrap();
        controls.enroll_saved_profile(&c, id(2)).unwrap();
        let after = controls.manager.statuses();
        assert!(controls.manager.current(&held.context));
        assert_eq!(
            (before[0].binding, before[0].connection, before[0].page),
            (after[0].binding, after[0].connection, after[0].page)
        );
        assert_eq!(
            plus.borrow().key_image(0).unwrap(),
            &vec![21; 120 * 120 * 3]
        );
        let released = edge(&mut controls, &plus, 0, false, 2);
        assert!(controls.resolve(&released).is_ok());
        assert!(controls.enroll_saved_profile(&c, id(2)).is_err());
        assert!(
            controls
                .manager
                .attachment_target(old_revision, &discovery(false), 1)
                .is_err()
        );
        let revision = controls.discover(discovery(false)).unwrap();
        assert_eq!(
            controls
                .manager
                .attachment_target(revision, &discovery(false), 1)
                .unwrap(),
            id(2)
        );
        let mk2 = Rc::new(RefCell::new(VirtualDeck::new(DeviceModel::Mk2)));
        controls
            .attach(revision, 1, Box::new(Probe(mk2.clone())))
            .unwrap();
        controls.poll_round();
        let e = edge(&mut controls, &mk2, 14, true, 3);
        assert_eq!(e.context.device, id(2));
        assert_eq!(mk2.borrow().brightness, 68);
        assert_eq!(plus.borrow().brightness, 39);
        assert_eq!(
            c.associations(&discovery(true)),
            vec![
                (0, decksmith_device::registry::Association::Assigned(id(2))),
                (1, decksmith_device::registry::Association::Assigned(id(1)))
            ]
        );
        let impostor = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("plus-private")),
            Candidate::new(1, DeviceModel::Mk2, Some("different-private")),
        ])
        .unwrap();
        assert!(
            controls
                .manager
                .attachment_target(revision, &impostor, 1)
                .is_err()
        );
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
            s.select(
                c.generation,
                if c.selected == Some(id(1)) {
                    id(2)
                } else {
                    id(1)
                },
            )
            .unwrap();
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
    fn fill(s: &Store) {
        while s.load().unwrap().generation < 16 {
            let c = s.load().unwrap();
            s.save_preferences_private(
                c.generation,
                format!("{{\"snapshot\":{}}}", c.generation + 1).as_bytes(),
            )
            .unwrap();
        }
    }
    #[test]
    fn verified_archive_reclaims_budget_and_recovers_old_identity_and_original_bytes() {
        let (temp, s) = pair();
        fill(&s);
        let before = s.load().unwrap();
        let path = temp.0.join("history");
        let report = s.archive_history(16, 8, &path).unwrap();
        assert_eq!((report.archived, report.retained), (8, 8));
        let after = s.load().unwrap();
        assert_eq!(after.generation, 16);
        assert_eq!(
            after.profile(id(1)).unwrap().document.bytes(),
            before.profile(id(1)).unwrap().document.bytes()
        );
        let archive = Store::open(&path).unwrap();
        assert_eq!(archive.inventory().unwrap(), (1..=8).collect::<Vec<_>>());
        archive
            .restore_generation(2, &temp.0.join("restored"))
            .unwrap();
        let restored = Store::open(&temp.0.join("restored"))
            .unwrap()
            .load()
            .unwrap();
        assert_eq!(
            restored.profile(id(2)).unwrap().document.model(),
            DeviceModel::Mk2
        );
        assert_eq!(
            &**restored.profile(id(1)).unwrap().legacy.as_ref().unwrap(),
            include_bytes!("../../../config/audio.json")
        );
        assert_eq!(
            s.save_preferences_private(16, br#"{"after":true}"#)
                .unwrap(),
            17
        );
        assert_eq!(s.history_status().unwrap(), (9, false));
        assert!(!path.join("retention.json").exists());
    }
    #[test]
    fn prepared_archive_failure_leaves_source_untouched_and_no_backup_is_deleted() {
        let (temp, s) = pair();
        fill(&s);
        let path = temp.0.join("history");
        assert!(s.archive_history_at(16, 8, &path, Some(1)).is_err());
        assert_eq!(s.history_status().unwrap(), (16, false));
        assert_eq!(s.load().unwrap().generation, 16);
        assert_eq!(Store::open(&path).unwrap().load().unwrap().generation, 8);
        assert!(s.dir(1).join("manifest.json").exists());
    }
    #[test]
    fn interrupted_prune_blocks_writes_and_resumes_only_verified_archive() {
        let (temp, s) = pair();
        fill(&s);
        let path = temp.0.join("history");
        assert!(s.archive_history_at(16, 8, &path, Some(2)).is_err());
        assert_eq!(s.load().unwrap().generation, 16);
        assert_eq!(
            s.save_preferences_private(16, b"{}"),
            Err("profile_retention_pending")
        );
        let other = Store::create(&temp.0.join("wrong")).unwrap();
        assert!(s.resume_archive(16, &other.root).is_err());
        assert!(s.root.join("retention.json").exists());
        let report = s.resume_archive(16, &path).unwrap();
        assert_eq!((report.archived, report.retained), (8, 8));
        assert_eq!(s.history_status().unwrap(), (8, false));
        assert!(s.save_preferences_private(16, b"{}").is_ok());
    }
    #[test]
    fn corrupt_archive_cannot_resume_pruning_or_release_write_guard() {
        let (temp, s) = pair();
        fill(&s);
        let path = temp.0.join("history");
        assert!(s.archive_history_at(16, 8, &path, Some(2)).is_err());
        let archived = Store::open(&path).unwrap();
        let file = archived.dir(1).join("preferences.json");
        let original = read(&file, PREFS_BYTES).unwrap();
        fs::write(&file, b"{\"tampered\":true}").unwrap();
        assert!(s.resume_archive(16, &path).is_err());
        assert!(s.root.join("retention.json").exists());
        assert!(s.dir(2).exists());
        assert_eq!(
            s.save_preferences_private(16, b"{}"),
            Err("profile_retention_pending")
        );
        fs::write(&file, original).unwrap();
        s.resume_archive(16, &path).unwrap();
    }
    #[test]
    fn stale_nested_incomplete_and_unexpected_history_fail_before_archiving() {
        let (temp, s) = pair();
        fill(&s);
        let path = temp.0.join("history");
        assert!(s.archive_history(15, 8, &path).is_err());
        assert!(!path.exists());
        assert!(s.archive_history(16, 1, &path).is_err());
        assert!(!path.exists());
        assert!(s.archive_history(16, 8, &s.root.join("nested")).is_err());
        assert!(!s.root.join("nested").exists());
        fs::write(s.dir(1).join("unknown"), b"preserve me").unwrap();
        assert!(s.archive_history(16, 8, &path).is_err());
        assert!(!path.exists());
        assert!(s.dir(1).join("unknown").exists());
        fs::remove_file(s.dir(1).join("unknown")).unwrap();
        fs::remove_file(s.dir(1).join("manifest.json")).unwrap();
        assert!(s.archive_history(16, 8, &path).is_err());
        assert!(!path.exists());
        assert_eq!(s.load().unwrap().generation, 16);
    }
    #[test]
    fn selection_noop_never_consumes_generation_and_archive_respects_writer_lock() {
        let (temp, s) = pair();
        let c = s.load().unwrap();
        assert_eq!(
            s.select(c.generation, c.selected.unwrap()).unwrap(),
            c.generation
        );
        fill(&s);
        let _lock = s.lock().unwrap();
        assert!(s.archive_history(16, 8, &temp.0.join("history")).is_err());
        assert!(!temp.0.join("history").exists());
    }
}
