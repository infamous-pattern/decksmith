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
    time::Duration,
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
pub struct Runtime {
    controls: Controls,
    catalog: Catalog,
    layouts: BTreeMap<DeviceId, Arc<[u8]>>,
    touch: BTreeMap<DeviceId, TouchState>,
    leases: BTreeMap<DeviceId, actor::Lease>,
    io: actor::Pool,
    renders: device_render::Pool,
    actions: shared_worker::Service,
    mics: Option<shared_microphone::Service>,
    mirror: Arc<RwLock<View>>,
    dirty: BTreeSet<DeviceId>,
    strip_dirty: BTreeSet<DeviceId>,
    render_pending: BTreeSet<DeviceId>,
    painted: BTreeMap<DeviceId, DeviceContext>,
    holds: BTreeMap<(DeviceId, ControlAddress), HoldToken>,
    press: u64,
    stopped: bool,
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
        Ok(Self {
            controls,
            catalog,
            layouts,
            touch,
            leases: BTreeMap::new(),
            io: actor::Pool::default(),
            renders,
            actions,
            mics,
            mirror,
            dirty: ids.into_iter().collect(),
            strip_dirty: BTreeSet::new(),
            render_pending: BTreeSet::new(),
            painted: BTreeMap::new(),
            holds: BTreeMap::new(),
            press: 0,
            stopped: false,
        })
    }
    pub fn context(&self, id: DeviceId, control: ControlAddress) -> Result<DeviceContext> {
        self.controls.manager().context(id, control)
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
                if self.dirty.contains(&s.id)
                    || self.strip_dirty.contains(&s.id)
                    || self.render_pending.contains(&s.id)
                {
                    s.display_ready = false;
                }
                s
            })
            .collect()
    }
    fn revoke(&mut self, id: DeviceId) {
        self.mirror.write().unwrap().entries.remove(&id);
        self.renders.invalidate(id);
        self.render_pending.remove(&id);
        self.painted.remove(&id);
        self.dirty.insert(id);
        self.actions.invalidate();
        if let Some(m) = &self.mics {
            let _ = m.invalidate();
        }
    }
    fn sync(&mut self) {
        let mut view = self.mirror.write().unwrap();
        view.entries.clear();
        for status in self.statuses() {
            if let (Ok(context), Some(lease)) = (
                self.controls
                    .manager()
                    .context(status.id, ControlAddress::Key(0)),
                self.leases.get(&status.id),
            ) {
                let epoch = lease.epoch();
                if lease.accepts(epoch) {
                    view.entries.insert(
                        status.id,
                        Entry {
                            context,
                            lease: lease.clone(),
                            epoch,
                            model: status.model,
                        },
                    );
                }
            }
        }
        self.holds.retain(|_, token| view.accepts(&token.scope));
        drop(view);
        self.actions.invalidate();
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
        self.revoke(context.device);
        let result = self.controls.navigate(context, page).map(|_| ());
        self.sync();
        result
    }
    pub fn gate(&mut self, locked: bool, suspended: bool) -> Result<()> {
        let ids: Vec<_> = self.layouts.keys().copied().collect();
        for id in ids {
            self.revoke(id);
        }
        let result = self.controls.gate(locked, suspended);
        self.sync();
        result
    }
    pub fn foreground(&mut self, revision: u64) -> Result<()> {
        let previous = self.mirror.read().unwrap().foreground;
        if revision == previous {
            return Ok(());
        }
        if revision < previous {
            return Err("stale_foreground_revision");
        }
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
        self.revoke(id);
        self.controls.apply_saved_profile(&catalog, id, binding)?;
        self.layouts
            .insert(id, Arc::from(catalog.profile(id)?.document().bytes()));
        self.catalog = catalog;
        self.sync();
        Ok(())
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
        let Some(action) = self.controls.resolve(&event)? else {
            return Ok(None);
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
            Action::MediaTarget { player, command } => self
                .actions
                .submit(
                    self.scope(event.context),
                    Compiled::media(&player, &command)?,
                    event.received_at,
                )
                .map(Some),
            audio @ (Action::AudioAdjust { .. } | Action::AudioMute { .. }) => self
                .actions
                .submit(
                    self.scope(event.context),
                    Compiled::audio(audio)?,
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
        self.io.reap();
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
        if let Some(m) = &self.mics {
            for (id, lease) in &self.leases {
                if lease.recent(Duration::from_millis(200))
                    && self.mirror.read().unwrap().entries.contains_key(id)
                {
                    let _ = m.heartbeat(*id);
                }
            }
        }
        while let Ok(Some(reply)) = self.actions.poll() {
            feedback.push(Feedback {
                scope: reply.completion.scope,
                result: reply.completion.result,
            });
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
    pub fn stop(
        &mut self,
    ) -> Vec<(
        DeviceId,
        std::result::Result<(), decksmith_device::DeviceError>,
    )> {
        self.stopped = true;
        self.mirror.write().unwrap().entries.clear();
        self.actions.stop();
        if let Some(m) = &self.mics {
            let _ = m.stop();
        }
        self.renders.stop();
        self.controls.stop();
        self.io.join()
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop();
    }
}
