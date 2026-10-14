//! Dormant multi-device execution bridge. One bounded service, never one copy
//! per Deck. Native session wiring and microphone guardians are separate gates.
use crate::pages::Action;
use decksmith_core::{
    device_context::DeviceId,
    shared_dispatch::{
        ActionScope, Completion, Dispatcher, Effect, Job, MAX_INFLIGHT, RequestId, TargetId,
    },
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
pub type Authority = Arc<dyn Fn(&ActionScope) -> bool + Send + Sync>;
const MAX_REPLIES: usize = 64;

/// Identity and payload travel together; a target label alone cannot compile a
/// command. Aliases/default targets must be resolved and fenced by the host.
#[derive(Clone, PartialEq, Eq)]
pub struct Compiled {
    target: TargetId,
    action: Action,
    effect: Effect,
    media: Option<crate::shared_native_media::Session>,
}
impl std::fmt::Debug for Compiled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Compiled([private])")
    }
}
impl Compiled {
    /// Initial native slice: named inputs/outputs and explicitly selected MPRIS
    /// players. App aliases, system-sounds, plugins and target selection require
    /// a shared canonical resolver before entering this service.
    pub fn audio(action: Action) -> Result<Self> {
        let (target, effect) = match &action {
            Action::AudioAdjust { target, percent }
                if *percent != 0 && (-20..=20).contains(percent) =>
            {
                (target, Effect::Adjust(*percent))
            }
            Action::AudioMute { target } => (target, Effect::Toggle),
            _ => return Err("unsupported_shared_audio_action"),
        };
        if !crate::audio_target::valid(target)
            || !target.starts_with("input:") && !target.starts_with("output:")
        {
            return Err("shared_audio_requires_named_target");
        }
        let name = target.split_once(':').unwrap().1;
        if name.starts_with('@') || name.bytes().all(|b| b.is_ascii_digit()) {
            return Err("shared_audio_requires_named_target");
        }
        let key = TargetId::new(&format!("audio:{target}"))?;
        Ok(Self {
            target: key,
            action,
            effect,
            media: None,
        })
    }
    pub fn media(player: &str, command: &str) -> Result<Self> {
        if !crate::launch::valid_player(player)
            || !matches!(command, "play_pause" | "next" | "previous")
        {
            return Err("invalid_shared_media_action");
        }
        Ok(Self {
            target: TargetId::new(&format!("media:{player}"))?,
            action: Action::MediaTarget {
                player: player.into(),
                command: command.into(),
            },
            effect: if command == "play_pause" {
                Effect::Toggle
            } else {
                Effect::Activate
            },
            media: None,
        })
    }
    /// Saved app and event-sound selectors use the single guarded native helper.
    /// Default microphone mute remains reserved for the shared hold guardian.
    pub fn saved_audio(action: Action) -> Result<Self> {
        let (target, effect) = match &action {
            Action::AudioAdjust { target, percent }
                if *percent != 0 && (-20..=20).contains(percent) =>
            {
                (target, Effect::Adjust(*percent))
            }
            Action::AudioMute { target } => (target, Effect::Toggle),
            _ => return Err("unsupported_shared_audio_action"),
        };
        if target != "system" && target != "default_output" && !target.starts_with("app:") {
            return Self::audio(action);
        }
        if !crate::audio_target::valid(target) {
            return Err("invalid_shared_native_target");
        }
        Ok(Self {
            target: TargetId::new(&format!("audio:{target}"))?,
            action,
            effect,
            media: None,
        })
    }
    pub fn media_bound(
        session: crate::shared_native_media::Session,
        command: &str,
    ) -> Result<Self> {
        session.validate()?;
        let mut compiled = Self::media(&session.name, command)?;
        compiled.action = Action::MediaTarget {
            player: session.selector.clone(),
            command: command.into(),
        };
        compiled.target = TargetId::new(&format!("media:{}", session.owner))?;
        compiled.media = Some(session);
        Ok(compiled)
    }
    fn action(&self, job: &Job) -> Result<Action> {
        if self.target != job.target {
            return Err("shared_payload_target_mismatch");
        }
        match (&self.action, job.effect) {
            (Action::AudioAdjust { target, .. }, Effect::Adjust(percent)) => {
                Ok(Action::AudioAdjust {
                    target: target.clone(),
                    percent,
                })
            }
            (_, effect) if effect == self.effect => Ok(self.action.clone()),
            _ => Err("shared_payload_effect_mismatch"),
        }
    }
}

/// Passed into the backend, which must recheck at every actual write boundary,
/// including after discovery/waits. A check does not revoke an issued write.
#[derive(Clone)]
pub struct Fence {
    scope: ActionScope,
    authority: Authority,
    shared: Arc<Shared>,
    media: Option<crate::shared_native_media::Session>,
}
impl Fence {
    pub fn media_session(&self) -> Option<&crate::shared_native_media::Session> {
        self.media.as_ref()
    }
    pub fn accepts(&self) -> bool {
        !self.shared.state.lock().unwrap().stopped && (self.authority)(&self.scope)
    }
}
/// Implementations must bound/terminate their owned I/O before returning, and
/// check the Fence immediately before each external write. Errors never retry.
/// Coalesced adjustments can reach 100%; adapters must preserve same-direction
/// clamping and recheck the Fence between bounded writes if splitting the step.
pub trait Backend: Send + 'static {
    fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()>;
}
pub struct Reply {
    pub completion: Completion,
    pub receipt_to_dispatch_us: Option<u64>,
    pub submissions: u16,
}
struct State {
    dispatch: Dispatcher,
    payloads: BTreeMap<RequestId, Compiled>,
    replies: VecDeque<Reply>,
    stopped: bool,
    workers: usize,
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}
pub struct Service {
    shared: Arc<Shared>,
    authority: Authority,
    threads: Vec<JoinHandle<()>>,
}
impl Service {
    /// Construct backends before starting; no partially started pool on invalid
    /// owner/budget input. The shared service owns at most four executor threads.
    pub fn start(
        runtime: DeviceId,
        devices: &[DeviceId],
        authority: Authority,
        backends: Vec<Box<dyn Backend>>,
    ) -> Result<Self> {
        if backends.is_empty() || backends.len() > MAX_INFLIGHT {
            return Err("invalid_shared_worker_count");
        }
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dispatch: Dispatcher::new(runtime, devices)?,
                payloads: BTreeMap::new(),
                replies: VecDeque::new(),
                stopped: false,
                workers: backends.len(),
            }),
            wake: Condvar::new(),
        });
        let mut threads = Vec::new();
        for backend in backends {
            let owner = shared.clone();
            let current = authority.clone();
            // Fallible spawn: stop and join the already started subset on failure.
            match std::thread::Builder::new()
                .name("decksmith-shared-action".into())
                .spawn(move || run(owner, current, backend))
            {
                Ok(thread) => threads.push(thread),
                Err(_) => {
                    let mut state = shared.state.lock().unwrap();
                    state.stopped = true;
                    state.dispatch.stop();
                    drop(state);
                    shared.wake.notify_all();
                    for thread in threads {
                        let _ = thread.join();
                    }
                    return Err("shared_worker_spawn_failed");
                }
            }
        }
        Ok(Self {
            shared,
            authority,
            threads,
        })
    }
    pub fn submit(
        &self,
        scope: ActionScope,
        action: Compiled,
        received_at: Option<Instant>,
    ) -> Result<RequestId> {
        let mut state = self.shared.state.lock().unwrap();
        if state.stopped {
            return Err("shared_worker_stopped");
        }
        let id = state.dispatch.submit(
            scope,
            action.target.clone(),
            action.effect,
            received_at,
            &*self.authority,
        )?;
        // Coalescing retains the original immutable payload; only its scalar
        // changes in the scheduler. The compilation limits guarantee same meaning.
        state.payloads.entry(id).or_insert(action);
        prune(&mut state, &self.authority);
        drop(state);
        self.shared.wake.notify_all();
        Ok(id)
    }
    /// Call after a page/binding/disconnect/lock/foreground fence changes. The
    /// backend checks authority independently; stale payloads are removed here.
    pub fn invalidate(&self) {
        let mut state = self.shared.state.lock().unwrap();
        prune(&mut state, &self.authority);
        drop(state);
        self.shared.wake.notify_all();
    }
    /// Final current-scope check at consumer receipt, not only worker completion.
    pub fn poll(&self) -> Result<Option<Reply>> {
        let mut state = self.shared.state.lock().unwrap();
        while let Some(reply) = state.replies.pop_front() {
            if !state.stopped && (self.authority)(&reply.completion.scope) {
                return Ok(Some(reply));
            }
        }
        if state.stopped {
            Err("shared_worker_stopped")
        } else {
            Ok(None)
        }
    }
    /// Nonblocking cancellation; join separately after the HID owner stops.
    pub fn stop(&self) {
        let mut state = self.shared.state.lock().unwrap();
        state.stopped = true;
        state.dispatch.stop();
        state.payloads.clear();
        state.replies.clear();
        drop(state);
        self.shared.wake.notify_all();
    }
    pub fn quiesce(&mut self) {
        self.stop();
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
    pub fn join(mut self) {
        self.quiesce();
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.stop();
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}
fn prune(state: &mut State, current: &Authority) {
    state.dispatch.cancel_stale(&**current);
    state
        .replies
        .retain(|reply| current(&reply.completion.scope));
    // Payloads carry scopes below, in the scheduler's bounded request inventory.
    let ids = state.dispatch.request_ids();
    state.payloads.retain(|id, _| ids.contains(id));
}
fn run(shared: Arc<Shared>, authority: Authority, mut backend: Box<dyn Backend>) {
    loop {
        let (job, action, media) = {
            let mut state = shared.state.lock().unwrap();
            loop {
                if state.stopped {
                    return;
                }
                prune(&mut state, &authority);
                if let Some(selected) = state.dispatch.select(&*authority) {
                    let Ok(job) = state.dispatch.begin(selected.id, &*authority) else {
                        continue;
                    };
                    let compiled = state
                        .payloads
                        .remove(&job.id)
                        .ok_or("missing_shared_payload");
                    let media = compiled.as_ref().ok().and_then(|p| p.media.clone());
                    let action = compiled.and_then(|p| p.action(&job));
                    break (job, action, media);
                }
                state = shared.wake.wait(state).unwrap();
            }
        };
        let fence = Fence {
            scope: job.scope,
            authority: authority.clone(),
            shared: shared.clone(),
            media,
        };
        let receipt_to_dispatch_us = crate::audio_worker::elapsed_us(job.received_at);
        // Backend panic cannot poison the service mutex or free an active target
        // before unwinding its owned writer. Do not retry an uncertain effect.
        let (result, panicked) = match action {
            Ok(action) if fence.accepts() => {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    backend.execute(&action, &fence)
                })) {
                    Ok(result) => (result, false),
                    Err(_) => (Err("shared_backend_panicked"), true),
                }
            }
            Ok(_) => (Err("stale_action_scope"), false),
            Err(code) => (Err(code), false),
        };
        // Destroy a failed backend before releasing its reservation. A new worker
        // is not started automatically. Remaining workers can serve other targets.
        if panicked {
            let destroyed =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(backend))).is_ok();
            let mut state = shared.state.lock().unwrap();
            state.workers -= 1;
            if state.workers == 0 || !destroyed {
                state.stopped = true;
                state.dispatch.stop();
                state.payloads.clear();
            }
            drop(state);
            if destroyed {
                complete(&shared, &authority, &job, result, receipt_to_dispatch_us);
            }
            // A panicking destructor cannot prove its writer ended. Leave the
            // reservation held and stop the service; never start a replacement.
            shared.wake.notify_all();
            return;
        }
        complete(&shared, &authority, &job, result, receipt_to_dispatch_us);
    }
}
fn complete(
    shared: &Shared,
    authority: &Authority,
    job: &Job,
    result: Result<()>,
    receipt_to_dispatch_us: Option<u64>,
) {
    let mut state = shared.state.lock().unwrap();
    if let Ok(Some(completion)) = state.dispatch.finish(job.id, result, &**authority) {
        if state.replies.len() == MAX_REPLIES {
            // Stop rather than dropping failure feedback or allocating unboundedly.
            state.stopped = true;
            state.dispatch.stop();
            state.payloads.clear();
        } else {
            state.replies.push_back(Reply {
                completion,
                receipt_to_dispatch_us,
                submissions: job.submissions,
            });
        }
    }
    drop(state);
    shared.wake.notify_all();
}

/// Initial fixed native adapter, constructed lazily and never started by the
/// current daemon. Named volume and output mute only. Input mute remains gated
/// on the shared microphone guardian; media awaits a guarded MPRIS adapter.
#[derive(Default)]
pub struct NamedAudioBackend;
impl Backend for NamedAudioBackend {
    fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
        execute_named(action, || fence.accepts(), native_command)
    }
}
fn execute_named(
    action: &Action,
    current: impl Fn() -> bool,
    mut command: impl FnMut(&[&str], bool, &dyn Fn() -> bool) -> Result<Vec<u8>>,
) -> Result<()> {
    let target = match action {
        Action::AudioAdjust { target, percent }
            if *percent != 0 && (-100..=100).contains(percent) =>
        {
            target
        }
        Action::AudioMute { target } if target.starts_with("output:") => target,
        _ => return Err("unsupported_shared_native_action"),
    };
    if !crate::audio_target::valid(target) {
        return Err("invalid_shared_native_target");
    }
    let (kind, name) = if let Some(name) = target.strip_prefix("input:") {
        ("source", name)
    } else if let Some(name) = target.strip_prefix("output:") {
        ("sink", name)
    } else {
        return Err("shared_audio_requires_named_target");
    };
    if !current() {
        return Err("stale_action_scope");
    }
    let (operation, value) = match action {
        Action::AudioAdjust { percent, .. } => {
            let read = format!("get-{kind}-volume");
            let bytes = command(&[&read, name], false, &current)?;
            let level = parse_percent(&bytes)?;
            (
                format!("set-{kind}-volume"),
                format!(
                    "{}%",
                    (i32::from(level) + i32::from(*percent)).clamp(0, 100)
                ),
            )
        }
        Action::AudioMute { .. } => {
            let read = format!("get-{kind}-mute");
            let bytes = command(&[&read, name], false, &current)?;
            let muted = match bytes.as_slice() {
                b"Mute: yes\n" => true,
                b"Mute: no\n" => false,
                _ => return Err("invalid_shared_audio_state"),
            };
            (
                format!("set-{kind}-mute"),
                if muted { "0" } else { "1" }.into(),
            )
        }
        _ => unreachable!(),
    };
    // Discovery/read may wait: the native transport rechecks right before the
    // actual write spawn, and issues exactly one clamped write per adjustment.
    command(&[&operation, name, &value], true, &current)?;
    Ok(())
}
fn parse_percent(bytes: &[u8]) -> Result<u16> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid_shared_audio_state")?;
    if !text.starts_with("Volume:") {
        return Err("invalid_shared_audio_state");
    }
    let values = text
        .split_whitespace()
        .filter_map(|s| s.strip_suffix('%'))
        .map(|s| s.parse::<u16>().ok().filter(|n| *n <= 1000))
        .collect::<Option<Vec<_>>>()
        .ok_or("invalid_shared_audio_state")?;
    values.into_iter().max().ok_or("invalid_shared_audio_state")
}
fn native_command(args: &[&str], write: bool, current: &dyn Fn() -> bool) -> Result<Vec<u8>> {
    use std::{
        io::Read,
        os::unix::process::CommandExt,
        process::{Command, Stdio},
    };
    if !current() {
        return Err("stale_action_scope");
    }
    let mut child = Command::new("/usr/bin/pactl")
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(if write { Stdio::null() } else { Stdio::piped() })
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|_| "shared_audio_spawn_failed")?;
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err("shared_audio_command_failed");
                }
                let mut bytes = Vec::new();
                if let Some(output) = child.stdout.take() {
                    output
                        .take(4097)
                        .read_to_end(&mut bytes)
                        .map_err(|_| "shared_audio_read_failed")?;
                }
                if bytes.len() > 4096 {
                    return Err("invalid_shared_audio_state");
                }
                return Ok(bytes);
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            _ => {
                // The owned unreaped child owns this group. No command from
                // settings can select another program/group or supply shell text.
                unsafe {
                    libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
                }
                let _ = child.kill();
                let _ = child.wait();
                return Err("shared_audio_timeout");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_core::{device_context::DeviceContext, device_model::ControlAddress};
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    };
    fn id(n: u8) -> DeviceId {
        DeviceId::from_bytes([n; 16]).unwrap()
    }
    fn scope(n: u8, binding: u64) -> ActionScope {
        ActionScope {
            runtime: id(99),
            foreground: 0,
            device: DeviceContext {
                supervisor: 1,
                device: id(n),
                connection: 1,
                binding,
                lock_epoch: 0,
                control: ControlAddress::Key(0),
            },
        }
    }
    fn adjust(name: &str, percent: i16) -> Compiled {
        Compiled::audio(Action::AudioAdjust {
            target: format!("output:{name}"),
            percent,
        })
        .unwrap()
    }
    fn mute(name: &str) -> Compiled {
        Compiled::audio(Action::AudioMute {
            target: format!("output:{name}"),
        })
        .unwrap()
    }
    fn authority() -> (Arc<AtomicU64>, Authority) {
        let revision = Arc::new(AtomicU64::new(1));
        let read = revision.clone();
        (
            revision,
            Arc::new(move |s| s.device.binding == read.load(Ordering::Acquire)),
        )
    }
    struct Controlled {
        entered: SyncSender<Action>,
        release: Receiver<()>,
        written: SyncSender<Action>,
    }
    impl Backend for Controlled {
        fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
            self.entered.send(action.clone()).unwrap();
            self.release.recv_timeout(Duration::from_secs(3)).unwrap();
            if !fence.accepts() {
                return Err("stale_action_scope");
            }
            self.written.send(action.clone()).unwrap();
            Ok(())
        }
    }
    struct Probe {
        entered: Receiver<Action>,
        release: SyncSender<()>,
        written: Receiver<Action>,
    }
    fn controlled() -> (Box<dyn Backend>, Probe) {
        let (entered, observed) = mpsc::sync_channel(16);
        let (release, waiting) = mpsc::sync_channel(16);
        let (written, writes) = mpsc::sync_channel(16);
        (
            Box::new(Controlled {
                entered,
                release: waiting,
                written,
            }),
            Probe {
                entered: observed,
                release,
                written: writes,
            },
        )
    }
    fn reply(service: &Service) -> Reply {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(reply) = service.poll().unwrap() {
                return reply;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
    }
    #[test]
    fn compile_rejects_aliases_plugins_bad_steps_and_default_players() {
        for target in [
            "microphone",
            "default_output",
            "system",
            "app:application.name=Browser",
            "input:-option",
            "input:a\nb",
        ] {
            assert!(
                Compiled::audio(Action::AudioAdjust {
                    target: target.into(),
                    percent: 1
                })
                .is_err()
            );
        }
        for percent in [0, 21, -21, 101] {
            assert!(
                Compiled::audio(Action::AudioAdjust {
                    target: "output:desk".into(),
                    percent
                })
                .is_err()
            );
        }
        assert!(
            Compiled::audio(Action::AudioSelect {
                target: "output:desk".into()
            })
            .is_err()
        );
        assert!(Compiled::media("", "next").is_err());
        assert!(Compiled::media("org.mpris.MediaPlayer2.player", "shell").is_err());
        assert!(Compiled::media("org.mpris.MediaPlayer2.player", "next").is_ok());
        assert!(!format!("{:?}", adjust("private-device", 1)).contains("private-device"));
    }
    #[test]
    fn independent_targets_execute_in_parallel_but_same_target_waits_for_completion() {
        let (_, current) = authority();
        let (a, pa) = controlled();
        let (b, pb) = controlled();
        let service = Service::start(id(99), &[id(1), id(2)], current, vec![a, b]).unwrap();
        service
            .submit(scope(1, 1), adjust("left", 1), None)
            .unwrap();
        // Either pool worker may take the first job; use one aggregate observer
        // per worker and select the matching one, without relying on thread order.
        let deadline = Instant::now() + Duration::from_secs(2);
        let first = loop {
            if let Ok(action) = pa.entered.try_recv() {
                break (&pa, &pb, action);
            }
            if let Ok(action) = pb.entered.try_recv() {
                break (&pb, &pa, action);
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        };
        service
            .submit(scope(2, 1), adjust("right", 2), None)
            .unwrap();
        first
            .1
            .entered
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        service.submit(scope(2, 1), mute("left"), None).unwrap();
        first.1.release.send(()).unwrap();
        first
            .1
            .written
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        let r = reply(&service);
        assert_eq!(r.completion.scope.device.device, id(2));
        // Left is still owned by a blocked writer; the queued toggle cannot pass.
        assert!(
            first
                .1
                .entered
                .recv_timeout(Duration::from_millis(30))
                .is_err()
        );
        first.0.release.send(()).unwrap();
        first
            .0
            .written
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        let r = reply(&service);
        assert_eq!(r.completion.scope.device.device, id(1));
        let deadline = Instant::now() + Duration::from_secs(2);
        let last = loop {
            if let Ok(action) = pa.entered.try_recv() {
                break (&pa, action);
            }
            if let Ok(action) = pb.entered.try_recv() {
                break (&pb, action);
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        };
        assert!(matches!(last.1, Action::AudioMute { .. }));
        last.0.release.send(()).unwrap();
        assert_eq!(reply(&service).completion.result, Ok(()));
        service.join();
    }
    #[test]
    fn stale_scope_cancels_queued_actions_and_is_checked_again_after_backend_wait() {
        let (revision, current) = authority();
        let (backend, p) = controlled();
        let service = Service::start(id(99), &[id(1), id(2)], current, vec![backend]).unwrap();
        service
            .submit(scope(1, 1), adjust("desk", 1), None)
            .unwrap();
        p.entered.recv_timeout(Duration::from_secs(2)).unwrap();
        service.submit(scope(2, 1), mute("other"), None).unwrap();
        revision.store(2, Ordering::Release);
        service.invalidate();
        p.release.send(()).unwrap();
        service
            .submit(scope(2, 2), adjust("fresh", 1), None)
            .unwrap();
        p.entered.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(p.written.try_recv().is_err());
        p.release.send(()).unwrap();
        let r = reply(&service);
        assert_eq!(r.completion.scope, scope(2, 2));
        assert_eq!(r.completion.result, Ok(()));
        service.join();
    }
    #[test]
    fn fast_steps_coalesce_with_payload_and_timing_while_toggle_stays_a_barrier() {
        let (_, current) = authority();
        let (backend, p) = controlled();
        let service = Service::start(id(99), &[id(1)], current, vec![backend]).unwrap();
        service.submit(scope(1, 1), mute("blocker"), None).unwrap();
        p.entered.recv_timeout(Duration::from_secs(2)).unwrap();
        let received = Instant::now();
        let first = service
            .submit(scope(1, 1), adjust("desk", 20), Some(received))
            .unwrap();
        assert_eq!(
            service
                .submit(scope(1, 1), adjust("desk", 20), None)
                .unwrap(),
            first
        );
        service.submit(scope(1, 1), mute("desk"), None).unwrap();
        service
            .submit(scope(1, 1), adjust("desk", -1), None)
            .unwrap();
        p.release.send(()).unwrap();
        reply(&service);
        assert!(matches!(
            p.entered.recv_timeout(Duration::from_secs(2)).unwrap(),
            Action::AudioAdjust { percent: 40, .. }
        ));
        p.release.send(()).unwrap();
        let r = reply(&service);
        assert_eq!(r.submissions, 2);
        assert!(r.receipt_to_dispatch_us.is_some());
        assert!(matches!(
            p.entered.recv_timeout(Duration::from_secs(2)).unwrap(),
            Action::AudioMute { .. }
        ));
        p.release.send(()).unwrap();
        reply(&service);
        assert!(matches!(
            p.entered.recv_timeout(Duration::from_secs(2)).unwrap(),
            Action::AudioAdjust { percent: -1, .. }
        ));
        p.release.send(()).unwrap();
        reply(&service);
        service.join();
    }
    #[test]
    fn queue_full_and_stop_remain_nonblocking_with_an_active_writer() {
        let (_, current) = authority();
        let (backend, p) = controlled();
        let service = Service::start(id(99), &[id(1)], current, vec![backend]).unwrap();
        service.submit(scope(1, 1), mute("blocker"), None).unwrap();
        p.entered.recv_timeout(Duration::from_secs(2)).unwrap();
        for _ in 0..16 {
            service.submit(scope(1, 1), mute("desk"), None).unwrap();
        }
        assert_eq!(
            service.submit(scope(1, 1), mute("desk"), None),
            Err("action_queue_full")
        );
        assert!(service.poll().unwrap().is_none());
        service.stop();
        assert!(matches!(service.poll(), Err("shared_worker_stopped")));
        assert_eq!(
            service.submit(scope(1, 1), mute("desk"), None),
            Err("shared_worker_stopped")
        );
        p.release.send(()).unwrap();
        service.join();
        assert!(p.written.try_recv().is_err());
    }
    #[test]
    fn reply_is_discarded_if_binding_changes_after_completion_before_poll() {
        let (revision, current) = authority();
        let (backend, p) = controlled();
        let service = Service::start(id(99), &[id(1)], current, vec![backend]).unwrap();
        service
            .submit(scope(1, 1), adjust("desk", 1), None)
            .unwrap();
        p.entered.recv_timeout(Duration::from_secs(2)).unwrap();
        p.release.send(()).unwrap();
        p.written.recv_timeout(Duration::from_secs(2)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if !service.shared.state.lock().unwrap().replies.is_empty() {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        revision.store(2, Ordering::Release);
        assert!(service.poll().unwrap().is_none());
        service.join();
    }
    #[test]
    fn backend_panic_ends_its_writer_without_automatic_retry_and_last_worker_stops_service() {
        struct PanicBackend;
        impl Backend for PanicBackend {
            fn execute(&mut self, _: &Action, _: &Fence) -> Result<()> {
                panic!("fixture provider failure")
            }
        }
        let (_, current) = authority();
        let service =
            Service::start(id(99), &[id(1)], current, vec![Box::new(PanicBackend)]).unwrap();
        service.submit(scope(1, 1), mute("desk"), None).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if service.shared.state.lock().unwrap().stopped {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(matches!(service.poll(), Err("shared_worker_stopped")));
        assert_eq!(service.shared.state.lock().unwrap().dispatch.inflight(), 0);
        service.join();
    }
    #[test]
    fn failed_provider_is_source_specific_and_other_worker_remains_available() {
        struct FailsOnce;
        impl Backend for FailsOnce {
            fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
                assert!(fence.accepts());
                if matches!(action, Action::AudioMute { target } if target == "output:broken") {
                    panic!("fixture provider");
                }
                Ok(())
            }
        }
        let (_, current) = authority();
        let service = Service::start(
            id(99),
            &[id(1), id(2)],
            current,
            vec![Box::new(FailsOnce), Box::new(FailsOnce)],
        )
        .unwrap();
        service.submit(scope(1, 1), mute("broken"), None).unwrap();
        let r = reply(&service);
        assert_eq!(r.completion.scope, scope(1, 1));
        assert_eq!(r.completion.result, Err("shared_backend_panicked"));
        service.submit(scope(2, 1), mute("healthy"), None).unwrap();
        let r = reply(&service);
        assert_eq!(r.completion.scope, scope(2, 1));
        assert_eq!(r.completion.result, Ok(()));
        service.join();
    }
    #[test]
    fn stalled_reply_consumer_stops_at_fixed_budget_without_unbounded_growth() {
        struct Fast;
        impl Backend for Fast {
            fn execute(&mut self, _: &Action, fence: &Fence) -> Result<()> {
                if fence.accepts() {
                    Ok(())
                } else {
                    Err("stale_action_scope")
                }
            }
        }
        let (_, current) = authority();
        let service = Service::start(id(99), &[id(1)], current, vec![Box::new(Fast)]).unwrap();
        for count in 1..=MAX_REPLIES + 1 {
            service.submit(scope(1, 1), mute("desk"), None).unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let state = service.shared.state.lock().unwrap();
                let done = state.replies.len() == count.min(MAX_REPLIES)
                    && (count <= MAX_REPLIES || state.stopped);
                drop(state);
                if done {
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
        }
        assert!(matches!(service.poll(), Err("shared_worker_stopped")));
        assert_eq!(service.shared.state.lock().unwrap().dispatch.inflight(), 0);
        service.join();
    }
    #[test]
    fn failed_backend_destructor_keeps_reservation_and_forbids_replacement_writes() {
        struct BrokenDrop;
        impl Backend for BrokenDrop {
            fn execute(&mut self, _: &Action, _: &Fence) -> Result<()> {
                panic!("fixture execution");
            }
        }
        impl Drop for BrokenDrop {
            fn drop(&mut self) {
                panic!("fixture destructor");
            }
        }
        let (_, current) = authority();
        let service =
            Service::start(id(99), &[id(1)], current, vec![Box::new(BrokenDrop)]).unwrap();
        service.submit(scope(1, 1), mute("desk"), None).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if service.shared.state.lock().unwrap().stopped {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(service.shared.state.lock().unwrap().dispatch.inflight(), 1);
        assert_eq!(
            service.submit(scope(1, 1), mute("desk"), None),
            Err("shared_worker_stopped")
        );
        service.join();
    }
    #[test]
    fn native_named_plan_clamps_combined_steps_and_rechecks_after_read_without_audio_access() {
        let calls = std::cell::RefCell::new(Vec::new());
        execute_named(
            &Action::AudioAdjust {
                target: "input:mic".into(),
                percent: 40,
            },
            || true,
            |args, write, guard| {
                assert!(guard());
                calls.borrow_mut().push((
                    args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    write,
                ));
                Ok(if write {
                    Vec::new()
                } else {
                    b"Volume: mono: 60000 / 92% / -0.5 dB\n".to_vec()
                })
            },
        )
        .unwrap();
        assert_eq!(
            calls.borrow()[1],
            (
                vec!["set-source-volume".into(), "mic".into(), "100%".into()],
                true
            )
        );
        let alive = std::cell::Cell::new(true);
        let writes = std::cell::Cell::new(0);
        let r = execute_named(
            &Action::AudioMute {
                target: "output:desk".into(),
            },
            || alive.get(),
            |_, write, guard| {
                if !guard() {
                    return Err("stale_action_scope");
                }
                if write {
                    writes.set(writes.get() + 1);
                } else {
                    alive.set(false);
                }
                Ok(b"Mute: yes\n".to_vec())
            },
        );
        assert_eq!(r, Err("stale_action_scope"));
        assert_eq!(writes.get(), 0);
        assert!(
            execute_named(
                &Action::AudioMute {
                    target: "input:mic".into()
                },
                || true,
                |_, _, _| panic!("input mute must remain gated")
            )
            .is_err()
        );
        for bad in [
            b"garbage 50%".as_slice(),
            b"Volume: mono: NaN%",
            b"Volume: mono: 1001%",
            b"Volume: none",
        ] {
            assert!(parse_percent(bad).is_err());
        }
    }
}
