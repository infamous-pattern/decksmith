//! Latest application identity only. No titles, focus polling, or manual override timer.
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
pub const SOURCE: &str = "cc.senecal.Decksmith.Foreground";
pub const STUDIO: &str = "cc.senecal.Decksmith.Studio.desktop";
#[derive(Clone, Default, Serialize)]
pub struct Snapshot {
    pub available: bool,
    pub application: Option<String>,
    pub revision: u64,
}
#[derive(Clone, Default)]
pub struct Foreground(Arc<Mutex<Snapshot>>);
impl Foreground {
    pub fn snapshot(&self) -> Snapshot {
        self.0.lock().unwrap().clone()
    }
    pub fn report(&self, id: &str) -> Result<(), &'static str> {
        if !id.is_empty() && !crate::launch::valid_application(id) {
            return Err("invalid_application_identity");
        }
        let mut s = self.0.lock().unwrap();
        let app = if id.is_empty() { None } else { Some(id.into()) };
        if !s.available || s.application != app {
            s.revision += 1;
            s.application = app;
        }
        s.available = true;
        Ok(())
    }
    pub fn unavailable(&self) {
        let mut s = self.0.lock().unwrap();
        if s.available {
            s.available = false;
            s.application = None;
            s.revision += 1;
        }
    }
    pub fn monitor(&self) {
        let weak = Arc::downgrade(&self.0);
        std::thread::spawn(move || {
            while let Some(inner) = weak.upgrade() {
                let present = zbus::blocking::connection::Builder::session()
                    .ok()
                    .and_then(|b| b.method_timeout(Duration::from_millis(250)).build().ok())
                    .and_then(|c| {
                        zbus::blocking::Proxy::new(
                            &c,
                            "org.freedesktop.DBus",
                            "/org/freedesktop/DBus",
                            "org.freedesktop.DBus",
                        )
                        .ok()?
                        .call::<_, _, bool>("NameHasOwner", &(SOURCE,))
                        .ok()
                    })
                    .unwrap_or(false);
                let context = Foreground(inner);
                if !present {
                    context.unavailable();
                }
                drop(context);
                // Source liveness only; application changes arrive as events.
                std::thread::sleep(Duration::from_secs(2));
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_changes_only_and_missing_source_retains_no_stale_target() {
        let f = Foreground::default();
        f.report("brave-browser.desktop").unwrap();
        let n = f.snapshot().revision;
        f.report("brave-browser.desktop").unwrap();
        assert_eq!(f.snapshot().revision, n);
        f.report("teams.desktop").unwrap();
        assert_eq!(f.snapshot().revision, n + 1);
        f.unavailable();
        assert!(!f.snapshot().available);
        assert!(f.snapshot().application.is_none());
        assert!(f.report("window title").is_err());
    }
}
