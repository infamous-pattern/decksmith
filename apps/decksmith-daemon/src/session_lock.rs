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
            let mut session = None;
            let mut system = None;
            let mut gnome_seen = false;
            while let Some(inner) = weak.upgrade() {
                let gate = Gate(inner);
                if session
                    .as_ref()
                    .is_some_and(zbus::blocking::Connection::is_closed)
                {
                    session = None;
                }
                if system
                    .as_ref()
                    .is_some_and(zbus::blocking::Connection::is_closed)
                {
                    system = None;
                }
                if session.is_none() {
                    session = zbus::blocking::connection::Builder::session()
                        .ok()
                        .and_then(|b| b.method_timeout(Duration::from_millis(250)).build().ok());
                }
                if system.is_none() {
                    system = zbus::blocking::connection::Builder::system()
                        .ok()
                        .and_then(|b| b.method_timeout(Duration::from_millis(250)).build().ok());
                }
                let gnome = session.as_ref().and_then(|c| {
                    zbus::blocking::Proxy::new(
                        c,
                        "org.gnome.ScreenSaver",
                        "/org/gnome/ScreenSaver",
                        "org.gnome.ScreenSaver",
                    )
                    .ok()?
                    .call::<_, _, bool>("GetActive", &())
                    .ok()
                });
                gnome_seen |= gnome.is_some();
                let logind = system.as_ref().and_then(|c| {
                    let p = zbus::blocking::Proxy::new(
                        c,
                        "org.freedesktop.login1",
                        "/org/freedesktop/login1/session/auto",
                        "org.freedesktop.login1.Session",
                    )
                    .ok()?;
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
