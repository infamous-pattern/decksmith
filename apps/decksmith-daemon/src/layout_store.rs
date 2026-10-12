//! Offline migration generations. Never reads the live config path implicitly.
use crate::layout_document::{Document, LIMIT};
use decksmith_core::device_model::DeviceModel;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
};

const MAX_FILES: usize = 256;
const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_GENERATIONS: usize = 16;
type Result<T> = std::result::Result<T, &'static str>;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn io<T>(result: std::io::Result<T>) -> Result<T> {
    result.map_err(|_| "layout_store_io")
}
fn own(meta: &fs::Metadata) -> bool {
    meta.uid() == unsafe { libc::geteuid() } && meta.mode() & 0o022 == 0
}
fn directory(path: &Path) -> Result<()> {
    let meta = io(fs::symlink_metadata(path))?;
    if !meta.is_dir() || !own(&meta) {
        return Err("unsafe_layout_directory");
    }
    Ok(())
}
fn read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut file = io(OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path))?;
    let meta = io(file.metadata())?;
    if !meta.is_file() || !own(&meta) || meta.len() > limit as u64 {
        return Err("unsafe_layout_file");
    }
    let mut bytes = Vec::new();
    io((&mut file).take(limit as u64 + 1).read_to_end(&mut bytes))?;
    if bytes.len() > limit {
        return Err("layout_snapshot_too_large");
    }
    Ok(bytes)
}
fn mkdir(path: &Path) -> Result<()> {
    io(fs::DirBuilder::new().mode(0o700).create(path))
}
fn sync(path: &Path) -> Result<()> {
    io(io(File::open(path))?.sync_all())
}
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = io(OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path))?;
    io(file.write_all(bytes))?;
    io(file.sync_all())
}
fn relative(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && Path::new(name)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

struct LegacyFile {
    name: String,
    bytes: Vec<u8>,
}
struct Snapshot {
    files: Vec<LegacyFile>,
}
impl Snapshot {
    fn capture(root: &Path) -> Result<Self> {
        fn walk(
            root: &Path,
            at: &Path,
            depth: usize,
            files: &mut Vec<LegacyFile>,
            total: &mut usize,
            nodes: &mut usize,
        ) -> Result<()> {
            directory(at)?;
            if depth > 8 {
                return Err("layout_snapshot_too_deep");
            }
            for entry in io(fs::read_dir(at))? {
                *nodes += 1;
                if *nodes > 512 {
                    return Err("layout_snapshot_too_many_entries");
                }
                let path = io(entry)?.path();
                let meta = io(fs::symlink_metadata(&path))?;
                if meta.is_dir() {
                    walk(root, &path, depth + 1, files, total, nodes)?;
                } else {
                    if files.len() >= MAX_FILES {
                        return Err("layout_snapshot_too_many_files");
                    }
                    let bytes = read(&path, 2 * LIMIT)?;
                    *total += bytes.len();
                    if *total > MAX_BYTES {
                        return Err("layout_snapshot_too_large");
                    }
                    let name = path
                        .strip_prefix(root)
                        .map_err(|_| "invalid_snapshot_path")?
                        .to_str()
                        .ok_or("invalid_snapshot_path")?
                        .to_owned();
                    if !relative(&name) {
                        return Err("invalid_snapshot_path");
                    }
                    files.push(LegacyFile { name, bytes });
                }
            }
            Ok(())
        }
        let mut files = Vec::new();
        walk(root, root, 0, &mut files, &mut 0, &mut 0)?;
        files.sort_by(|a, b| a.name.cmp(&b.name));
        let snapshot = Self { files };
        let original = Document::for_model(snapshot.layout()?, DeviceModel::Plus)?;
        if original.version() != 1 {
            return Err("migration_requires_legacy_plus");
        }
        Ok(snapshot)
    }
    fn layout(&self) -> Result<&[u8]> {
        self.files
            .iter()
            .find(|file| file.name == "layout.json")
            .map(|file| file.bytes.as_slice())
            .ok_or("legacy_layout_missing")
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    sha256: String,
    bytes: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    generation: u64,
    plus: Entry,
    mk2: Entry,
    legacy: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    version: u8,
    generation: u64,
}
struct Store {
    root: PathBuf,
}
/// Explicit unlock prevents an unrelated fork/duplicated descriptor from
/// retaining this file-description lock after the writer's own fd is dropped.
struct WriterLock(File);
impl WriterLock {
    fn acquire(file: File) -> Result<Self> {
        file.try_lock().map_err(|_| "layout_store_busy")?;
        Ok(Self(file))
    }
}
impl Drop for WriterLock {
    fn drop(&mut self) {
        if self.0.unlock().is_err() {
            tracing::warn!(
                error_code = "layout_unlock_failed",
                "Configuration writer lock release failed"
            );
        }
    }
}
impl Store {
    fn open(root: &Path) -> Result<Self> {
        directory(root)?;
        if io(fs::metadata(root))?.mode() & 0o077 != 0 {
            return Err("layout_store_not_private");
        }
        directory(&root.join("generations"))?;
        Ok(Self { root: root.into() })
    }
    fn create(root: &Path) -> Result<Self> {
        // Exclusive creation: never adopt or overwrite a foreign destination.
        mkdir(root)?;
        mkdir(&root.join("generations"))?;
        sync(root)?;
        sync(root.parent().ok_or("invalid_store_path")?)?;
        Self::open(root)
    }
    fn pointer(&self) -> Result<Option<u64>> {
        let path = self.root.join("current.json");
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err("layout_store_io"),
            Ok(_) => {
                let pointer: Pointer = serde_json::from_slice(&read(&path, 4096)?)
                    .map_err(|_| "invalid_store_pointer")?;
                if pointer.version != 1 || pointer.generation == 0 {
                    return Err("invalid_store_pointer");
                }
                Ok(Some(pointer.generation))
            }
        }
    }
    fn generation(&self, number: u64) -> PathBuf {
        self.root
            .join("generations")
            .join(format!("g{number:016x}"))
    }
    fn next(&self) -> Result<u64> {
        let mut max = 0;
        for (count, entry) in io(fs::read_dir(self.root.join("generations")))?.enumerate() {
            if count + 1 >= MAX_GENERATIONS {
                return Err("layout_generation_limit");
            }
            let path = io(entry)?.path();
            directory(&path)?;
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("invalid_generation")?;
            if name.len() != 17 || !name.starts_with('g') {
                return Err("invalid_generation");
            }
            let number = u64::from_str_radix(&name[1..], 16).map_err(|_| "invalid_generation")?;
            max = max.max(number);
        }
        max.checked_add(1).ok_or("layout_generation_overflow")
    }
    fn commit(&self, snapshot: &Snapshot, expected: Option<u64>, stop: Option<u8>) -> Result<u64> {
        let lock = io(OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.root.join("store.lock")))?;
        let meta = io(lock.metadata())?;
        if !meta.is_file() || !own(&meta) || meta.nlink() != 1 {
            return Err("unsafe_layout_file");
        }
        let _lock = WriterLock::acquire(lock)?;
        if self.pointer()? != expected {
            return Err("stale_layout_generation");
        }
        if let Some(previous) = expected {
            self.load(previous)?;
        }
        let plus = Document::for_model(snapshot.layout()?, DeviceModel::Plus)?.upgraded()?;
        let mk2 = Document::mk2_defaults()?;
        let generation = self.next()?;
        let dir = self.generation(generation);
        mkdir(&dir)?;
        write(&dir.join("plus.json"), plus.bytes())?;
        write(&dir.join("mk2.json"), mk2.bytes())?;
        if stop == Some(1) {
            return Err("migration_interrupted");
        }
        mkdir(&dir.join("legacy"))?;
        for file in &snapshot.files {
            let path = dir.join("legacy").join(&file.name);
            make_parents(
                &dir.join("legacy"),
                path.parent().ok_or("invalid_snapshot_path")?,
            )?;
            write(&path, &file.bytes)?;
        }
        let manifest = Manifest {
            version: 1,
            generation,
            plus: entry("plus.json", plus.bytes()),
            mk2: entry("mk2.json", mk2.bytes()),
            legacy: snapshot
                .files
                .iter()
                .map(|file| entry(&file.name, &file.bytes))
                .collect(),
        };
        write(
            &dir.join("manifest.json"),
            &serde_json::to_vec(&manifest).map_err(|_| "invalid_manifest")?,
        )?;
        if stop == Some(2) {
            return Err("migration_interrupted");
        }
        sync_tree(&dir)?;
        sync(&self.root.join("generations"))?;
        // Verify the complete generation before publishing the pointer.
        self.load(generation)?;
        if stop == Some(3) {
            return Err("migration_interrupted");
        }
        let temp = self.root.join(format!("current-{generation:016x}.tmp"));
        write(
            &temp,
            &serde_json::to_vec(&Pointer {
                version: 1,
                generation,
            })
            .map_err(|_| "invalid_store_pointer")?,
        )?;
        io(fs::rename(temp, self.root.join("current.json")))?;
        // A sync failure after rename is an uncertain commit, never auto-retried.
        sync(&self.root).map_err(|_| "layout_commit_uncertain")?;
        Ok(generation)
    }
    fn load(&self, number: u64) -> Result<Snapshot> {
        let dir = self.generation(number);
        directory(&dir)?;
        let manifest: Manifest =
            serde_json::from_slice(&read(&dir.join("manifest.json"), 512 * 1024)?)
                .map_err(|_| "invalid_manifest")?;
        if manifest.version != 1
            || manifest.generation != number
            || manifest.legacy.len() > MAX_FILES
            || manifest.plus.path != "plus.json"
            || manifest.mk2.path != "mk2.json"
        {
            return Err("invalid_manifest");
        }
        let plus = checked(&dir, &manifest.plus, LIMIT)?;
        let mk2 = checked(&dir, &manifest.mk2, LIMIT)?;
        let plus_doc = Document::for_model(&plus, DeviceModel::Plus)?;
        let mk2_doc = Document::for_model(&mk2, DeviceModel::Mk2)?;
        if plus_doc.version() != 2 || mk2_doc.version() != 2 {
            return Err("invalid_manifest");
        }
        let mut seen = BTreeSet::new();
        let mut files = Vec::new();
        let mut total = 0;
        directory(&dir.join("legacy"))?;
        for entry in &manifest.legacy {
            if !seen.insert(entry.path.clone()) {
                return Err("invalid_manifest");
            }
            let bytes = checked(&dir.join("legacy"), entry, 2 * LIMIT)?;
            total += bytes.len();
            if total > MAX_BYTES {
                return Err("layout_snapshot_too_large");
            }
            files.push(LegacyFile {
                name: entry.path.clone(),
                bytes,
            });
        }
        let snapshot = Snapshot { files };
        let original = Document::for_model(snapshot.layout()?, DeviceModel::Plus)?;
        if original.version() != 1 || original.upgraded()?.bytes() != plus {
            return Err("layout_backup_mismatch");
        }
        Ok(snapshot)
    }
}
fn entry(path: &str, bytes: &[u8]) -> Entry {
    Entry {
        path: path.into(),
        sha256: hash(bytes),
        bytes: bytes.len(),
    }
}
fn checked(root: &Path, entry: &Entry, limit: usize) -> Result<Vec<u8>> {
    if !relative(&entry.path) || entry.bytes > limit {
        return Err("invalid_manifest");
    }
    let path = root.join(&entry.path);
    let mut parent = path.parent().ok_or("invalid_manifest")?;
    while parent != root {
        directory(parent)?;
        parent = parent.parent().ok_or("invalid_manifest")?;
    }
    let bytes = read(&path, limit)?;
    if bytes.len() != entry.bytes || hash(&bytes) != entry.sha256 {
        return Err("layout_checksum_failed");
    }
    Ok(bytes)
}
fn make_parents(root: &Path, path: &Path) -> Result<()> {
    let tail = path
        .strip_prefix(root)
        .map_err(|_| "invalid_snapshot_path")?;
    let mut parent = root.to_path_buf();
    for part in tail.components() {
        if !matches!(part, Component::Normal(_)) {
            return Err("invalid_snapshot_path");
        }
        parent.push(part);
        if !parent.exists() {
            mkdir(&parent)?;
        }
        directory(&parent)?;
    }
    Ok(())
}
fn sync_tree(root: &Path) -> Result<()> {
    for entry in io(fs::read_dir(root))? {
        let path = io(entry)?.path();
        if io(fs::symlink_metadata(&path))?.is_dir() {
            sync_tree(&path)?;
        }
    }
    sync(root)
}

pub fn prepare(source: &Path, destination: &Path) -> Result<u64> {
    directory(source)?;
    let source = io(source.canonicalize())?;
    directory(&source)?;
    let parent = io(destination
        .parent()
        .ok_or("invalid_store_path")?
        .canonicalize())?;
    if parent.starts_with(&source) {
        return Err("store_inside_legacy_config");
    }
    let snapshot = Snapshot::capture(&source)?;
    // Re-read to reject changing source/configuration during snapshot capture.
    let second = Snapshot::capture(&source)?;
    if snapshot.files.len() != second.files.len()
        || snapshot
            .files
            .iter()
            .zip(second.files.iter())
            .any(|(a, b)| a.name != b.name || a.bytes != b.bytes)
    {
        return Err("legacy_config_changed");
    }
    drop(second);
    let store = Store::create(destination)?;
    store.commit(&snapshot, None, None)
}
pub fn restore(source: &Path, destination: &Path) -> Result<()> {
    let parent = io(destination
        .parent()
        .ok_or("invalid_store_path")?
        .canonicalize())?;
    if parent.starts_with(io(source.canonicalize())?) {
        return Err("restore_inside_layout_store");
    }
    let store = Store::open(source)?;
    let generation = store.pointer()?.ok_or("layout_store_incomplete")?;
    let snapshot = store.load(generation)?;
    mkdir(destination)?;
    for file in snapshot.files {
        let path = destination.join(&file.name);
        make_parents(destination, path.parent().ok_or("invalid_snapshot_path")?)?;
        write(&path, &file.bytes)?;
    }
    sync_tree(destination)?;
    sync(destination.parent().ok_or("invalid_store_path")?)
}

/// Explicit development commands only; normal startup does not migrate state.
/// Reports contain counts/identity metadata, never settings, credentials or labels.
pub fn request(args: &[String]) -> Option<Result<String>> {
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();
    let result = match args.as_slice() {
        ["--layout-info", path] => (|| {
            let document = Document::parse(&read(Path::new(path),LIMIT)?)?;
            let identities = document.bindings()?.into_iter().map(|binding| (binding.provider,binding.action,binding.schema)).collect::<BTreeSet<_>>();
            Ok(serde_json::json!({"version":document.version(),"model":document.model(),
                "descriptor":document.descriptor(),"action_definitions":identities.len()}).to_string())
        })(),
        ["--prepare-layout-store", source, destination] => prepare(Path::new(source),Path::new(destination))
            .map(|generation| serde_json::json!({"generation":generation,"profiles":["legacy-plus","mk2-default"],"activated":false}).to_string()),
        ["--restore-legacy-layout-store", source, destination] => restore(Path::new(source),Path::new(destination))
            .map(|()| serde_json::json!({"restored":true,"activated":false}).to_string()),
        _ => return None,
    };
    Some(result)
}

#[cfg(test)]
#[path = "layout_store_tests.rs"]
mod tests;
