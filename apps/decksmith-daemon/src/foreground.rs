//! Latest application identity only. No titles, focus polling, or manual override timer.
use serde::Serialize;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
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
        if let Some(dir) = hyprland_dir() {
            std::thread::spawn(move || hyprland(weak, dir));
            return;
        }
        std::thread::spawn(move || {
            while let Some(inner) = weak.upgrade() {
                let present = futures_lite::future::block_on(async {
                    let c = zbus::connection::Builder::session()
                        .method_timeout(Duration::from_millis(250))
                        .build()
                        .await
                        .ok()?;
                    let proxy = zbus::Proxy::new(
                        &c,
                        "org.freedesktop.DBus",
                        "/org/freedesktop/DBus",
                        "org.freedesktop.DBus",
                    )
                    .await
                    .ok()?;
                    proxy
                        .call::<_, _, bool>("NameHasOwner", &(SOURCE,))
                        .await
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
fn hyprland_dir() -> Option<PathBuf> {
    let signature = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
    Some(PathBuf::from(runtime).join("hypr").join(signature))
}
/// Hyprland reports focus on its event socket; no extension or D-Bus source.
fn hyprland(weak: std::sync::Weak<Mutex<Snapshot>>, dir: PathBuf) {
    while weak.strong_count() > 0 {
        if let Ok(events) = UnixStream::connect(dir.join(".socket2.sock")) {
            let _ = events.set_read_timeout(Some(Duration::from_secs(2)));
            let class = active_class(&dir).unwrap_or_default();
            let report = |class: &str| match weak.upgrade() {
                Some(inner) => {
                    // Desktop entries can be added, removed or edited while the
                    // daemon is running. Resolve on focus events so both misses
                    // and previously successful mappings can recover.
                    report_class(&Foreground(inner), class, &application_dirs());
                    true
                }
                None => false,
            };
            if !report(&class) {
                return;
            }
            let mut reader = BufReader::new(events);
            let mut line = String::new();
            loop {
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let class = event_class(&line);
                        if class.is_some_and(|c| !report(c)) {
                            return;
                        }
                        line.clear();
                    }
                    // Timeouts keep partial lines and only check for shutdown.
                    Err(e) if timed_out(&e) => {
                        if weak.strong_count() == 0 {
                            return;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        match weak.upgrade() {
            Some(inner) => Foreground(inner).unavailable(),
            None => return,
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}
fn report_class(foreground: &Foreground, class: &str, dirs: &[PathBuf]) {
    let _ = foreground.report(&desktop_id(class, dirs));
}
/// `activewindow>>CLASS,TITLE` -> CLASS.
fn event_class(line: &str) -> Option<&str> {
    let event = line.trim_end().strip_prefix("activewindow>>")?;
    Some(event.split_once(',').map_or(event, |(class, _)| class))
}
fn timed_out(e: &std::io::Error) -> bool {
    matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
fn active_class(dir: &Path) -> Option<String> {
    let mut s = UnixStream::connect(dir.join(".socket.sock")).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(1))).ok()?;
    s.write_all(b"j/activewindow").ok()?;
    let mut reply = String::new();
    s.read_to_string(&mut reply).ok()?;
    let value: serde_json::Value = serde_json::from_str(&reply).ok()?;
    Some(value["class"].as_str()?.to_owned())
}
fn application_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            let home = PathBuf::from(std::env::var_os("HOME")?);
            Some(home.join(".local/share"))
        });
    let system = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    home.into_iter()
        .chain(system.split(':').map(PathBuf::from))
        .map(|d| d.join("applications"))
        .collect()
}
/// Maps a window class to its desktop entry: `<class>.desktop`, then
/// StartupWMClass. shortcut: top-level entries only, add subdirectory
/// ids if needed.
fn desktop_id(class: &str, dirs: &[PathBuf]) -> String {
    if class.is_empty() {
        return String::new();
    }
    for name in [class.to_owned(), class.to_lowercase()] {
        let id = format!("{name}.desktop");
        let installed = dirs.iter().any(|d| d.join(&id).is_file());
        if crate::launch::valid_application(&id) && installed {
            return id;
        }
    }
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let id = entry.file_name().to_string_lossy().into_owned();
            if !crate::launch::valid_application(&id) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            if text.lines().any(|l| {
                l.strip_prefix("StartupWMClass=")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(class))
            }) {
                return id;
            }
        }
    }
    String::new()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_resolves_entries_again_after_install_edit_and_removal() {
        let root =
            std::env::temp_dir().join(format!("decksmith-focus-refresh-{}", std::process::id()));
        let apps = root.join("applications");
        std::fs::create_dir_all(&apps).unwrap();
        let dirs = [apps.clone()];
        let f = Foreground::default();
        report_class(&f, "DynamicApp", &dirs);
        assert_eq!(f.snapshot().application, None);
        let entry = apps.join("dynamic.desktop");
        std::fs::write(&entry, "[Desktop Entry]\nStartupWMClass=DynamicApp\n").unwrap();
        report_class(&f, "DynamicApp", &dirs);
        assert_eq!(f.snapshot().application.as_deref(), Some("dynamic.desktop"));
        std::fs::write(&entry, "[Desktop Entry]\nStartupWMClass=Other\n").unwrap();
        report_class(&f, "DynamicApp", &dirs);
        assert_eq!(f.snapshot().application, None);
        std::fs::write(&entry, "[Desktop Entry]\nStartupWMClass=DynamicApp\n").unwrap();
        report_class(&f, "DynamicApp", &dirs);
        std::fs::remove_file(&entry).unwrap();
        report_class(&f, "DynamicApp", &dirs);
        assert_eq!(f.snapshot().application, None);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn hyprland_event_line_yields_window_class() {
        let line = "activewindow>>google-chrome,Inbox, mail\n";
        assert_eq!(event_class(line), Some("google-chrome"));
        assert_eq!(event_class("activewindow>>,\n"), Some(""));
        assert_eq!(event_class("workspace>>2\n"), None);
    }
    #[test]
    fn hyprland_class_maps_to_desktop_entry() {
        let name = format!("decksmith-apps-{}", std::process::id());
        let root = std::env::temp_dir().join(name);
        let apps = root.join("applications");
        std::fs::create_dir_all(&apps).unwrap();
        let entry = "[Desktop Entry]\n";
        std::fs::write(apps.join("brave-browser.desktop"), entry).unwrap();
        std::fs::write(
            apps.join("code.desktop"),
            "[Desktop Entry]\nStartupWMClass=Code\n",
        )
        .unwrap();
        let dirs = [apps];
        assert_eq!(desktop_id("brave-browser", &dirs), "brave-browser.desktop");
        assert_eq!(desktop_id("Brave-browser", &dirs), "brave-browser.desktop");
        assert_eq!(desktop_id("Code", &dirs), "code.desktop");
        assert_eq!(desktop_id("unknown", &dirs), "");
        assert_eq!(desktop_id("", &dirs), "");
        std::fs::remove_dir_all(root).unwrap();
    }
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
