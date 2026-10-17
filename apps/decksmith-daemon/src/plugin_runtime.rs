//! Experimental per-user Unix bridge. Separate bounded worker; no network on HID thread.
use crate::{
    audio_worker::{Reply, Request},
    pages::Action,
};
use std::{
    io::{Read, Write},
    os::unix::{fs::MetadataExt, net::UnixStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    time::{Duration, Instant},
};

#[derive(Clone)]
struct Job {
    request: Request,
    epoch: u64,
    focus: u64,
    display: u64,
    updated: Instant,
}
impl Job {
    fn compatible(&self, other: &Self) -> bool {
        if (self.epoch, self.focus, self.display) != (other.epoch, other.focus, other.display) {
            return false;
        }
        matches!((&self.request, &other.request),
            (Request::Action { page:a, session:b, action:Action::Plugin { binding:c, ticks:d }, .. },
             Request::Action { page:e, session:f, action:Action::Plugin { binding:g, ticks:h }, .. })
            if a==e && b==f && c==g && *d!=0 && *h!=0)
    }
    fn ticks(&mut self) -> &mut i16 {
        let Request::Action {
            action: Action::Plugin { ticks, .. },
            ..
        } = &mut self.request
        else {
            unreachable!()
        };
        ticks
    }
}
#[derive(Default)]
struct Queue {
    active: Option<Job>,
    pending: Option<Job>,
}
impl Queue {
    fn submit(&mut self, mut job: Job) -> Result<bool, &'static str> {
        if let Some(active) = &self.active {
            if !active.compatible(&job) {
                return Err("plugin_busy");
            }
            if let Some(pending) = &mut self.pending {
                // One bounded relative intent, never an unbounded event backlog.
                let total = (*pending.ticks())
                    .saturating_add(*job.ticks())
                    .clamp(-100, 100);
                *job.ticks() = total;
            }
            self.pending = Some(job);
            Ok(false)
        } else {
            self.active = Some(job);
            Ok(true)
        }
    }
    fn next(&mut self, success: bool) -> Option<Job> {
        self.active = None;
        let mut job = self.pending.take()?;
        if !success || job.updated.elapsed() > Duration::from_secs(2) || *job.ticks() == 0 {
            return None;
        }
        let total = *job.ticks();
        *job.ticks() = total.clamp(-20, 20);
        if total != *job.ticks() {
            let mut remainder = job.clone();
            *remainder.ticks() = total - *job.ticks();
            self.pending = Some(remainder);
        }
        self.active = Some(job.clone());
        Some(job)
    }
}
pub struct Worker {
    send: SyncSender<()>,
    replies: Receiver<Reply>,
    queue: Arc<Mutex<Queue>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    gate: crate::session_lock::Gate,
    foreground: crate::foreground::Foreground,
}
impl Worker {
    pub fn start(
        gate: crate::session_lock::Gate,
        foreground: crate::foreground::Foreground,
        active: Arc<AtomicU64>,
    ) -> Self {
        let (send, incoming) = mpsc::sync_channel::<()>(1);
        let (out, replies) = mpsc::sync_channel(4);
        let queue = Arc::new(Mutex::new(Queue::default()));
        let running = queue.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let g = gate.clone();
        let f = foreground.clone();
        let thread = std::thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                if incoming.recv_timeout(Duration::from_millis(50)).is_err() {
                    continue;
                }
                let mut next = running.lock().unwrap().active.clone();
                while let Some(job) = next {
                    let mut success = false;
                    if let Request::Action {
                        page,
                        session,
                        action,
                        input,
                    } = job.request
                    {
                        let allowed = || {
                            !stopped.load(Ordering::Acquire)
                                && session != 0
                                && active.load(Ordering::Acquire) == session
                                && DISPLAY_EPOCH.load(Ordering::Acquire) == job.display
                                && g.accepts(job.epoch)
                                && f.snapshot().revision == job.focus
                        };
                        let result = if allowed() {
                            exchange(&action, allowed)
                        } else {
                            Err("plugin_cancelled")
                        };
                        success = result.is_ok();
                        let _ = out.try_send(Reply::Action {
                            page,
                            session,
                            action,
                            input,
                            result,
                            report_receipt_to_dispatch_us: None,
                        });
                    }
                    next = running.lock().unwrap().next(success);
                }
            }
        });
        Self {
            send,
            replies,
            queue,
            stop,
            thread: Some(thread),
            gate,
            foreground,
        }
    }
    pub fn submit(&self, request: Request) -> Result<(), &'static str> {
        let job = Job {
            request,
            epoch: self.gate.snapshot().epoch,
            focus: self.foreground.snapshot().revision,
            display: DISPLAY_EPOCH.load(Ordering::Acquire),
            updated: Instant::now(),
        };
        let mut queue = self.queue.lock().unwrap();
        if queue.submit(job)? && self.send.try_send(()).is_err() {
            queue.active = None;
            queue.pending = None;
            return Err("plugin_unavailable");
        }
        Ok(())
    }
    pub fn poll(&self) -> Option<Reply> {
        self.replies.try_recv().ok()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
fn exchange(action: &Action, allowed: impl Fn() -> bool) -> Result<(), &'static str> {
    let path = std::env::var_os("XDG_RUNTIME_DIR")
        .ok_or("plugin_unavailable")
        .map(std::path::PathBuf::from)?
        .join("decksmith-plugin-lab.sock");
    exchange_at(action, &path, allowed)
}
fn exchange_at(
    action: &Action,
    path: &std::path::Path,
    allowed: impl Fn() -> bool,
) -> Result<(), &'static str> {
    let meta = std::fs::symlink_metadata(path).map_err(|_| "plugin_unavailable")?;
    use std::os::unix::fs::FileTypeExt;
    if !meta.file_type().is_socket()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
    {
        return Err("plugin_unavailable");
    }
    let mut stream = UnixStream::connect(path).map_err(|_| "plugin_unavailable")?;
    stream
        .set_read_timeout(Some(Duration::from_millis(50)))
        .map_err(|_| "plugin_unavailable")?;
    stream
        .set_write_timeout(Some(Duration::from_millis(100)))
        .map_err(|_| "plugin_unavailable")?;
    let Action::Plugin { binding, ticks } = action else {
        return Err("plugin_unsupported");
    };
    if !binding.supported(*ticks != 0) {
        return Err("plugin_unsupported");
    }
    let payload = serde_json::to_vec(&serde_json::json!({"binding":binding,"ticks":ticks}))
        .map_err(|_| "plugin_unsupported")?;
    if !allowed() {
        return Err("plugin_cancelled");
    }
    stream
        .write_all(&payload)
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|_| "plugin_unavailable")?;
    let start = Instant::now();
    let mut buffer = Vec::new();
    loop {
        if !allowed() {
            return Err("plugin_cancelled");
        }
        if start.elapsed() > Duration::from_secs(5) {
            return Err("plugin_unknown");
        }
        let mut part = [0; 512];
        match stream.read(&mut part) {
            Ok(0) => return Err("plugin_unknown"),
            Ok(n) => {
                buffer.extend_from_slice(&part[..n]);
                if buffer.len() > 8192 {
                    return Err("plugin_unknown");
                }
                if buffer.contains(&b'\n') {
                    let value: serde_json::Value =
                        serde_json::from_slice(&buffer).map_err(|_| "plugin_unknown")?;
                    return if value["ok"] == true {
                        Ok(())
                    } else {
                        Err("plugin_failed")
                    };
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return Err("plugin_unknown"),
        }
    }
}

/// A separate lane keeps optional Homebridge network/readback latency from
/// delaying audio/media. Uncertain actions are quarantined, never replayed.
#[cfg(any(feature = "hardware", test))]
#[derive(Default)]
pub(crate) struct HomebridgeBackend {
    blocked: bool,
    uncertain: Arc<AtomicBool>,
}
#[cfg(any(feature = "hardware", test))]
impl HomebridgeBackend {
    pub(crate) fn health(&self) -> Arc<AtomicBool> {
        self.uncertain.clone()
    }
}
#[cfg(any(feature = "hardware", test))]
impl crate::shared_worker::Backend for HomebridgeBackend {
    fn execute(
        &mut self,
        action: &Action,
        fence: &crate::shared_worker::Fence,
    ) -> Result<(), &'static str> {
        if self.blocked {
            return Err("plugin_recovery_required");
        }
        crate::shared_worker::Compiled::homebridge(action.clone())?;
        let Action::Plugin { binding, ticks } = action else {
            return Err("plugin_unsupported");
        };
        let mut remaining = *ticks;
        loop {
            let step = remaining.clamp(-20, 20);
            let action = Action::Plugin {
                binding: binding.clone(),
                ticks: step,
            };
            let result = exchange_shared(&action, || fence.accepts());
            if result == Err("plugin_unknown") {
                self.blocked = true;
                self.uncertain.store(true, Ordering::Release);
            }
            result?;
            remaining -= step;
            if remaining == 0 {
                return Ok(());
            }
        }
    }
}

/// Existing experimental Homebridge bridge only. Version 2 adds a late grant;
/// older companions reject the envelope before any accessory action is issued.
#[cfg(any(feature = "hardware", test))]
pub(crate) fn exchange_shared(
    action: &Action,
    allowed: impl Fn() -> bool,
) -> Result<(), &'static str> {
    let path =
        std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").ok_or("plugin_unavailable")?)
            .join("decksmith-plugin-lab.sock");
    exchange_shared_at(action, &path, allowed)
}
#[cfg(any(feature = "hardware", test))]
fn exchange_shared_at(
    action: &Action,
    path: &std::path::Path,
    allowed: impl Fn() -> bool,
) -> Result<(), &'static str> {
    use std::os::unix::fs::FileTypeExt;
    let Action::Plugin { binding, ticks } = action else {
        return Err("plugin_unsupported");
    };
    binding.validate()?;
    if !binding.supported(*ticks != 0) || !(-20..=20).contains(ticks) {
        return Err("plugin_unsupported");
    }
    let meta = std::fs::symlink_metadata(path).map_err(|_| "plugin_unavailable")?;
    if !meta.file_type().is_socket()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
    {
        return Err("plugin_unavailable");
    }
    let mut stream = connect_bounded(path)?;
    stream
        .set_read_timeout(Some(Duration::from_millis(50)))
        .map_err(|_| "plugin_unavailable")?;
    stream
        .set_write_timeout(Some(Duration::from_millis(100)))
        .map_err(|_| "plugin_unavailable")?;
    let payload =
        serde_json::to_vec(&serde_json::json!({"bridge":2,"binding":binding,"ticks":ticks}))
            .map_err(|_| "plugin_unsupported")?;
    if !allowed() {
        return Err("plugin_cancelled");
    }
    stream
        .write_all(&payload)
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|_| "plugin_unavailable")?;
    let started = Instant::now();
    let mut buffer = Vec::new();
    let mut granted = false;
    loop {
        if !allowed() {
            return Err(if granted {
                "plugin_unknown"
            } else {
                "plugin_cancelled"
            });
        }
        if started.elapsed() > Duration::from_secs(5) {
            return Err(if granted {
                "plugin_unknown"
            } else {
                "plugin_unavailable"
            });
        }
        let mut part = [0; 512];
        match stream.read(&mut part) {
            Ok(0) => {
                return Err(if granted {
                    "plugin_unknown"
                } else {
                    "plugin_unavailable"
                });
            }
            Ok(n) => {
                buffer.extend_from_slice(&part[..n]);
                if buffer.len() > 8192 {
                    return Err("plugin_unknown");
                }
                if buffer.contains(&b'\n') {
                    let value: serde_json::Value =
                        serde_json::from_slice(&buffer).map_err(|_| "plugin_unknown")?;
                    buffer.clear();
                    if value == serde_json::json!({"ready":true}) && !granted {
                        let live = allowed();
                        let response = if live {
                            b"{\"decision\":\"apply\"}\n".as_slice()
                        } else {
                            b"{\"decision\":\"abort\"}\n".as_slice()
                        };
                        granted = live;
                        stream.write_all(response).map_err(|_| "plugin_unknown")?;
                        if !live {
                            return Err("plugin_cancelled");
                        }
                    } else if (value == serde_json::json!({"ok":true}) && granted)
                        || (value == serde_json::json!({"ok":true,"effect":false}) && !granted)
                    {
                        return Ok(());
                    } else if value == serde_json::json!({"ok":false}) {
                        return Err("plugin_failed");
                    } else {
                        return Err("plugin_unknown");
                    }
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => {
                return Err(if granted {
                    "plugin_unknown"
                } else {
                    "plugin_unavailable"
                });
            }
        }
    }
}
#[cfg(any(feature = "hardware", test))]
fn connect_bounded(path: &std::path::Path) -> Result<UnixStream, &'static str> {
    use std::os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::ffi::OsStrExt,
    };
    let bytes = path.as_os_str().as_bytes();
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if bytes.contains(&0) || bytes.len() >= address.sun_path.len() {
        return Err("plugin_unavailable");
    }
    address.sun_family = libc::AF_UNIX as _;
    for (slot, byte) in address.sun_path.iter_mut().zip(bytes) {
        *slot = *byte as _;
    }
    let fd = unsafe {
        libc::socket(
            libc::AF_UNIX,
            libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
            0,
        )
    };
    if fd < 0 {
        return Err("plugin_unavailable");
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    let result = unsafe {
        libc::connect(
            fd.as_raw_fd(),
            &address as *const _ as *const libc::sockaddr,
            std::mem::size_of_val(&address) as _,
        )
    };
    if result != 0 {
        if !matches!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EINPROGRESS) | Some(libc::EAGAIN)
        ) {
            return Err("plugin_unavailable");
        }
        let mut poll = libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLOUT,
            revents: 0,
        };
        if unsafe { libc::poll(&mut poll, 1, 100) } != 1 {
            return Err("plugin_unavailable");
        }
        let mut error = 0i32;
        let mut len = std::mem::size_of_val(&error) as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                fd.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                &mut error as *mut _ as *mut libc::c_void,
                &mut len,
            )
        } != 0
            || error != 0
        {
            return Err("plugin_unavailable");
        }
    }
    let stream = UnixStream::from(fd);
    stream
        .set_nonblocking(false)
        .map_err(|_| "plugin_unavailable")?;
    Ok(stream)
}

static DISPLAY_EPOCH: AtomicU64 = AtomicU64::new(1);
pub fn invalidate() {
    DISPLAY_EPOCH.fetch_add(1, Ordering::AcqRel);
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub on: bool,
    pub brightness: u8,
    pub revision: u64,
    pub available: bool,
}
#[derive(Clone, PartialEq, Eq)]
struct SnapshotKey {
    path: std::path::PathBuf,
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
struct Snapshot {
    key: SnapshotKey,
    value: serde_json::Value,
    revision: u64,
}
thread_local! {
    static SNAPSHOT: std::cell::RefCell<Option<std::sync::Arc<Snapshot>>> = const { std::cell::RefCell::new(None) };
}
fn snapshot_key(path: &std::path::Path, meta: &std::fs::Metadata) -> Option<SnapshotKey> {
    if !meta.is_file()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
        || meta.len() > 131072
        || meta.modified().ok()?.elapsed().ok()? > Duration::from_secs(3)
    {
        return None;
    }
    Some(SnapshotKey {
        path: path.into(),
        device: meta.dev(),
        inode: meta.ino(),
        length: meta.len(),
        modified: (meta.mtime(), meta.mtime_nsec()),
        changed: (meta.ctime(), meta.ctime_nsec()),
    })
}
fn snapshot_at(path: &std::path::Path) -> Option<std::sync::Arc<Snapshot>> {
    use std::os::unix::fs::OpenOptionsExt;
    let key = snapshot_key(path, &std::fs::symlink_metadata(path).ok()?)?;
    SNAPSHOT.with(|cache| {
        if let Some(previous) = cache.borrow().as_ref().filter(|p| p.key == key) {
            return Some(previous.clone());
        }
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)
            .ok()?;
        let key = snapshot_key(path, &file.metadata().ok()?)?;
        let mut raw = Vec::new();
        file.take(131073).read_to_end(&mut raw).ok()?;
        if raw.len() > 131072 {
            return None;
        }
        let value: serde_json::Value = serde_json::from_slice(&raw).ok()?;
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        value["accessories"].to_string().hash(&mut hash);
        value["catalog_ready"].to_string().hash(&mut hash);
        value["actual"].to_string().hash(&mut hash);
        value["ready"].to_string().hash(&mut hash);
        let result = std::sync::Arc::new(Snapshot {
            key,
            value,
            revision: hash.finish(),
        });
        *cache.borrow_mut() = Some(result.clone());
        Some(result)
    })
}
fn snapshot() -> Option<std::sync::Arc<Snapshot>> {
    let path = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?)
        .join("decksmith-plugin-state.json");
    snapshot_at(&path)
}
/// Background private-state observer; the input coordinator polls only memory.
#[cfg(any(feature = "hardware", test))]
type ObservationShared = Arc<(Mutex<(bool, Option<u64>)>, std::sync::Condvar)>;
#[cfg(any(feature = "hardware", test))]
pub(crate) struct Observer {
    shared: ObservationShared,
    thread: Option<std::thread::JoinHandle<()>>,
    shown: Option<Option<u64>>,
}
#[cfg(any(feature = "hardware", test))]
impl Observer {
    pub fn start() -> Result<Self, &'static str> {
        let shared = Arc::new((Mutex::new((false, None)), std::sync::Condvar::new()));
        let owner = shared.clone();
        let thread = std::thread::Builder::new()
            .name("decksmith-homebridge-observer".into())
            .spawn(move || {
                loop {
                    let revision = snapshot().map(|s| s.revision);
                    let mut state = owner.0.lock().unwrap();
                    if state.0 {
                        return;
                    }
                    state.1 = revision;
                    let (state, _) = owner
                        .1
                        .wait_timeout(state, Duration::from_millis(500))
                        .unwrap();
                    if state.0 {
                        return;
                    }
                }
            })
            .map_err(|_| "plugin_observer_spawn_failed")?;
        Ok(Self {
            shared,
            thread: Some(thread),
            shown: None,
        })
    }
    pub fn snapshot(&mut self) -> Option<Option<u64>> {
        let current = self.shared.0.lock().unwrap().1;
        if self.shown == Some(current) {
            return None;
        }
        self.shown = Some(current);
        Some(current)
    }
}
#[cfg(any(feature = "hardware", test))]
impl Drop for Observer {
    fn drop(&mut self) {
        self.shared.0.lock().unwrap().0 = true;
        self.shared.1.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub fn state() -> Option<State> {
    let snapshot = snapshot()?;
    let v = &snapshot.value;
    if v["target"] != "b3d109c968d17f5cc965ddfa89a1fa695a2d60832200b819ad08127ee15634b4" {
        return None;
    }
    let brightness = v["actual"]["brightness"].as_f64().unwrap_or(0.0);
    if !(0.0..=100.0).contains(&brightness) {
        return None;
    }
    Some(State {
        on: v["actual"]["on"].as_bool().unwrap_or(false),
        available: v["ready"] == true,
        brightness: brightness.round() as u8,
        revision: snapshot.revision,
    })
}

/// Read the private, short-lived catalogue snapshot. No network calls on rendering.
pub fn caption(binding: &crate::plugin_binding::Binding, dial: bool) -> Option<String> {
    if binding.schema != 2 || !binding.supported(dial) {
        return None;
    }
    let snapshot = snapshot()?;
    let v = &snapshot.value;
    if v["catalog_ready"] != true {
        return None;
    }
    let item = &v["accessories"][binding.settings["accessoryId"].as_str()?];
    if dial {
        let level = item["level"].as_f64()?;
        if !(0.0..=100.0).contains(&level) {
            return None;
        }
        Some(format!(
            "{:.0}%{}",
            level,
            if item["on"] == false { " Off" } else { "" }
        ))
    } else {
        let text = item["text"].as_str()?;
        (text.len() <= 64).then(|| text.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::{fs::PermissionsExt, net::UnixListener};
    fn action() -> Action {
        Action::Plugin {
            binding: crate::plugin_binding::Binding {
                provider: "com.infamous-pattern.openhomeb".into(),
                action: "com.infamous-pattern.openhomeb.set".into(),
                schema: 1,
                settings: serde_json::json!({"accessoryId":"b3d109c968d17f5cc965ddfa89a1fa695a2d60832200b819ad08127ee15634b4","characteristicType":"On","targetValue":true}),
            },
            ticks: 0,
        }
    }
    fn job(ticks: i16) -> Job {
        let mut action = action();
        if let Action::Plugin { ticks: value, .. } = &mut action {
            *value = ticks;
        }
        Job {
            request: Request::Action {
                page: 3,
                session: 4,
                action,
                input: decksmith_core::InputEvent {
                    timestamp_ms: 0,
                    event: decksmith_core::RawEvent::DialRotate { index: 0, ticks },
                },
            },
            epoch: 1,
            focus: 2,
            display: 3,
            updated: Instant::now(),
        }
    }
    #[test]
    fn snapshots_reuse_only_unchanged_private_fresh_files() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!(
            "decksmith-cache-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("state.json");
        let write = |path: &std::path::Path, text: &str| {
            std::fs::write(path, text).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        };
        write(
            &path,
            r#"{"accessories":{"a":{"text":"On"}},"catalog_ready":true}"#,
        );
        let first = snapshot_at(&path).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &snapshot_at(&path).unwrap()));
        let replacement = root.join("new.json");
        write(
            &replacement,
            r#"{"accessories":{"a":{"text":"No"}},"catalog_ready":true}"#,
        );
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        std::fs::File::open(&replacement)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        std::fs::rename(&replacement, &path).unwrap();
        let second = snapshot_at(&path).unwrap();
        assert_ne!(first.revision, second.revision);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(snapshot_at(&path).is_none());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::File::open(&path)
            .unwrap()
            .set_times(
                std::fs::FileTimes::new()
                    .set_modified(std::time::SystemTime::now() - Duration::from_secs(4)),
            )
            .unwrap();
        assert!(snapshot_at(&path).is_none());
        std::fs::remove_file(&path).unwrap();
        assert!(snapshot_at(&path).is_none());
        std::os::unix::fs::symlink("/dev/zero", &path).unwrap();
        assert!(snapshot_at(&path).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn fast_turns_coalesce_into_bounded_steps() {
        let mut queue = Queue::default();
        assert_eq!(queue.submit(job(5)), Ok(true));
        for _ in 0..6 {
            assert_eq!(queue.submit(job(5)), Ok(false));
        }
        assert_eq!(*queue.next(true).unwrap().ticks(), 20);
        assert_eq!(*queue.next(true).unwrap().ticks(), 10);
        assert!(queue.next(true).is_none());
        assert!(queue.active.is_none());
        queue.submit(job(5)).unwrap();
        for _ in 0..1000 {
            queue.submit(job(5)).unwrap();
        }
        assert_eq!(*queue.pending.as_mut().unwrap().ticks(), 100);
    }
    #[test]
    fn failed_or_stale_turns_are_discarded_not_replayed() {
        for stale in [false, true] {
            let mut queue = Queue::default();
            queue.submit(job(5)).unwrap();
            let mut pending = job(5);
            if stale {
                pending.updated = Instant::now() - Duration::from_secs(3);
            }
            queue.submit(pending).unwrap();
            assert!(queue.next(stale).is_none());
            assert!(queue.pending.is_none());
            assert!(queue.active.is_none());
            assert_eq!(queue.submit(job(-5)), Ok(true));
        }
    }
    #[test]
    fn reversals_cancel_pending_motion_and_contexts_do_not_mix() {
        let mut queue = Queue::default();
        queue.submit(job(5)).unwrap();
        queue.submit(job(10)).unwrap();
        queue.submit(job(-10)).unwrap();
        assert!(queue.next(true).is_none());
        queue.submit(job(5)).unwrap();
        let mut changed = job(5);
        changed.display += 1;
        assert_eq!(queue.submit(changed), Err("plugin_busy"));
        assert_eq!(queue.submit(job(0)), Err("plugin_busy"));
        assert!(queue.pending.is_none());
    }
    #[test]
    fn transport_confirms_once_and_cancel_closes_connection() {
        for cancel in [false, true] {
            let path = std::env::temp_dir().join(format!(
                "decksmith-plugin-test-{}-{cancel}.sock",
                std::process::id()
            ));
            let listener = UnixListener::bind(&path).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            let allowed = Arc::new(AtomicBool::new(true));
            let flag = allowed.clone();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0];
                while stream.read(&mut byte).unwrap() == 1 {
                    request.push(byte[0]);
                    if byte[0] == b'\n' {
                        break;
                    }
                }
                assert!(
                    serde_json::from_slice::<serde_json::Value>(&request).unwrap()["binding"]
                        .is_object()
                );
                if cancel {
                    flag.store(false, Ordering::Release);
                    assert_eq!(stream.read(&mut byte).unwrap(), 0);
                } else {
                    stream.write_all(b"{\"ok\":true}\n").unwrap();
                }
            });
            let result = exchange_at(&action(), &path, || allowed.load(Ordering::Acquire));
            assert_eq!(
                result,
                if cancel {
                    Err("plugin_cancelled")
                } else {
                    Ok(())
                }
            );
            server.join().unwrap();
            std::fs::remove_file(path).unwrap();
        }
    }
    #[test]
    fn page_epoch_invalidates_queued_context() {
        let old = DISPLAY_EPOCH.load(Ordering::Acquire);
        invalidate();
        assert_ne!(old, DISPLAY_EPOCH.load(Ordering::Acquire));
    }
    #[test]
    fn shared_bridge_never_falls_back_and_rechecks_after_discovery() {
        for mode in ["apply", "stale", "old", "readonly"] {
            let root = std::env::temp_dir().join(format!(
                "decksmith-shared-hb-{}-{}",
                std::process::id(),
                mode
            ));
            std::fs::create_dir_all(&root).unwrap();
            let path = root.join("bridge.sock");
            let listener = UnixListener::bind(&path).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            let allowed = Arc::new(AtomicBool::new(true));
            let live = allowed.clone();
            let task = std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let (mut peer, _) = listener.accept().unwrap();
                peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut reader = BufReader::new(peer.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let payload: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert_eq!(payload["bridge"], 2);
                if mode == "old" {
                    peer.write_all(b"{\"ok\":false}\n").unwrap();
                    return;
                }
                if mode == "readonly" {
                    peer.write_all(b"{\"ok\":true,\"effect\":false}\n").unwrap();
                    return;
                }
                if mode == "stale" {
                    live.store(false, Ordering::Release);
                }
                if let Err(error) = peer.write_all(b"{\"ready\":true}\n") {
                    // Cancellation may close the client before it reads readiness.
                    // Only the stale case may close without a protocol response.
                    assert_eq!(mode, "stale");
                    assert!(matches!(
                        error.kind(),
                        std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                    ));
                    return;
                }
                line.clear();
                let decision = reader.read_line(&mut line);
                if mode == "stale" {
                    // Closing with unread readiness bytes can produce reset rather
                    // than EOF. Neither is permission to apply the action.
                    match decision {
                        Ok(0) => assert!(line.is_empty()),
                        Ok(_) => assert_eq!(line, "{\"decision\":\"abort\"}\n"),
                        Err(error) => {
                            assert_eq!(error.kind(), std::io::ErrorKind::ConnectionReset);
                            assert!(line.is_empty());
                        }
                    }
                } else {
                    decision.unwrap();
                    assert_eq!(line, "{\"decision\":\"apply\"}\n");
                    peer.write_all(b"{\"ok\":true}\n").unwrap();
                }
            });
            let result = exchange_shared_at(&action(), &path, || allowed.load(Ordering::Acquire));
            assert_eq!(
                result,
                match mode {
                    "stale" => Err("plugin_cancelled"),
                    "old" => Err("plugin_failed"),
                    _ => Ok(()),
                }
            );
            task.join().unwrap();
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
