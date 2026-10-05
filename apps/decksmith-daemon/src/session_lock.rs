//! Conservative session lock gate. Unknown/stale state blocks when enabled.
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub enabled: bool,
    pub locked: bool,
    pub available: bool,
    pub source: &'static str,
    pub epoch: u64,
}
struct State {
    view: Snapshot,
    observed: Option<bool>,
    checked: Instant,
}
#[derive(Clone)]
pub struct Gate(Arc<Mutex<State>>);

fn lock_proxy(
    connection: &zbus::blocking::Connection,
    destination: &'static str,
    path: &'static str,
    interface: &'static str,
) -> Option<zbus::blocking::Proxy<'static>> {
    zbus::blocking::proxy::Builder::new(connection)
        .destination(destination)
        .ok()?
        .path(path)
        .ok()?
        .interface(interface)
        .ok()?
        // session/auto is an alias: property-change signals use the canonical
        // path. Always read lock properties afresh, even with a reusable proxy.
        .cache_properties(zbus::proxy::CacheProperties::No)
        .build()
        .ok()
}

fn connect_lock_proxy(session: bool) -> Option<zbus::blocking::Proxy<'static>> {
    let connection = if session {
        zbus::blocking::connection::Builder::session()
    } else {
        zbus::blocking::connection::Builder::system()
    }
    .ok()?
    .method_timeout(Duration::from_millis(250))
    .build()
    .ok()?;
    if session {
        lock_proxy(
            &connection,
            "org.gnome.ScreenSaver",
            "/org/gnome/ScreenSaver",
            "org.gnome.ScreenSaver",
        )
    } else {
        lock_proxy(
            &connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto",
            "org.freedesktop.login1.Session",
        )
    }
}

impl Default for Gate {
    fn default() -> Self {
        Self::new(false)
    }
}
impl Gate {
    pub fn new(enabled: bool) -> Self {
        Self(Arc::new(Mutex::new(State {
            view: Snapshot {
                enabled,
                locked: enabled,
                available: false,
                source: "unavailable",
                epoch: 1,
            },
            observed: None,
            checked: Instant::now(),
        })))
    }
    fn refresh(s: &mut State) {
        s.view.available = s.observed.is_some() && s.checked.elapsed() < Duration::from_secs(1);
        let blocked = s.view.enabled && (!s.view.available || s.observed != Some(false));
        if blocked != s.view.locked {
            s.view.epoch += 1;
            s.view.locked = blocked;
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let mut s = self.0.lock().unwrap();
        Self::refresh(&mut s);
        s.view.clone()
    }
    pub fn enable(&self, enabled: bool) {
        let mut s = self.0.lock().unwrap();
        if s.view.enabled != enabled {
            s.view.enabled = enabled;
            s.view.epoch += 1;
        }
        Self::refresh(&mut s);
    }
    pub fn observe(&self, value: Option<bool>, source: &'static str) {
        let mut s = self.0.lock().unwrap();
        s.observed = value;
        s.checked = Instant::now();
        s.view.source = source;
        Self::refresh(&mut s);
    }
    pub fn accepts(&self, epoch: u64) -> bool {
        let s = self.snapshot();
        !s.locked && s.epoch == epoch
    }
    pub fn monitor(&self) {
        let weak = Arc::downgrade(&self.0);
        std::thread::spawn(move || {
            let mut session: Option<zbus::blocking::Proxy<'static>> = None;
            let mut system: Option<zbus::blocking::Proxy<'static>> = None;
            let mut gnome_seen = false;
            while let Some(inner) = weak.upgrade() {
                let gate = Gate(inner);
                if session
                    .as_ref()
                    .is_some_and(|proxy| proxy.connection().is_closed())
                {
                    session = None;
                }
                if system
                    .as_ref()
                    .is_some_and(|proxy| proxy.connection().is_closed())
                {
                    system = None;
                }
                if session.is_none() {
                    session = connect_lock_proxy(true);
                }
                if system.is_none() {
                    system = connect_lock_proxy(false);
                }
                let gnome = session
                    .as_ref()
                    .and_then(|p| p.call::<_, _, bool>("GetActive", &()).ok());
                gnome_seen |= gnome.is_some();
                let logind = system.as_ref().and_then(|p| {
                    let kind = p.get_property::<String>("Type").ok()?;
                    if !matches!(kind.as_str(), "wayland" | "x11") {
                        return None;
                    }
                    p.get_property::<bool>("LockedHint").ok()
                });
                let (value, source) = if gnome_seen {
                    (gnome.map(|v| v || logind == Some(true)), "GNOME")
                } else {
                    (logind, "systemd-logind")
                };
                gate.observe(value, source);
                drop(gate);
                std::thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
pub fn paint(
    display: &mut impl decksmith_device::DeckDevice,
) -> Result<(), decksmith_device::DeviceError> {
    static FRAMES: std::sync::OnceLock<(Vec<u8>, Vec<u8>)> = std::sync::OnceLock::new();
    let (key, strip) = FRAMES.get_or_init(|| {
        let mut key = vec![0; 120 * 120 * 3];
        crate::key_text::draw(&mut key, "Locked", 1, false, false, [245, 190, 115]);
        (key, vec![0; 800 * 100 * 3])
    });
    display.set_key_image(0, key)?;
    let blank_key = &strip[..120 * 120 * 3];
    for i in 1..8 {
        display.set_key_image(i, blank_key)?;
    }
    display.set_touch_image(strip)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Probe(std::sync::Arc<std::sync::atomic::AtomicBool>);
    #[zbus::interface(name = "cc.senecal.Decksmith.LockProbe")]
    impl Probe {
        #[zbus(property)]
        fn locked_hint(&self) -> bool {
            self.0.load(std::sync::atomic::Ordering::Acquire)
        }
    }

    #[test]
    #[ignore = "requires an isolated dbus-run-session"]
    fn reusable_proxy_reads_unsignalled_lock_changes_and_recovers() {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();
        assert!(address.starts_with("unix:path=/tmp/dbus-"));
        let locked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let server = zbus::blocking::connection::Builder::session()
            .unwrap()
            .name("cc.senecal.Decksmith.LockProbe")
            .unwrap()
            .serve_at("/test/lock", Probe(locked.clone()))
            .unwrap()
            .build()
            .unwrap();
        let client = zbus::blocking::connection::Builder::session()
            .unwrap()
            .method_timeout(Duration::from_millis(250))
            .build()
            .unwrap();
        let proxy = lock_proxy(
            &client,
            "cc.senecal.Decksmith.LockProbe",
            "/test/lock",
            "cc.senecal.Decksmith.LockProbe",
        )
        .unwrap();
        let gate = Gate::new(true);
        gate.observe(proxy.get_property::<bool>("LockedHint").ok(), "test");
        assert!(!gate.snapshot().locked);
        // Deliberately emit no PropertiesChanged signal. The same proxy must
        // still fetch the current value, including the transition back to false.
        locked.store(true, std::sync::atomic::Ordering::Release);
        gate.observe(proxy.get_property::<bool>("LockedHint").ok(), "test");
        assert!(gate.snapshot().locked);
        locked.store(false, std::sync::atomic::Ordering::Release);
        gate.observe(proxy.get_property::<bool>("LockedHint").ok(), "test");
        assert!(!gate.snapshot().locked);
        server
            .object_server()
            .remove::<Probe, _>("/test/lock")
            .unwrap();
        gate.observe(proxy.get_property::<bool>("LockedHint").ok(), "test");
        assert!(gate.snapshot().locked);
        assert!(!gate.snapshot().available);
        server
            .object_server()
            .at("/test/lock", Probe(locked))
            .unwrap();
        gate.observe(proxy.get_property::<bool>("LockedHint").ok(), "test");
        assert!(!gate.snapshot().locked);
        assert!(gate.snapshot().available);
    }

    #[test]
    fn unknown_and_stale_never_unlock() {
        let gate = Gate::new(true);
        assert!(gate.snapshot().locked);
        gate.observe(Some(false), "test");
        assert!(!gate.snapshot().locked);
        gate.observe(Some(true), "test");
        let old = gate.snapshot().epoch;
        gate.observe(None, "test");
        assert!(gate.snapshot().locked);
        gate.observe(Some(false), "test");
        assert!(!gate.accepts(old));
        gate.0.lock().unwrap().checked = Instant::now() - Duration::from_secs(2);
        assert!(gate.snapshot().locked);
    }
    #[test]
    fn brief_lock_invalidates_prelock_requests() {
        let g = Gate::new(true);
        g.observe(Some(false), "test");
        let old = g.snapshot().epoch;
        g.observe(Some(true), "test");
        g.observe(Some(false), "test");
        assert!(!g.accepts(old));
        assert!(g.accepts(g.snapshot().epoch));
    }
}
