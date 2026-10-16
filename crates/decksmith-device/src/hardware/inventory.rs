//! Read-only, bounded HID inventory. USB paths/serials never enter public status.
use super::{PhysicalDeck, UsbTransport};
use crate::{
    DeviceError,
    registry::{Candidate, Discovery, MAX_DEVICES, MAX_DISCOVERED, PhysicalIdentity},
};
use decksmith_core::device_model::DeviceModel;
use elgato_streamdeck::{StreamDeck, info::Kind};
use std::sync::{Arc, Mutex};

mod cache;
pub use cache::InventoryCache;

struct Locator {
    model: DeviceModel,
    identity: Option<PhysicalIdentity>,
    path: Vec<u8>,
}
impl Locator {
    fn candidate(&self, slot: u8) -> Candidate {
        Candidate::new(slot, self.model, self.identity.as_ref().map(|i| i.serial()))
    }
}
/// Slots belong only to this snapshot. Resolve with the saved registry before
/// opening; enrollment of a new device remains an explicit caller decision.
pub struct Inventory {
    locators: Vec<Locator>,
    discovery: Discovery,
}
impl std::fmt::Debug for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inventory")
            .field("discovery", &self.discovery)
            .finish()
    }
}
impl Inventory {
    pub fn discover() -> Result<Self, DeviceError> {
        let api = elgato_streamdeck::new_hidapi().map_err(|_| DeviceError::Transport)?;
        Self::from_locators(api.device_list().filter_map(|d| {
            let model = match Kind::from_vid_pid(d.vendor_id(), d.product_id())? {
                Kind::Plus => DeviceModel::Plus,
                Kind::Mk2 => DeviceModel::Mk2,
                _ => return None,
            };
            Some(Locator {
                model,
                identity: d
                    .serial_number()
                    .and_then(|s| PhysicalIdentity::new(model, s).ok()),
                path: d.path().to_bytes().to_vec(),
            })
        }))
    }
    fn from_locators(locators: impl IntoIterator<Item = Locator>) -> Result<Self, DeviceError> {
        // Do not let enumeration size allocate an unbounded inventory.
        let mut locators: Vec<_> = locators.into_iter().take(MAX_DISCOVERED + 1).collect();
        if locators.len() > MAX_DISCOVERED {
            return Err(DeviceError::DeviceUnavailable);
        }
        // Retain duplicates: upstream list_devices deduplicates equal serials,
        // which hides ambiguity that must instead block association/opening.
        locators.sort_by(|a, b| a.path.cmp(&b.path));
        let discovery = Discovery::new(
            locators
                .iter()
                .enumerate()
                .map(|(slot, l)| l.candidate(slot as u8))
                .collect(),
        )
        .map_err(|_| DeviceError::DeviceUnavailable)?;
        Ok(Self {
            locators,
            discovery,
        })
    }
    pub fn discovery(&self) -> &Discovery {
        &self.discovery
    }
    pub fn snapshot(&self) -> Discovery {
        Discovery::new(self.discovery.candidates().to_vec()).expect("validated inventory")
    }
    fn selected(&self, slot: u8) -> Result<&Locator, DeviceError> {
        let l = self
            .locators
            .get(usize::from(slot))
            .ok_or(DeviceError::DeviceUnavailable)?;
        let identity = l.identity.as_ref().ok_or(DeviceError::DeviceUnavailable)?;
        if self
            .locators
            .iter()
            .filter(|other| other.identity.as_ref() == Some(identity))
            .count()
            != 1
            || self
                .locators
                .iter()
                .filter(|other| other.path == l.path)
                .count()
                != 1
        {
            return Err(DeviceError::DeviceUnavailable);
        }
        Ok(l)
    }
    fn revalidate(&self, old: &Locator) -> Result<(), DeviceError> {
        let slot = self
            .locators
            .iter()
            .position(|l| l.identity == old.identity)
            .ok_or(DeviceError::DeviceUnavailable)?;
        let current = self.selected(slot as u8)?;
        if current.model != old.model || current.path != old.path {
            return Err(DeviceError::DeviceUnavailable);
        }
        Ok(())
    }
}
/// One cooperative process-wide lease shared by all its live physical sessions.
/// This is not an OS HID exclusivity guarantee against unrelated software.
pub struct HardwareOwner {
    lease: Arc<std::fs::File>,
    claims: Arc<Mutex<Vec<PhysicalIdentity>>>,
}
pub(super) struct Claim {
    identity: PhysicalIdentity,
    claims: Arc<Mutex<Vec<PhysicalIdentity>>>,
}
impl Drop for Claim {
    fn drop(&mut self) {
        // Poison blocks future opens; never panic while unwinding a failed open.
        if let Ok(mut claims) = self.claims.lock() {
            claims.retain(|i| i != &self.identity);
        }
    }
}
impl HardwareOwner {
    pub fn acquire() -> Result<Self, DeviceError> {
        Ok(Self {
            lease: Arc::new(crate::ownership::acquire()?),
            claims: Arc::default(),
        })
    }
    fn claim(&self, identity: &PhysicalIdentity) -> Result<Claim, DeviceError> {
        let mut claims = self.claims.lock().map_err(|_| DeviceError::DeviceBusy)?;
        if claims.len() >= MAX_DEVICES || claims.contains(identity) {
            return Err(DeviceError::DeviceUnavailable);
        }
        claims.push(identity.clone());
        Ok(Claim {
            identity: identity.clone(),
            claims: self.claims.clone(),
        })
    }
    /// Opens only an unchanged unique identity from the supplied snapshot.
    /// No reset, brightness or image write. Do not call on the GTK thread.
    pub fn open(&mut self, inventory: &Inventory, slot: u8) -> Result<PhysicalDeck, DeviceError> {
        let locator = inventory.selected(slot)?;
        Inventory::discover()?.revalidate(locator)?;
        let identity = locator
            .identity
            .as_ref()
            .ok_or(DeviceError::DeviceUnavailable)?;
        let claim = self.claim(identity)?;
        let api = elgato_streamdeck::new_hidapi().map_err(|_| DeviceError::Transport)?;
        // Recheck the exact enumeration used by open_serial, not just the earlier
        // probe. The dependency exposes no safe constructor for an opened path.
        let now = Inventory::from_locators(api.device_list().filter_map(|d| {
            let model = match Kind::from_vid_pid(d.vendor_id(), d.product_id())? {
                Kind::Plus => DeviceModel::Plus,
                Kind::Mk2 => DeviceModel::Mk2,
                _ => return None,
            };
            Some(Locator {
                model,
                identity: d
                    .serial_number()
                    .and_then(|s| PhysicalIdentity::new(model, s).ok()),
                path: d.path().to_bytes().to_vec(),
            })
        }))?;
        now.revalidate(locator)?;
        let kind = match locator.model {
            DeviceModel::Plus => Kind::Plus,
            DeviceModel::Mk2 => Kind::Mk2,
        };
        let deck = StreamDeck::connect(&api, kind, identity.serial())
            .map_err(|_| DeviceError::Transport)?;
        let actual = deck.serial_number().map_err(|_| DeviceError::Transport)?;
        if actual != identity.serial() {
            return Err(DeviceError::DeviceUnavailable);
        }
        Inventory::discover()?.revalidate(locator)?;
        let mut physical =
            PhysicalDeck::with_model_transport(locator.model, Box::new(UsbTransport(deck)));
        physical._ownership = Some(self.lease.clone());
        physical._claim = Some(claim);
        Ok(physical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn locator(model: DeviceModel, serial: Option<&str>, path: &str) -> Locator {
        Locator {
            model,
            identity: serial.and_then(|s| PhysicalIdentity::new(model, s).ok()),
            path: path.as_bytes().to_vec(),
        }
    }
    #[test]
    fn missing_duplicate_and_oversize_inventory_never_select() {
        let missing = Inventory::from_locators([locator(DeviceModel::Mk2, None, "a")]).unwrap();
        assert!(missing.selected(0).is_err());
        let duplicate = Inventory::from_locators([
            locator(DeviceModel::Mk2, Some("unit-a"), "a"),
            locator(DeviceModel::Mk2, Some("unit-a"), "b"),
        ])
        .unwrap();
        assert!(duplicate.selected(0).is_err());
        assert!(duplicate.selected(1).is_err());
        assert!(
            Inventory::from_locators((0..17).map(|_| locator(DeviceModel::Plus, None, "a")))
                .is_err()
        );
    }
    #[test]
    fn reordered_inventory_retains_identity_but_changed_path_model_or_duplicate_is_rejected() {
        let original =
            Inventory::from_locators([locator(DeviceModel::Mk2, Some("unit-a"), "a")]).unwrap();
        let l = original.selected(0).unwrap();
        let reordered = Inventory::from_locators([
            locator(DeviceModel::Plus, Some("unit-b"), "0"),
            locator(DeviceModel::Mk2, Some("unit-a"), "a"),
        ])
        .unwrap();
        reordered.revalidate(l).unwrap();
        for other in [
            vec![locator(DeviceModel::Mk2, Some("unit-a"), "b")],
            vec![locator(DeviceModel::Plus, Some("unit-a"), "a")],
            vec![
                locator(DeviceModel::Mk2, Some("unit-a"), "a"),
                locator(DeviceModel::Mk2, Some("unit-a"), "b"),
            ],
            vec![],
        ] {
            assert!(
                Inventory::from_locators(other)
                    .unwrap()
                    .revalidate(l)
                    .is_err()
            );
        }
        assert!(!format!("{reordered:?}").contains("unit-a"));
    }
    #[test]
    fn shared_claims_bound_duplicate_handles_and_release_failed_open() {
        let lease = std::fs::File::open("/dev/null").unwrap();
        let owner = HardwareOwner {
            lease: Arc::new(lease),
            claims: Arc::default(),
        };
        let a = PhysicalIdentity::new(DeviceModel::Mk2, "a").unwrap();
        let first = owner.claim(&a).unwrap();
        assert!(owner.claim(&a).is_err());
        drop(first);
        let mut claims = vec![owner.claim(&a).unwrap()];
        for index in 1..MAX_DEVICES {
            claims.push(
                owner
                    .claim(&PhysicalIdentity::new(DeviceModel::Plus, &index.to_string()).unwrap())
                    .unwrap(),
            );
        }
        assert!(
            owner
                .claim(&PhysicalIdentity::new(DeviceModel::Plus, "extra").unwrap())
                .is_err()
        );
        drop(claims);
        assert!(owner.claim(&a).is_ok());
    }
}
