//! Avoid repeated full HID enumeration when Linux's HID topology is unchanged.
//! This only caches discovery: HardwareOwner still revalidates every actual open.
use super::Inventory;
use crate::DeviceError;
use std::{
    io,
    time::{Duration, Instant},
};

const FULL_SCAN_INTERVAL: Duration = Duration::from_secs(30);

// Deliberately not Debug/Serialize: topology and device metadata stay private.
#[derive(PartialEq, Eq)]
struct Fingerprint(Vec<TopologyNode>);

#[derive(PartialEq, Eq)]
struct TopologyNode {
    name: std::ffi::OsString,
    target: std::path::PathBuf,
    properties: Vec<u8>,
    node: Option<NodeMetadata>,
}

#[derive(PartialEq, Eq)]
struct NodeMetadata {
    inode: u64,
    device: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    rdev: u64,
    changed: (i64, i64),
}

/// The caller keeps its existing scan cadence. A cheap Linux topology check
/// triggers full enumeration on change, failure, or at least every 30 seconds.
/// Missing sysfs/non-Linux falls back to uncached full discovery each time.
#[derive(Default)]
pub struct InventoryCache {
    inventory: Option<Inventory>,
    fingerprint: Option<Fingerprint>,
    refreshed: Option<Instant>,
}

impl InventoryCache {
    /// The exact inventory returned by the last successful scan. Opening a slot
    /// must use this snapshot, not silently replace it under the same sequence.
    pub fn current(&self) -> Option<&Inventory> {
        self.inventory.as_ref()
    }

    pub fn refresh(&mut self) -> Result<&Inventory, DeviceError> {
        self.refresh_with(Instant::now(), fingerprint(), Inventory::discover)
    }

    fn refresh_with(
        &mut self,
        now: Instant,
        fingerprint: io::Result<Fingerprint>,
        discover: impl FnOnce() -> Result<Inventory, DeviceError>,
    ) -> Result<&Inventory, DeviceError> {
        let fingerprint = fingerprint.ok();
        if self.inventory.is_none()
            || fingerprint.is_none()
            || fingerprint != self.fingerprint
            || self
                .refreshed
                .is_none_or(|t| now.duration_since(t) >= FULL_SCAN_INTERVAL)
        {
            // A failed refresh must never admit stale handles or delay retry.
            self.inventory = None;
            self.fingerprint = None;
            self.refreshed = None;
            self.inventory = Some(discover()?);
            // Keep the pre-enumeration fingerprint: a concurrent topology
            // change is detected on the next poll, rather than hidden by cache.
            self.fingerprint = fingerprint;
            self.refreshed = Some(now);
        }
        Ok(self
            .inventory
            .as_ref()
            .expect("successful inventory refresh"))
    }
}

#[cfg(target_os = "linux")]
fn fingerprint() -> io::Result<Fingerprint> {
    fingerprint_at(
        std::path::Path::new("/sys/class/hidraw"),
        std::path::Path::new("/dev"),
    )
}

#[cfg(not(target_os = "linux"))]
fn fingerprint() -> io::Result<Fingerprint> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "no topology probe",
    ))
}

#[cfg(target_os = "linux")]
fn fingerprint_at(class: &std::path::Path, devices: &std::path::Path) -> io::Result<Fingerprint> {
    use std::{fs, io::Read, os::unix::fs::MetadataExt};
    let mut entries = Vec::new();
    for entry in fs::read_dir(class)? {
        let entry = entry?;
        // Bound memory and file reads even on an unexpectedly large HID tree.
        if entries.len() >= 256 {
            return Err(io::Error::other("HID topology exceeds probe bound"));
        }
        let target = fs::read_link(entry.path())?;
        let mut properties = Vec::new();
        fs::File::open(entry.path().join("device/uevent"))?
            .take(16385)
            .read_to_end(&mut properties)?;
        if properties.len() > 16384 {
            return Err(io::Error::other("HID properties exceed probe bound"));
        }
        let node = match fs::metadata(devices.join(entry.file_name())) {
            Ok(m) => Some(NodeMetadata {
                inode: m.ino(),
                device: m.dev(),
                mode: m.mode(),
                uid: m.uid(),
                gid: m.gid(),
                rdev: m.rdev(),
                changed: (m.ctime(), m.ctime_nsec()),
            }),
            // udev may create the node after the sysfs entry. Its appearance
            // changes this fingerprint; other failures force full enumeration.
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        // Include the hidraw name as well as its target to detect renumbering.
        entries.push(TopologyNode {
            name: entry.file_name(),
            target,
            properties,
            node,
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Fingerprint(entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(n: u8) -> io::Result<Fingerprint> {
        Ok(Fingerprint(vec![TopologyNode {
            name: "hidraw0".into(),
            target: "usb-a".into(),
            properties: vec![n],
            node: None,
        }]))
    }
    fn empty() -> Result<Inventory, DeviceError> {
        Inventory::from_locators([])
    }

    #[test]
    fn unchanged_topology_reuses_inventory_but_periodic_scan_is_not_postponed() {
        let start = Instant::now();
        let mut cache = InventoryCache::default();
        cache.refresh_with(start, stamp(1), empty).unwrap();
        for second in 1..30 {
            cache
                .refresh_with(start + Duration::from_secs(second), stamp(1), || {
                    panic!("unchanged topology must not enumerate")
                })
                .unwrap();
        }
        let mut scanned = false;
        cache
            .refresh_with(start + FULL_SCAN_INTERVAL, stamp(1), || {
                scanned = true;
                empty()
            })
            .unwrap();
        assert!(scanned);
    }

    #[test]
    fn changed_or_unreadable_topology_refreshes_and_failed_scan_retries_immediately() {
        let now = Instant::now();
        let mut cache = InventoryCache::default();
        cache.refresh_with(now, stamp(1), empty).unwrap();
        assert!(
            cache
                .refresh_with(now, stamp(2), || Err(DeviceError::Transport))
                .is_err()
        );
        assert!(cache.inventory.is_none());
        let mut scans = 0;
        for fingerprint in [
            stamp(2),
            Err(io::Error::other("sysfs unavailable")),
            stamp(2),
        ] {
            cache
                .refresh_with(now, fingerprint, || {
                    scans += 1;
                    empty()
                })
                .unwrap();
        }
        assert_eq!(scans, 3);
        // Even with a previously successful inventory, an unavailable probe
        // must use the original full-scan behavior on every request.
        for _ in 0..2 {
            cache
                .refresh_with(now, Err(io::Error::other("sysfs unavailable")), || {
                    scans += 1;
                    empty()
                })
                .unwrap();
        }
        assert_eq!(scans, 5);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_probe_detects_replug_metadata_and_node_permission_changes() {
        use std::{
            fs,
            os::unix::fs::{PermissionsExt, symlink},
        };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "decksmith-hid-probe-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let class = root.join("class");
        let dev = root.join("dev");
        fs::create_dir_all(&class).unwrap();
        fs::create_dir_all(&dev).unwrap();
        for name in ["usb-a", "usb-b"] {
            fs::create_dir_all(root.join(name).join("device")).unwrap();
            fs::write(
                root.join(name).join("device/uevent"),
                b"HID_ID=0003:00000FD9:00000084\nHID_UNIQ=test\n",
            )
            .unwrap();
        }
        symlink("../usb-a", class.join("hidraw0")).unwrap();
        let absent = fingerprint_at(&class, &dev).unwrap();
        fs::write(dev.join("hidraw0"), []).unwrap();
        let before = fingerprint_at(&class, &dev).unwrap();
        assert!(absent != before);
        fs::set_permissions(dev.join("hidraw0"), fs::Permissions::from_mode(0o600)).unwrap();
        // Choose a different mode regardless of the process umask.
        fs::set_permissions(dev.join("hidraw0"), fs::Permissions::from_mode(0o400)).unwrap();
        assert!(before != fingerprint_at(&class, &dev).unwrap());
        let before = fingerprint_at(&class, &dev).unwrap();
        fs::remove_file(class.join("hidraw0")).unwrap();
        symlink("../usb-b", class.join("hidraw0")).unwrap();
        assert!(before != fingerprint_at(&class, &dev).unwrap());
        let before = fingerprint_at(&class, &dev).unwrap();
        fs::write(root.join("usb-b/device/uevent"), b"HID_UNIQ=replacement\n").unwrap();
        assert!(before != fingerprint_at(&class, &dev).unwrap());
        fs::remove_file(class.join("hidraw0")).unwrap();
        assert!(fingerprint_at(&class, &dev).unwrap().0.is_empty());
        fs::remove_dir_all(root).unwrap();
    }
}
