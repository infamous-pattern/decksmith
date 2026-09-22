//! Bounded read-only target checks and compact, actionable saved-layout feedback.
use crate::pages::Action;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Check {
    pub page: u8,
    pub slot: u8,
    pub label: String,
    pub kind: String,
    pub target: String,
    pub command: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Notice {
    pub check: Check,
    pub target_name: String,
    pub status: String,
    pub detail: String,
    pub hint: String,
    pub short: String,
    pub next_step: String,
}
impl Notice {
    pub fn attention(&self) -> bool {
        self.status != "available"
    }
}
pub fn check(page: u8, slot: u8, label: &str, action: &Action) -> Option<Check> {
    let (kind, target, command) = match action {
        Action::System { command } => ("desktop", command.as_str(), ""),
        Action::AudioAdjust { target, .. } => ("audio", target.as_str(), "volume"),
        Action::AudioMute { target } | Action::PushToTalk { target } => {
            ("audio", target.as_str(), "mute")
        }
        Action::AudioSelect { target } => ("audio", target.as_str(), "select"),
        Action::VolumeUp | Action::VolumeDown | Action::VolumeAdjust { .. } => {
            ("audio", "system", "volume")
        }
        Action::MuteToggle => ("audio", "system", "mute"),
        Action::MediaTarget { player, command } => ("media", player.as_str(), command.as_str()),
        Action::MediaPlayPause => ("media", "automatic", "play_pause"),
        Action::MediaNext => ("media", "automatic", "next"),
        Action::MediaPrevious => ("media", "automatic", "previous"),
        Action::OpenApplication { desktop_id } => ("application", desktop_id.as_str(), ""),
        _ => return None,
    };
    Some(Check {
        page,
        slot,
        label: label.into(),
        kind: kind.into(),
        target: target.into(),
        command: command.into(),
    })
}
pub fn target_name(check: &Check) -> String {
    match check.target.as_str() {
        "system" => "System sounds".into(),
        "microphone" => "Default microphone".into(),
        "automatic" => "Media player".into(),
        target if target.starts_with("app:") => target
            .split_once('=')
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| check.label.clone()),
        target if target.starts_with("org.mpris.MediaPlayer2.") => {
            target.trim_start_matches("org.mpris.MediaPlayer2.").into()
        }
        _ => check.label.clone(),
    }
}
pub fn failed(check: Check, name: String, code: &str) -> Notice {
    let (detail, hint, short, next) = match code {
        "ptt_guard_failed" => (
            "Push-to-talk could not be maintained",
            "Release the control, check the microphone, then press again.",
            "Hold failed",
            "Release / retry",
        ),
        "media_unavailable" => (
            "No matching media player is running",
            "Open the selected player and load media, then try again.",
            "No player",
            "Open player",
        ),
        "media_unsupported" => (
            "The selected player does not support this action",
            "Load a playlist or select another player, then retry.",
            "Unsupported",
            "Check player",
        ),
        "application_unavailable" => (
            "The application is unavailable",
            "Install the app or select another application.",
            "App missing",
            "Edit app",
        ),
        "audio_target_unsupported" => (
            "This target does not expose the required control",
            "Choose another audio target or check its settings.",
            "Unsupported",
            "Edit target",
        ),
        "audio_target_unavailable" => (
            "The audio target could not be controlled",
            "Start app playback or reconnect the selected device, then retry.",
            "Unavailable",
            "Check target",
        ),
        "audio_queue_full" => (
            "Controls are busy; this action was not queued",
            "Wait a moment, then try again.",
            "Busy",
            "Retry",
        ),
        "audio_worker_stopped" => (
            "The action worker stopped",
            "Restart Background controls in Decksmith.",
            "Stopped",
            "Restart controls",
        ),
        "brightness_not_set" => (
            "No brightness level is set yet",
            "Set a brightness level in the Decksmith panel first.",
            "Not set",
            "Set brightness",
        ),
        code if code.ends_with("timeout") => (
            "The action timed out; completion is unknown",
            "Check the target before retrying; a partial change may have occurred.",
            "Timed out",
            "Check target",
        ),
        _ => (
            "The action failed; a partial change may have occurred",
            "Check the target's settings, then retry. No success has been confirmed.",
            "Failed",
            "Check target",
        ),
    };
    Notice {
        check,
        target_name: name,
        status: "failed".into(),
        detail: detail.into(),
        hint: hint.into(),
        short: short.into(),
        next_step: next.into(),
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Job {
    session: u64,
    checks: Vec<Check>,
}
#[derive(Default)]
struct Shared {
    job: Option<Job>,
    result: Option<(Job, Vec<Notice>)>,
    stop: bool,
}
pub struct Worker {
    shared: Arc<Mutex<Shared>>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start() -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let data = shared.clone();
        let thread = std::thread::spawn(move || {
            let mut client = Client::default();
            let mut previous = None;
            let mut checked = Instant::now() - Duration::from_secs(2);
            loop {
                let job = {
                    let state = data.lock().unwrap();
                    if state.stop {
                        break;
                    }
                    state.job.clone()
                };
                if let Some(job) = job
                    && (previous.as_ref() != Some(&job)
                        || checked.elapsed() >= Duration::from_secs(1))
                {
                    let notices = client.inspect(&job.checks, &data);
                    checked = Instant::now();
                    previous = Some(job.clone());
                    data.lock().unwrap().result = Some((job, notices));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        Self {
            shared,
            thread: Some(thread),
        }
    }
    pub fn poll(&self, session: u64, checks: Vec<Check>) -> Option<Vec<Notice>> {
        let mut state = self.shared.lock().unwrap();
        state.job = if session == 0 {
            None
        } else {
            Some(Job { session, checks })
        };
        state
            .result
            .as_ref()
            .filter(|(job, _)| Some(job) == state.job.as_ref())
            .map(|(_, n)| n.clone())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.lock().unwrap().stop = true;
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
struct Reader {
    child: std::process::Child,
    input: std::sync::mpsc::SyncSender<Vec<u8>>,
    output: std::sync::mpsc::Receiver<Vec<u8>>,
}
impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[derive(Default)]
struct Client {
    reader: Option<Reader>,
}
impl Client {
    fn request(&mut self, checks: &[Check], shared: &Arc<Mutex<Shared>>) -> Option<Vec<Notice>> {
        use std::io::{BufRead, BufReader, Write};
        if self.reader.is_none() {
            let mut child = Command::new("/usr/bin/python3")
                .arg("-B")
                .arg(crate::runtime_paths::helper("control_health.py"))
                .arg("serve")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let mut stdin = child.stdin.take()?;
            let stdout = child.stdout.take()?;
            let (input, requests) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
            std::thread::spawn(move || {
                while let Ok(raw) = requests.recv() {
                    if stdin.write_all(&raw).and_then(|_| stdin.flush()).is_err() {
                        break;
                    }
                }
            });
            let (results, output) = std::sync::mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut raw = Vec::new();
                    if reader
                        .by_ref()
                        .take(65537)
                        .read_until(b'\n', &mut raw)
                        .is_err()
                        || raw.len() > 65536
                        || raw.last() != Some(&b'\n')
                    {
                        break;
                    }
                    if results.send(raw).is_err() {
                        break;
                    }
                }
            });
            self.reader = Some(Reader {
                child,
                input,
                output,
            });
        }
        let mut raw = serde_json::to_vec(checks).ok()?;
        if raw.len() >= 65536 {
            return None;
        }
        raw.push(b'\n');
        let reader = self.reader.as_ref()?;
        reader.input.try_send(raw).ok()?;
        let deadline = Instant::now() + Duration::from_secs(4);
        while !shared.lock().unwrap().stop && Instant::now() < deadline {
            match reader.output.recv_timeout(Duration::from_millis(50)) {
                Ok(raw) => {
                    let notices: Vec<Notice> = serde_json::from_slice(&raw).ok()?;
                    if notices.len() != checks.len()
                        || notices.iter().zip(checks).any(|(n, c)| &n.check != c)
                    {
                        return None;
                    }
                    return Some(notices);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
                Err(_) => return None,
            }
        }
        None
    }
    fn inspect(&mut self, checks: &[Check], shared: &Arc<Mutex<Shared>>) -> Vec<Notice> {
        if checks.is_empty() {
            self.reader = None;
            return vec![];
        }
        let result = self.request(checks, shared);
        // A failed or late response must never be consumed by a subsequent request.
        if result.is_none() {
            self.reader = None;
        }
        result.unwrap_or_else(|| {
            checks
                .iter()
                .cloned()
                .map(|check| Notice {
                    target_name: target_name(&check),
                    check,
                    status: "unknown".into(),
                    detail: "Target status could not be checked".into(),
                    hint: "Check desktop audio/media settings; automatic retry is running.".into(),
                    short: "Check target".into(),
                    next_step: "Retrying".into(),
                })
                .collect()
        })
    }
}
pub fn badge(rgb: &mut [u8], failed: bool) {
    let color = if failed {
        [255, 85, 75]
    } else {
        [255, 190, 65]
    };
    for y in 0..120 {
        for x in 0..120 {
            if !(3..117).contains(&x) || !(3..117).contains(&y) {
                rgb[(y * 120 + x) * 3..(y * 120 + x) * 3 + 3].copy_from_slice(&color);
            }
        }
    }
    if failed {
        return;
    } // The full failure card already names the error in text.
    for y in 7..28 {
        for x in 96..114 {
            let i = (y * 120 + x) * 3;
            rgb[i..i + 3].copy_from_slice(&[24, 24, 24]);
        }
    }
    for y in 10..25 {
        if y == 19 || y == 20 {
            continue;
        }
        for x in 103..107 {
            let i = (y * 120 + x) * 3;
            rgb[i..i + 3].copy_from_slice(&color);
        }
    }
}

pub fn key_failure(rgb: &mut [u8], notice: &Notice) {
    for pixel in rgb.chunks_exact_mut(3) {
        pixel.copy_from_slice(&[24, 27, 32]);
    }
    let target: String = notice.target_name.chars().take(24).collect();
    for (label, anchor, scale, color) in [
        (&target, 0, 0.68, [240, 240, 240]),
        (&notice.short, 1, 0.64, [255, 100, 80]),
        (&notice.next_step, 2, 0.48, [240, 240, 240]),
    ] {
        crate::key_text::draw_styled(
            rgb,
            label,
            anchor,
            true,
            false,
            color,
            crate::key_text::Typography { font: 0, scale },
        );
    }
    badge(rgb, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture_client(raw: Vec<u8>) -> Client {
        let child = Command::new("/usr/bin/python3")
            .args(["-c", "import time; time.sleep(30)"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let (input, requests) = std::sync::mpsc::sync_channel(2);
        let (results, output) = std::sync::mpsc::sync_channel(2);
        std::thread::spawn(move || {
            while requests.recv().is_ok() {
                if results.send(raw.clone()).is_err() {
                    break;
                }
            }
        });
        Client {
            reader: Some(Reader {
                child,
                input,
                output,
            }),
        }
    }
    #[test]
    fn persistent_health_reader_reuses_process_and_rejects_mismatched_results() {
        let check = check(0, 0, "Volume", &Action::VolumeUp).unwrap();
        let notice = failed(check.clone(), "Volume".into(), "audio_target_failed");
        let shared = Arc::new(Mutex::new(Shared::default()));
        let mut client = fixture_client(serde_json::to_vec(&vec![notice]).unwrap());
        let pid = client.reader.as_ref().unwrap().child.id();
        for _ in 0..2 {
            assert_eq!(
                client.inspect(std::slice::from_ref(&check), &shared).len(),
                1
            );
            assert_eq!(client.reader.as_ref().unwrap().child.id(), pid);
        }
        let mut client = fixture_client(b"[]".to_vec());
        assert_eq!(client.inspect(&[check], &shared)[0].status, "unknown");
        assert!(client.reader.is_none());
    }
    #[test]
    fn stopped_health_reader_is_discarded_without_waiting_for_timeout() {
        let shared = Arc::new(Mutex::new(Shared {
            stop: true,
            ..Default::default()
        }));
        let mut client = fixture_client(b"[]".to_vec());
        let check = check(0, 0, "Volume", &Action::VolumeUp).unwrap();
        assert_eq!(client.inspect(&[check], &shared)[0].status, "unknown");
        assert!(client.reader.is_none());
    }
    #[test]
    fn stale_probe_results_never_cross_session_page_or_binding_changes() {
        let c = check(
            0,
            0,
            "Brave",
            &Action::AudioMute {
                target: "app:application.process.binary=brave".into(),
            },
        )
        .unwrap();
        let job = Job {
            session: 1,
            checks: vec![c.clone()],
        };
        let n = failed(c.clone(), "Brave".into(), "audio_target_unavailable");
        let worker = Worker {
            shared: Arc::new(Mutex::new(Shared {
                result: Some((job, vec![n])),
                ..Default::default()
            })),
            thread: None,
        };
        assert!(worker.poll(1, vec![c.clone()]).is_some());
        assert!(worker.poll(2, vec![c.clone()]).is_none());
        let mut moved = c.clone();
        moved.page = 1;
        assert!(worker.poll(1, vec![moved]).is_none());
        let mut changed = c;
        changed.target = "input:mic".into();
        assert!(worker.poll(1, vec![changed]).is_none());
        assert!(worker.poll(0, vec![]).is_none());
    }
    #[test]
    fn failure_copy_distinguishes_unsupported_queue_and_unknown_completion() {
        let c = check(0, 0, "Next", &Action::MediaNext).unwrap();
        let unsupported = failed(c.clone(), "MPZ".into(), "media_unsupported");
        assert!(unsupported.hint.contains("playlist"));
        let busy = failed(c.clone(), "MPZ".into(), "audio_queue_full");
        assert!(busy.detail.contains("not queued"));
        let timed_out = failed(c, "MPZ".into(), "launch_timeout");
        assert!(timed_out.detail.contains("unknown"));
        assert!(timed_out.hint.contains("before retrying"));
    }
}
