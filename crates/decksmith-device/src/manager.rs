//! Inert multi-device supervisor seam. Owns only adapters supplied by its caller;
//! does not enumerate/open HID, acquire ownership, render layouts, start helpers
//! or execute actions. The production Plus worker does not use this module yet.
use crate::{
    DeckDevice, DeviceError,
    registry::{Association, Discovery, MAX_DEVICES, Registry},
};
use decksmith_core::{
    InputEvent, RawEvent,
    device_context::{DeviceContext, DeviceId},
    device_model::{ControlAddress, DeviceModel},
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

type Result<T> = std::result::Result<T, &'static str>;
const DRAIN_BUDGET: usize = 32;
static SUPERVISOR_EPOCH: AtomicU64 = AtomicU64::new(0);

/// Topology from a validated document, not a replacement for layout validation.
#[derive(Debug, Clone, Copy)]
pub struct SessionLayout {
    model: DeviceModel,
    pages: u8,
    default_page: u8,
}
impl SessionLayout {
    pub fn new(model: DeviceModel, pages: u8, default_page: u8) -> Result<Self> {
        if !(1..=16).contains(&pages) || default_page >= pages {
            return Err("invalid_session_layout");
        }
        Ok(Self {
            model,
            pages,
            default_page,
        })
    }
}
struct Session {
    id: DeviceId,
    model: DeviceModel,
    brightness: u8,
    layout: Option<SessionLayout>,
    page: Option<u8>,
    connection: u64,
    binding: u64,
    device: Option<Box<dyn DeckDevice>>,
    draining: bool,
    held: BTreeSet<ControlAddress>,
    armed: BTreeSet<ControlAddress>,
    last_timestamp: Option<u64>,
    error: Option<DeviceError>,
}
impl Session {
    fn fence(&mut self) {
        self.armed.clear();
        self.draining = true;
    }
    fn failed(&mut self, error: DeviceError) {
        self.device = None;
        self.error = Some(error);
        self.fence();
        self.held.clear();
        self.last_timestamp = None;
    }
    fn valid_input(&self, event: &InputEvent) -> bool {
        let descriptor = decksmith_core::device_model::DeviceDescriptor::for_model(self.model);
        if !descriptor.supports(event.event.control())
            || self
                .last_timestamp
                .is_some_and(|last| event.timestamp_ms < last)
        {
            return false;
        }
        match &event.event {
            RawEvent::DialRotate { ticks, .. } => *ticks != 0,
            RawEvent::Touch { x, y, gesture } => descriptor
                .capabilities()
                .touch
                .is_some_and(|t| *x < t.pixels.0 && *y < t.pixels.1 && t.supports(gesture)),
            _ => true,
        }
    }
    fn accept(&mut self, event: &InputEvent, blocked: bool) -> bool {
        self.last_timestamp = Some(event.timestamp_ms);
        let control = event.event.control();
        match event.event {
            RawEvent::Key { pressed, .. } | RawEvent::DialPush { pressed, .. } => {
                if pressed {
                    // A held control across a fence must be released before a
                    // fresh down can arm it. Repeated downs cannot rearm it.
                    if self.held.insert(control) && !blocked {
                        self.armed.insert(control);
                        return true;
                    }
                    false
                } else {
                    self.held.remove(&control);
                    self.armed.remove(&control) && !blocked
                }
            }
            _ => !blocked,
        }
    }
}

#[derive(Debug)]
pub struct ReceivedInput {
    pub context: DeviceContext,
    pub input: InputEvent,
    /// Preserve the adapter's actual monotonic receipt time. Virtual adapters
    /// leave it absent rather than inventing a physical latency measurement.
    pub received_at: Option<std::time::Instant>,
}
#[derive(Serialize)]
pub struct SessionStatus {
    pub id: DeviceId,
    pub model: DeviceModel,
    pub connected: bool,
    pub input_ready: bool,
    pub connection: u64,
    pub binding: u64,
    pub page: Option<u8>,
    pub brightness: u8,
    pub error: Option<String>,
}
pub struct DeviceManager {
    epoch: u64,
    registry: Registry,
    discovery: Discovery,
    discovery_revision: u64,
    sessions: Vec<Session>,
    cursor: usize,
    locked: bool,
    suspended: bool,
    lock_epoch: u64,
}
impl DeviceManager {
    pub fn new(registry: Registry) -> Result<Self> {
        let epoch = SUPERVISOR_EPOCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_add(1)
            })
            .map_err(|_| "supervisor_epoch_exhausted")?
            + 1;
        let sessions = registry
            .records()
            .iter()
            .map(|r| Session {
                id: r.id(),
                model: r.model(),
                brightness: r.brightness(),
                layout: None,
                page: None,
                connection: 0,
                binding: 0,
                device: None,
                draining: true,
                held: BTreeSet::new(),
                armed: BTreeSet::new(),
                last_timestamp: None,
                error: None,
            })
            .collect();
        Ok(Self {
            epoch,
            registry,
            discovery: Discovery::new(Vec::new()).expect("empty snapshot"),
            discovery_revision: 0,
            sessions,
            cursor: 0,
            locked: false,
            suspended: false,
            lock_epoch: 0,
        })
    }
    fn session(&self, id: DeviceId) -> Result<&Session> {
        self.sessions
            .iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")
    }
    fn session_mut(&mut self, id: DeviceId) -> Result<&mut Session> {
        self.sessions
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or("unknown_device")
    }
    pub fn active_page(&self, id: DeviceId) -> Result<u8> {
        self.session(id)?.page.ok_or("device_layout_unassigned")
    }
    pub fn statuses(&self) -> Vec<SessionStatus> {
        self.sessions
            .iter()
            .map(|s| SessionStatus {
                id: s.id,
                model: s.model,
                connected: s.device.is_some(),
                input_ready: s.device.is_some()
                    && !s.draining
                    && !self.locked
                    && !self.suspended
                    && s.layout.is_some(),
                connection: s.connection,
                binding: s.binding,
                page: s.page,
                brightness: s.brightness,
                error: s.error.map(|e| e.to_string()),
            })
            .collect()
    }
    /// Add exactly one explicitly saved profile without recreating the supervisor
    /// or fencing unrelated live sessions. Old identities may not be reassigned.
    pub fn enroll_saved(
        &mut self,
        registry: Registry,
        id: DeviceId,
        layout: SessionLayout,
        page: u8,
    ) -> Result<()> {
        if self.locked || self.suspended {
            return Err("devices_blocked");
        }
        let r = registry.get(id).ok_or("unknown_device")?;
        if self.registry.get(id).is_some()
            || registry.records().len() != self.registry.records().len() + 1
            || registry.revision() <= self.registry.revision()
            || self.registry.records().iter().any(|old| {
                !registry
                    .get(old.id())
                    .is_some_and(|new| old.same_identity(new))
            })
        {
            return Err("invalid_registry_extension");
        }
        if layout.model != r.model() || page >= layout.pages {
            return Err("invalid_session_layout");
        }
        let discovery_revision = self
            .discovery_revision
            .checked_add(1)
            .ok_or("discovery_revision_exhausted")?;
        self.sessions.push(Session {
            id,
            model: r.model(),
            brightness: r.brightness(),
            layout: Some(layout),
            page: Some(page),
            connection: 0,
            binding: 1,
            device: None,
            draining: true,
            held: BTreeSet::new(),
            armed: BTreeSet::new(),
            last_timestamp: None,
            error: None,
        });
        self.registry = registry;
        self.discovery_revision = discovery_revision;
        Ok(())
    }
    /// Validate private snapshot identity, not merely model/slot, before an opener
    /// can acquire a handle. No HID call or display write occurs here.
    pub fn attachment_target(
        &self,
        revision: u64,
        discovery: &Discovery,
        slot: u8,
    ) -> Result<DeviceId> {
        if revision != self.discovery_revision
            || discovery.candidates() != self.discovery.candidates()
        {
            return Err("stale_discovery_revision");
        }
        let candidate = discovery
            .candidates()
            .iter()
            .find(|c| c.slot() == slot)
            .ok_or("unknown_candidate")?;
        let Association::Assigned(id) = self.registry.resolve(discovery, candidate) else {
            return Err("device_unassigned");
        };
        if self.session(id)?.device.is_some() {
            return Err("device_already_owned");
        }
        Ok(id)
    }
    /// Disconnected records persist. A reversed enumeration never swaps them.
    /// Ambiguous/disappearing identity closes only its existing session.
    pub fn reconcile(&mut self, discovery: Discovery) -> Result<u64> {
        let revision = self
            .discovery_revision
            .checked_add(1)
            .ok_or("discovery_revision_exhausted")?;
        for session in &mut self.sessions {
            let present = discovery
                .candidates()
                .iter()
                .any(|c| self.registry.resolve(&discovery, c) == Association::Assigned(session.id));
            if !present && session.device.is_some() {
                session.failed(DeviceError::Disconnected);
            }
        }
        self.discovery = discovery;
        self.discovery_revision = revision;
        Ok(revision)
    }
    /// The host must open/verify this snapshot's actual unit under its one
    /// manager-wide ownership lease. Matching descriptor alone does not verify
    /// a physical serial. This seam does not implement the physical opener.
    pub fn attach(
        &mut self,
        revision: u64,
        slot: u8,
        mut device: Box<dyn DeckDevice>,
    ) -> Result<DeviceId> {
        if revision != self.discovery_revision {
            return Err("stale_discovery_revision");
        }
        let candidate = self
            .discovery
            .candidates()
            .iter()
            .find(|c| c.slot() == slot)
            .ok_or("unknown_candidate")?;
        let Association::Assigned(id) = self.registry.resolve(&self.discovery, candidate) else {
            return Err("device_unassigned");
        };
        let session = self.session_mut(id)?;
        if session.device.is_some() {
            return Err("device_already_owned");
        }
        if device.descriptor().model() != session.model {
            return Err("device_model_mismatch");
        }
        let generation = session
            .connection
            .checked_add(1)
            .ok_or("connection_generation_exhausted")?;
        session.connection = generation;
        session.fence();
        session.held.clear();
        session.last_timestamp = None;
        if let Err(error) = device.set_brightness(session.brightness) {
            session.failed(error);
            return Err("device_attach_failed");
        }
        session.error = None;
        session.device = Some(device);
        Ok(id)
    }
    pub fn disconnect(&mut self, id: DeviceId) -> Result<()> {
        self.session_mut(id)?.failed(DeviceError::Disconnected);
        Ok(())
    }
    pub fn configure(&mut self, id: DeviceId, expected: u64, layout: SessionLayout) -> Result<u64> {
        if self.locked || self.suspended {
            return Err("devices_blocked");
        }
        let s = self.session_mut(id)?;
        if expected != s.binding {
            return Err("stale_binding_revision");
        }
        if s.model != layout.model {
            return Err("layout_model_mismatch");
        }
        let revision = s
            .binding
            .checked_add(1)
            .ok_or("binding_revision_exhausted")?;
        s.layout = Some(layout);
        s.page = Some(layout.default_page);
        s.binding = revision;
        s.fence();
        Ok(revision)
    }
    pub fn page(&mut self, id: DeviceId, expected: u64, page: u8) -> Result<u64> {
        if self.locked || self.suspended {
            return Err("devices_blocked");
        }
        let s = self.session_mut(id)?;
        if expected != s.binding {
            return Err("stale_binding_revision");
        }
        if page >= s.layout.ok_or("device_layout_unassigned")?.pages {
            return Err("invalid_page");
        }
        if s.page == Some(page) {
            return Ok(s.binding);
        }
        let revision = s
            .binding
            .checked_add(1)
            .ok_or("binding_revision_exhausted")?;
        s.page = Some(page);
        s.binding = revision;
        s.fence();
        Ok(revision)
    }
    /// Device-addressed brightness. Keep the latest desired value for this
    /// unit's reconnect, with no global/default-device fallback.
    pub fn brightness(&mut self, id: DeviceId, expected: u64, percent: u8) -> Result<()> {
        if self.locked || self.suspended {
            return Err("devices_blocked");
        }
        if percent > 100 {
            return Err("invalid_brightness");
        }
        let s = self.session_mut(id)?;
        if expected != s.binding {
            return Err("stale_binding_revision");
        }
        s.brightness = percent;
        if let Some(device) = &mut s.device
            && let Err(error) = device.set_brightness(percent)
        {
            s.failed(error);
            return Err("device_brightness_failed");
        }
        Ok(())
    }
    /// Apply a pre-rendered model-sized frame to one exact connection/binding.
    /// Validate every surface before the first write. On transport failure,
    /// attempt a full blank of this owner and close only its adapter.
    pub fn paint(
        &mut self,
        context: DeviceContext,
        keys: &[u8],
        touch: Option<&[u8]>,
    ) -> Result<()> {
        if !self.current(&context) {
            return Err("stale_device_context");
        }
        let s = self.session_mut(context.device)?;
        let descriptor = decksmith_core::device_model::DeviceDescriptor::for_model(s.model);
        if keys.len() != descriptor.key_rgb_bytes() * usize::from(descriptor.key_count())
            || match (descriptor.capabilities().touch, touch) {
                (Some(surface), Some(bytes)) => bytes.len() != surface.rgb_bytes(),
                (None, None) => false,
                _ => true,
            }
        {
            return Err("invalid_device_frame");
        }
        let device = s.device.as_mut().ok_or("device_not_ready")?;
        let result = (|| {
            for (index, frame) in keys.chunks_exact(descriptor.key_rgb_bytes()).enumerate() {
                device.set_key_image(index as u8, frame)?;
            }
            if let Some(frame) = touch {
                device.set_touch_image(frame)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            let _ = device.blank();
            s.failed(error);
            return Err("device_frame_failed");
        }
        Ok(())
    }
    /// Shared desktop gate, independent physical drains on both edges. Nothing
    /// automatically unlocks a session or replays held/queued input.
    pub fn gate(&mut self, locked: bool, suspended: bool) -> Result<()> {
        if self.locked == locked && self.suspended == suspended {
            return Ok(());
        }
        let epoch = self
            .lock_epoch
            .checked_add(1)
            .ok_or("lock_epoch_exhausted")?;
        self.locked = locked;
        self.suspended = suspended;
        self.lock_epoch = epoch;
        for session in &mut self.sessions {
            session.fence();
        }
        Ok(())
    }
    pub fn context(&self, id: DeviceId, control: ControlAddress) -> Result<DeviceContext> {
        let s = self.session(id)?;
        let descriptor = decksmith_core::device_model::DeviceDescriptor::for_model(s.model);
        if !descriptor.supports(control) {
            return Err("invalid_control");
        }
        if self.locked || self.suspended || s.device.is_none() || s.layout.is_none() || s.draining {
            return Err("device_not_ready");
        }
        Ok(DeviceContext {
            supervisor: self.epoch,
            device: id,
            connection: s.connection,
            binding: s.binding,
            lock_epoch: self.lock_epoch,
            control,
        })
    }
    pub fn current(&self, context: &DeviceContext) -> bool {
        self.context(context.device, context.control)
            .is_ok_and(|now| now == *context)
    }
    /// Fair bounded nonblocking polling: at most one deliverable input per unit
    /// per turn; each draining unit gets at most 32 reads. The supplied adapter
    /// must implement genuinely nonblocking polling before physical integration.
    pub fn poll_round(&mut self) -> Vec<ReceivedInput> {
        let mut output = Vec::with_capacity(self.sessions.len());
        let count = self.sessions.len();
        debug_assert!(count <= MAX_DEVICES);
        for offset in 0..count {
            let s = &mut self.sessions[(self.cursor + offset) % count];
            let blocked = self.locked || self.suspended || s.layout.is_none();
            let budget = if s.draining { DRAIN_BUDGET } else { 1 };
            for _ in 0..budget {
                let Some(device) = s.device.as_mut() else {
                    break;
                };
                let polled = device.poll_event_nonblocking();
                let received_at = device.event_received_at();
                match polled {
                    Err(error) => {
                        s.failed(error);
                        break;
                    }
                    Ok(None) => {
                        s.draining = false;
                        break;
                    }
                    Ok(Some(input)) => {
                        if !s.valid_input(&input) {
                            s.failed(DeviceError::InvalidReport);
                            break;
                        }
                        if s.accept(&input, blocked || s.draining) {
                            output.push(ReceivedInput {
                                context: DeviceContext {
                                    supervisor: self.epoch,
                                    device: s.id,
                                    connection: s.connection,
                                    binding: s.binding,
                                    lock_epoch: self.lock_epoch,
                                    control: input.event.control(),
                                },
                                input,
                                received_at,
                            });
                            break;
                        }
                    }
                }
            }
        }
        if count != 0 {
            self.cursor = (self.cursor + 1) % count;
        }
        output
    }
    /// Feedback is fenced before transport access; a failure closes only this
    /// owner. Current action backends do not yet produce/consume this context.
    pub fn write_key(&mut self, context: &DeviceContext, frame: &[u8]) -> Result<()> {
        if !self.current(context) {
            return Err("stale_device_context");
        }
        let ControlAddress::Key(index) = context.control else {
            return Err("invalid_control");
        };
        let s = self.session_mut(context.device)?;
        let descriptor = decksmith_core::device_model::DeviceDescriptor::for_model(s.model);
        if frame.len() != descriptor.key_rgb_bytes() {
            return Err("invalid_key_image");
        }
        if let Err(error) = s
            .device
            .as_mut()
            .ok_or("device_not_ready")?
            .set_key_image(index, frame)
        {
            s.failed(error);
            return Err("device_write_failed");
        }
        Ok(())
    }
    /// Best effort all-surface blanking for every owned unit, continuing after
    /// failures. Blanking synchrony/time bounds depend on the eventual adapter.
    pub fn stop(&mut self) -> Vec<DeviceId> {
        // Stop permanently fences these sessions until an explicit new attach.
        let mut failures = Vec::new();
        for s in &mut self.sessions {
            if let Some(mut device) = s.device.take()
                && let Err(error) = device.blank()
            {
                failures.push(s.id);
                s.error = Some(error);
            }
            s.fence();
            s.held.clear();
            s.last_timestamp = None;
        }
        failures
    }
}
