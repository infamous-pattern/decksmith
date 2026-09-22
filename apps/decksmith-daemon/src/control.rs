//! Versioned session-bus control API; all mutations cross the device worker queue.
use crate::pages::Pages;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SyncSender},
    },
    time::Duration,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub layout: String,
    pub brightness: Option<u8>,
}
#[derive(Serialize)]
pub struct Status {
    pub attention: Vec<crate::feedback::Notice>,
    pub api_version: u8,
    pub connected: bool,
    pub layout: String,
    pub brightness: Option<u8>,
    pub display_ready: bool,
    pub active_page: Option<u8>,
    #[serde(skip)]
    pub layout_json: String,
    #[serde(skip)]
    pub touch_frame: Option<Vec<u8>>,
    #[serde(skip)]
    pub touch_state: crate::pages::TouchState,
}
pub type Shared = Arc<Mutex<Status>>;
type Answer = SyncSender<Result<(), &'static str>>;
pub enum Command {
    TestControl {
        layout_json: String,
        page: u8,
        slot: u8,
        answer: Answer,
    },
    Page {
        page: u8,
        answer: Answer,
    },
    Brightness {
        percent: u8,
        answer: Answer,
    },
    Layout {
        pages: Box<Pages>,
        name: String,
        answer: Answer,
    },
}
pub struct Context {
    pub foreground: crate::foreground::Foreground,
    pub gate: crate::session_lock::Gate,
    pub incoming: Receiver<(Command, u64, u64)>,
    pub status: Shared,
}
fn path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config"));
    base.join("decksmith/control-panel.json")
}
fn preset(name: &str) -> Result<Pages, &'static str> {
    if name == "custom" {
        use std::io::Read;
        let file = std::fs::File::open(path().with_file_name("layout.json"))
            .map_err(|_| "custom_layout_not_saved")?;
        let mut bytes = Vec::new();
        file.take(1048577)
            .read_to_end(&mut bytes)
            .map_err(|_| "layout_read_failed")?;
        return Pages::parse(&bytes);
    }
    let file = match name {
        "audio" => "audio.json",
        "navigation" => "navigation.json",
        _ => return Err("unknown_layout"),
    };
    let bytes =
        std::fs::read(crate::runtime_paths::config(file)).map_err(|_| "layout_read_failed")?;
    Pages::parse(&bytes)
}
pub fn startup(default_path: &str) -> Result<(Pages, Settings), &'static str> {
    let settings = match std::fs::read(path()) {
        Ok(bytes) if bytes.len() <= 4096 => {
            serde_json::from_slice::<Settings>(&bytes).map_err(|_| "invalid_settings")?
        }
        Ok(_) => return Err("invalid_settings"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Settings {
            layout: if default_path.ends_with("navigation.json") {
                "navigation"
            } else {
                "audio"
            }
            .into(),
            brightness: None,
        },
        Err(_) => return Err("settings_read_failed"),
    };
    if settings.brightness.is_some_and(|n| n > 100) {
        return Err("invalid_settings");
    }
    let pages = preset(&settings.layout)?;
    Ok((pages, settings))
}
static SAVE_LOCK: Mutex<()> = Mutex::new(());
pub(crate) fn save_shared(shared: &Shared) -> zbus::fdo::Result<()> {
    let _guard = SAVE_LOCK.lock().unwrap();
    let status = shared.lock().unwrap();
    let settings = Settings {
        layout: status.layout.clone(),
        brightness: status.brightness,
    };
    drop(status);
    let path = path();
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(path.parent().unwrap())?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(&settings)?)?;
        std::fs::rename(temporary, path)?;
        Ok(())
    })();
    result.map_err(|_| zbus::fdo::Error::Failed("Applied, but settings could not be saved".into()))
}
pub fn persist_later(shared: Shared) {
    static SAVER: std::sync::OnceLock<SyncSender<Shared>> = std::sync::OnceLock::new();
    let sender = SAVER.get_or_init(|| {
        let (sender, receiver) = mpsc::sync_channel::<Shared>(1);
        std::thread::spawn(move || {
            while let Ok(shared) = receiver.recv() {
                if save_shared(&shared).is_err() {
                    tracing::warn!("Brightness preference could not be saved");
                }
            }
        });
        sender
    });
    let _ = sender.try_send(shared);
}
struct Api {
    foreground: crate::foreground::Foreground,
    gate: crate::session_lock::Gate,
    preview: Mutex<Option<(String, String, Pages)>>,
    queue: SyncSender<(Command, u64, u64)>,
    status: Shared,
}
impl Api {
    fn wait(&self, build: impl FnOnce(Answer) -> Command) -> zbus::fdo::Result<()> {
        if self.gate.snapshot().locked {
            return Err(zbus::fdo::Error::Failed("Session is locked".into()));
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.queue
            .try_send((
                build(tx),
                self.gate.snapshot().epoch,
                self.foreground.snapshot().revision,
            ))
            .map_err(|_| zbus::fdo::Error::Failed("Control queue unavailable".into()))?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| zbus::fdo::Error::Failed("Control request timed out".into()))?
            .map_err(|error| zbus::fdo::Error::Failed(error.into()))
    }
    fn save(&self) -> zbus::fdo::Result<()> {
        save_shared(&self.status)
    }
}
#[zbus::interface(name = "cc.senecal.Decksmith.Control1")]
impl Api {
    fn get_status(&self) -> String {
        let mut value = serde_json::to_value(&*self.status.lock().unwrap()).unwrap();
        value["auto_lock"] = serde_json::to_value(self.gate.snapshot()).unwrap();
        value["automatic_pages"] = serde_json::to_value(self.foreground.snapshot()).unwrap();
        value.to_string()
    }
    fn report_foreground(&self, desktop_id: &str) -> zbus::fdo::Result<()> {
        self.foreground
            .report(desktop_id)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e.into()))
    }
    fn get_layout(&self) -> String {
        self.status.lock().unwrap().layout_json.clone()
    }
    fn preview_keys(&self, layout_json: &str, page: u8) -> zbus::fdo::Result<Vec<u8>> {
        if layout_json.len() > 1_048_576 {
            return Err(zbus::fdo::Error::InvalidArgs("Layout too large".into()));
        }
        let mut cached = self.preview.lock().unwrap();
        if cached.as_ref().is_none_or(|(raw, _, _)| raw != layout_json) {
            let pages = Pages::parse(layout_json.as_bytes())
                .map_err(|e| zbus::fdo::Error::InvalidArgs(e.into()))?;
            *cached = Some((layout_json.into(), pages.json(), pages));
        }
        let status = self.status.lock().unwrap();
        let mut state = status.touch_state.clone();
        if !status.connected {
            state.targets.clear();
        }
        if !status.connected
            || status.active_page != Some(page)
            || cached.as_ref().unwrap().1 != status.layout_json
        {
            state.feedback.clear();
        }
        drop(status);
        cached
            .as_mut()
            .unwrap()
            .2
            .preview_keys(page, state)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e.into()))
    }
    fn preview_touch(&self, layout_json: &str, page: u8) -> zbus::fdo::Result<(Vec<u8>, String)> {
        if layout_json.len() > 1_048_576 {
            return Err(zbus::fdo::Error::InvalidArgs("Layout too large".into()));
        }
        {
            let status = self.status.lock().unwrap();
            if status.connected
                && status.active_page == Some(page)
                && layout_json == status.layout_json
                && let Some(frame) = &status.touch_frame
            {
                return Ok((frame.clone(), "live".into()));
            }
        }
        let mut cached = self.preview.lock().unwrap();
        if cached.as_ref().is_none_or(|(raw, _, _)| raw != layout_json) {
            let pages = Pages::parse(layout_json.as_bytes())
                .map_err(|error| zbus::fdo::Error::InvalidArgs(error.into()))?;
            *cached = Some((layout_json.into(), pages.json(), pages));
        }
        let (_, normalized, pages) = cached.as_mut().unwrap();
        let (mut state, connected) = {
            let status = self.status.lock().unwrap();
            if status.connected
                && status.active_page == Some(page)
                && *normalized == status.layout_json
                && let Some(frame) = &status.touch_frame
            {
                return Ok((frame.clone(), "live".into()));
            }
            (status.touch_state.clone(), status.connected)
        };
        state.feedback.clear(); // Drafts never inherit saved-layout action warnings.
        if !connected {
            state.audio = Some(None);
            state.targets.clear();
            state.levels.clear();
        }
        let missing = pages
            .audio_targets()
            .into_iter()
            .any(|target| !state.targets.iter().any(|(known, _)| known == &target));
        let frame = pages
            .preview_touch(page, state)
            .map_err(|error| zbus::fdo::Error::InvalidArgs(error.into()))?;
        Ok((
            frame,
            if !connected {
                "offline"
            } else if missing {
                "draft-missing-target"
            } else {
                "draft"
            }
            .into(),
        ))
    }
    fn get_previous_layout(&self) -> zbus::fdo::Result<String> {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path().with_file_name("layout.previous.json"))
            .and_then(|file| file.take(1048577).read_to_end(&mut bytes))
            .map_err(|_| zbus::fdo::Error::Failed("No previous layout is available".into()))?;
        Pages::parse(&bytes)
            .map(|pages| pages.json())
            .map_err(|error| zbus::fdo::Error::Failed(error.into()))
    }
    fn validate_layout(&self, layout_json: &str) -> zbus::fdo::Result<String> {
        Pages::parse(layout_json.as_bytes())
            .map(|pages| pages.json())
            .map_err(|error| zbus::fdo::Error::InvalidArgs(error.into()))
    }
    fn save_layout(&mut self, layout_json: &str) -> zbus::fdo::Result<()> {
        if self.gate.snapshot().locked {
            return Err(zbus::fdo::Error::Failed("Session is locked".into()));
        }
        let pages = Pages::parse(layout_json.as_bytes())
            .map_err(|error| zbus::fdo::Error::InvalidArgs(error.into()))?;
        let file = path().with_file_name("layout.json");
        let temporary = file.with_extension("json.tmp");
        if file.exists() {
            let backup = file.with_file_name("layout.previous.json");
            let staged = backup.with_extension("json.tmp");
            std::fs::copy(&file, &staged)
                .and_then(|_| std::fs::rename(&staged, &backup))
                .map_err(|_| {
                    zbus::fdo::Error::Failed(
                        "Could not back up existing layout; nothing changed".into(),
                    )
                })?;
        }
        std::fs::create_dir_all(file.parent().unwrap())
            .and_then(|_| std::fs::write(&temporary, pages.json()))
            .and_then(|_| std::fs::rename(&temporary, &file))
            .map_err(|_| zbus::fdo::Error::Failed("Could not save custom layout".into()))?;
        self.wait(|answer| Command::Layout {
            pages: Box::new(pages),
            name: "custom".into(),
            answer,
        })?;
        self.save()
    }
    fn set_auto_lock(&mut self, enabled: bool) -> zbus::fdo::Result<()> {
        let path = path().with_file_name("auto-lock.json");
        std::fs::create_dir_all(path.parent().unwrap())
            .and_then(|_| {
                std::fs::write(
                    path.with_extension("json.tmp"),
                    if enabled { "true" } else { "false" },
                )
            })
            .and_then(|_| std::fs::rename(path.with_extension("json.tmp"), &path))
            .map_err(|_| zbus::fdo::Error::Failed("Could not save Auto-Lock preference".into()))?;
        self.gate.enable(enabled);
        Ok(())
    }
    fn set_brightness(&mut self, percent: u8) -> zbus::fdo::Result<()> {
        if percent > 100 {
            return Err(zbus::fdo::Error::InvalidArgs(
                "Brightness must be 0 through 100".into(),
            ));
        }
        self.wait(|answer| Command::Brightness { percent, answer })?;
        self.save()
    }
    /// Explicit editor testing of a saved control; never accepts an arbitrary action.
    fn test_control(&mut self, layout_json: &str, page: u8, slot: u8) -> zbus::fdo::Result<()> {
        if slot >= 12 || layout_json.len() > 1_048_576 {
            return Err(zbus::fdo::Error::InvalidArgs("Invalid control".into()));
        }
        let normalized = Pages::parse(layout_json.as_bytes())
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e.into()))?
            .json();
        self.wait(|answer| Command::TestControl {
            layout_json: normalized,
            page,
            slot,
            answer,
        })
    }
    fn show_page(&mut self, page: u8) -> zbus::fdo::Result<()> {
        self.wait(|answer| Command::Page { page, answer })
    }
    fn set_layout(&mut self, name: &str) -> zbus::fdo::Result<()> {
        let pages = preset(name).map_err(|error| zbus::fdo::Error::InvalidArgs(error.into()))?;
        self.wait(|answer| Command::Layout {
            pages: Box::new(pages),
            name: name.into(),
            answer,
        })?;
        self.save()
    }
}
pub fn start(
    settings: Settings,
    layout_json: String,
) -> Result<(zbus::blocking::Connection, Context), &'static str> {
    let status = Arc::new(Mutex::new(Status {
        api_version: 1,
        connected: false,
        layout: settings.layout,
        brightness: settings.brightness,
        display_ready: false,
        active_page: None,
        layout_json,
        touch_frame: None,
        touch_state: Default::default(),
        attention: Vec::new(),
    }));
    let enabled = match std::fs::read(path().with_file_name("auto-lock.json")) {
        Ok(bytes) => serde_json::from_slice::<bool>(&bytes).unwrap_or(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => true,
    };
    let gate = crate::session_lock::Gate::new(enabled);
    gate.monitor();
    let foreground = crate::foreground::Foreground::default();
    foreground.monitor();
    let (queue, incoming) = mpsc::sync_channel(16);
    let connection = zbus::blocking::connection::Builder::session()
        .map_err(|_| "bus_unavailable")?
        .name("cc.senecal.Decksmith")
        .map_err(|_| "bus_name_failed")?
        .serve_at(
            "/cc/senecal/Decksmith",
            Api {
                foreground: foreground.clone(),
                gate: gate.clone(),
                preview: Mutex::new(None),
                queue,
                status: status.clone(),
            },
        )
        .map_err(|_| "bus_interface_failed")?
        .build()
        .map_err(|_| "bus_registration_failed")?;
    Ok((
        connection,
        Context {
            foreground,
            incoming,
            status,
            gate,
        },
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn touch_preview_returns_sent_bytes_without_queuing_a_device_action() {
        let pages = preset("audio").unwrap();
        let json = pages.json();
        let frame = vec![81; 800 * 100 * 3];
        let status = Arc::new(Mutex::new(Status {
            api_version: 1,
            connected: true,
            layout: "audio".into(),
            brightness: Some(50),
            display_ready: true,
            active_page: Some(0),
            layout_json: json.clone(),
            touch_frame: Some(frame.clone()),
            touch_state: Default::default(),
            attention: Vec::new(),
        }));
        let (queue, incoming) = mpsc::sync_channel(1);
        let api = Api {
            foreground: Default::default(),
            gate: Default::default(),
            preview: Mutex::new(None),
            queue,
            status: status.clone(),
        };
        assert_eq!(api.preview_touch(&json, 0).unwrap(), (frame, "live".into()));
        assert!(incoming.try_recv().is_err());
        status.lock().unwrap().connected = false;
        let (frame, mode) = api.preview_touch(&json, 0).unwrap();
        assert_eq!(mode, "offline");
        assert_eq!(frame.len(), 800 * 100 * 3);
        assert!(api.preview_touch(&json, 255).is_err());
        assert!(api.preview_touch("{}", 0).is_err());
    }

    #[test]
    fn unknown_layout_cannot_be_used_as_a_file_path() {
        assert!(preset("../../private").is_err());
        assert!(preset("audio").is_ok());
    }
}
