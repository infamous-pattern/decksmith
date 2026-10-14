//! Bounded private inventory codec and explicit association. No discovery I/O or
//! implicit persistence: the host must atomically commit these bytes privately.
use decksmith_core::{device_context::DeviceId, device_model::DeviceModel};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_DEVICES: usize = decksmith_core::shared_dispatch::MAX_DEVICES;
pub const MAX_DISCOVERED: usize = 16;
pub const REGISTRY_BYTES: usize = 65_536;
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, PartialEq, Eq)]
pub struct PhysicalIdentity {
    model: DeviceModel,
    serial: String,
}
impl std::fmt::Debug for PhysicalIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalIdentity")
            .field("model", &self.model)
            .field("serial", &"[private]")
            .finish()
    }
}
impl PhysicalIdentity {
    pub fn new(model: DeviceModel, serial: &str) -> Result<Self> {
        if serial.is_empty()
            || serial.len() > 64
            || !serial
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err("invalid_device_serial");
        }
        Ok(Self {
            model,
            serial: serial.into(),
        })
    }
    pub fn model(&self) -> DeviceModel {
        self.model
    }
    #[cfg(feature = "hardware")]
    pub(crate) fn serial(&self) -> &str {
        &self.serial
    }
}

/// Snapshot-local locator, not a stable device ID. The host retains its private
/// USB-path mapping separately and must not reuse a snapshot when opening HID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    slot: u8,
    model: DeviceModel,
    identity: Option<PhysicalIdentity>,
}
impl Candidate {
    pub fn new(slot: u8, model: DeviceModel, serial: Option<&str>) -> Self {
        Self {
            slot,
            model,
            identity: serial.and_then(|s| PhysicalIdentity::new(model, s).ok()),
        }
    }
    pub fn slot(&self) -> u8 {
        self.slot
    }
    pub fn model(&self) -> DeviceModel {
        self.model
    }
}
#[derive(Debug)]
pub struct Discovery {
    candidates: Vec<Candidate>,
}
impl Discovery {
    pub fn new(candidates: Vec<Candidate>) -> Result<Self> {
        if candidates.len() > MAX_DISCOVERED
            || candidates
                .iter()
                .map(|c| c.slot)
                .collect::<BTreeSet<_>>()
                .len()
                != candidates.len()
        {
            return Err("invalid_device_discovery");
        }
        Ok(Self { candidates })
    }
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }
    fn unique(&self, candidate: &Candidate) -> bool {
        candidate.identity.as_ref().is_some_and(|identity| {
            self.candidates
                .iter()
                .filter(|c| c.identity.as_ref() == Some(identity))
                .count()
                == 1
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnassignedReason {
    MissingIdentity,
    AmbiguousIdentity,
    NewDevice,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum Association {
    Assigned(DeviceId),
    Unassigned(UnassignedReason),
}
#[derive(Debug, Clone)]
pub struct Record {
    id: DeviceId,
    name: String,
    brightness: u8,
    identity: PhysicalIdentity,
}
impl Record {
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        self.id == other.id && self.identity == other.identity
    }
    pub fn id(&self) -> DeviceId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn model(&self) -> DeviceModel {
        self.identity.model
    }
    pub fn brightness(&self) -> u8 {
        self.brightness
    }
}
#[derive(Debug, Default, Clone)]
pub struct Registry {
    revision: u64,
    records: Vec<Record>,
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.len() <= 256
        && name.chars().count() <= 64
        && !name.chars().any(char::is_control)
}
// These wire types are private. Public summaries never serialize serials.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRecord {
    id: DeviceId,
    name: String,
    brightness: u8,
    model: DeviceModel,
    serial: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u8,
    revision: u64,
    devices: Vec<WireRecord>,
}
#[derive(Serialize)]
pub struct Summary<'a> {
    pub id: DeviceId,
    pub name: &'a str,
    pub model: DeviceModel,
    pub brightness: u8,
}
impl Registry {
    /// Compare private associations without exposing them or comparing editable
    /// names/brightness. Live execution cannot adopt a reassigned identity.
    pub fn same_associations(&self, other: &Self) -> bool {
        self.records.len() == other.records.len()
            && self
                .records
                .iter()
                .all(|r| other.get(r.id).is_some_and(|o| r.same_identity(o)))
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    pub fn get(&self, id: DeviceId) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }
    pub fn summaries(&self) -> Vec<Summary<'_>> {
        self.records
            .iter()
            .map(|r| Summary {
                id: r.id,
                name: &r.name,
                model: r.model(),
                brightness: r.brightness,
            })
            .collect()
    }
    pub fn resolve(&self, discovery: &Discovery, candidate: &Candidate) -> Association {
        // Require the candidate to belong to this exact snapshot, not just share
        // a serial with it. The manager adds a snapshot revision to attachment.
        if !discovery.candidates.iter().any(|c| {
            c.slot == candidate.slot
                && c.model == candidate.model
                && c.identity == candidate.identity
        }) || !discovery.unique(candidate)
        {
            return Association::Unassigned(if candidate.identity.is_none() {
                UnassignedReason::MissingIdentity
            } else {
                UnassignedReason::AmbiguousIdentity
            });
        }
        match self
            .records
            .iter()
            .find(|r| Some(&r.identity) == candidate.identity.as_ref())
        {
            Some(record) => Association::Assigned(record.id),
            None => Association::Unassigned(UnassignedReason::NewDevice),
        }
    }
    /// Explicit enrollment only. A missing/duplicate serial cannot be persisted
    /// safely in this slice; show unassigned rather than guess by path/order.
    pub fn associate(
        &mut self,
        expected: u64,
        id: DeviceId,
        name: &str,
        brightness: u8,
        discovery: &Discovery,
        slot: u8,
    ) -> Result<()> {
        if expected != self.revision {
            return Err("stale_registry_revision");
        }
        let candidate = discovery
            .candidates
            .iter()
            .find(|c| c.slot == slot)
            .ok_or("unknown_candidate")?;
        if self.resolve(discovery, candidate)
            != Association::Unassigned(UnassignedReason::NewDevice)
        {
            return Err("device_association_unavailable");
        }
        if self.records.len() >= MAX_DEVICES {
            return Err("device_registry_full");
        }
        if self.get(id).is_some() || !valid_name(name) || brightness > 100 {
            return Err("invalid_device_record");
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("registry_revision_exhausted")?;
        self.records.push(Record {
            id,
            name: name.into(),
            brightness,
            identity: candidate.identity.clone().ok_or("invalid_device_serial")?,
        });
        self.revision = revision;
        Ok(())
    }
    /// Metadata changes retain the exact physical association and stable ID.
    pub fn update(
        &mut self,
        expected: u64,
        id: DeviceId,
        name: &str,
        brightness: u8,
    ) -> Result<()> {
        if expected != self.revision {
            return Err("stale_registry_revision");
        }
        if !valid_name(name) || brightness > 100 {
            return Err("invalid_device_record");
        }
        let index = self
            .records
            .iter()
            .position(|r| r.id == id)
            .ok_or("unknown_device")?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("registry_revision_exhausted")?;
        self.records[index].name = name.into();
        self.records[index].brightness = brightness;
        self.revision = revision;
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > REGISTRY_BYTES {
            return Err("device_registry_too_large");
        }
        let wire: Wire = serde_json::from_slice(bytes).map_err(|_| "invalid_device_registry")?;
        if wire.version != 1
            || wire.devices.len() > MAX_DEVICES
            || wire.revision < wire.devices.len() as u64
        {
            return Err("invalid_device_registry");
        }
        let mut registry = Self {
            revision: wire.revision,
            records: Vec::new(),
        };
        for r in wire.devices {
            let identity = PhysicalIdentity::new(r.model, &r.serial)?;
            if !valid_name(&r.name)
                || r.brightness > 100
                || registry.get(r.id).is_some()
                || registry.records.iter().any(|old| old.identity == identity)
            {
                return Err("invalid_device_record");
            }
            registry.records.push(Record {
                id: r.id,
                name: r.name,
                brightness: r.brightness,
                identity,
            });
        }
        Ok(registry)
    }
    /// Private persistence bytes only. Never put this in layout exports/status,
    /// logs, crash reports or a public repository. No filesystem access here.
    pub fn encode_private(&self) -> Result<Vec<u8>> {
        let devices = self
            .records
            .iter()
            .map(|r| WireRecord {
                id: r.id,
                name: r.name.clone(),
                brightness: r.brightness,
                model: r.model(),
                serial: r.identity.serial.clone(),
            })
            .collect();
        let bytes = serde_json::to_vec(&Wire {
            version: 1,
            revision: self.revision,
            devices,
        })
        .map_err(|_| "invalid_device_registry")?;
        if bytes.len() > REGISTRY_BYTES {
            return Err("device_registry_too_large");
        }
        Ok(bytes)
    }
}
