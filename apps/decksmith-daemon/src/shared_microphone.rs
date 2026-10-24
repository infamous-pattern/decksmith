//! Dormant shared microphone owner. Releases update latest desired state outside
//! the ordinary action queue; a separate owner drives at most eight guardians.
use crate::shared_worker::Authority;
use decksmith_core::{
    device_context::DeviceId,
    microphone_holds::{DesiredMic, HoldToken, MAX_TARGETS, MicUpdate, MicrophoneHolds},
    shared_dispatch::{ActionScope, TargetId},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
const GUARD_TIMEOUT_MS: u64 = 2500;
const RETRY_MS: u64 = 250;
const MAX_ATTEMPTS: u8 = 3;
const RECONNECT_MS: u64 = 5000;

/// A structurally validated named microphone. Aliases/labels are never identity.
#[derive(Clone)]
pub struct Microphone(TargetId);
impl Microphone {
    pub fn named(target: &str) -> Result<Self> {
        if !target.starts_with("input:")
            || !crate::audio_target::valid(target)
            || target.strip_prefix("input:").is_some_and(|name| {
                name.starts_with('@') || name.bytes().all(|b| b.is_ascii_digit())
            })
        {
            return Err("shared_mic_requires_named_input");
        }
        Ok(Self(TargetId::new(target)?))
    }
}
#[derive(Default)]
pub struct Observation {
    pub live: bool,
    pub muted: bool,
    pub exited: Option<bool>,
    pub committed: bool,
}
/// Trusted adapters: all methods are bounded/nonblocking except terminate,
/// which must confirm owned writers ended. Never replace a still-running writer.
pub trait Guardian: Send {
    fn pulse(&mut self) -> Result<()>;
    fn close(&mut self);
    fn poll(&mut self) -> Result<Observation>;
    fn terminate(&mut self) -> bool;
    fn commit(&mut self) -> Result<()> {
        Err("shared_mic_commit_unsupported")
    }
}
pub trait Factory: Send + 'static {
    /// Only adapters that can safely revalidate an exact physical target may
    /// retry a disconnected target after the initial bounded mute attempts.
    fn reconnectable(&self, _target: &str) -> bool {
        false
    }
    fn start(&mut self, target: &str, desired: DesiredMic) -> Result<Box<dyn Guardian>>;
    fn toggle(&mut self, _target: &str) -> Result<Box<dyn Guardian>> {
        Err("shared_mic_toggle_unsupported")
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct ManualRequest {
    scope: ActionScope,
    sequence: u64,
}
struct State {
    holds: MicrophoneHolds,
    blocked: BTreeSet<TargetId>,
    failures: BTreeMap<TargetId, &'static str>,
    stopped: bool,
    now: u64,
    retry_requests: BTreeMap<TargetId, u64>,
    retry_sequence: u64,
    fatal: Option<&'static str>,
    manual: BTreeMap<TargetId, ManualRequest>,
}
impl State {
    // A tick sampled before a concurrent API call may arrive after that call.
    // Serialize sampled host times at one high-water mark; core remains strict.
    fn at(&mut self, now: u64) -> u64 {
        self.now = self.now.max(now);
        self.now
    }
    fn prepare(&mut self, update: &MicUpdate, now: u64, current: &Authority) -> bool {
        let now = self.at(now);
        self.holds
            .prepare_update(update, now, &**current)
            .unwrap_or(false)
    }
    fn ack(&mut self, update: &MicUpdate, now: u64, current: &Authority) {
        let now = self.at(now);
        let _ = self.holds.acknowledge(update, true, now, &**current);
    }
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
    origin: Instant,
}
impl Shared {
    fn now(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}
pub struct Service {
    shared: Arc<Shared>,
    current: Authority,
    thread: Option<JoinHandle<()>>,
}
impl Service {
    pub fn start(
        runtime: DeviceId,
        devices: &[DeviceId],
        current: Authority,
        factory: Box<dyn Factory>,
    ) -> Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                holds: MicrophoneHolds::new(runtime, devices)?,
                blocked: BTreeSet::new(),
                failures: BTreeMap::new(),
                stopped: false,
                now: 0,
                retry_requests: BTreeMap::new(),
                retry_sequence: 0,
                fatal: None,
                manual: BTreeMap::new(),
            }),
            wake: Condvar::new(),
            origin: Instant::now(),
        });
        let owner = shared.clone();
        let authority = current.clone();
        let thread = std::thread::Builder::new()
            .name("decksmith-shared-mic".into())
            .spawn(move || run(owner, authority, factory))
            .map_err(|_| "shared_mic_spawn_failed")?;
        Ok(Self {
            shared,
            current,
            thread: Some(thread),
        })
    }
    /// A cancelled hold is not a completed mute. Muted targets leave `states`
    /// only after the guardian reported mute and a successful owned child exit.
    pub fn quiet(&self) -> bool {
        let state = self.shared.state.lock().unwrap();
        state.holds.states().is_empty()
            && state.manual.is_empty()
            && state.blocked.is_empty()
            && state.fatal.is_none()
    }
    pub fn held(&self) -> usize {
        self.shared.state.lock().unwrap().holds.held()
    }
    pub fn acquire(&self, token: HoldToken, target: Microphone) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped {
            return Err("shared_mic_stopped");
        }
        let now = state.at(self.shared.now());
        let targets = state.holds.states();
        let reserved = state
            .manual
            .keys()
            .filter(|key| **key != target.0 && !targets.iter().any(|u| u.target == **key))
            .count();
        let result = state.holds.acquire_with_limit(
            token,
            target.0.clone(),
            now,
            &*self.current,
            MAX_TARGETS.saturating_sub(reserved),
        );
        if result.is_ok() {
            state.manual.remove(&target.0);
        }
        let result = if result.is_ok() && state.blocked.contains(&target.0) {
            // Consume the genuine down while recovery is pending. Never let a
            // held key silently reopen the mic when the mute retry later succeeds.
            state.holds.cancel_target(&target.0)?;
            Err("shared_mic_recovery_pending")
        } else {
            result
        };
        drop(state);
        self.shared.wake.notify_one();
        result
    }
    pub fn release(&self, token: HoldToken) -> Result<bool> {
        let mut state = self.shared.state.lock().unwrap();
        let now = state.at(self.shared.now());
        let result = state.holds.release(token, now, &*self.current);
        drop(state);
        self.shared.wake.notify_one();
        result
    }
    pub fn heartbeat(&self, device: DeviceId) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        let now = state.at(self.shared.now());
        let result = state.holds.heartbeat(device, now, &*self.current);
        let active = !state.holds.states().is_empty() || !state.manual.is_empty();
        drop(state);
        if active {
            self.shared.wake.notify_one();
        }
        result
    }
    pub fn invalidate(&self) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        let now = state.at(self.shared.now());
        let result = state.holds.maintain(now, &*self.current);
        let active = !state.holds.states().is_empty() || !state.manual.is_empty();
        drop(state);
        if active {
            self.shared.wake.notify_one();
        }
        result
    }
    pub fn mute(&self, target: Microphone) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped {
            return Err("shared_mic_stopped");
        }
        let targets: BTreeSet<_> = state
            .holds
            .states()
            .into_iter()
            .map(|u| u.target)
            .chain(state.manual.keys().cloned())
            .collect();
        if !targets.contains(&target.0) && targets.len() >= MAX_TARGETS {
            return Err("shared_mic_target_limit");
        }
        state.holds.explicit_mute(&target.0)?;
        state.manual.remove(&target.0);
        state.retry_sequence = state
            .retry_sequence
            .checked_add(1)
            .ok_or("shared_mic_retry_exhausted")?;
        let sequence = state.retry_sequence;
        state.retry_requests.insert(target.0, sequence);
        // Only this explicit API request resets an exhausted mute retry budget.
        drop(state);
        self.shared.wake.notify_one();
        Ok(())
    }
    /// Ordinary mute toggles run on this same owner. With any active hold,
    /// Mute always cancels every hold rather than reopening a live microphone.
    pub fn toggle(&self, scope: ActionScope, target: Microphone) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped || !scope.structurally_valid() || !(self.current)(&scope) {
            return Err("stale_action_scope");
        }
        let now = state.at(self.shared.now());
        state.holds.maintain(now, &*self.current)?;
        if state.holds.states().iter().any(|u| u.target == target.0)
            || state.blocked.contains(&target.0)
        {
            drop(state);
            return self.mute(target);
        }
        if state.manual.contains_key(&target.0) {
            return Err("shared_mic_toggle_pending");
        }
        if state.manual.len() + state.holds.states().len() >= MAX_TARGETS {
            return Err("shared_mic_target_limit");
        }
        state.retry_sequence = state
            .retry_sequence
            .checked_add(1)
            .ok_or("shared_mic_retry_exhausted")?;
        let sequence = state.retry_sequence;
        state
            .manual
            .insert(target.0, ManualRequest { scope, sequence });
        drop(state);
        self.shared.wake.notify_one();
        Ok(())
    }
    /// Target keys are private resource identifiers, not public display labels.
    pub fn take_failures(&self) -> Vec<(TargetId, &'static str)> {
        std::mem::take(&mut self.shared.state.lock().unwrap().failures)
            .into_iter()
            .collect()
    }
    pub fn fatal_error(&self) -> Option<&'static str> {
        self.shared.state.lock().unwrap().fatal
    }
    pub fn stop(&self) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        state.stopped = true;
        state.manual.clear();
        let result = state.holds.stop();
        drop(state);
        self.shared.wake.notify_one();
        result
    }
    pub fn join(mut self) {
        let _ = self.stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
struct Slot {
    update: MicUpdate,
    guard: Option<Box<dyn Guardian>>,
    closing: bool,
    muted: bool,
    started: u64,
    attempts: u8,
    retry_at: u64,
    failed: bool,
    retry_request: u64,
}
struct Driver {
    slots: BTreeMap<TargetId, Slot>,
    factory: Box<dyn Factory>,
    manual: BTreeMap<TargetId, ManualSlot>,
}
struct ManualSlot {
    request: ManualRequest,
    guard: Box<dyn Guardian>,
    closing: bool,
    committed: bool,
    acknowledged: bool,
    muted: bool,
    started: u64,
}
impl Driver {
    fn fail(shared: &Shared, target: &TargetId, code: &'static str) {
        let mut state = shared.state.lock().unwrap();
        // Manual failures also need a pending safe mute, never replay a toggle.
        state.manual.remove(target);
        let _ = state.holds.explicit_mute(target);
        state.blocked.insert(target.clone());
        if state.failures.contains_key(target) || state.failures.len() < MAX_TARGETS {
            state.failures.insert(target.clone(), code);
        } else {
            state.fatal = Some("shared_mic_failure_budget");
            state.stopped = true;
            let _ = state.holds.stop();
        }
    }
    fn latest(shared: &Shared, current: &Authority, now: u64) -> Vec<MicUpdate> {
        let mut state = shared.state.lock().unwrap();
        let now = state.at(now);
        if state.holds.maintain(now, &**current).is_err() {
            state.stopped = true;
        }
        state.holds.states()
    }
    fn step(&mut self, shared: &Shared, current: &Authority, now: u64) {
        self.step_manual(shared, current, now);
        let states = Self::latest(shared, current, now);
        for update in &states {
            if !self.slots.contains_key(&update.target) {
                assert!(self.slots.len() < MAX_TARGETS);
                self.slots.insert(
                    update.target.clone(),
                    Slot {
                        update: update.clone(),
                        guard: None,
                        closing: false,
                        muted: false,
                        started: now,
                        attempts: 0,
                        retry_at: now,
                        failed: false,
                        retry_request: shared
                            .state
                            .lock()
                            .unwrap()
                            .retry_requests
                            .get(&update.target)
                            .copied()
                            .unwrap_or(0),
                    },
                );
            }
        }
        for (target, slot) in &mut self.slots {
            let latest = Self::latest(shared, current, now)
                .into_iter()
                .find(|u| u.target == *target);
            if let Some(guard) = &mut slot.guard {
                let wants_live = latest.as_ref().is_some_and(|u| {
                    u.desired == DesiredMic::Live && u.revision == slot.update.revision
                });
                if !slot.closing && !wants_live {
                    guard.close();
                    slot.closing = true;
                    slot.started = now;
                }
                if !slot.closing {
                    // Check leases once more before sending a heartbeat. The
                    // helper also drains EOF after discovery before opening.
                    let allowed = shared
                        .state
                        .lock()
                        .unwrap()
                        .prepare(&slot.update, now, current);
                    if !allowed {
                        guard.close();
                        slot.closing = true;
                        slot.started = now;
                    } else if guard.pulse().is_err() {
                        guard.close();
                        slot.closing = true;
                        slot.started = now;
                        Self::fail(shared, target, "shared_mic_heartbeat_failed");
                        slot.failed = true;
                    }
                }
                let observation = guard.poll();
                match observation {
                    Ok(observation) => {
                        if observation.live
                            && !observation.muted
                            && observation.exited.is_none()
                            && !slot.closing
                        {
                            shared.state.lock().unwrap().ack(&slot.update, now, current);
                        }
                        if observation.exited.is_some() && !slot.closing {
                            // Even a successful watchdog mute is unexpected
                            // while the host still wants Live. Consume those holds.
                            Self::fail(shared, target, "shared_mic_unexpected_exit");
                            slot.failed = true;
                        }
                        slot.muted |= observation.muted;
                        if let Some(success) = observation.exited {
                            slot.guard.take();
                            if success && slot.muted {
                                let mut state = shared.state.lock().unwrap();
                                if let Some(update) =
                                    state.holds.states().into_iter().find(|u| {
                                        u.target == *target && u.desired == DesiredMic::Muted
                                    })
                                {
                                    state.ack(&update, now, current);
                                    state.blocked.remove(target);
                                    state.retry_requests.remove(target);
                                }
                                slot.attempts = 0;
                                slot.failed = false;
                                slot.retry_at = now;
                            } else {
                                Self::fail(shared, target, "shared_mic_guard_failed");
                                slot.failed = true;
                                slot.retry_at = now + RETRY_MS;
                            }
                        }
                    }
                    Err(_) => {
                        if !slot.closing {
                            slot.started = now;
                        }
                        guard.close();
                        slot.closing = true;
                        if !slot.failed {
                            Self::fail(shared, target, "shared_mic_protocol_failed");
                        }
                        slot.failed = true;
                    }
                }
                if let Some(guard) = &mut slot.guard
                    && now.saturating_sub(slot.started) >= GUARD_TIMEOUT_MS
                    && slot.closing
                {
                    if guard.terminate() {
                        slot.guard.take();
                        Self::fail(shared, target, "shared_mic_guard_timeout");
                        slot.failed = true;
                        slot.retry_at = now + RETRY_MS;
                    } else {
                        // Never replace this target's unknown writer. Stop Live
                        // ownership; other targets must still attempt safe mute.
                        Self::fail(shared, target, "shared_mic_writer_not_terminated");
                        let mut state = shared.state.lock().unwrap();
                        state.stopped = true;
                        state.fatal = Some("shared_mic_writer_not_terminated");
                        let _ = state.holds.stop();
                    }
                }
            }
            if slot.guard.is_none() {
                if self.manual.contains_key(target) {
                    continue;
                }
                let next = Self::latest(shared, current, now)
                    .into_iter()
                    .find(|u| u.target == *target);
                if let Some(update) = next {
                    let retry_request = shared
                        .state
                        .lock()
                        .unwrap()
                        .retry_requests
                        .get(target)
                        .copied()
                        .unwrap_or(0);
                    if slot.failed && retry_request != 0 && retry_request != slot.retry_request {
                        slot.attempts = 0;
                        slot.retry_at = now;
                        slot.retry_request = retry_request;
                    }
                    // One slow mute-only probe after the initial burst. No writer
                    // may remain, and stopping never schedules reconnection.
                    if slot.attempts >= MAX_ATTEMPTS
                        && update.desired == DesiredMic::Muted
                        && !shared.state.lock().unwrap().stopped
                        && now >= slot.retry_at.saturating_add(RECONNECT_MS)
                        && self.factory.reconnectable(target.as_str())
                    {
                        slot.attempts = MAX_ATTEMPTS - 1;
                    }
                    if now < slot.retry_at || slot.attempts >= MAX_ATTEMPTS {
                        continue;
                    }
                    slot.update = update.clone();
                    let allowed = shared.state.lock().unwrap().prepare(&update, now, current);
                    if !allowed {
                        continue;
                    }
                    if update.desired == DesiredMic::Muted {
                        slot.attempts += 1;
                    }
                    match self.factory.start(target.as_str(), update.desired) {
                        Ok(mut guard) => {
                            slot.closing = update.desired == DesiredMic::Muted;
                            slot.muted = false;
                            slot.started = now;
                            if slot.closing {
                                guard.close();
                            }
                            slot.guard = Some(guard);
                        }
                        Err(_) => {
                            Self::fail(shared, target, "shared_mic_spawn_failed");
                            slot.failed = true;
                            slot.retry_at = now + RETRY_MS;
                        }
                    }
                    if slot.failed
                        && let Some(update) = Self::latest(shared, current, now)
                            .into_iter()
                            .find(|u| u.target == *target)
                    {
                        slot.update = update;
                    }
                }
            }
        }
        let targets = Self::latest(shared, current, now);
        self.slots.retain(|target, slot| {
            slot.guard.is_some() || targets.iter().any(|u| u.target == *target)
        });
    }
    fn step_manual(&mut self, shared: &Shared, current: &Authority, now: u64) {
        let requests = {
            let mut state = shared.state.lock().unwrap();
            state.manual.retain(|_, request| current(&request.scope));
            state.manual.clone()
        };
        for (target, request) in requests {
            if self.slots.contains_key(&target) || self.manual.contains_key(&target) {
                continue;
            }
            match self.factory.toggle(target.as_str()) {
                Ok(guard) => {
                    self.manual.insert(
                        target,
                        ManualSlot {
                            request,
                            guard,
                            closing: false,
                            committed: false,
                            acknowledged: false,
                            muted: false,
                            started: now,
                        },
                    );
                }
                Err(_) => {
                    shared.state.lock().unwrap().manual.remove(&target);
                    Self::fail(shared, &target, "shared_mic_toggle_failed");
                }
            }
        }
        let mut ended = Vec::new();
        for (target, slot) in &mut self.manual {
            let allowed = {
                let state = shared.state.lock().unwrap();
                !state.stopped
                    && state.manual.get(target) == Some(&slot.request)
                    && current(&slot.request.scope)
            };
            if !slot.closing && !slot.committed && (!allowed || slot.guard.pulse().is_err()) {
                slot.guard.close();
                slot.closing = true;
            }
            match slot.guard.poll() {
                Ok(observation) => {
                    if observation.live {
                        slot.muted = false;
                    }
                    if observation.muted {
                        slot.muted = true;
                    }
                    slot.acknowledged |= observation.committed;
                    if (observation.live || observation.muted) && !slot.closing && !slot.committed {
                        // Discovery/write may have waited. Recheck the exact
                        // request and scope immediately before persistence.
                        let state = shared.state.lock().unwrap();
                        if !state.stopped
                            && state.manual.get(target) == Some(&slot.request)
                            && current(&slot.request.scope)
                            && slot.guard.commit().is_ok()
                        {
                            slot.committed = true;
                        } else {
                            slot.guard.close();
                            slot.closing = true;
                        }
                    }
                    if let Some(success) = observation.exited {
                        ended.push(target.clone());
                        let mut state = shared.state.lock().unwrap();
                        if state.manual.get(target) == Some(&slot.request) {
                            state.manual.remove(target);
                        }
                        drop(state);
                        if !success
                            || !(slot.committed && slot.acknowledged || slot.closing && slot.muted)
                        {
                            Self::fail(shared, target, "shared_mic_toggle_uncertain");
                        }
                    }
                }
                Err(_) => {
                    slot.guard.close();
                    slot.closing = true;
                }
            }
            if !ended.contains(target) && now.saturating_sub(slot.started) >= GUARD_TIMEOUT_MS {
                if slot.guard.terminate() {
                    ended.push(target.clone());
                    shared.state.lock().unwrap().manual.remove(target);
                    Self::fail(shared, target, "shared_mic_toggle_timeout");
                } else {
                    let mut state = shared.state.lock().unwrap();
                    state.stopped = true;
                    state.manual.clear();
                    state.fatal = Some("shared_mic_writer_not_terminated");
                    let _ = state.holds.stop();
                }
            }
        }
        for target in ended {
            self.manual.remove(&target);
        }
    }
    fn quiesce(&mut self) {
        for slot in self.manual.values_mut() {
            slot.guard.close();
            let _ = slot.guard.terminate();
        }
        for slot in self.slots.values_mut() {
            if let Some(guard) = &mut slot.guard {
                guard.close();
                let _ = guard.terminate();
            }
        }
    }
}
fn run(shared: Arc<Shared>, current: Authority, factory: Box<dyn Factory>) {
    let mut driver = Driver {
        slots: BTreeMap::new(),
        factory,
        manual: BTreeMap::new(),
    };
    let mut stop_at = None;
    loop {
        driver.step(&shared, &current, shared.now());
        let state = shared.state.lock().unwrap();
        if state.stopped {
            let deadline = *stop_at.get_or_insert_with(|| Instant::now() + Duration::from_secs(10));
            let exhausted = !driver.slots.is_empty()
                && driver.manual.is_empty()
                && driver
                    .slots
                    .values()
                    .all(|slot| slot.guard.is_none() && slot.attempts >= MAX_ATTEMPTS);
            if exhausted || Instant::now() >= deadline {
                tracing::warn!(
                    error_code = "shared_mic_shutdown_unconfirmed",
                    "Microphone mute could not be confirmed before shutdown"
                );
            }
            if driver.slots.is_empty() && driver.manual.is_empty()
                || exhausted
                || Instant::now() >= deadline
            {
                drop(state);
                driver.quiesce();
                return;
            }
        }
        if driver.manual.is_empty()
            && (driver.slots.is_empty()
                || !state.stopped
                    && driver
                        .slots
                        .values()
                        .all(|slot| slot.guard.is_none() && slot.attempts >= MAX_ATTEMPTS))
        {
            if driver
                .slots
                .keys()
                .any(|target| driver.factory.reconnectable(target.as_str()))
            {
                drop(
                    shared
                        .wake
                        .wait_timeout(state, Duration::from_millis(RECONNECT_MS))
                        .unwrap(),
                );
            } else {
                drop(shared.wake.wait(state).unwrap());
            }
        } else {
            drop(
                shared
                    .wake
                    .wait_timeout(state, Duration::from_millis(20))
                    .unwrap(),
            );
        }
    }
}

pub struct NativeFactory {
    root: std::path::PathBuf,
    pins: BTreeMap<String, std::path::PathBuf>,
}
impl NativeFactory {
    pub fn new() -> Result<Self> {
        use std::os::unix::fs::DirBuilderExt;
        let root = std::env::temp_dir().join(format!(
            "decksmith-mic-{}",
            crate::device_profiles::random_device_id()?
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .map_err(|_| "shared_mic_pin_directory_failed")?;
        Ok(Self {
            root,
            pins: BTreeMap::new(),
        })
    }
    fn pin(&mut self, target: &str) -> Result<std::path::PathBuf> {
        Microphone::named(target)?;
        if !self.pins.contains_key(target) {
            if self.pins.len() >= MAX_TARGETS {
                return Err("shared_mic_target_limit");
            }
            self.pins.insert(
                target.into(),
                self.root.join(format!("{}.json", self.pins.len())),
            );
        }
        Ok(self.pins[target].clone())
    }
}
impl Drop for NativeFactory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
impl Factory for NativeFactory {
    fn reconnectable(&self, target: &str) -> bool {
        self.pins.contains_key(target)
    }
    fn start(&mut self, target: &str, desired: DesiredMic) -> Result<Box<dyn Guardian>> {
        let pin = self.pin(target)?;
        NativeGuardian::start(
            target,
            if desired == DesiredMic::Live {
                "hold"
            } else {
                "mute"
            },
            &pin,
        )
        .map(|g| Box::new(g) as Box<dyn Guardian>)
    }
    fn toggle(&mut self, target: &str) -> Result<Box<dyn Guardian>> {
        let pin = self.pin(target)?;
        let mut guard = NativeGuardian::start(target, "toggle", &pin)?;
        guard.manual = true;
        Ok(Box::new(guard))
    }
}
struct NativeGuardian {
    child: std::process::Child,
    input: Option<std::process::ChildStdin>,
    output: std::process::ChildStdout,
    bytes: Vec<u8>,
    live_seen: bool,
    muted_seen: bool,
    committed_seen: bool,
    manual: bool,
}
impl NativeGuardian {
    fn start(target: &str, mode: &str, pin: &std::path::Path) -> Result<Self> {
        use std::{
            os::unix::process::CommandExt,
            process::{Command, Stdio},
        };
        let child = Command::new("/usr/bin/python3")
            .arg("-B")
            .arg(crate::runtime_paths::helper("shared_microphone_native.py"))
            .arg(target)
            .arg(mode)
            .arg(pin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|_| "shared_mic_spawn_failed")?;
        Self::from_child(child)
    }
    fn from_child(mut child: std::process::Child) -> Result<Self> {
        use std::os::unix::io::AsRawFd;
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        for fd in [input.as_raw_fd(), output.as_raw_fd()] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
            {
                unsafe {
                    libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
                }
                let _ = child.kill();
                let _ = child.wait();
                return Err("shared_mic_pipe_failed");
            }
        }
        Ok(Self {
            child,
            input: Some(input),
            output,
            bytes: Vec::new(),
            live_seen: false,
            muted_seen: false,
            committed_seen: false,
            manual: false,
        })
    }
}
impl Guardian for NativeGuardian {
    fn commit(&mut self) -> Result<()> {
        use std::io::Write;
        if !self.manual || self.committed_seen {
            return Err("shared_mic_commit_unsupported");
        }
        self.input
            .as_mut()
            .ok_or("shared_mic_pipe_closed")?
            .write_all(b"commit\n")
            .map_err(|_| "shared_mic_pipe_failed")
    }
    fn pulse(&mut self) -> Result<()> {
        use std::io::Write;
        self.input
            .as_mut()
            .ok_or("shared_mic_pipe_closed")?
            .write_all(b"hold\n")
            .map_err(|_| "shared_mic_pipe_failed")
    }
    fn close(&mut self) {
        self.input.take();
    }
    fn poll(&mut self) -> Result<Observation> {
        use std::io::Read;
        let mut o = Observation {
            exited: self
                .child
                .try_wait()
                .map_err(|_| "shared_mic_wait_failed")?
                .map(|status| status.success()),
            ..Default::default()
        };
        let mut bytes = [0; 64];
        match self.output.read(&mut bytes) {
            Ok(n) => self.bytes.extend_from_slice(&bytes[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => return Err("shared_mic_pipe_failed"),
        }
        if self.bytes.len() > 64 {
            return Err("shared_mic_protocol_failed");
        }
        while let Some(index) = self.bytes.iter().position(|b| *b == b'\n') {
            match &self.bytes[..index] {
                b"live" if !self.live_seen && !self.muted_seen => {
                    o.live = true;
                    self.live_seen = true;
                }
                b"muted" if !self.muted_seen => {
                    o.muted = true;
                    self.muted_seen = true;
                }
                b"committed"
                    if self.manual
                        && !self.committed_seen
                        && (self.live_seen || self.muted_seen) =>
                {
                    o.committed = true;
                    self.committed_seen = true;
                }
                _ => return Err("shared_mic_protocol_failed"),
            }
            self.bytes.drain(..=index);
        }
        if o.exited.is_some() && !self.bytes.is_empty() {
            return Err("shared_mic_protocol_failed");
        }
        Ok(o)
    }
    fn terminate(&mut self) -> bool {
        self.close();
        if self.child.try_wait().ok().flatten().is_some() {
            return true;
        }
        unsafe {
            libc::killpg(self.child.id() as libc::pid_t, libc::SIGKILL);
        }
        let _ = self.child.kill();
        // A killed helper may be waiting in the kernel. Never block the owner
        // forever in wait(); retain ownership if exit cannot yet be confirmed.
        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return true,
                Err(_) => return false,
                Ok(None) if Instant::now() >= deadline => return false,
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    }
}
impl Drop for NativeGuardian {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_core::{
        device_context::DeviceContext, device_model::ControlAddress, shared_dispatch::ActionScope,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    #[derive(Default)]
    struct Fake {
        target: String,
        desired: Option<DesiredMic>,
        closed: bool,
        pulses: usize,
        delayed: bool,
        terminated: bool,
        event: Observation,
        manual: bool,
    }
    #[derive(Default)]
    struct Trace {
        guards: Vec<Arc<Mutex<Fake>>>,
        fail: bool,
        reconnect: bool,
        starts: Vec<DesiredMic>,
    }
    struct FakeFactory(Arc<Mutex<Trace>>);
    struct FakeGuard(Arc<Mutex<Fake>>);
    impl Factory for FakeFactory {
        fn reconnectable(&self, _target: &str) -> bool {
            self.0.lock().unwrap().reconnect
        }
        fn toggle(&mut self, target: &str) -> Result<Box<dyn Guardian>> {
            let fake = Arc::new(Mutex::new(Fake {
                target: target.into(),
                manual: true,
                ..Default::default()
            }));
            self.0.lock().unwrap().guards.push(fake.clone());
            Ok(Box::new(FakeGuard(fake)))
        }
        fn start(&mut self, target: &str, desired: DesiredMic) -> Result<Box<dyn Guardian>> {
            let mut trace = self.0.lock().unwrap();
            trace.starts.push(desired);
            if trace.fail {
                return Err("fixture_spawn_failed");
            }
            let fake = Arc::new(Mutex::new(Fake {
                target: target.into(),
                desired: Some(desired),
                ..Default::default()
            }));
            trace.guards.push(fake.clone());
            Ok(Box::new(FakeGuard(fake)))
        }
    }
    impl Guardian for FakeGuard {
        fn commit(&mut self) -> Result<()> {
            let mut g = self.0.lock().unwrap();
            if !g.manual || g.closed {
                return Err("fixture_invalid_commit");
            }
            g.event.committed = true;
            g.event.exited = Some(true);
            Ok(())
        }
        fn pulse(&mut self) -> Result<()> {
            let mut g = self.0.lock().unwrap();
            if g.closed {
                return Err("fixture_closed");
            }
            g.pulses += 1;
            Ok(())
        }
        fn close(&mut self) {
            let mut g = self.0.lock().unwrap();
            if !g.closed {
                g.closed = true;
                if !g.delayed {
                    g.event.muted = true;
                    g.event.exited = Some(true);
                }
            }
        }
        fn poll(&mut self) -> Result<Observation> {
            let mut g = self.0.lock().unwrap();
            let o = std::mem::take(&mut g.event);
            if o.exited.is_some() {
                g.terminated = true;
            }
            Ok(o)
        }
        fn terminate(&mut self) -> bool {
            self.0.lock().unwrap().terminated = true;
            true
        }
    }
    fn id(n: u8) -> DeviceId {
        DeviceId::from_bytes([n; 16]).unwrap()
    }
    fn token(device: u8, press: u64, binding: u64) -> HoldToken {
        HoldToken {
            scope: ActionScope {
                runtime: id(99),
                foreground: 0,
                device: DeviceContext {
                    supervisor: 1,
                    device: id(device),
                    connection: 1,
                    binding,
                    lock_epoch: 0,
                    control: if device == 1 {
                        ControlAddress::DialPush(0)
                    } else {
                        ControlAddress::Key(14)
                    },
                },
            },
            press,
        }
    }
    struct Rig {
        shared: Shared,
        current: Authority,
        revision: Arc<AtomicU64>,
        driver: Driver,
        trace: Arc<Mutex<Trace>>,
    }
    impl Rig {
        fn new() -> Self {
            let revision = Arc::new(AtomicU64::new(1));
            let read = revision.clone();
            let trace = Arc::new(Mutex::new(Trace::default()));
            Self {
                shared: Shared {
                    state: Mutex::new(State {
                        holds: MicrophoneHolds::new(id(99), &[id(1), id(2)]).unwrap(),
                        blocked: BTreeSet::new(),
                        failures: BTreeMap::new(),
                        stopped: false,
                        now: 0,
                        retry_requests: BTreeMap::new(),
                        retry_sequence: 0,
                        fatal: None,
                        manual: BTreeMap::new(),
                    }),
                    wake: Condvar::new(),
                    origin: Instant::now(),
                },
                current: Arc::new(move |s| s.device.binding == read.load(Ordering::Acquire)),
                revision,
                driver: Driver {
                    slots: BTreeMap::new(),
                    factory: Box::new(FakeFactory(trace.clone())),
                    manual: BTreeMap::new(),
                },
                trace,
            }
        }
        fn acquire(&self, t: HoldToken, name: &str, now: u64) -> Result<()> {
            let mut state = self.shared.state.lock().unwrap();
            let now = state.at(now);
            state
                .holds
                .acquire(t, TargetId::new(name).unwrap(), now, &*self.current)
        }
        fn release(&self, t: HoldToken, now: u64) {
            let mut state = self.shared.state.lock().unwrap();
            let now = state.at(now);
            state.holds.release(t, now, &*self.current).unwrap();
        }
        fn heartbeat(&self, device: u8, now: u64) {
            let mut state = self.shared.state.lock().unwrap();
            let now = state.at(now);
            state
                .holds
                .heartbeat(id(device), now, &*self.current)
                .unwrap();
        }
        fn step(&mut self, now: u64) {
            self.driver.step(&self.shared, &self.current, now);
        }
        fn guard(&self, n: usize) -> Arc<Mutex<Fake>> {
            self.trace.lock().unwrap().guards[n].clone()
        }
        fn live(&self, n: usize) {
            self.guard(n).lock().unwrap().event.live = true;
        }
        fn held(&self) -> usize {
            self.shared.state.lock().unwrap().holds.held()
        }
        fn explicit_mute(&self, name: &str) {
            let mut state = self.shared.state.lock().unwrap();
            let target = TargetId::new(name).unwrap();
            state.holds.explicit_mute(&target).unwrap();
            state.retry_sequence += 1;
            let sequence = state.retry_sequence;
            state.retry_requests.insert(target, sequence);
        }
    }
    #[test]
    fn sleep_quiet_waits_for_actual_mute_and_owned_guardian_exit() {
        let revision = Arc::new(AtomicU64::new(1));
        let read = revision.clone();
        let authority: Authority =
            Arc::new(move |scope| scope.device.binding == read.load(Ordering::Acquire));
        let trace = Arc::new(Mutex::new(Trace::default()));
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            authority,
            Box::new(FakeFactory(trace.clone())),
        )
        .unwrap();
        service
            .acquire(token(1, 1, 1), Microphone::named("input:mic").unwrap())
            .unwrap();
        let end = Instant::now() + Duration::from_secs(3);
        while trace.lock().unwrap().guards.is_empty() {
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
        let guard = trace.lock().unwrap().guards[0].clone();
        guard.lock().unwrap().delayed = true;
        revision.store(2, Ordering::Release);
        service.invalidate().unwrap();
        assert_eq!(service.held(), 0);
        assert!(!service.quiet());
        while !guard.lock().unwrap().closed {
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
        guard.lock().unwrap().event.muted = true;
        std::thread::sleep(Duration::from_millis(30));
        assert!(
            !service.quiet(),
            "mute acknowledgement without owned child exit is insufficient"
        );
        guard.lock().unwrap().event.exited = Some(true);
        while !service.quiet() {
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
        service.join();
    }
    #[test]
    fn plus_and_mk2_overlap_share_one_guardian_until_the_last_release() {
        let mut r = Rig::new();
        let a = token(1, 1, 1);
        let b = token(2, 1, 1);
        r.acquire(a, "input:mic", 0).unwrap();
        r.acquire(b, "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        assert_eq!(r.trace.lock().unwrap().starts, vec![DesiredMic::Live]);
        assert_eq!(r.guard(0).lock().unwrap().target, "input:mic");
        r.release(a, 2);
        r.step(2);
        assert!(!r.guard(0).lock().unwrap().closed);
        assert_eq!(r.held(), 1);
        r.release(b, 3);
        r.step(3);
        assert!(r.guard(0).lock().unwrap().closed);
        assert!(r.driver.slots.is_empty());
        assert!(r.shared.state.lock().unwrap().holds.states().is_empty());
    }
    #[test]
    fn delayed_old_close_finishes_before_a_fresh_hold_starts_a_new_guardian() {
        let mut r = Rig::new();
        let a = token(1, 1, 1);
        r.acquire(a, "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        r.guard(0).lock().unwrap().delayed = true;
        r.release(a, 2);
        r.step(2);
        r.acquire(token(1, 2, 1), "input:mic", 3).unwrap();
        r.step(3);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 1);
        r.guard(0).lock().unwrap().event = Observation {
            muted: true,
            exited: Some(true),
            ..Default::default()
        };
        r.step(4);
        assert!(r.guard(0).lock().unwrap().terminated);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 2);
        assert_eq!(r.guard(1).lock().unwrap().desired, Some(DesiredMic::Live));
    }
    #[test]
    fn stale_live_ack_cannot_reopen_after_release_or_binding_change() {
        let mut r = Rig::new();
        let a = token(1, 1, 1);
        r.acquire(a, "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.release(a, 1);
        r.step(1);
        assert!(r.driver.slots.is_empty());
        assert_eq!(r.trace.lock().unwrap().starts.len(), 1);
        r.acquire(token(2, 1, 1), "input:mic", 2).unwrap();
        r.step(2);
        r.revision.store(2, Ordering::Release);
        r.live(1);
        r.step(3);
        assert_eq!(r.held(), 0);
        assert!(r.driver.slots.is_empty());
        assert!(r.acquire(token(2, 1, 1), "input:mic", 4).is_err());
        r.acquire(token(2, 2, 2), "input:mic", 4).unwrap();
        r.step(4);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 3);
    }
    #[test]
    fn one_device_heartbeat_cannot_extend_the_other_devices_hold() {
        let mut r = Rig::new();
        r.acquire(token(1, 1, 1), "input:mic", 0).unwrap();
        r.acquire(token(2, 1, 1), "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        r.heartbeat(2, 400);
        r.step(600);
        assert_eq!(r.held(), 1);
        assert!(!r.guard(0).lock().unwrap().closed);
        r.step(900);
        assert_eq!(r.held(), 0);
        assert!(r.driver.slots.is_empty());
    }
    #[test]
    fn explicit_mute_consumes_holds_and_can_mute_a_target_before_any_hold() {
        let mut r = Rig::new();
        let t = token(1, 1, 1);
        r.acquire(t, "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        r.explicit_mute("input:mic");
        r.step(2);
        assert_eq!(r.held(), 0);
        assert!(r.acquire(t, "input:mic", 3).is_err());
        r.explicit_mute("input:other");
        r.step(3);
        assert_eq!(r.guard(1).lock().unwrap().desired, Some(DesiredMic::Muted));
        r.step(4);
        assert!(r.driver.slots.is_empty());
    }
    #[test]
    fn unexpected_watchdog_exit_consumes_holds_instead_of_restarting_live() {
        let mut r = Rig::new();
        let t = token(1, 1, 1);
        r.acquire(t, "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        r.guard(0).lock().unwrap().event = Observation {
            muted: true,
            exited: Some(true),
            ..Default::default()
        };
        r.step(2);
        assert_eq!(r.held(), 0);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 1);
        assert!(r.acquire(t, "input:mic", 3).is_err());
        assert!(
            r.shared
                .state
                .lock()
                .unwrap()
                .failures
                .values()
                .any(|c| *c == "shared_mic_unexpected_exit")
        );
    }
    #[test]
    fn failed_mute_retries_are_bounded_and_only_explicit_mute_restarts_them() {
        let mut r = Rig::new();
        r.trace.lock().unwrap().fail = true;
        r.explicit_mute("input:mic");
        for now in [0, 250, 500, 750, 1000, 1500, 2000] {
            r.step(now);
        }
        assert_eq!(r.trace.lock().unwrap().starts, vec![DesiredMic::Muted; 3]);
        r.acquire(token(1, 1, 1), "input:mic", 2001).unwrap();
        // Reproduce the API's blocked-press consumption; it is not retry consent.
        r.shared
            .state
            .lock()
            .unwrap()
            .holds
            .cancel_target(&TargetId::new("input:mic").unwrap())
            .unwrap();
        r.step(2001);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 3);
        r.explicit_mute("input:mic");
        r.step(2002);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 4);
    }
    #[test]
    fn hung_close_is_terminated_before_the_fallback_mute_guardian_starts() {
        let mut r = Rig::new();
        let t = token(1, 1, 1);
        r.acquire(t, "input:mic", 0).unwrap();
        r.step(0);
        r.guard(0).lock().unwrap().delayed = true;
        r.release(t, 1);
        r.step(1);
        r.step(2501);
        assert!(r.guard(0).lock().unwrap().terminated);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 1);
        r.step(2751);
        assert_eq!(r.guard(1).lock().unwrap().desired, Some(DesiredMic::Muted));
        r.step(2752);
        assert!(r.driver.slots.is_empty());
    }
    #[test]
    fn sampled_tick_older_than_a_concurrent_heartbeat_does_not_reverse_core_time() {
        let mut r = Rig::new();
        r.acquire(token(1, 1, 1), "input:mic", 0).unwrap();
        r.step(0);
        r.heartbeat(1, 101);
        r.step(100);
        assert_eq!(r.held(), 1);
        assert!(!r.shared.state.lock().unwrap().stopped);
    }
    #[test]
    fn eight_targets_bound_processes_and_urgent_stop_closes_every_guard() {
        let mut r = Rig::new();
        for n in 0..8 {
            r.explicit_mute(&format!("input:mic{n}"));
        }
        r.step(0);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 8);
        r.step(1);
        assert!(r.driver.slots.is_empty());
        r.acquire(token(1, 1, 1), "input:mic", 2).unwrap();
        r.step(2);
        r.shared.state.lock().unwrap().holds.stop().unwrap();
        r.step(3);
        assert!(r.driver.slots.is_empty());
        for target in [
            "microphone",
            "output:desk",
            "system",
            "input:-argument",
            "input:x\ny",
        ] {
            assert!(Microphone::named(target).is_err());
        }
    }
    #[test]
    fn blocked_press_is_consumed_until_confirmed_mute_and_requires_a_new_down() {
        let trace = Arc::new(Mutex::new(Trace {
            fail: true,
            ..Default::default()
        }));
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(FakeFactory(trace.clone())),
        )
        .unwrap();
        let mic = Microphone::named("input:fixture").unwrap();
        service.acquire(token(1, 1, 1), mic.clone()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if service
                .shared
                .state
                .lock()
                .unwrap()
                .blocked
                .contains(&mic.0)
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        let rejected = token(2, 1, 1);
        assert_eq!(
            service.acquire(rejected, mic.clone()),
            Err("shared_mic_recovery_pending")
        );
        trace.lock().unwrap().fail = false;
        service.mute(mic.clone()).unwrap();
        loop {
            if !service
                .shared
                .state
                .lock()
                .unwrap()
                .blocked
                .contains(&mic.0)
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(
            service.acquire(rejected, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        service.acquire(token(2, 2, 1), mic).unwrap();
        assert!(!service.take_failures().is_empty());
        assert_eq!(service.fatal_error(), None);
        service.join();
    }
    #[test]
    fn failure_of_one_target_keeps_the_other_held_microphone_guardian_live() {
        let mut r = Rig::new();
        r.acquire(token(1, 1, 1), "input:good", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        r.trace.lock().unwrap().fail = true;
        r.acquire(token(2, 1, 1), "input:bad", 2).unwrap();
        r.step(2);
        r.step(252);
        assert_eq!(r.held(), 1);
        assert!(!r.guard(0).lock().unwrap().closed);
        assert_eq!(r.guard(0).lock().unwrap().target, "input:good");
        assert!(
            r.shared
                .state
                .lock()
                .unwrap()
                .blocked
                .contains(&TargetId::new("input:bad").unwrap())
        );
        assert!(r.guard(0).lock().unwrap().pulses > 0);
    }
    #[test]
    fn failure_consumer_backpressure_stops_instead_of_growing_diagnostics() {
        let mut r = Rig::new();
        let mut now = 0;
        for n in 0..9 {
            r.trace.lock().unwrap().fail = true;
            r.explicit_mute(&format!("input:mic{n}"));
            r.step(now);
            if n < 8 {
                r.trace.lock().unwrap().fail = false;
                now += 250;
                r.step(now);
                now += 1;
                r.step(now);
            }
        }
        let state = r.shared.state.lock().unwrap();
        assert!(state.stopped);
        assert_eq!(state.failures.len(), MAX_TARGETS);
        assert_eq!(state.fatal, Some("shared_mic_failure_budget"));
    }
    fn harmless_child(script: &str) -> NativeGuardian {
        use std::{
            os::unix::process::CommandExt,
            process::{Command, Stdio},
        };
        let child = Command::new("/usr/bin/python3")
            .args(["-c", script])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .unwrap();
        NativeGuardian::from_child(child).unwrap()
    }
    #[test]
    fn native_nonblocking_pipes_acknowledge_live_then_eof_mute_and_reap_the_child() {
        let mut guard = harmless_child(
            "import sys\nline=sys.stdin.buffer.readline()\nif line:\n print('live',flush=True)\n for line in sys.stdin.buffer:pass\nprint('muted',flush=True)\n",
        );
        guard.pulse().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let o = guard.poll().unwrap();
            if o.live {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        guard.close();
        let mut muted = false;
        loop {
            let o = guard.poll().unwrap();
            muted |= o.muted;
            if let Some(success) = o.exited {
                assert!(success);
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(muted);
        assert!(guard.terminate());
    }
    #[test]
    fn malformed_native_pipe_response_is_rejected_and_owned_child_is_terminated() {
        let mut guard =
            harmless_child("import sys,time\nprint('unexpected',flush=True)\ntime.sleep(10)\n");
        let pid = guard.child.id();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if guard.poll().is_err() {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(guard.terminate());
        assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    }
    #[test]
    fn real_service_thread_routes_overlap_and_shutdown_without_native_audio() {
        let trace = Arc::new(Mutex::new(Trace::default()));
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(FakeFactory(trace.clone())),
        )
        .unwrap();
        let a = token(1, 1, 1);
        let b = token(2, 1, 1);
        let mic = Microphone::named("input:fixture").unwrap();
        service.acquire(a, mic.clone()).unwrap();
        service.acquire(b, mic).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if !trace.lock().unwrap().guards.is_empty() {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        service.heartbeat(id(1)).unwrap();
        service.heartbeat(id(2)).unwrap();
        service.release(a).unwrap();
        service.invalidate().unwrap();
        service.release(b).unwrap();
        service.join();
        assert_eq!(trace.lock().unwrap().starts, vec![DesiredMic::Live]);
        assert!(trace.lock().unwrap().guards[0].lock().unwrap().terminated);
    }
    fn until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !condition() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    #[test]
    fn ordinary_mute_cancels_both_holds_and_only_fresh_press_reopens() {
        let trace = Arc::new(Mutex::new(Trace::default()));
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(FakeFactory(trace.clone())),
        )
        .unwrap();
        let mic = Microphone::named("input:fixture").unwrap();
        let a = token(1, 1, 1);
        let b = token(2, 1, 1);
        service.acquire(a, mic.clone()).unwrap();
        service.acquire(b, mic.clone()).unwrap();
        until(|| !trace.lock().unwrap().guards.is_empty());
        service.toggle(a.scope, mic.clone()).unwrap();
        assert_eq!(service.held(), 0);
        until(|| trace.lock().unwrap().guards[0].lock().unwrap().terminated);
        assert_eq!(
            service.acquire(a, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        assert_eq!(
            service.acquire(b, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        service.acquire(token(1, 2, 1), mic).unwrap();
        until(|| trace.lock().unwrap().guards.len() == 2);
        assert!(!trace.lock().unwrap().guards[1].lock().unwrap().manual);
        service.join();
    }
    #[test]
    fn manual_transactions_reserve_the_same_eight_target_budget_as_holds() {
        let trace = Arc::new(Mutex::new(Trace::default()));
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(FakeFactory(trace)),
        )
        .unwrap();
        for index in 0..MAX_TARGETS {
            service
                .toggle(
                    token(1, 1, 1).scope,
                    Microphone::named(&format!("input:fixture{index}")).unwrap(),
                )
                .unwrap();
        }
        let held = token(2, 1, 1);
        let mic = Microphone::named("input:ninth").unwrap();
        assert_eq!(service.acquire(held, mic.clone()), Err("hold_target_limit"));
        assert_eq!(service.held(), 0);
        service.shared.state.lock().unwrap().manual.clear();
        assert_eq!(service.acquire(held, mic), Err("hold_requires_fresh_press"));
        service.join();
        for target in [
            "input:@DEFAULT_SOURCE@",
            "input:31",
            "microphone",
            "output:fixture",
        ] {
            assert!(Microphone::named(target).is_err());
        }
    }
    #[test]
    fn ordinary_unmute_requires_current_scope_commit_and_does_not_create_hold() {
        let mut r = Rig::new();
        let target = TargetId::new("input:fixture").unwrap();
        let request = ManualRequest {
            scope: token(1, 1, 1).scope,
            sequence: 1,
        };
        r.shared
            .state
            .lock()
            .unwrap()
            .manual
            .insert(target.clone(), request);
        r.step(0);
        r.guard(0).lock().unwrap().event.live = true;
        r.step(1);
        r.step(2);
        assert!(r.driver.manual.is_empty());
        assert!(r.shared.state.lock().unwrap().manual.is_empty());
        assert_eq!(r.held(), 0);
        r.revision.store(2, Ordering::Release);
        r.step(3);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 1); // No page-change mute for explicit Live.

        let request = ManualRequest {
            scope: token(1, 2, 2).scope,
            sequence: 2,
        };
        r.shared
            .state
            .lock()
            .unwrap()
            .manual
            .insert(target, request);
        r.step(4);
        r.guard(1).lock().unwrap().event.live = true;
        r.revision.store(3, Ordering::Release);
        r.step(5);
        assert!(r.guard(1).lock().unwrap().closed);
        assert!(
            !r.driver
                .manual
                .contains_key(&TargetId::new("input:fixture").unwrap())
        );
    }
    #[test]
    fn fresh_hold_waits_for_a_cancelled_manual_writer_to_finish() {
        let mut r = Rig::new();
        let target = TargetId::new("input:fixture").unwrap();
        r.shared.state.lock().unwrap().manual.insert(
            target.clone(),
            ManualRequest {
                scope: token(1, 1, 1).scope,
                sequence: 1,
            },
        );
        r.step(0);
        r.guard(0).lock().unwrap().delayed = true;
        r.shared.state.lock().unwrap().manual.clear();
        r.acquire(token(2, 1, 1), "input:fixture", 1).unwrap();
        r.step(1);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 1);
        r.guard(0).lock().unwrap().event = Observation {
            muted: true,
            exited: Some(true),
            ..Default::default()
        };
        r.step(2);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 2);
        assert_eq!(r.guard(1).lock().unwrap().desired, Some(DesiredMic::Live));
    }
    #[test]
    #[ignore = "requires an explicitly isolated PipeWire microphone fixture"]
    fn private_native_microphone_pair_acceptance() {
        let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
        assert!(runtime.starts_with("/tmp/ds-mic-"));
        assert_eq!(
            std::env::var("DECKSMITH_PRIVATE_MIC_TEST").as_deref(),
            Ok("1")
        );
        assert_eq!(
            std::env::var("PULSE_SERVER").unwrap(),
            format!("unix:{runtime}/pulse/native")
        );
        let binding = Arc::new(AtomicU64::new(1));
        let read = binding.clone();
        let factory = NativeFactory::new().unwrap();
        let pin_directory = factory.root.clone();
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(move |s| s.device.binding == read.load(Ordering::Acquire)),
            Box::new(factory),
        )
        .unwrap();
        let mic = Microphone::named("input:fixture-mic").unwrap();
        let muted = || {
            let output = std::process::Command::new("/usr/bin/pactl")
                .args(["get-source-mute", "fixture-mic"])
                .env("LC_ALL", "C")
                .output()
                .unwrap();
            assert!(output.status.success());
            match output.stdout.as_slice() {
                b"Mute: yes\n" => true,
                b"Mute: no\n" => false,
                _ => panic!("invalid private mute state"),
            }
        };
        let settle = |expected| {
            until(|| {
                service.heartbeat(id(1)).unwrap();
                service.heartbeat(id(2)).unwrap();
                muted() == expected
            })
        };
        service.mute(mic.clone()).unwrap();
        settle(true);
        until(|| {
            service
                .shared
                .state
                .lock()
                .unwrap()
                .holds
                .states()
                .is_empty()
        });
        let a = token(1, 1, 1);
        let b = token(2, 1, 1);
        service.acquire(a, mic.clone()).unwrap();
        service.acquire(b, mic.clone()).unwrap();
        settle(false);
        service.release(a).unwrap();
        assert!(!muted());
        service.release(b).unwrap();
        settle(true);
        until(|| {
            service
                .shared
                .state
                .lock()
                .unwrap()
                .holds
                .states()
                .is_empty()
        });
        let a = token(1, 2, 1);
        let b = token(2, 2, 1);
        service.acquire(a, mic.clone()).unwrap();
        service.acquire(b, mic.clone()).unwrap();
        settle(false);
        service.toggle(a.scope, mic.clone()).unwrap();
        assert_eq!(service.held(), 0);
        settle(true);
        assert_eq!(
            service.acquire(a, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        assert_eq!(
            service.acquire(b, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        until(|| {
            service
                .shared
                .state
                .lock()
                .unwrap()
                .holds
                .states()
                .is_empty()
        });
        service.toggle(a.scope, mic.clone()).unwrap();
        settle(false);
        until(|| service.shared.state.lock().unwrap().manual.is_empty());
        binding.store(2, Ordering::Release);
        service.invalidate().unwrap();
        assert!(!muted());
        service.toggle(token(1, 3, 2).scope, mic.clone()).unwrap();
        settle(true);
        until(|| service.shared.state.lock().unwrap().manual.is_empty());
        service.acquire(token(1, 3, 2), mic).unwrap();
        settle(false);
        std::thread::sleep(Duration::from_millis(900));
        assert!(muted());
        assert_eq!(service.held(), 0);
        assert!(service.take_failures().is_empty());
        service.join();
        assert!(muted());
        assert!(!pin_directory.exists());
    }
    #[test]
    fn exhausted_mute_retries_stop_promptly_without_claiming_success() {
        let trace = Arc::new(Mutex::new(Trace {
            fail: true,
            reconnect: true,
            ..Default::default()
        }));
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(FakeFactory(trace.clone())),
        )
        .unwrap();
        service
            .mute(Microphone::named("input:disconnected-fixture").unwrap())
            .unwrap();
        until(|| trace.lock().unwrap().starts.len() >= MAX_ATTEMPTS as usize);
        let observed = service.shared.clone();
        let began = Instant::now();
        service.join();
        assert!(began.elapsed() < Duration::from_secs(1));
        let state = observed.state.lock().unwrap();
        assert!(
            !state.holds.states().is_empty(),
            "unconfirmed mute must not be acknowledged"
        );
        assert!(!state.blocked.is_empty());
        assert!(
            state
                .failures
                .values()
                .any(|code| *code == "shared_mic_spawn_failed")
        );
        assert_eq!(trace.lock().unwrap().starts.len(), MAX_ATTEMPTS as usize);
    }
    #[test]
    fn disconnected_target_uses_slow_mute_only_probes_and_never_replays_old_hold() {
        let mut r = Rig::new();
        r.trace.lock().unwrap().reconnect = true;
        let held = token(1, 1, 1);
        r.acquire(held, "input:mic", 0).unwrap();
        r.step(0);
        r.live(0);
        r.step(1);
        r.guard(0).lock().unwrap().event.exited = Some(false);
        r.trace.lock().unwrap().fail = true;
        r.step(2);
        for now in [252, 502, 752, 1002, 2000, 5000] {
            r.step(now);
        }
        assert_eq!(r.held(), 0);
        assert_eq!(
            r.trace.lock().unwrap().starts,
            vec![
                DesiredMic::Live,
                DesiredMic::Muted,
                DesiredMic::Muted,
                DesiredMic::Muted
            ]
        );
        r.step(6002);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 5);
        r.step(6003);
        r.step(10000);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 5);
        r.trace.lock().unwrap().fail = false;
        r.step(11252);
        r.step(11253);
        assert!(r.driver.slots.is_empty());
        assert!(r.shared.state.lock().unwrap().blocked.is_empty());
        assert!(r.acquire(held, "input:mic", 11254).is_err());
        r.acquire(token(1, 2, 1), "input:mic", 11255).unwrap();
        r.step(11255);
        assert_eq!(r.guard(2).lock().unwrap().desired, Some(DesiredMic::Live));
    }
    #[test]
    fn failed_manual_toggle_schedules_mute_instead_of_replaying_toggle() {
        let mut r = Rig::new();
        let target = TargetId::new("input:mic").unwrap();
        r.shared.state.lock().unwrap().manual.insert(
            target.clone(),
            ManualRequest {
                scope: token(1, 1, 1).scope,
                sequence: 1,
            },
        );
        r.step(0);
        r.guard(0).lock().unwrap().event.exited = Some(false);
        r.step(1);
        assert!(r.guard(0).lock().unwrap().terminated);
        assert!(r.driver.manual.is_empty());
        assert!(r.shared.state.lock().unwrap().manual.is_empty());
        assert_eq!(r.guard(1).lock().unwrap().desired, Some(DesiredMic::Muted));
        r.step(2);
        assert!(r.driver.slots.is_empty());
        r.step(10000);
        assert_eq!(r.trace.lock().unwrap().guards.len(), 2);
    }
    #[test]
    fn no_reconnect_probe_replaces_an_unretired_writer_or_runs_while_stopping() {
        let mut r = Rig::new();
        r.trace.lock().unwrap().reconnect = true;
        let t = token(1, 1, 1);
        r.acquire(t, "input:mic", 0).unwrap();
        r.step(0);
        r.guard(0).lock().unwrap().delayed = true;
        r.release(t, 1);
        r.step(1);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 1);
        r.step(2501);
        assert!(r.guard(0).lock().unwrap().terminated);
        r.trace.lock().unwrap().fail = true;
        for now in [2751, 3001, 3251] {
            r.step(now);
        }
        r.shared.state.lock().unwrap().stopped = true;
        r.step(20000);
        assert_eq!(r.trace.lock().unwrap().starts.len(), 4);
    }
    #[test]
    #[ignore = "requires an explicitly isolated PipeWire reconnection fixture"]
    fn private_native_microphone_reconnection_acceptance() {
        let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();
        assert!(runtime.starts_with("/tmp/ds-mic-"));
        assert_eq!(
            std::env::var("DECKSMITH_PRIVATE_MIC_TEST").as_deref(),
            Ok("1")
        );
        assert_eq!(
            std::env::var("PULSE_SERVER").unwrap(),
            format!("unix:{runtime}/pulse/native")
        );
        let factory = NativeFactory::new().unwrap();
        let pins = factory.root.clone();
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(factory),
        )
        .unwrap();
        let mic = Microphone::named("input:fixture-mic").unwrap();
        let control = |action| {
            assert!(
                std::process::Command::new("python3")
                    .arg("-B")
                    .arg(crate::runtime_paths::root().join("tests/private_microphone_recovery.py"))
                    .args(["control", action])
                    .status()
                    .unwrap()
                    .success()
            );
        };
        let muted = || {
            let output = std::process::Command::new("pactl")
                .args(["get-source-mute", "fixture-mic"])
                .env("LC_ALL", "C")
                .output()
                .unwrap();
            assert!(output.status.success());
            match output.stdout.as_slice() {
                b"Mute: yes\n" => true,
                b"Mute: no\n" => false,
                _ => panic!("unexpected mute state"),
            }
        };
        let wait = |condition: &dyn Fn() -> bool| {
            let end = Instant::now() + Duration::from_secs(12);
            while !condition() {
                assert!(Instant::now() < end, "private reconnection deadline");
                service.heartbeat(id(1)).unwrap();
                service.heartbeat(id(2)).unwrap();
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        let a = token(1, 1, 1);
        let b = token(2, 1, 1);
        service.acquire(a, mic.clone()).unwrap();
        service.acquire(b, mic.clone()).unwrap();
        wait(&|| !muted());
        control("remove");
        wait(&|| service.held() == 0 && !service.shared.state.lock().unwrap().blocked.is_empty());
        std::thread::sleep(Duration::from_secs(6));
        control("different");
        std::thread::sleep(Duration::from_secs(6));
        assert!(!service.quiet());
        assert_eq!(service.held(), 0);
        control("same");
        wait(&|| muted() && service.quiet());
        assert_eq!(
            service.acquire(a, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        assert_eq!(
            service.acquire(b, mic.clone()),
            Err("hold_requires_fresh_press")
        );
        let fresh = token(2, 2, 1);
        service.acquire(fresh, mic.clone()).unwrap();
        wait(&|| !muted());
        service.release(fresh).unwrap();
        wait(&|| muted() && service.quiet());
        service.toggle(a.scope, mic.clone()).unwrap();
        wait(&|| !muted() && service.shared.state.lock().unwrap().manual.is_empty());
        control("same");
        service.toggle(a.scope, mic.clone()).unwrap();
        wait(&|| muted() && service.quiet());
        assert_eq!(service.held(), 0);
        let began = Instant::now();
        service.join();
        assert!(began.elapsed() < Duration::from_secs(1));
        assert!(muted());
        assert!(!pins.exists());
        eprintln!("native recovered-mic join: {:?}", began.elapsed());
        let failed = Service::start(
            id(99),
            &[id(1), id(2)],
            Arc::new(|_| true),
            Box::new(NativeFactory::new().unwrap()),
        )
        .unwrap();
        failed
            .mute(Microphone::named("input:missing-fixture").unwrap())
            .unwrap();
        std::thread::sleep(Duration::from_millis(1500));
        let observed = failed.shared.clone();
        let began = Instant::now();
        failed.join();
        assert!(began.elapsed() < Duration::from_secs(1));
        assert!(!observed.state.lock().unwrap().holds.states().is_empty());
        assert!(!observed.state.lock().unwrap().blocked.is_empty());
        eprintln!(
            "native unavailable-mic join: {:?}; mute remains unconfirmed",
            began.elapsed()
        );
    }
}
