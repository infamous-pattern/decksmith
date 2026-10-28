//! Owned, bounded logind sleep observation. Never requests a power operation.
//! A delay inhibitor is released only after the coordinator acknowledges real
//! microphone/action/I/O settlement, or a recorded preparation deadline expires.
use async_channel::{Receiver, Sender};
use futures_lite::{StreamExt, future};
use serde::Serialize;
use std::{
    os::fd::OwnedFd,
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
const CALL: Duration = Duration::from_millis(300);
const STALE: Duration = Duration::from_secs(1);
const CHECK: Duration = Duration::from_millis(200);
#[derive(Clone, Serialize)]
pub(crate) struct Snapshot {
    pub epoch: u64,
    pub available: bool,
    pub preparing: bool,
    pub blocked: bool,
    pub error: Option<&'static str>,
    pub preparation_failures: u64,
    pub last_preparation_error: Option<&'static str>,
}
struct State {
    snapshot: Snapshot,
    checked: Instant,
    queued_ack: Option<u64>,
}
impl State {
    fn publish(&mut self, available: bool, preparing: bool, error: Option<&'static str>) {
        if self.snapshot.available != available || self.snapshot.preparing != preparing {
            self.snapshot.epoch = self.snapshot.epoch.saturating_add(1);
            self.queued_ack = None;
        }
        self.snapshot.available = available;
        self.snapshot.preparing = preparing;
        self.snapshot.blocked = !available || preparing;
        self.snapshot.error = error;
        self.checked = Instant::now();
    }
    fn unknown(&mut self, reason: &'static str) {
        self.publish(false, self.snapshot.preparing, Some(reason));
    }
}
enum Command {
    Acknowledge(u64, bool),
    Refresh,
}
pub(crate) struct Observer {
    state: Arc<Mutex<State>>,
    commands: Sender<Command>,
    thread: Option<JoinHandle<()>>,
}
impl Observer {
    pub(crate) fn start() -> Result<Self> {
        Self::start_with(None, "org.freedesktop.login1")
    }
    fn start_with(address: Option<String>, destination: &'static str) -> Result<Self> {
        let state = Arc::new(Mutex::new(State {
            snapshot: Snapshot {
                epoch: 1,
                available: false,
                preparing: false,
                blocked: true,
                error: Some("sleep_watch_unavailable"),
                preparation_failures: 0,
                last_preparation_error: None,
            },
            checked: Instant::now(),
            queued_ack: None,
        }));
        let (commands, receiver) = async_channel::bounded(16);
        let owner = state.clone();
        let thread = std::thread::Builder::new()
            .name("decksmith-sleep-watch".into())
            .spawn(move || async_io::block_on(run(owner, receiver, address, destination)))
            .map_err(|_| "sleep_watch_spawn_failed")?;
        Ok(Self {
            state,
            commands,
            thread: Some(thread),
        })
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        let mut state = self.state.lock().unwrap();
        if state.checked.elapsed() > STALE {
            state.unknown("sleep_watch_stale");
        }
        state.snapshot.clone()
    }
    pub(crate) fn acknowledge(&self, epoch: u64, success: bool) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        if state.snapshot.epoch != epoch
            || !state.snapshot.preparing
            || state.queued_ack == Some(epoch)
        {
            return Ok(());
        }
        self.commands
            .try_send(Command::Acknowledge(epoch, success))
            .map_err(|_| "sleep_watch_queue_full")?;
        state.queued_ack = Some(epoch);
        Ok(())
    }
    /// CLOCK_MONOTONIC excludes sleep: never trust an apparently fresh pre-sleep
    /// Awake snapshot after the independent BOOTTIME monitor reports a gap.
    pub(crate) fn resumed(&self) {
        let mut state = self.state.lock().unwrap();
        state.snapshot.epoch = state.snapshot.epoch.saturating_add(1);
        state.unknown("sleep_resume_revalidation");
        drop(state);
        let _ = self.commands.try_send(Command::Refresh);
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        self.commands.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
async fn bounded<T>(job: impl std::future::Future<Output = zbus::Result<T>>) -> Result<T> {
    future::race(
        async { job.await.map_err(|_| "sleep_bus_unavailable") },
        async {
            async_io::Timer::after(CALL).await;
            Err("sleep_bus_timeout")
        },
    )
    .await
}
async fn connect(address: Option<&str>) -> Result<zbus::Connection> {
    let builder = match address {
        Some(address) => zbus::connection::Builder::address(address),
        None => zbus::connection::Builder::system(),
    };
    bounded(builder.method_timeout(CALL).build()).await
}
async fn run(
    state: Arc<Mutex<State>>,
    commands: Receiver<Command>,
    address: Option<String>,
    destination: &'static str,
) {
    loop {
        if commands.is_closed() {
            break;
        }
        let connection = match connect(address.as_deref()).await {
            Ok(connection) => connection,
            Err(error) => {
                state.lock().unwrap().unknown(error);
                if future::race(async { commands.recv().await.is_err() }, async {
                    async_io::Timer::after(Duration::from_secs(1)).await;
                    false
                })
                .await
                {
                    break;
                }
                continue;
            }
        };
        let result = connected(&state, &commands, &connection, destination).await;
        if commands.is_closed() {
            break;
        }
        state
            .lock()
            .unwrap()
            .unknown(result.err().unwrap_or("sleep_bus_disconnected"));
        if future::race(async { commands.recv().await.is_err() }, async {
            async_io::Timer::after(Duration::from_secs(1)).await;
            false
        })
        .await
        {
            break;
        }
    }
}
async fn connected(
    state: &Arc<Mutex<State>>,
    commands: &Receiver<Command>,
    connection: &zbus::Connection,
    destination: &'static str,
) -> Result<()> {
    let proxy = bounded(
        zbus::proxy::Builder::<zbus::Proxy<'_>>::new(connection)
            .destination(destination)
            .path("/org/freedesktop/login1")
            .interface("org.freedesktop.login1.Manager")
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build(),
    )
    .await?;
    // Register both streams before taking the uncached initial property sample.
    let mut signals = bounded(proxy.receive_signal("PrepareForSleep")).await?;
    let mut owners = bounded(proxy.receive_owner_changed()).await?;
    let max_delay: u64 = bounded(proxy.get_property("InhibitDelayMaxUSec")).await?;
    let delay = Duration::from_micros(max_delay).min(Duration::from_secs(4));
    if delay < Duration::from_millis(200) {
        return Err("sleep_delay_too_short");
    }
    let mut inhibitor = Some(inhibit(&proxy).await?);
    let epoch = state.lock().unwrap().snapshot.epoch;
    let preparing: bool = bounded(proxy.get_property("PreparingForSleep")).await?;
    let fresh = {
        let mut current = state.lock().unwrap();
        let fresh = current.snapshot.epoch == epoch;
        if fresh {
            current.publish(true, preparing, None);
        }
        fresh
    };
    let mut deadline =
        (fresh && preparing).then(|| Instant::now() + delay - Duration::from_millis(100));
    loop {
        if commands.is_closed() {
            return Ok(());
        }
        enum Event {
            Command(std::result::Result<Command, async_channel::RecvError>),
            Signal(Option<zbus::Message>),
            Owner,
            Check,
        }
        let event = future::race(
            future::race(async { Event::Command(commands.recv().await) }, async {
                Event::Signal(signals.next().await)
            }),
            future::race(
                async {
                    let _ = owners.next().await;
                    Event::Owner
                },
                async {
                    let wait = deadline.map_or(CHECK, |end| {
                        end.saturating_duration_since(Instant::now()).min(CHECK)
                    });
                    async_io::Timer::after(wait).await;
                    Event::Check
                },
            ),
        )
        .await;
        let (preparing, sampled_epoch) = match event {
            Event::Command(Err(_)) => return Ok(()),
            Event::Command(Ok(Command::Acknowledge(epoch, success))) => {
                let mut current = state.lock().unwrap();
                if current.snapshot.epoch == epoch
                    && current.snapshot.preparing
                    && deadline.is_some()
                {
                    inhibitor.take();
                    deadline = None;
                    if !success {
                        current.snapshot.preparation_failures =
                            current.snapshot.preparation_failures.saturating_add(1);
                        current.snapshot.error = Some("sleep_preparation_failed");
                        current.snapshot.last_preparation_error = Some("sleep_preparation_failed");
                    }
                }
                continue;
            }
            Event::Command(Ok(Command::Refresh)) | Event::Check => {
                // Release before a bounded property request can consume the
                // preparation budget. The OS must never wait for our read.
                if deadline.is_some_and(|end| Instant::now() >= end) {
                    inhibitor.take();
                    deadline = None;
                    let mut current = state.lock().unwrap();
                    current.snapshot.preparation_failures =
                        current.snapshot.preparation_failures.saturating_add(1);
                    current.snapshot.error = Some("sleep_preparation_timeout");
                    current.snapshot.last_preparation_error = Some("sleep_preparation_timeout");
                }
                if connection.is_closed() {
                    return Err("sleep_bus_disconnected");
                }
                let epoch = state.lock().unwrap().snapshot.epoch;
                let value = bounded(proxy.get_property::<bool>("PreparingForSleep")).await?;
                (value, Some(epoch))
            }
            Event::Signal(Some(message)) => (
                message
                    .body()
                    .deserialize::<(bool,)>()
                    .map_err(|_| "sleep_signal_invalid")?
                    .0,
                None,
            ),
            Event::Signal(None) | Event::Owner => return Err("sleep_owner_changed"),
        };
        if preparing {
            let mut current = state.lock().unwrap();
            if sampled_epoch.is_some_and(|epoch| epoch != current.snapshot.epoch) {
                continue;
            }
            if !current.snapshot.preparing {
                current.publish(
                    inhibitor.is_some(),
                    true,
                    if inhibitor.is_some() {
                        None
                    } else {
                        Some("sleep_inhibitor_unavailable")
                    },
                );
                deadline = Some(Instant::now() + delay - Duration::from_millis(100));
            } else {
                current.checked = Instant::now();
            }
            if deadline.is_some_and(|end| Instant::now() >= end) {
                inhibitor.take();
                deadline = None;
                current.snapshot.preparation_failures =
                    current.snapshot.preparation_failures.saturating_add(1);
                current.snapshot.error = Some("sleep_preparation_timeout");
                current.snapshot.last_preparation_error = Some("sleep_preparation_timeout");
            }
        } else {
            if sampled_epoch.is_some_and(|epoch| epoch != state.lock().unwrap().snapshot.epoch) {
                continue;
            }
            if deadline.take().is_some() {
                inhibitor.take();
                let mut current = state.lock().unwrap();
                current.snapshot.preparation_failures =
                    current.snapshot.preparation_failures.saturating_add(1);
                current.snapshot.last_preparation_error = Some("sleep_preparation_unacknowledged");
            }
            if inhibitor.is_none() {
                inhibitor = Some(inhibit(&proxy).await?);
            }
            let mut current = state.lock().unwrap();
            if sampled_epoch.is_none_or(|epoch| epoch == current.snapshot.epoch) {
                current.publish(true, false, None);
            }
        }
    }
}
async fn inhibit(proxy: &zbus::Proxy<'_>) -> Result<OwnedFd> {
    let fd: zbus::OwnedFd = bounded(proxy.call(
        "Inhibit",
        &(
            "sleep",
            "Decksmith",
            "Settle microphone and device controls",
            "delay",
        ),
    ))
    .await?;
    Ok(fd.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{ErrorKind, Read},
        os::unix::net::UnixStream,
        sync::atomic::{AtomicBool, Ordering},
    };
    type SampleGate = Arc<(Mutex<(bool, usize, usize)>, std::sync::Condvar)>;
    struct Probe {
        preparing: Arc<AtomicBool>,
        peers: Arc<Mutex<Vec<UnixStream>>>,
        delay: u64,
        gate: SampleGate,
    }
    #[zbus::interface(name = "org.freedesktop.login1.Manager")]
    impl Probe {
        #[zbus(property(emits_changed_signal = "false"))]
        fn preparing_for_sleep(&self) -> bool {
            let (lock, wake) = &*self.gate;
            let mut gate = lock.lock().unwrap();
            if gate.0 {
                gate.1 += 1;
                let request = gate.1;
                wake.notify_all();
                while gate.0 && gate.2 < request {
                    gate = wake.wait(gate).unwrap();
                }
            }
            self.preparing.load(Ordering::Acquire)
        }
        #[zbus(property, name = "InhibitDelayMaxUSec")]
        fn inhibit_delay_max_usec(&self) -> u64 {
            self.delay
        }
        fn inhibit(
            &self,
            what: &str,
            _who: &str,
            _why: &str,
            mode: &str,
        ) -> zbus::fdo::Result<zbus::OwnedFd> {
            assert_eq!((what, mode), ("sleep", "delay"));
            let (reader, writer) = UnixStream::pair().unwrap();
            reader.set_nonblocking(true).unwrap();
            self.peers.lock().unwrap().push(reader);
            Ok(OwnedFd::from(writer).into())
        }
    }
    struct PrivateBus(std::process::Child);
    impl Drop for PrivateBus {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    struct Rig {
        observer: Observer,
        server: zbus::Connection,
        preparing: Arc<AtomicBool>,
        peers: Arc<Mutex<Vec<UnixStream>>>,
        gate: SampleGate,
        _bus: PrivateBus,
    }
    impl Rig {
        fn new(delay: u64) -> Self {
            use std::io::BufRead;
            let mut child = std::process::Command::new("dbus-daemon")
                .args(["--session", "--nofork", "--print-address=1", "--nopidfile"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("private dbus-daemon required");
            let mut address = String::new();
            std::io::BufReader::new(child.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            let address = address.trim().to_owned();
            assert!(!address.is_empty());
            let bus = PrivateBus(child);
            let preparing = Arc::new(AtomicBool::new(false));
            let peers: Arc<Mutex<Vec<UnixStream>>> = Arc::default();
            let gate = Arc::new((Mutex::new((false, 0, 0)), std::sync::Condvar::new()));
            let destination: &'static str = Box::leak(
                format!(
                    "cc.senecal.Decksmith.SleepProbe{}",
                    crate::device_profiles::random_device_id().unwrap()
                )
                .into_boxed_str(),
            );
            let server = futures_lite::future::block_on(
                zbus::connection::Builder::address(address.as_str())
                    .name(destination)
                    .serve_at(
                        "/org/freedesktop/login1",
                        Probe {
                            preparing: preparing.clone(),
                            peers: peers.clone(),
                            delay,
                            gate: gate.clone(),
                        },
                    )
                    .build(),
            )
            .unwrap();
            let observer = Observer::start_with(Some(address), destination).unwrap();
            Self {
                observer,
                server,
                preparing,
                peers,
                gate,
                _bus: bus,
            }
        }
        fn wait(&self, check: impl Fn(&Snapshot) -> bool) -> Snapshot {
            let end = Instant::now() + Duration::from_secs(5);
            loop {
                let s = self.observer.snapshot();
                if check(&s) {
                    return s;
                }
                assert!(
                    Instant::now() < end,
                    "sleep fixture deadline: {:?}",
                    s.error
                );
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        fn sleep(&self, preparing: bool) {
            self.preparing.store(preparing, Ordering::Release);
            futures_lite::future::block_on(self.server.emit_signal(
                None::<&str>,
                "/org/freedesktop/login1",
                "org.freedesktop.login1.Manager",
                "PrepareForSleep",
                &(preparing,),
            ))
            .unwrap();
        }
        fn closed(&self, index: usize) -> bool {
            let mut peers = self.peers.lock().unwrap();
            let mut byte = [0];
            match peers[index].read(&mut byte) {
                Ok(0) => true,
                Err(error) if error.kind() == ErrorKind::WouldBlock => false,
                other => panic!("unexpected inhibitor state {other:?}"),
            }
        }
    }
    #[test]
    fn inhibitor_waits_for_matching_settlement_and_reacquires_after_resume() {
        let r = Rig::new(2_000_000);
        let first = r.wait(|s| s.available && !s.blocked);
        assert!(!r.closed(0));
        r.sleep(true);
        let prep = r.wait(|s| s.available && s.preparing);
        assert!(prep.epoch > first.epoch);
        assert!(!r.closed(0));
        r.observer.acknowledge(first.epoch, true).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        assert!(!r.closed(0));
        r.observer.acknowledge(prep.epoch, true).unwrap();
        let end = Instant::now() + Duration::from_secs(2);
        while !r.closed(0) {
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(5));
        }
        r.sleep(false);
        r.wait(|s| s.available && !s.blocked && s.epoch > prep.epoch);
        assert_eq!(r.peers.lock().unwrap().len(), 2);
        assert!(!r.closed(1));
        let start = Instant::now();
        drop(r.observer);
        assert!(start.elapsed() < Duration::from_secs(1));
        let mut peers = r.peers.lock().unwrap();
        let mut byte = [0];
        assert_eq!(peers[1].read(&mut byte).unwrap(), 0);
    }
    #[test]
    fn deadline_is_reported_and_unknown_owner_never_unlocks() {
        let r = Rig::new(350_000);
        r.wait(|s| s.available && !s.blocked);
        r.sleep(true);
        let failure = r.wait(|s| s.preparation_failures == 1);
        assert_eq!(failure.error, Some("sleep_preparation_timeout"));
        assert!(failure.blocked);
        assert!(r.closed(0));
        futures_lite::future::block_on(r.server.clone().close()).unwrap();
        r.wait(|s| !s.available && s.blocked);
        let start = Instant::now();
        drop(r.observer);
        assert!(start.elapsed() < Duration::from_secs(1));
    }
    #[test]
    fn resume_clock_gap_invalidates_awake_until_uncached_revalidation() {
        let r = Rig::new(2_000_000);
        let first = r.wait(|s| s.available && !s.blocked);
        r.observer.resumed();
        let pending = r.observer.snapshot();
        // A fast uncached refresh may already have crossed the resume fence.
        assert!(!pending.available && pending.blocked || pending.epoch > first.epoch + 2);
        assert!(pending.epoch > first.epoch);
        r.wait(|s| s.available && !s.blocked && s.epoch > first.epoch + 2);
    }
    #[test]
    fn pre_resume_property_reply_cannot_clear_resume_fence() {
        let r = Rig::new(2_000_000);
        r.wait(|s| s.available && !s.blocked);
        let (lock, wake) = &*r.gate;
        lock.lock().unwrap().0 = true;
        let wait_request = |n| {
            let end = Instant::now() + Duration::from_secs(3);
            let mut gate = lock.lock().unwrap();
            while gate.1 < n {
                let (next, timeout) = wake
                    .wait_timeout(gate, end.saturating_duration_since(Instant::now()))
                    .unwrap();
                gate = next;
                assert!(!timeout.timed_out(), "property request did not arrive");
            }
        };
        wait_request(1);
        r.observer.resumed();
        {
            let mut gate = lock.lock().unwrap();
            gate.2 = 1;
            wake.notify_all();
        }
        wait_request(2);
        let pending = r.observer.snapshot();
        assert!(!pending.available && pending.blocked);
        {
            let mut gate = lock.lock().unwrap();
            gate.0 = false;
            wake.notify_all();
        }
        r.wait(|s| s.available && !s.blocked);
    }
}
