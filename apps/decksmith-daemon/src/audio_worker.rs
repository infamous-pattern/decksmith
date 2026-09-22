//! Bounded audio execution, isolated from HID ownership.
use crate::{audio, pages::Action};
use decksmith_core::InputEvent;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::{thread::JoinHandle, time::Duration};

#[derive(Clone)]
pub enum Request {
    Action {
        page: u8,
        session: u64,
        action: Action,
        input: InputEvent,
    },
    Read {
        system: Vec<String>,
        session: u64,
        targets: Vec<String>,
    },
}
pub enum Reply {
    Action {
        page: u8,
        session: u64,
        action: Action,
        input: InputEvent,
        result: Result<(), &'static str>,
    },
    State {
        system: crate::system_actions::States,
        session: u64,
        state: Option<audio::State>,
        targets: Vec<(String, Option<audio::State>)>,
    },
}
trait Backend: Send + 'static {
    fn read_system(&mut self, _commands: &[String]) -> crate::system_actions::States {
        Default::default()
    }
    fn execute(&mut self, action: Action) -> Result<(), &'static str>;
    fn read(&mut self) -> Option<audio::State>;
    fn read_snapshot(
        &mut self,
        targets: &[String],
    ) -> (Option<audio::State>, Vec<Option<audio::State>>) {
        let states = self.read_targets(targets);
        (self.read(), states)
    }
    fn read_targets(&mut self, targets: &[String]) -> Vec<Option<audio::State>> {
        vec![None; targets.len()]
    }
}
#[derive(Default)]
struct System {
    system: crate::system_actions::Client,
    audio: crate::audio_target::Client,
}
impl Backend for System {
    fn read_system(&mut self, commands: &[String]) -> crate::system_actions::States {
        self.system.read(commands)
    }
    fn execute(&mut self, action: Action) -> Result<(), &'static str> {
        match &action {
            Action::System { command } => self.system.execute(command),
            Action::AudioAdjust { .. } | Action::AudioMute { .. } | Action::AudioSelect { .. } => {
                crate::audio_target::execute(&action)
            }
            Action::OpenApplication { .. }
            | Action::OpenWebsite { .. }
            | Action::MediaTarget { .. }
            | Action::MediaPlayPause
            | Action::MediaNext
            | Action::MediaPrevious => crate::launch::execute(&action),
            _ => audio::execute(action),
        }
    }
    fn read(&mut self) -> Option<audio::State> {
        self.audio
            .read(&["default_output".into()])
            .into_iter()
            .next()
            .flatten()
            .or_else(|| audio::read().ok())
    }
    fn read_targets(&mut self, targets: &[String]) -> Vec<Option<audio::State>> {
        self.audio.read(targets)
    }
    fn read_snapshot(
        &mut self,
        targets: &[String],
    ) -> (Option<audio::State>, Vec<Option<audio::State>>) {
        let mut all = targets.to_vec();
        all.push("default_output".into());
        let mut states = self.audio.read(&all);
        let state = states.pop().flatten().or_else(|| audio::read().ok());
        (state, states)
    }
}
pub struct Worker {
    plugin: crate::plugin_runtime::Worker,
    requests: SyncSender<(Request, u64, u64)>,
    gate: crate::session_lock::Gate,
    foreground: crate::foreground::Foreground,
    read_pending: Arc<AtomicBool>,
    replies: Receiver<Reply>,
    session: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start_with_context(
        gate: crate::session_lock::Gate,
        foreground: crate::foreground::Foreground,
    ) -> Self {
        Self::with_context(System::default(), gate, foreground)
    }
    #[cfg(test)]
    fn with_backend(backend: impl Backend) -> Self {
        Self::with_gate(backend, Default::default())
    }
    #[cfg(test)]
    fn with_gate(backend: impl Backend, gate: crate::session_lock::Gate) -> Self {
        Self::with_context(backend, gate, Default::default())
    }
    fn with_context(
        mut backend: impl Backend,
        gate: crate::session_lock::Gate,
        foreground: crate::foreground::Foreground,
    ) -> Self {
        let (requests, incoming) = mpsc::sync_channel::<(Request, u64, u64)>(32);
        let (outgoing, replies) = mpsc::sync_channel(64);
        let session = Arc::new(AtomicU64::new(0));
        let plugin =
            crate::plugin_runtime::Worker::start(gate.clone(), foreground.clone(), session.clone());
        let stop = Arc::new(AtomicBool::new(false));
        let read_pending = Arc::new(AtomicBool::new(false));
        let pending = read_pending.clone();
        let active = session.clone();
        let stopped = stop.clone();
        let action_gate = gate.clone();
        let action_focus = foreground.clone();
        let thread = std::thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                let (request, epoch, focus_revision) =
                    match incoming.recv_timeout(Duration::from_millis(20)) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(_) => break,
                    };
                let id = match &request {
                    Request::Action { session, .. } | Request::Read { session, .. } => *session,
                };
                if stopped.load(Ordering::Acquire)
                    || !action_gate.accepts(epoch)
                    || focus_revision != action_focus.snapshot().revision
                    || id == 0
                    || id != active.load(Ordering::Acquire)
                {
                    if matches!(request, Request::Read { .. }) {
                        pending.store(false, Ordering::Release);
                    }
                    continue;
                }
                let reply = match request {
                    Request::Action {
                        page,
                        session,
                        action,
                        input,
                    } => Reply::Action {
                        page,
                        session,
                        action: action.clone(),
                        input,
                        result: backend.execute(action),
                    },
                    Request::Read {
                        session,
                        targets,
                        system,
                    } => {
                        let system = backend.read_system(&system);
                        let (state, states) = backend.read_snapshot(&targets);
                        pending.store(false, Ordering::Release);
                        Reply::State {
                            system,
                            session,
                            state,
                            targets: targets.into_iter().zip(states).collect(),
                        }
                    }
                };
                // A stalled consumer ends the worker; it never blocks the audio thread forever.
                if outgoing.try_send(reply).is_err() {
                    break;
                }
            }
        });
        Self {
            plugin,
            foreground,
            gate,
            requests,
            read_pending,
            replies,
            session,
            stop,
            thread: Some(thread),
        }
    }
    pub fn session(&self, id: u64) {
        self.session.store(id, Ordering::Release);
    }
    pub fn submit(&self, request: Request) -> Result<(), &'static str> {
        if matches!(
            &request,
            Request::Action {
                action: Action::Plugin { .. },
                ..
            }
        ) {
            return self.plugin.submit(request);
        }
        let is_read = matches!(request, Request::Read { .. });
        if is_read && self.read_pending.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let result = self
            .requests
            .try_send((
                request,
                self.gate.snapshot().epoch,
                self.foreground.snapshot().revision,
            ))
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => "audio_queue_full",
                mpsc::TrySendError::Disconnected(_) => "audio_worker_stopped",
            });
        if is_read && result.is_err() {
            self.read_pending.store(false, Ordering::Release);
        }
        result
    }
    pub fn poll(&self) -> Result<Option<Reply>, &'static str> {
        if let Some(reply) = self.plugin.poll() {
            return Ok(Some(reply));
        }
        match self.replies.try_recv() {
            Ok(reply) => Ok(Some(reply)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err("audio_worker_stopped"),
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.session(0);
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Blocked {
        entered: SyncSender<()>,
        release: Receiver<()>,
        calls: Arc<AtomicU64>,
    }
    impl Backend for Blocked {
        fn execute(&mut self, _: Action) -> Result<(), &'static str> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.entered.send(()).unwrap();
            self.release.recv().unwrap();
            Ok(())
        }
        fn read(&mut self) -> Option<audio::State> {
            None
        }
    }
    fn request(session: u64) -> Request {
        Request::Action {
            page: 0,
            session,
            action: Action::MuteToggle,
            input: InputEvent {
                timestamp_ms: 0,
                event: decksmith_core::RawEvent::Key {
                    index: 4,
                    pressed: false,
                },
            },
        }
    }
    #[test]
    fn foreground_change_discards_actions_queued_for_previous_application() {
        let (entered, waiting) = mpsc::sync_channel(1);
        let (release, blocked) = mpsc::sync_channel(1);
        let calls = Arc::new(AtomicU64::new(0));
        let focus = crate::foreground::Foreground::default();
        focus.report("brave.desktop").unwrap();
        let worker = Worker::with_context(
            Blocked {
                entered,
                release: blocked,
                calls: calls.clone(),
            },
            Default::default(),
            focus.clone(),
        );
        worker.session(1);
        worker.submit(request(1)).unwrap();
        waiting.recv_timeout(Duration::from_secs(2)).unwrap();
        worker.submit(request(1)).unwrap();
        focus.report("teams.desktop").unwrap();
        release.send(()).unwrap();
        worker
            .submit(Request::Read {
                system: Vec::new(),
                session: 1,
                targets: vec![],
            })
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if matches!(worker.poll().unwrap(), Some(Reply::State { .. })) {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn lock_cycle_discards_queued_actions_even_without_device_worker_progress() {
        let (entered, waiting) = mpsc::sync_channel(1);
        let (release, blocked) = mpsc::sync_channel(1);
        let calls = Arc::new(AtomicU64::new(0));
        let gate = crate::session_lock::Gate::new(true);
        gate.observe(Some(false), "test");
        let worker = Worker::with_gate(
            Blocked {
                entered,
                release: blocked,
                calls: calls.clone(),
            },
            gate.clone(),
        );
        worker.session(1);
        worker.submit(request(1)).unwrap();
        waiting.recv_timeout(Duration::from_secs(2)).unwrap();
        worker.submit(request(1)).unwrap();
        gate.observe(Some(true), "test");
        gate.observe(Some(false), "test");
        release.send(()).unwrap();
        worker
            .submit(Request::Read {
                system: Vec::new(),
                session: 1,
                targets: vec![],
            })
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if matches!(worker.poll().unwrap(), Some(Reply::State { .. })) {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn blocked_audio_leaves_submit_and_poll_nonblocking_and_rejects_overflow() {
        let (entered, waiting) = mpsc::sync_channel(1);
        let (release, blocked) = mpsc::sync_channel(1);
        let calls = Arc::new(AtomicU64::new(0));
        let worker = Worker::with_backend(Blocked {
            entered,
            release: blocked,
            calls: calls.clone(),
        });
        worker.session(1);
        worker.submit(request(1)).unwrap();
        waiting.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(worker.poll().unwrap().is_none());
        for _ in 0..32 {
            worker.submit(request(1)).unwrap();
        }
        assert_eq!(worker.submit(request(1)), Err("audio_queue_full"));
        worker.session(2); // queued old-session toggles must not execute
        release.send(()).unwrap();
        // A fresh read serves as a barrier after all stale queued actions.
        loop {
            if worker
                .submit(Request::Read {
                    system: Vec::new(),
                    session: 2,
                    targets: Vec::new(),
                })
                .is_ok()
            {
                break;
            }
            std::thread::yield_now();
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if matches!(
                worker.poll().unwrap(),
                Some(Reply::State { session: 2, .. })
            ) {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
