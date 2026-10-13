//! One deterministic desired-state coordinator for all device owners. No audio
//! access/helpers here: the host must drive a fail-safe guardian and acknowledge
//! actual writes. "Live" below is desired state, never proof a mic is live.
use crate::{
    device_context::DeviceId,
    device_model::ControlAddress,
    shared_dispatch::{ActionScope, MAX_DEVICES, TargetId},
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
pub const LEASE_MS: u64 = 500;
pub const MAX_TARGETS: usize = 8;
const MAX_CONTROLS: usize = MAX_DEVICES * 19;
type Result<T> = std::result::Result<T, &'static str>;
static HOLD_EPOCH: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldToken {
    pub scope: ActionScope,
    pub press: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesiredMic {
    Muted,
    Live,
}
#[derive(Debug, Clone)]
pub struct MicUpdate {
    runtime: DeviceId,
    coordinator: u64,
    pub target: TargetId,
    pub revision: u64,
    pub desired: DesiredMic,
}
struct Target {
    desired: DesiredMic,
    revision: u64,
    dirty: bool,
}
struct Lease {
    token: HoldToken,
    target: TargetId,
    touched: u64,
}
type Owner = (DeviceId, ControlAddress);
pub struct MicrophoneHolds {
    epoch: u64,
    runtime: DeviceId,
    devices: Vec<DeviceId>,
    leases: BTreeMap<Owner, Lease>,
    consumed: BTreeMap<Owner, (ActionScope, u64)>,
    targets: BTreeMap<TargetId, Target>,
    clock: u64,
    revision: u64,
    stopped: bool,
}
impl MicrophoneHolds {
    pub fn new(runtime: DeviceId, devices: &[DeviceId]) -> Result<Self> {
        if devices.len() > MAX_DEVICES
            || devices
                .iter()
                .enumerate()
                .any(|(i, id)| devices[..i].contains(id))
        {
            return Err("invalid_hold_devices");
        }
        let epoch = HOLD_EPOCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_add(1)
            })
            .map_err(|_| "hold_epoch_exhausted")?
            + 1;
        Ok(Self {
            epoch,
            runtime,
            devices: devices.to_vec(),
            leases: BTreeMap::new(),
            consumed: BTreeMap::new(),
            targets: BTreeMap::new(),
            clock: 0,
            revision: 0,
            stopped: false,
        })
    }
    pub fn held(&self) -> usize {
        self.leases.len()
    }
    fn time(&mut self, now: u64) -> Result<()> {
        if now < self.clock {
            self.stop()?;
            return Err("hold_time_reversed");
        }
        self.clock = now;
        Ok(())
    }
    fn scope_allowed(&self, scope: &ActionScope, current: &impl Fn(&ActionScope) -> bool) -> bool {
        !self.stopped
            && scope.runtime == self.runtime
            && scope.structurally_valid()
            && self.devices.contains(&scope.device.device)
            && current(scope)
    }
    fn desired(&mut self, target: &TargetId, desired: DesiredMic, force: bool) -> Result<()> {
        if self
            .targets
            .get(target)
            .is_some_and(|t| t.desired == desired)
            && !force
        {
            return Ok(());
        }
        let Some(revision) = self.revision.checked_add(1) else {
            // Fail closed even if the counter is exhausted: no old Live ack can
            // match Muted, and no further acquisition can reopen any target.
            self.stopped = true;
            self.leases.clear();
            for t in self.targets.values_mut() {
                t.desired = DesiredMic::Muted;
                t.dirty = true;
            }
            return Err("hold_revision_exhausted");
        };
        self.revision = revision;
        self.targets.insert(
            target.clone(),
            Target {
                desired,
                revision,
                dirty: true,
            },
        );
        Ok(())
    }
    fn remove(&mut self, owner: Owner) -> Result<()> {
        let Some(lease) = self.leases.remove(&owner) else {
            return Ok(());
        };
        if !self.leases.values().any(|l| l.target == lease.target) {
            self.desired(&lease.target, DesiredMic::Muted, false)?;
        }
        Ok(())
    }
    pub fn maintain(&mut self, now: u64, current: impl Fn(&ActionScope) -> bool) -> Result<()> {
        self.time(now)?;
        let stale = self
            .leases
            .iter()
            .filter(|(_, l)| {
                !self.scope_allowed(&l.token.scope, &current) || now - l.touched >= LEASE_MS
            })
            .map(|(o, _)| *o)
            .collect::<Vec<_>>();
        for owner in stale {
            self.remove(owner)?;
        }
        Ok(())
    }
    pub fn acquire(
        &mut self,
        token: HoldToken,
        target: TargetId,
        now: u64,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<()> {
        self.maintain(now, &current)?;
        if !self.scope_allowed(&token.scope, &current)
            || token.press == 0
            || !matches!(
                token.scope.device.control,
                ControlAddress::Key(_) | ControlAddress::DialPush(_)
            )
        {
            return Err("invalid_hold_scope");
        }
        let owner = (token.scope.device.device, token.scope.device.control);
        if let Some(active) = self.leases.get(&owner) {
            return if active.token == token && active.target == target {
                Ok(())
            } else {
                Err("hold_control_busy")
            };
        }
        if self
            .consumed
            .get(&owner)
            .is_some_and(|(scope, press)| *scope == token.scope && token.press <= *press)
        {
            return Err("hold_requires_fresh_press");
        }
        if !self.consumed.contains_key(&owner) && self.consumed.len() >= MAX_CONTROLS {
            return Err("hold_control_limit");
        }
        // Consume this down even when the target budget is full. Do not unmute
        // a still-held key later merely because resource space becomes free.
        self.consumed.insert(owner, (token.scope, token.press));
        if !self.targets.contains_key(&target) && self.targets.len() >= MAX_TARGETS {
            return Err("hold_target_limit");
        }
        self.desired(&target, DesiredMic::Live, false)?;
        self.leases.insert(
            owner,
            Lease {
                token,
                target,
                touched: now,
            },
        );
        Ok(())
    }
    pub fn heartbeat(
        &mut self,
        device: DeviceId,
        now: u64,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<()> {
        self.maintain(now, &current)?; // Expired/cancelled holds can never revive.
        for lease in self
            .leases
            .values_mut()
            .filter(|l| l.token.scope.device.device == device)
        {
            lease.touched = now;
        }
        Ok(())
    }
    pub fn release(
        &mut self,
        token: HoldToken,
        now: u64,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<bool> {
        self.maintain(now, &current)?;
        let owner = (token.scope.device.device, token.scope.device.control);
        if self.leases.get(&owner).is_some_and(|l| l.token == token) {
            self.remove(owner)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Explicit mute or guardian failure consumes all holds for this target.
    /// Only another genuine down/press sequence may acquire again.
    pub fn cancel_target(&mut self, target: &TargetId) -> Result<()> {
        self.leases.retain(|_, l| l.target != *target);
        if self.targets.contains_key(target) {
            self.desired(target, DesiredMic::Muted, true)?;
        }
        Ok(())
    }
    /// At most eight latest desired states. Releases/cancellation never enter
    /// the ordinary action queue and remain pending until the host acknowledges.
    pub fn updates(&self) -> Vec<MicUpdate> {
        self.targets
            .iter()
            .filter(|(_, t)| t.dirty)
            .map(|(target, t)| MicUpdate {
                runtime: self.runtime,
                coordinator: self.epoch,
                target: target.clone(),
                revision: t.revision,
                desired: t.desired,
            })
            .collect()
    }
    /// Bounded desired-state inventory for persistent guardians, including Live
    /// states whose write has already been acknowledged. This is not actual state.
    pub fn states(&self) -> Vec<MicUpdate> {
        self.targets
            .iter()
            .map(|(target, t)| MicUpdate {
                runtime: self.runtime,
                coordinator: self.epoch,
                target: target.clone(),
                revision: t.revision,
                desired: t.desired,
            })
            .collect()
    }
    /// An explicit mute must work even before the first hold. It consumes active
    /// leases and leaves a bounded latest request for the native guardian.
    pub fn explicit_mute(&mut self, target: &TargetId) -> Result<()> {
        if !self.targets.contains_key(target) && self.targets.len() >= MAX_TARGETS {
            return Err("hold_target_limit");
        }
        self.leases.retain(|_, l| l.target != *target);
        self.desired(target, DesiredMic::Muted, true)
    }
    pub fn accepts_update(&self, update: &MicUpdate) -> bool {
        if update.runtime != self.runtime || update.coordinator != self.epoch {
            return false;
        }
        self.targets
            .get(&update.target)
            .is_some_and(|t| t.revision == update.revision && t.desired == update.desired)
    }
    /// Revalidate owner leases immediately before issuing the guardian command.
    /// An aggregate Live update remains valid only while a current owner holds it.
    pub fn prepare_update(
        &mut self,
        update: &MicUpdate,
        now: u64,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<bool> {
        self.maintain(now, current)?;
        Ok(self.accepts_update(update))
    }
    pub fn acknowledge(
        &mut self,
        update: &MicUpdate,
        success: bool,
        now: u64,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<bool> {
        self.maintain(now, current)?;
        if !self.accepts_update(update) {
            return Ok(false);
        }
        if !success {
            self.cancel_target(&update.target)?;
            return Ok(true);
        }
        if update.desired == DesiredMic::Muted
            && !self.leases.values().any(|l| l.target == update.target)
        {
            self.targets.remove(&update.target);
        } else if let Some(target) = self.targets.get_mut(&update.target) {
            target.dirty = false;
        }
        Ok(true)
    }
    pub fn stop(&mut self) -> Result<()> {
        self.stopped = true;
        self.leases.clear();
        let targets = self.targets.keys().cloned().collect::<Vec<_>>();
        for target in targets {
            self.desired(&target, DesiredMic::Muted, true)?;
        }
        Ok(())
    }
}
