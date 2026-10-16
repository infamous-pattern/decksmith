//! Connected supplied-adapter execution path. The normal V1 worker and saved-only
//! workspace do not activate it. Canonical named audio/media are explicit;
//! unsupported providers/aliases never fall back to another target.
use crate::{
    device_profiles::{Catalog, Controls, random_device_id},
    device_render,
    pages::{Action, TouchState},
    shared_microphone::{self, Microphone},
    shared_worker::{self, Authority, Compiled},
};
use decksmith_core::{
    RawEvent,
    device_context::{DeviceContext, DeviceId},
    device_model::ControlAddress,
    microphone_holds::HoldToken,
    shared_dispatch::{ActionScope, RequestId},
};
use decksmith_device::{
    DeckDevice, actor,
    registry::{Association, Discovery},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
struct Entry {
    context: DeviceContext,
    lease: actor::Lease,
    epoch: u64,
    model: decksmith_core::device_model::DeviceModel,
}
struct View {
    runtime: DeviceId,
    foreground: u64,
    entries: BTreeMap<DeviceId, Entry>,
}
impl View {
    fn accepts(&self, scope: &ActionScope) -> bool {
        self.runtime == scope.runtime
            && self.foreground == scope.foreground
            && self.entries.get(&scope.device.device).is_some_and(|e| {
                decksmith_core::device_model::DeviceDescriptor::for_model(e.model)
                    .supports(scope.device.control)
                    && e.lease.accepts(e.epoch)
                    && DeviceContext {
                        control: scope.device.control,
                        ..e.context
                    } == scope.device
            })
    }
}
pub struct Feedback {
    pub scope: ActionScope,
    pub result: Result<()>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct EditTicket {
    pub context: DeviceContext,
    epoch: u64,
}
pub(crate) struct PreparedEdit {
    controls: crate::device_profiles::PreparedControls,
    plan: crate::saved_native::Plan,
    bindings: BTreeMap<String, crate::shared_native_media::Session>,
    layout: Arc<[u8]>,
}
impl PreparedEdit {
    pub(crate) fn new(
        catalog: &Catalog,
        id: DeviceId,
        bindings: Vec<crate::shared_native_media::Session>,
    ) -> Result<Self> {
        let plan = crate::saved_native::Plan::from_catalog(catalog)?;
        let mut book = BTreeMap::new();
        if bindings.len() > 8 {
            return Err("invalid_native_media_bindings");
        }
        for binding in bindings {
            binding.validate()?;
            if !plan.media.contains(&binding.selector)
                || book.insert(binding.selector.clone(), binding).is_some()
            {
                return Err("invalid_native_media_bindings");
            }
        }
        Ok(Self {
            controls: crate::device_profiles::PreparedControls::new(catalog, id)?,
            plan,
            bindings: book,
            layout: Arc::from(catalog.profile(id)?.document().bytes()),
        })
    }
}
pub struct Runtime {
    controls: Controls,
    catalog: Catalog,
    layouts: BTreeMap<DeviceId, Arc<[u8]>>,
    touch: BTreeMap<DeviceId, TouchState>,
    leases: BTreeMap<DeviceId, actor::Lease>,
    io: actor::Pool,
    renders: device_render::Pool,
    actions: shared_worker::Service,
    homebridge_actions: Option<shared_worker::Service>,
    homebridge_uncertain: Option<Arc<std::sync::atomic::AtomicBool>>,
    mics: Option<shared_microphone::Service>,
    mic_fatal_reported: bool,
    mirror: Arc<RwLock<View>>,
    dirty: BTreeSet<DeviceId>,
    strip_dirty: BTreeSet<DeviceId>,
    render_pending: BTreeSet<DeviceId>,
    painted: BTreeMap<DeviceId, DeviceContext>,
    holds: BTreeMap<(DeviceId, ControlAddress), HoldToken>,
    press: u64,
    stopped: bool,
    media_bindings: Option<BTreeMap<String, crate::shared_native_media::Session>>,
    native_plan: Option<crate::saved_native::Plan>,
    native_uncertain: Option<Arc<std::sync::atomic::AtomicBool>>,
    native_observer: Option<crate::shared_native_state::Observer>,
    native_states: Vec<(String, Option<crate::audio::State>)>,
    native_contexts: BTreeMap<DeviceId, DeviceContext>,
    system_states: crate::system_actions::States,
    system_contexts: BTreeMap<DeviceId, Option<u8>>,
    native_meter: Option<crate::meter::Worker>,
    native_meter_targets: Vec<String>,
    native_meter_epoch: u64,
    next_meter: Instant,
    edits: BTreeMap<DeviceId, EditTicket>,
    edit_epoch: u64,
    gate_state: (bool, bool),
    manual_gate: (bool, bool),
    host_locked: bool,
    host_sleeping: bool,
    host_sleep_epoch: Option<u64>,
    reopen_after: Option<Instant>,
    retired_errors: BTreeMap<DeviceId, decksmith_device::DeviceError>,
    dynamic_media: bool,
    host_lock_epoch: Option<u64>,
    gated_frames: BTreeMap<DeviceId, Arc<crate::profile_control::Frame>>,
    gated_painted: BTreeMap<DeviceId, u64>,
}
impl Runtime {
    pub fn start(
        catalog: Catalog,
        backends: Vec<Box<dyn shared_worker::Backend>>,
        microphones: Option<Box<dyn shared_microphone::Factory>>,
    ) -> Result<Self> {
        let runtime = random_device_id()?;
        let ids: Vec<_> = catalog.summaries().iter().map(|s| s.id).collect();
        if ids.is_empty() {
            return Err("execution_requires_saved_devices");
        }
        let controls = Controls::from_catalog(&catalog)?;
        let mirror = Arc::new(RwLock::new(View {
            runtime,
            foreground: 1,
            entries: BTreeMap::new(),
        }));
        let view = mirror.clone();
        let authority: Authority = Arc::new(move |s| view.read().unwrap().accepts(s));
        let actions = shared_worker::Service::start(runtime, &ids, authority.clone(), backends)?;
        let mics = microphones
            .map(|factory| {
                shared_microphone::Service::start(runtime, &ids, authority.clone(), factory)
            })
            .transpose()?;
        let renders = device_render::Pool::start(&ids, authority)?;
        let mut layouts = BTreeMap::new();
        let mut touch = BTreeMap::new();
        for id in &ids {
            layouts.insert(*id, Arc::from(catalog.profile(*id)?.document().bytes()));
            touch.insert(*id, TouchState::default());
        }
        let mut model_frames: Vec<(
            decksmith_core::device_model::DeviceModel,
            Arc<crate::profile_control::Frame>,
        )> = Vec::new();
        let mut gated_frames = BTreeMap::new();
        for unit in catalog.summaries() {
            let frame = match model_frames.iter().find(|(model, _)| *model == unit.model) {
                Some((_, frame)) => frame.clone(),
                None => {
                    let frame = Arc::new(crate::session_lock::model_frame(unit.model));
                    model_frames.push((unit.model, frame.clone()));
                    frame
                }
            };
            gated_frames.insert(unit.id, frame);
        }
        Ok(Self {
            controls,
            catalog,
            layouts,
            touch,
            leases: BTreeMap::new(),
            io: actor::Pool::default(),
            renders,
            actions,
            homebridge_actions: None,
            homebridge_uncertain: None,
            mics,
            mic_fatal_reported: false,
            mirror,
            dirty: ids.into_iter().collect(),
            strip_dirty: BTreeSet::new(),
            render_pending: BTreeSet::new(),
            painted: BTreeMap::new(),
            holds: BTreeMap::new(),
            press: 0,
            stopped: false,
            media_bindings: None,
            native_plan: None,
            native_uncertain: None,
            native_observer: None,
            native_states: Vec::new(),
            native_contexts: BTreeMap::new(),
            system_states: Default::default(),
            system_contexts: BTreeMap::new(),
            native_meter: None,
            native_meter_targets: Vec::new(),
            native_meter_epoch: 1,
            next_meter: Instant::now(),
            edits: BTreeMap::new(),
            edit_epoch: 1,
            gate_state: (false, false),
            manual_gate: (false, false),
            host_locked: false,
            host_sleeping: false,
            host_sleep_epoch: None,
            reopen_after: None,
            retired_errors: BTreeMap::new(),
            dynamic_media: false,
            host_lock_epoch: None,
            gated_frames,
            gated_painted: BTreeMap::new(),
        })
    }
    /// Prepared off the input coordinator. Exactly one native executor serializes
    /// overlapping logical audio aliases; one deduplicated reader serves all Decks.
    pub fn start_saved_native(
        catalog: Catalog,
        backend: crate::shared_native_media::NativeBackend,
        bindings: Vec<crate::shared_native_media::Session>,
    ) -> Result<Self> {
        let plan = crate::saved_native::Plan::from_catalog(&catalog)?;
        let uncertain = backend.health();
        if bindings.iter().any(|s| !plan.media.contains(&s.selector)) {
            return Err("native_media_payload_mismatch");
        }
        let observer = Some(crate::shared_native_state::Observer::start_many(
            plan.audio.clone(),
        )?);
        let microphones = Box::new(shared_microphone::NativeFactory::new()?);
        let mut runtime = Self::start(catalog, vec![Box::new(backend)], Some(microphones))?;
        runtime.bind_native_media(bindings)?;
        runtime.native_plan = Some(plan);
        runtime.native_uncertain = Some(uncertain);
        runtime.native_observer = observer;
        runtime.native_meter = Some(crate::meter::Worker::start_shared());
        Ok(runtime)
    }
    pub fn context(&self, id: DeviceId, control: ControlAddress) -> Result<DeviceContext> {
        if self.edits.contains_key(&id) {
            return Err("profile_update_pending");
        }
        self.controls.manager().context(id, control)
    }
    pub(crate) fn instance(&self) -> DeviceId {
        self.mirror.read().unwrap().runtime
    }
    pub(crate) fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub(crate) fn bindings(&self) -> Vec<crate::shared_native_media::Session> {
        self.media_bindings
            .as_ref()
            .map_or_else(Vec::new, |map| map.values().cloned().collect())
    }
    pub(crate) fn reserve_edit(&mut self, context: DeviceContext) -> Result<EditTicket> {
        if self.stopped || self.edits.contains_key(&context.device) {
            return Err("profile_update_pending");
        }
        if !self.controls.manager().current(&context) {
            return Err("stale_device_context");
        }
        self.revoke(context.device);
        let binding = self.controls.fence_input(context.device, context.binding)?;
        let ticket = EditTicket {
            context: DeviceContext { binding, ..context },
            epoch: self.edit_epoch,
        };
        self.edits.insert(context.device, ticket);
        self.sync();
        Ok(ticket)
    }
    pub(crate) fn cancel_edit(&mut self, ticket: EditTicket) -> Result<()> {
        let id = ticket.context.device;
        if self.edits.get(&id) != Some(&ticket) {
            return Err("stale_profile_update");
        }
        let binding = self
            .controls
            .manager()
            .statuses()
            .iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")?
            .binding;
        self.revoke(id);
        self.controls.fence_input(id, binding)?;
        self.edits.remove(&id);
        self.sync();
        Ok(())
    }
    pub(crate) fn apply_edit(
        &mut self,
        ticket: EditTicket,
        catalog: Catalog,
        prepared: PreparedEdit,
    ) -> Result<()> {
        let id = ticket.context.device;
        if self.stopped
            || self.edits.get(&id) != Some(&ticket)
            || self.edit_epoch != ticket.epoch
            || !self.controls.manager().statuses().iter().any(|s| {
                s.id == id
                    && s.connected
                    && s.binding == ticket.context.binding
                    && s.connection == ticket.context.connection
            })
        {
            return Err("stale_profile_update");
        }
        if !self.catalog.same_associations(&catalog) || !prepared.controls.matches(&catalog, id) {
            return Err("invalid_prepared_profile");
        }
        for summary in self.catalog.summaries() {
            let old = self.catalog.profile(summary.id)?;
            let new = catalog.profile(summary.id)?;
            if summary.id != id && old.revision != new.revision
                || summary.id == id && old.revision.checked_add(1) != Some(new.revision)
            {
                return Err("stale_device_profile");
            }
        }
        // Preserve pins used by a partner. A save is not permission to rebind
        // another Deck's player when it pauses, vanishes or changes ownership.
        if let Some(old_plan) = &self.native_plan {
            for (partner, pages) in &old_plan.pages {
                if *partner != id {
                    for selector in pages.iter().flat_map(|p| &p.media) {
                        if self.media_bindings.as_ref().and_then(|b| b.get(selector))
                            != prepared.bindings.get(selector)
                        {
                            return Err("partner_media_pin_changed");
                        }
                    }
                }
            }
        }
        if let Some(observer) = &mut self.native_observer {
            observer.configure(prepared.plan.audio.clone())?;
        }
        self.controls
            .apply_prepared(prepared.controls, ticket.context.binding)?;
        self.layouts.insert(id, prepared.layout);
        self.catalog = catalog;
        self.native_plan = Some(prepared.plan);
        self.media_bindings = Some(prepared.bindings);
        self.native_contexts.remove(&id);
        self.system_contexts.remove(&id);
        self.edits.remove(&id);
        self.touch.get_mut(&id).unwrap().levels.clear();
        self.next_meter = Instant::now();
        self.dirty.insert(id);
        self.sync();
        Ok(())
    }
    pub(crate) fn apply_selection(&mut self, catalog: Catalog) -> Result<()> {
        if !self.catalog.same_associations(&catalog)
            || self.catalog.summaries().iter().any(|s| {
                self.catalog.profile(s.id).map(|p| p.revision)
                    != catalog.profile(s.id).map(|p| p.revision)
            })
        {
            return Err("stale_device_profile");
        }
        self.catalog = catalog;
        Ok(())
    }
    pub fn bind_native_media(
        &mut self,
        bindings: Vec<crate::shared_native_media::Session>,
    ) -> Result<()> {
        if self.media_bindings.is_some() || bindings.len() > 8 {
            return Err("invalid_native_media_bindings");
        }
        let mut map = BTreeMap::new();
        for binding in bindings {
            binding.validate()?;
            if map.insert(binding.selector.clone(), binding).is_some() {
                return Err("invalid_native_media_bindings");
            }
        }
        self.media_bindings = Some(map);
        Ok(())
    }
    pub fn key_feedback(&mut self, context: DeviceContext, bytes: &[u8]) -> Result<()> {
        self.controls.paint_key(context, bytes)
    }
    pub fn held(&self) -> usize {
        self.mics.as_ref().map_or(0, |m| m.held())
    }
    pub fn statuses(&self) -> Vec<decksmith_device::manager::SessionStatus> {
        self.controls
            .manager()
            .statuses()
            .into_iter()
            .map(|mut s| {
                if self.edits.contains_key(&s.id) {
                    s.input_ready = false;
                    s.error = Some("profile_update_pending".into());
                }
                if !(self.gate_state.0 || self.gate_state.1)
                    && (self.dirty.contains(&s.id)
                        || self.strip_dirty.contains(&s.id)
                        || self.render_pending.contains(&s.id))
                {
                    s.display_ready = false;
                }
                s
            })
            .collect()
    }
    fn revoke(&mut self, id: DeviceId) {
        // Connection/page/desktop fences cancel any old edit reservation too.
        // reserve_edit installs its new ticket only after this revocation.
        self.edits.remove(&id);
        self.gated_painted.remove(&id);
        self.mirror.write().unwrap().entries.remove(&id);
        self.renders.invalidate(id);
        self.render_pending.remove(&id);
        self.painted.remove(&id);
        self.dirty.insert(id);
        self.actions.invalidate();
        if let Some(actions) = &self.homebridge_actions {
            actions.invalidate();
        }
        if let Some(m) = &self.mics {
            let _ = m.invalidate();
        }
    }
    fn sync(&mut self) {
        // Build a small candidate view without holding the authority lock while
        // reading device readiness. Live lease checks remain independent, so a
        // device failure rejects effects even before this coordinator observes it.
        let next: Vec<_> = self
            .statuses()
            .into_iter()
            .filter_map(|status| {
                if self.edits.contains_key(&status.id) {
                    return None;
                }
                let context = self
                    .controls
                    .manager()
                    .context(status.id, ControlAddress::Key(0))
                    .ok()?;
                let lease = self.leases.get(&status.id)?;
                let epoch = lease.epoch();
                lease
                    .accepts(epoch)
                    .then_some((status.id, context, lease, epoch, status.model))
            })
            .collect();
        let mut view = self.mirror.write().unwrap();
        if view.entries.len() == next.len()
            && next.iter().all(|(id, context, _, epoch, model)| {
                view.entries.get(id).is_some_and(|old| {
                    old.context == *context && old.epoch == *epoch && old.model == *model
                })
            })
        {
            // No fence changed: do not rebuild the map or wake idle providers.
            // Connection generations distinguish replacement leases. Explicit
            // revocation paths still invalidate immediately, even before sync.
            self.holds.retain(|_, token| view.accepts(&token.scope));
            drop(view);
            // Preserve the existing microphone maintenance cadence, including
            // a lease failure racing the unchanged-view check above.
            if let Some(m) = &self.mics {
                let _ = m.invalidate();
            }
            return;
        }
        view.entries = next
            .into_iter()
            .map(|(id, context, lease, epoch, model)| {
                (
                    id,
                    Entry {
                        context,
                        lease: lease.clone(),
                        epoch,
                        model,
                    },
                )
            })
            .collect();
        self.holds.retain(|_, token| view.accepts(&token.scope));
        drop(view);
        self.actions.invalidate();
        if let Some(actions) = &self.homebridge_actions {
            actions.invalidate();
        }
        if let Some(m) = &self.mics {
            let _ = m.invalidate();
        }
    }
    pub fn discover(&mut self, discovery: Discovery) -> Result<u64> {
        let present: BTreeSet<_> = self
            .catalog
            .associations(&discovery)
            .into_iter()
            .filter_map(|(_, a)| {
                if let Association::Assigned(id) = a {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        let lost: Vec<_> = self
            .statuses()
            .iter()
            .filter(|s| !present.contains(&s.id))
            .map(|s| s.id)
            .collect();
        for id in lost {
            self.revoke(id);
        }
        let revision = self.controls.discover(discovery)?;
        self.sync();
        Ok(revision)
    }
    pub fn attach(
        &mut self,
        revision: u64,
        discovery: &Discovery,
        slot: u8,
        device: Box<dyn DeckDevice + Send>,
    ) -> Result<DeviceId> {
        let id = self
            .controls
            .manager()
            .attachment_target(revision, discovery, slot)?;
        if self.gate_state.1
            || self
                .reopen_after
                .is_some_and(|deadline| Instant::now() < deadline)
        {
            return Err("device_resuming");
        }
        let model = self.catalog.profile(id)?.document().model();
        if device.descriptor().model() != model {
            return Err("device_model_mismatch");
        }
        let proxy = self
            .io
            .spawn(id, device)
            .map_err(|_| "device_actor_unavailable")?;
        let lease = proxy.lease();
        self.revoke(id);
        let attached = self.controls.attach(revision, slot, Box::new(proxy))?;
        self.leases.insert(id, lease);
        self.sync();
        Ok(attached)
    }
    pub fn disconnect(&mut self, id: DeviceId) -> Result<()> {
        self.revoke(id);
        let result = self.controls.disconnect(id);
        self.sync();
        result
    }
    pub fn navigate(&mut self, context: DeviceContext, page: u8) -> Result<()> {
        if self.edits.contains_key(&context.device) {
            return Err("profile_update_pending");
        }
        self.revoke(context.device);
        let result = self.controls.navigate(context, page).map(|_| ());
        self.sync();
        result
    }
    pub(crate) fn enable_homebridge(&mut self) -> Result<()> {
        if self.homebridge_actions.is_some() {
            return Ok(());
        }
        let view = self.mirror.clone();
        let authority: Authority = Arc::new(move |scope| view.read().unwrap().accepts(scope));
        let ids: Vec<_> = self.layouts.keys().copied().collect();
        let backend = crate::plugin_runtime::HomebridgeBackend::default();
        self.homebridge_uncertain = Some(backend.health());
        self.homebridge_actions = Some(shared_worker::Service::start(
            self.instance(),
            &ids,
            authority,
            vec![Box::new(backend)],
        )?);
        Ok(())
    }
    pub(crate) fn enable_dynamic_media(&mut self) {
        self.dynamic_media = true;
    }
    /// Observe an owned logind preparation generation. Unknown state is passed
    /// as blocked. Detach old handles even after a short complete sleep cycle.
    pub(crate) fn host_sleep(&mut self, blocked: bool, epoch: u64) -> Result<()> {
        if self.host_sleep_epoch.is_some_and(|old| epoch < old) {
            return Err("stale_host_sleep_epoch");
        }
        let changed = self.host_sleep_epoch != Some(epoch);
        let had_observation = self.host_sleep_epoch.is_some();
        let was_blocked = self.host_sleeping;
        let had_connection = self.statuses().iter().any(|unit| unit.connection > 0);
        if !changed && blocked == was_blocked {
            return Ok(());
        }
        self.host_sleeping = blocked;
        self.host_sleep_epoch = Some(epoch);
        self.apply_gate(true)?;
        if blocked || changed && had_observation {
            if blocked && !was_blocked {
                self.retired_errors.clear();
            }
            let ids: Vec<_> = self
                .statuses()
                .into_iter()
                .filter(|unit| unit.connected)
                .map(|unit| unit.id)
                .collect();
            for id in ids {
                self.disconnect(id)?;
            }
            self.reopen_after = had_connection.then(|| Instant::now() + Duration::from_secs(2));
        }
        if !blocked && was_blocked {
            self.reopen_after = had_connection.then(|| Instant::now() + Duration::from_secs(2));
        }
        Ok(())
    }
    /// Actual guardian acknowledgement plus finished target reservations and
    /// retired USB writers; no blocking join runs on this coordinator.
    pub(crate) fn sleep_ready(&self) -> bool {
        self.host_sleeping
            && self.io.outstanding() == 0
            && self.retired_errors.is_empty()
            && self
                .native_uncertain
                .as_ref()
                .is_none_or(|flag| !flag.load(std::sync::atomic::Ordering::Acquire))
            && self
                .homebridge_uncertain
                .as_ref()
                .is_none_or(|flag| !flag.load(std::sync::atomic::Ordering::Acquire))
            && self.actions.quiet()
            && self
                .homebridge_actions
                .as_ref()
                .is_none_or(|actions| actions.quiet())
            && self.mics.as_ref().is_none_or(|mic| mic.quiet())
    }
    pub(crate) fn sleep_errors(&self) -> Vec<(DeviceId, String)> {
        self.retired_errors
            .iter()
            .map(|(id, error)| (*id, error.to_string()))
            .collect()
    }
    /// One missing or retiring unit cannot prevent a healthy partner reopening.
    pub(crate) fn ready_to_attach(&self, id: DeviceId) -> bool {
        !self.gate_state.1
            && self
                .reopen_after
                .is_none_or(|deadline| Instant::now() >= deadline)
            && self.io.available(id)
    }
    pub fn gate(&mut self, locked: bool, suspended: bool) -> Result<()> {
        self.manual_gate = (locked, suspended);
        self.apply_gate(false)
    }
    /// Host observations cannot clear an independent/manual safety gate. Epochs
    /// retain a brief lock/unlock even when the coordinator sees only unlocked.
    pub(crate) fn host_lock(&mut self, locked: bool, epoch: u64) -> Result<()> {
        if self
            .host_lock_epoch
            .is_some_and(|previous| epoch < previous)
        {
            return Err("stale_host_lock_epoch");
        }
        let changed = self.host_lock_epoch != Some(epoch);
        self.host_locked = locked;
        self.host_lock_epoch = Some(epoch);
        self.apply_gate(changed)
    }
    fn apply_gate(&mut self, force: bool) -> Result<()> {
        let state = (
            self.manual_gate.0 || self.host_locked,
            self.manual_gate.1 || self.host_sleeping,
        );
        if !force && self.gate_state == state {
            return Ok(());
        }
        self.edit_epoch = self
            .edit_epoch
            .checked_add(1)
            .ok_or("profile_update_epoch_exhausted")?;
        let ids: Vec<_> = self.layouts.keys().copied().collect();
        for id in ids {
            self.revoke(id);
        }
        self.controls.fence_gate(state.0, state.1)?;
        // Reservations cannot survive a desktop lifecycle fence. A worker that
        // already published reports apply failure; it must never keep the
        // recovered physical session disabled or revive the stale ticket.
        self.edits.clear();
        self.gate_state = state;
        self.gated_painted.clear();
        self.next_meter = Instant::now();
        self.sync();
        Ok(())
    }
    fn paint_gated(&mut self) {
        if !self.gate_state.0 && !self.gate_state.1 {
            return;
        }
        for status in self.controls.manager().statuses() {
            let Some(lease) = self.leases.get(&status.id) else {
                continue;
            };
            let epoch = lease.epoch();
            if !status.connected
                || !lease.accepts(epoch)
                || self.gated_painted.get(&status.id) == Some(&epoch)
            {
                continue;
            }
            let Some(frame) = self.gated_frames.get(&status.id) else {
                continue;
            };
            if self
                .controls
                .paint_gated(status.id, &frame.keys, frame.touch.as_deref())
                .is_ok()
            {
                self.gated_painted.insert(status.id, epoch);
            }
        }
    }
    pub fn foreground(&mut self, revision: u64) -> Result<()> {
        let previous = self.mirror.read().unwrap().foreground;
        if revision == previous {
            return Ok(());
        }
        if revision < previous {
            return Err("stale_foreground_revision");
        }
        self.edit_epoch = self
            .edit_epoch
            .checked_add(1)
            .ok_or("profile_update_epoch_exhausted")?;
        for status in self.statuses() {
            self.revoke(status.id);
            self.controls.fence_input(status.id, status.binding)?;
        }
        self.mirror.write().unwrap().foreground = revision;
        self.sync();
        Ok(())
    }
    pub fn apply_saved(&mut self, catalog: Catalog, id: DeviceId, binding: u64) -> Result<()> {
        if !self.catalog.same_associations(&catalog) {
            return Err("execution_roster_change_requires_enrollment_bridge");
        }
        let plan = if let Some(previous) = &self.native_plan {
            let next = crate::saved_native::Plan::from_catalog(&catalog)?;
            if previous.audio != next.audio || previous.media != next.media {
                return Err("native_plan_refresh_required");
            }
            Some(next)
        } else {
            None
        };
        self.revoke(id);
        self.controls.apply_saved_profile(&catalog, id, binding)?;
        self.layouts
            .insert(id, Arc::from(catalog.profile(id)?.document().bytes()));
        self.catalog = catalog;
        self.native_plan = plan;
        self.native_contexts.remove(&id);
        self.system_contexts.remove(&id);
        self.touch.get_mut(&id).unwrap().levels.clear();
        self.next_meter = Instant::now();
        self.sync();
        Ok(())
    }
    /// A saved, currently visible control only; never execute draft JSON or arm
    /// a physical press. The caller's exact context is checked again by routing.
    pub(crate) fn test_control(
        &mut self,
        context: DeviceContext,
        raw: &str,
        page: u8,
        slot: u8,
    ) -> Result<()> {
        if raw.len() > crate::layout_document::LIMIT {
            return Err("layout_too_large");
        }
        let unit = self
            .statuses()
            .into_iter()
            .find(|unit| unit.id == context.device)
            .ok_or("unknown_device")?;
        if !unit.input_ready || !unit.display_ready {
            return Err("device_not_ready");
        }
        let profile = self.catalog.profile(context.device)?;
        let descriptor = profile.document().descriptor();
        let event = if slot < descriptor.key_count() {
            RawEvent::Key {
                index: slot,
                pressed: false,
            }
        } else if descriptor.model() == decksmith_core::device_model::DeviceModel::Plus
            && (8..12).contains(&slot)
        {
            RawEvent::DialPush {
                index: slot - 8,
                pressed: false,
            }
        } else {
            return Err("invalid_control");
        };
        let current = DeviceContext {
            control: event.control(),
            ..context
        };
        if self.context(context.device, event.control())? != current || unit.page != Some(page) {
            return Err("save_and_select_page_first");
        }
        let supplied: serde_json::Value =
            serde_json::from_str(raw).map_err(|_| "invalid_config")?;
        let saved: serde_json::Value =
            serde_json::from_slice(profile.document().bytes()).map_err(|_| "invalid_config")?;
        if supplied != saved {
            return Err("save_and_select_page_first");
        }
        let received = decksmith_device::manager::ReceivedInput {
            context: current,
            input: decksmith_core::InputEvent {
                timestamp_ms: 0,
                event,
            },
            received_at: None,
        };
        match self.controls.resolve(&received)? {
            Some(Action::PushToTalk { .. }) => return Err("test_push_to_talk_on_device"),
            None | Some(Action::None) => return Err("no_action_assigned"),
            _ => {}
        }
        self.route(received).map(|_| ())
    }
    pub(crate) fn refresh_plugin_display(&mut self) {
        let Some(plan) = &self.native_plan else {
            return;
        };
        for unit in self.controls.manager().statuses() {
            if unit.connected
                && unit
                    .page
                    .and_then(|page| {
                        plan.pages
                            .get(&unit.id)
                            .and_then(|pages| pages.get(usize::from(page)))
                    })
                    .is_some_and(|page| page.homebridge)
            {
                self.dirty.insert(unit.id);
            }
        }
    }
    pub(crate) fn update_system(&mut self, states: crate::system_actions::States) {
        if self.system_states != states {
            self.system_states = states;
            self.system_contexts.clear();
            self.sync_system();
        }
    }
    fn sync_system(&mut self) {
        let Some(plan) = &self.native_plan else {
            return;
        };
        for unit in self.controls.manager().statuses() {
            if self.system_contexts.get(&unit.id) == Some(&unit.page) {
                continue;
            }
            self.system_contexts.insert(unit.id, unit.page);
            let state = unit
                .page
                .and_then(|page| {
                    plan.pages
                        .get(&unit.id)
                        .and_then(|pages| pages.get(usize::from(page)))
                })
                .map(|page| {
                    self.system_states
                        .iter()
                        .filter(|(command, _)| page.system.contains(*command))
                        .map(|(command, state)| (command.clone(), state.clone()))
                        .collect()
                })
                .unwrap_or_default();
            if let Some(touch) = self.touch.get_mut(&unit.id)
                && touch.system != state
            {
                touch.system = state;
                self.dirty.insert(unit.id);
            }
        }
    }
    pub fn update_touch(&mut self, id: DeviceId, state: TouchState) -> Result<()> {
        if state.targets.len() > 32 || state.levels.len() > 32 || state.feedback.len() > 32 {
            return Err("render_state_too_large");
        }
        *self.touch.get_mut(&id).ok_or("unknown_device")? = state;
        self.dirty.insert(id);
        Ok(())
    }
    pub fn update_levels(&mut self, id: DeviceId, levels: crate::meter::Levels) -> Result<()> {
        if self.catalog.profile(id)?.document().model()
            != decksmith_core::device_model::DeviceModel::Plus
        {
            return Err("device_has_no_touch");
        }
        if levels.len() > 32
            || levels
                .iter()
                .any(|(key, value)| key.len() > 512 || value.is_some_and(|n| n > 100))
        {
            return Err("render_state_too_large");
        }
        self.touch.get_mut(&id).ok_or("unknown_device")?.levels = levels;
        self.strip_dirty.insert(id);
        Ok(())
    }
    fn scope(&self, context: DeviceContext) -> ActionScope {
        let v = self.mirror.read().unwrap();
        ActionScope {
            runtime: v.runtime,
            foreground: v.foreground,
            device: context,
        }
    }
    fn route(
        &mut self,
        event: decksmith_device::manager::ReceivedInput,
    ) -> Result<Option<RequestId>> {
        if self.edits.contains_key(&event.context.device) {
            return Err("profile_update_pending");
        }
        let Some(action) = self.controls.resolve(&event)? else {
            return Ok(None);
        };
        let action = if self.native_plan.is_some() {
            crate::saved_native::normalize(action)
        } else {
            action
        };
        let id = event.context.device;
        match action {
            Action::None => Ok(None),
            Action::GoToPage { page } => {
                self.navigate(event.context, page)?;
                Ok(None)
            }
            Action::BrightnessAdjust { percent } => {
                let b = self
                    .statuses()
                    .into_iter()
                    .find(|s| s.id == id)
                    .ok_or("unknown_device")?
                    .brightness;
                self.controls
                    .brightness(event.context, (i16::from(b) + percent).clamp(0, 100) as u8)?;
                self.dirty.insert(id);
                Ok(None)
            }
            Action::PushToTalk { target } => {
                let mic = self.mics.as_ref().ok_or("shared_microphone_unavailable")?;
                let key = (id, event.context.control);
                match event.input.event {
                    RawEvent::Key { pressed: true, .. }
                    | RawEvent::DialPush { pressed: true, .. } => {
                        self.press = self
                            .press
                            .checked_add(1)
                            .ok_or("press_sequence_exhausted")?;
                        let token = HoldToken {
                            scope: self.scope(event.context),
                            press: self.press,
                        };
                        mic.acquire(token, Microphone::named(&target)?)?;
                        self.holds.insert(key, token);
                    }
                    RawEvent::Key { pressed: false, .. }
                    | RawEvent::DialPush { pressed: false, .. } => {
                        if let Some(token) = self.holds.remove(&key) {
                            mic.release(token)?;
                        }
                    }
                    _ => return Err("invalid_microphone_trigger"),
                };
                Ok(None)
            }
            plugin @ Action::Plugin { .. } => self
                .homebridge_actions
                .as_ref()
                .ok_or("homebridge_unavailable")?
                .submit(
                    self.scope(event.context),
                    Compiled::homebridge(plugin)?,
                    event.received_at,
                )
                .map(Some),
            builtin @ (Action::OpenApplication { .. }
            | Action::OpenWebsite { .. }
            | Action::System { .. }
            | Action::AudioSelect { .. }) => self
                .actions
                .submit(
                    self.scope(event.context),
                    Compiled::builtin(builtin)?,
                    event.received_at,
                )
                .map(Some),
            Action::MediaTarget { player, command } => self
                .actions
                .submit(
                    self.scope(event.context),
                    if self.dynamic_media {
                        Compiled::media_selector(&player, &command)?
                    } else {
                        match &self.media_bindings {
                            Some(bindings) => Compiled::media_bound(
                                bindings.get(&player).ok_or("media_unavailable")?.clone(),
                                &command,
                            )?,
                            None => Compiled::media(&player, &command)?,
                        }
                    },
                    event.received_at,
                )
                .map(Some),
            Action::AudioMute { target } if target.starts_with("input:") => {
                let mic = self.mics.as_ref().ok_or("shared_microphone_unavailable")?;
                mic.toggle(self.scope(event.context), Microphone::named(&target)?)?;
                Ok(None)
            }
            audio @ (Action::AudioAdjust { .. } | Action::AudioMute { .. }) => self
                .actions
                .submit(
                    self.scope(event.context),
                    if self.native_plan.is_some() {
                        Compiled::saved_audio(audio)?
                    } else {
                        Compiled::audio(audio)?
                    },
                    event.received_at,
                )
                .map(Some),
            _ => Err("shared_action_provider_not_connected"),
        }
    }
    /// No rendering or hardware calls run on this coordinator thread. Newest
    /// snapshots replace pending render state; feedback remains context tagged.
    pub fn tick(&mut self) -> Vec<Feedback> {
        if self.stopped {
            return Vec::new();
        }
        for (id, outcome) in self.io.reap() {
            if let Err(error) = outcome {
                self.retired_errors.insert(id, error);
            }
        }
        self.paint_gated();
        let events = self.controls.poll_round();
        self.sync();
        let mut feedback = Vec::new();
        for event in events {
            let scope = self.scope(event.context);
            if let Err(error) = self.route(event) {
                feedback.push(Feedback {
                    scope,
                    result: Err(error),
                });
            }
        }
        self.sync();
        self.sync_system();
        self.poll_native_meters();
        if let Some(plan) = &self.native_plan {
            let changed = self
                .native_observer
                .as_mut()
                .and_then(|observer| observer.snapshot());
            let refreshed = changed.is_some();
            if let Some(states) = changed {
                self.native_states = states;
            }
            for status in self.controls.manager().statuses() {
                let Ok(context) = self
                    .controls
                    .manager()
                    .context(status.id, ControlAddress::Key(0))
                else {
                    continue;
                };
                if !refreshed && self.native_contexts.get(&status.id) == Some(&context) {
                    continue;
                }
                self.native_contexts.insert(status.id, context);
                let Some(page) = status.page.and_then(|page| {
                    plan.pages
                        .get(&status.id)
                        .and_then(|rows| rows.get(usize::from(page)))
                }) else {
                    continue;
                };
                let wanted: Vec<_> = self
                    .native_states
                    .iter()
                    .filter(|(target, _)| page.audio.contains(target))
                    .cloned()
                    .collect();
                let audio = Some(
                    wanted
                        .iter()
                        .find(|(t, _)| t == "default_output")
                        .and_then(|(_, state)| *state),
                );
                let state = self.touch.get_mut(&status.id).unwrap();
                if state.targets != wanted || state.audio != audio {
                    let keys_changed = page.key_audio.iter().any(|target| {
                        state.targets.iter().find(|(t, _)| t == target)
                            != wanted.iter().find(|(t, _)| t == target)
                    });
                    state.targets = wanted;
                    state.audio = audio;
                    if keys_changed {
                        self.dirty.insert(status.id);
                    } else if status.model == decksmith_core::device_model::DeviceModel::Plus {
                        self.strip_dirty.insert(status.id);
                    }
                }
            }
        }
        if let Some(m) = &self.mics {
            for (id, lease) in &self.leases {
                if lease.recent(Duration::from_millis(200))
                    && self.mirror.read().unwrap().entries.contains_key(id)
                {
                    let _ = m.heartbeat(*id);
                }
            }
        }
        if let Some(mics) = &self.mics {
            let failures = mics.take_failures();
            let fatal = if self.mic_fatal_reported {
                None
            } else {
                mics.fatal_error()
            };
            self.mic_fatal_reported |= fatal.is_some();
            for status in self.statuses() {
                let Some(page) = self
                    .native_plan
                    .as_ref()
                    .and_then(|plan| plan.pages.get(&status.id))
                    .and_then(|pages| status.page.and_then(|page| pages.get(usize::from(page))))
                else {
                    continue;
                };
                let Some(error) = fatal.or_else(|| {
                    failures
                        .iter()
                        .find(|(target, _)| page.audio.contains(target.as_str()))
                        .map(|(_, error)| *error)
                }) else {
                    continue;
                };
                if let Ok(context) = self.context(status.id, ControlAddress::Key(0)) {
                    feedback.push(Feedback {
                        scope: self.scope(context),
                        result: Err(error),
                    });
                }
            }
        }
        while let Ok(Some(reply)) = self.actions.poll() {
            feedback.push(Feedback {
                scope: reply.completion.scope,
                result: reply.completion.result,
            });
        }
        if let Some(actions) = &self.homebridge_actions {
            while let Ok(Some(reply)) = actions.poll() {
                feedback.push(Feedback {
                    scope: reply.completion.scope,
                    result: reply.completion.result,
                });
            }
        }
        for reply in self.renders.poll() {
            self.render_pending.remove(&reply.scope.device.device);
            match reply.frame {
                Ok(device_render::Update::Full(frame)) => {
                    if let Err(e) =
                        self.controls
                            .paint(reply.scope.device, &frame.keys, frame.touch.as_deref())
                    {
                        feedback.push(Feedback {
                            scope: reply.scope,
                            result: Err(e),
                        });
                    } else {
                        self.painted
                            .insert(reply.scope.device.device, reply.scope.device);
                    }
                }
                Ok(device_render::Update::Touch(bytes)) => {
                    let context = DeviceContext {
                        control: ControlAddress::Touch,
                        ..reply.scope.device
                    };
                    if let Err(e) = self.controls.paint_touch(context, &bytes) {
                        feedback.push(Feedback {
                            scope: reply.scope,
                            result: Err(e),
                        });
                    }
                }
                Err(e) => feedback.push(Feedback {
                    scope: reply.scope,
                    result: Err(e),
                }),
            }
        }
        let dirty: Vec<_> = self.dirty.union(&self.strip_dirty).copied().collect();
        for id in dirty {
            let Ok(context) = self.controls.manager().context(id, ControlAddress::Key(0)) else {
                continue;
            };
            let Some(status) = self.statuses().into_iter().find(|s| s.id == id) else {
                continue;
            };
            // Meter bursts do not repeatedly supersede an initial full render.
            // Once its model/page frame is queued, only the latest strip is needed.
            if !self.dirty.contains(&id) && self.painted.get(&id) != Some(&context) {
                continue;
            }
            let full = self.dirty.contains(&id) || self.painted.get(&id) != Some(&context);
            let mut state = self.touch[&id].clone();
            state.brightness = Some(status.brightness);
            if self
                .renders
                .request(
                    self.scope(context),
                    status.model,
                    self.layouts[&id].clone(),
                    status.page.unwrap(),
                    state,
                    full,
                )
                .is_ok()
            {
                self.render_pending.insert(id);
                self.dirty.remove(&id);
                self.strip_dirty.remove(&id);
            }
        }
        // A reconnect or a completed post-fence drain requests a fresh frame.
        for status in self.statuses() {
            if let Ok(context) = self
                .controls
                .manager()
                .context(status.id, ControlAddress::Key(0))
                && self.painted.get(&status.id).is_some_and(|c| *c != context)
            {
                self.dirty.insert(status.id);
            }
        }
        self.sync();
        feedback
    }
    fn poll_native_meters(&mut self) {
        if Instant::now() < self.next_meter {
            return;
        }
        self.next_meter = Instant::now() + Duration::from_millis(50);
        let (Some(plan), Some(worker)) = (&self.native_plan, &self.native_meter) else {
            return;
        };
        let sessions = self.controls.manager().statuses();
        let subscriptions = plan.active_meters(&sessions, self.gate_state.0 || self.gate_state.1);
        let targets: Vec<_> = subscriptions
            .values()
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if targets != self.native_meter_targets {
            let Some(epoch) = self.native_meter_epoch.checked_add(1) else {
                let _ = worker.levels(0, Vec::new());
                return;
            };
            self.native_meter_epoch = epoch;
            self.native_meter_targets = targets.clone();
        }
        let levels = worker.levels(self.native_meter_epoch, targets);
        let mut routed = crate::saved_native::route_levels(&subscriptions, &levels);
        for s in sessions
            .iter()
            .filter(|s| s.model == decksmith_core::device_model::DeviceModel::Plus)
        {
            let wanted = routed.remove(&s.id).unwrap_or_default();
            let state = self.touch.get_mut(&s.id).unwrap();
            if state.levels != wanted {
                state.levels = wanted;
                self.strip_dirty.insert(s.id);
            }
        }
    }
    pub fn stop(
        &mut self,
    ) -> Vec<(
        DeviceId,
        std::result::Result<(), decksmith_device::DeviceError>,
    )> {
        self.stopped = true;
        self.mirror.write().unwrap().entries.clear();
        self.actions.stop();
        if let Some(actions) = &self.homebridge_actions {
            actions.stop();
        }
        if let Some(m) = &self.mics {
            let _ = m.stop();
        }
        self.renders.stop();
        self.controls.stop();
        let results = self.io.join();
        // Effects must settle before the caller reports/starts restoration. A
        // late native timeout still records its uncertainty after scope revoke.
        self.actions.quiesce();
        if let Some(actions) = &mut self.homebridge_actions {
            actions.quiesce();
        }
        if let Some(mics) = self.mics.take() {
            mics.join();
        }
        self.native_observer = None;
        self.native_meter = None;
        results
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod meter_tests {
    use super::*;
    use crate::device_profiles::{Enrollment, Store};
    use decksmith_core::device_model::{DeviceDescriptor, DeviceModel};
    use decksmith_device::{VirtualDeck, registry::Candidate};
    struct NoEffects;
    impl shared_worker::Backend for NoEffects {
        fn execute(&mut self, _: &crate::pages::Action, _: &shared_worker::Fence) -> Result<()> {
            Err("fixture_no_effects")
        }
    }
    #[test]
    fn cached_system_and_plugin_feedback_follow_page_without_new_observer_sample() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-provider-display-{}",
            random_device_id().unwrap()
        ));
        let store = Store::create(&root).unwrap();
        let id = random_device_id().unwrap();
        let discovery = Discovery::new(vec![Candidate::new(
            0,
            DeviceModel::Plus,
            Some("display-fixture"),
        )])
        .unwrap();
        store
            .enroll(0, &discovery, 0, Enrollment::new(id, "Display", 50, None))
            .unwrap();
        let keys = |command: &str, plugin: bool| {
            (0..8).map(|index| {
            if index==0 {serde_json::json!({"label":"System","action":{"type":"system","command":command}})}
            else if index==1&&plugin {serde_json::json!({"label":"Accessory","action":{"type":"none"},"plugin":{"provider":"com.infamous-pattern.openhomeb","action":"com.infamous-pattern.openhomeb.status","schema":2,"settings":{"accessoryId":"a".repeat(64)}}})}
            else {serde_json::json!({"label":"Empty","action":{"type":"none"}})}
        }).collect::<Vec<_>>()
        };
        let layout = serde_json::json!({"version":2,"model":"plus","pages":[{"name":"One","background":[30,34,39],"keys":keys("dnd",false)},{"name":"Two","background":[30,34,39],"keys":keys("bluetooth",true)}]});
        store
            .save_profile(
                id,
                1,
                &serde_json::to_vec(&layout).unwrap(),
                0,
                "Display",
                50,
            )
            .unwrap();
        let catalog = store.load().unwrap();
        let plan = crate::saved_native::Plan::from_catalog(&catalog).unwrap();
        let mut runtime = Runtime::start(catalog, vec![Box::new(NoEffects)], None).unwrap();
        runtime.native_plan = Some(plan);
        runtime.touch.get_mut(&id).unwrap().audio = Some(None);
        let states = [
            (
                "dnd".into(),
                crate::system_actions::State {
                    available: true,
                    active: true,
                    text: "On".into(),
                },
            ),
            (
                "bluetooth".into(),
                crate::system_actions::State {
                    available: true,
                    active: false,
                    text: "Off".into(),
                },
            ),
        ]
        .into_iter()
        .collect();
        runtime.update_system(states);
        assert_eq!(runtime.touch[&id].system.len(), 1);
        assert!(runtime.touch[&id].system["dnd"].active);
        assert_eq!(runtime.touch[&id].audio, Some(None));
        runtime.dirty.clear();
        runtime.refresh_plugin_display();
        assert!(runtime.dirty.is_empty());
        let revision = runtime
            .discover(Discovery::new(discovery.candidates().to_vec()).unwrap())
            .unwrap();
        runtime
            .attach(
                revision,
                &discovery,
                0,
                Box::new(VirtualDeck::new(DeviceModel::Plus)),
            )
            .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while !runtime.statuses()[0].input_ready || !runtime.statuses()[0].display_ready {
            assert!(Instant::now() < end);
            runtime.tick();
            std::thread::sleep(Duration::from_millis(2));
        }
        runtime
            .navigate(runtime.context(id, ControlAddress::Key(0)).unwrap(), 1)
            .unwrap();
        runtime.tick();
        assert_eq!(runtime.touch[&id].system.len(), 1);
        assert!(!runtime.touch[&id].system["bluetooth"].active);
        assert!(!runtime.touch[&id].system.contains_key("dnd"));
        runtime.dirty.clear();
        runtime.refresh_plugin_display();
        assert!(runtime.dirty.contains(&id));
        runtime.stop();
        drop(runtime);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn saved_meter_pipeline_routes_pages_and_disconnects_without_partner_or_key_repaint() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-saved-meter-{}",
            random_device_id().unwrap()
        ));
        let store = Store::create(&root).unwrap();
        let ids = [
            random_device_id().unwrap(),
            random_device_id().unwrap(),
            random_device_id().unwrap(),
        ];
        let models = [DeviceModel::Plus, DeviceModel::Plus, DeviceModel::Mk2];
        let discovery = || {
            Discovery::new(
                models
                    .into_iter()
                    .enumerate()
                    .map(|(i, m)| Candidate::new(i as u8, m, Some(&format!("meter-fixture-{i}"))))
                    .collect(),
            )
            .unwrap()
        };
        for (i, model) in models.into_iter().enumerate() {
            let d = discovery();
            store
                .enroll(
                    store.load().unwrap().generation,
                    &d,
                    i as u8,
                    Enrollment::new(ids[i], "Meter fixture", 50, None),
                )
                .unwrap();
            let keys = (0..DeviceDescriptor::for_model(model).key_count())
                .map(|_| serde_json::json!({"label":"Key","action":{"type":"none"}}))
                .collect::<Vec<_>>();
            let mut layout = serde_json::json!({"version":2,"model":model,"pages":[{"name":"One","background":[30,34,39],"keys":keys},{"name":"Two","background":[30,34,39],"keys":keys}]});
            if model == DeviceModel::Plus {
                let target = if i == 0 { "output:A" } else { "output:B" };
                layout["dials"] = serde_json::json!([
                    {"label":"Own target","rotation":"volume","step":1,"audio_target":target,"press":{"type":"none"}},
                    {"label":"Two","rotation":"brightness","step":1,"press":{"type":"none"}},
                    {"label":"Three","rotation":"brightness","step":1,"press":{"type":"none"}},
                    {"label":"Four","rotation":"brightness","step":1,"press":{"type":"none"}}
                ]);
                layout["pages"][1]["dial_overrides"] = serde_json::json!([{ "label":"Page target","rotation":"volume","step":1,"audio_target":"output:C","press":{"type":"none"}},null,null,null]);
            } else {
                layout["pages"][0]["keys"][0]["action"] =
                    serde_json::json!({"type":"audio_adjust","target":"output:C","percent":1});
            }
            store
                .save_profile(
                    ids[i],
                    1,
                    &serde_json::to_vec(&layout).unwrap(),
                    0,
                    "Meter fixture",
                    50,
                )
                .unwrap();
        }
        let catalog = store.load().unwrap();
        let plan = crate::saved_native::Plan::from_catalog(&catalog).unwrap();
        let mut runtime = Runtime::start(catalog, vec![Box::new(NoEffects)], None).unwrap();
        runtime.native_plan = Some(plan);
        let revision = runtime.discover(discovery()).unwrap();
        for (i, model) in models.into_iter().enumerate() {
            runtime
                .attach(
                    revision,
                    &discovery(),
                    i as u8,
                    Box::new(VirtualDeck::new(model)),
                )
                .unwrap();
        }
        let end = Instant::now() + Duration::from_secs(5);
        while !runtime.statuses().iter().all(|s| s.input_ready) {
            assert!(Instant::now() < end);
            runtime.tick();
            std::thread::sleep(Duration::from_millis(2));
        }
        let inject = |r: &mut Runtime, epoch, targets: Vec<&str>, values: Vec<(&str, u8)>| {
            r.native_meter = Some(crate::meter::Worker::fixture(
                epoch,
                targets.into_iter().map(str::to_string).collect(),
                values
                    .into_iter()
                    .map(|(t, v)| (t.into(), Some(v)))
                    .collect(),
            ));
            r.next_meter = Instant::now();
            r.poll_native_meters();
        };
        runtime.dirty.clear();
        runtime.strip_dirty.clear();
        inject(
            &mut runtime,
            2,
            vec!["output:A", "output:B"],
            vec![("output:A", 20), ("output:B", 80), ("output:C", 99)],
        );
        assert_eq!(runtime.native_meter_targets, vec!["output:A", "output:B"]);
        assert_eq!(
            runtime.touch[&ids[0]].levels.get("output:A"),
            Some(&Some(20))
        );
        assert_eq!(
            runtime.touch[&ids[1]].levels.get("output:B"),
            Some(&Some(80))
        );
        assert!(runtime.touch[&ids[2]].levels.is_empty());
        assert!(runtime.dirty.is_empty());
        let partner = runtime.context(ids[1], ControlAddress::Key(0)).unwrap();
        runtime
            .navigate(runtime.context(ids[0], ControlAddress::Key(0)).unwrap(), 1)
            .unwrap();
        inject(
            &mut runtime,
            3,
            vec!["output:B", "output:C"],
            vec![("output:B", 80), ("output:C", 30)],
        );
        assert_eq!(
            runtime.touch[&ids[0]].levels,
            [("output:C".into(), Some(30))].into_iter().collect()
        );
        assert_eq!(
            runtime.context(ids[1], ControlAddress::Key(0)).unwrap(),
            partner
        );
        assert!(!runtime.dirty.contains(&ids[1]));
        runtime.disconnect(ids[0]).unwrap();
        inject(&mut runtime, 4, vec!["output:B"], vec![("output:B", 80)]);
        assert!(runtime.touch[&ids[0]].levels.is_empty());
        assert_eq!(runtime.native_meter_targets, vec!["output:B"]);
        runtime.gate(true, false).unwrap();
        inject(&mut runtime, 5, vec![], vec![]);
        assert!(runtime.native_meter_targets.is_empty());
        assert!(runtime.touch[&ids[1]].levels.is_empty());
        assert!(runtime.stop().iter().all(|(_, r)| r.is_ok()));
        drop(runtime);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod coordinator_tests;
