//! Isolated, latest-only peak helper. No audio data enters the device worker.
use std::{
    collections::BTreeMap,
    io::Read,
    os::unix::net::UnixStream,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
pub type Levels = BTreeMap<String, Option<u8>>;
#[derive(Default, Clone, PartialEq, Eq)]
struct Desired {
    session: u64,
    targets: Vec<String>,
}
pub struct Worker {
    desired: Arc<Mutex<Desired>>,
    latest: Arc<Mutex<Option<(Desired, Instant, Levels)>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start() -> Self {
        let desired = Arc::new(Mutex::new(Desired::default()));
        let latest = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let (want, out, done) = (desired.clone(), latest.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            while !done.load(Ordering::Acquire) {
                let current = want.lock().unwrap().clone();
                if current.session == 0 || current.targets.is_empty() {
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
                let Ok((mut reader, writer)) = UnixStream::pair() else {
                    break;
                };
                let _ = reader.set_read_timeout(Some(Duration::from_millis(20)));
                let path = crate::runtime_paths::helper("audio_meter.py");
                let child = Command::new("/usr/bin/python3")
                    .arg(path)
                    .arg(serde_json::to_string(&current.targets).unwrap())
                    .stdin(Stdio::null())
                    .stdout(Stdio::from(std::os::fd::OwnedFd::from(writer)))
                    .stderr(Stdio::null())
                    .spawn();
                if let Ok(mut child) = child {
                    let mut pending = Vec::new();
                    let mut buffer = [0u8; 4096];
                    while !done.load(Ordering::Acquire) && *want.lock().unwrap() == current {
                        match reader.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                pending.extend_from_slice(&buffer[..n]);
                                if pending.len() > 8192 {
                                    break;
                                }
                                while let Some(end) = pending.iter().position(|b| *b == b'\n') {
                                    if let Ok(levels) =
                                        serde_json::from_slice::<Levels>(&pending[..end])
                                        && levels.len() <= 4
                                        && levels.iter().all(|(k, v)| {
                                            current.targets.contains(k)
                                                && v.is_none_or(|n| n <= 100)
                                        })
                                    {
                                        *out.lock().unwrap() =
                                            Some((current.clone(), Instant::now(), levels));
                                    }
                                    pending.drain(..=end);
                                }
                            }
                            Err(e)
                                if matches!(
                                    e.kind(),
                                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                                ) => {}
                            Err(_) => break,
                        }
                    }
                    let _ = child.kill();
                    let _ = child.wait();
                }
                *out.lock().unwrap() = None;
                // Bounded retry backoff, interruptible on shutdown/configuration change.
                for _ in 0..50 {
                    if done.load(Ordering::Acquire) || *want.lock().unwrap() != current {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        });
        Self {
            desired,
            latest,
            stop,
            thread: Some(thread),
        }
    }
    pub fn levels(&self, session: u64, targets: Vec<String>) -> Levels {
        let desired = Desired { session, targets };
        *self.desired.lock().unwrap() = desired.clone();
        self.latest
            .lock()
            .unwrap()
            .as_ref()
            .filter(|(d, t, _)| *d == desired && t.elapsed() < Duration::from_millis(400))
            .map(|(_, _, v)| v.clone())
            .unwrap_or_default()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub fn draw(rgb: &mut [u8], level: Option<u8>, muted: bool) {
    // Cache the original glossy artwork for each bounded activity value.
    // Only rows above the independently rendered default-device marker change.
    static BARS: std::sync::OnceLock<Mutex<std::collections::BTreeMap<u8, Vec<u8>>>> =
        std::sync::OnceLock::new();
    let value = if muted {
        0
    } else {
        level.unwrap_or(0).min(100)
    };
    let mut bars = BARS.get_or_init(Default::default).lock().unwrap();
    let bar = bars.entry(value).or_insert_with(|| {
        let mut panel = [30, 34, 39].repeat(200 * 100);
        crate::level_bar::draw_signal(&mut panel, value);
        panel[72 * 600..94 * 600].to_vec()
    });
    for y in 72..94 {
        rgb[y * 600 + 27..y * 600 + 573]
            .copy_from_slice(&bar[(y - 72) * 600 + 27..(y - 72) * 600 + 573]);
    }
    if level.is_none() && !muted {
        // Missing measurement is a dash, not a fabricated silent signal.
        for y in 83..85 {
            for x in 94..106 {
                rgb[(y * 200 + x) * 3..(y * 200 + x) * 3 + 3].copy_from_slice(&[175, 175, 185]);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn latest_snapshot_expires_and_rejects_old_sessions_or_targets() {
        let desired = Desired {
            session: 1,
            targets: vec!["system".into()],
        };
        let worker = Worker {
            desired: Arc::new(Mutex::new(desired.clone())),
            latest: Arc::new(Mutex::new(Some((
                desired.clone(),
                Instant::now(),
                [("system".into(), Some(75))].into_iter().collect(),
            )))),
            stop: Arc::new(AtomicBool::new(false)),
            thread: None,
        };
        let start = Instant::now();
        assert_eq!(
            worker.levels(1, desired.targets.clone()).get("system"),
            Some(&Some(75))
        );
        assert!(worker.levels(2, desired.targets.clone()).is_empty());
        assert!(worker.levels(1, vec!["microphone".into()]).is_empty());
        worker.latest.lock().unwrap().as_mut().unwrap().1 = Instant::now() - Duration::from_secs(1);
        assert!(worker.levels(1, desired.targets).is_empty());
        assert!(start.elapsed() < Duration::from_millis(20));
    }
    #[test]
    fn signal_colors_follow_exact_thresholds() {
        use crate::level_bar::signal_tint;
        assert_eq!(signal_tint(79), None);
        assert_eq!(signal_tint(80), signal_tint(89));
        assert_ne!(signal_tint(89), signal_tint(90));
        assert_eq!(signal_tint(90), signal_tint(94));
        assert_ne!(signal_tint(94), signal_tint(95));
        assert_eq!(signal_tint(95), signal_tint(100));
        for level in [79, 80, 89, 90, 94, 95, 100] {
            {
                let mut panel = [30, 34, 39].repeat(200 * 100);
                draw(&mut panel, Some(level), false);
                let pixel = panel[(84 * 200 + 40) * 3..(84 * 200 + 40) * 3 + 3].to_vec();
                match level {
                    0..=79 => assert!(pixel[2] > pixel[0]),
                    80..=89 => assert!(pixel[1] > 180 && pixel[2] < 100),
                    90..=94 => assert!(pixel[0] > 200 && pixel[1] < 180),
                    _ => assert!(pixel[0] > 200 && pixel[1] < 100),
                }
            }
        }
        if let Ok(path) = std::env::var("DECKSMITH_COLOR_PREVIEW") {
            let mut strip = vec![0; 800 * 100 * 3];
            for (index, level) in [79, 80, 90, 95].into_iter().enumerate() {
                let mut panel = [30, 34, 39].repeat(200 * 100);
                crate::key_text::draw_strip(
                    &mut panel,
                    &format!("Signal {level}"),
                    0,
                    [240, 230, 215],
                );
                crate::key_text::draw_strip(&mut panel, "50%", 1, [90, 170, 255]);
                draw(&mut panel, Some(level), false);
                for y in 0..100 {
                    let start = (y * 800 + index * 200) * 3;
                    strip[start..start + 600].copy_from_slice(&panel[y * 600..(y + 1) * 600]);
                }
            }
            image::RgbImage::from_raw(800, 100, strip)
                .unwrap()
                .save(path)
                .unwrap();
        }
    }
    #[test]
    fn silence_unavailable_and_muted_are_distinct_and_bounded() {
        let base = [30, 34, 39].repeat(200 * 100);
        let mut silent = base.clone();
        draw(&mut silent, Some(0), false);
        let mut unknown = base.clone();
        draw(&mut unknown, None, false);
        assert_ne!(silent, unknown);
        let mut full = base.clone();
        draw(&mut full, Some(100), false);
        assert_ne!(silent, full);
        let mut muted = base.clone();
        draw(&mut muted, Some(100), true);
        assert_eq!(silent, muted);
        for y in 0..100 {
            if !(72..94).contains(&y) {
                assert_eq!(&full[y * 600..(y + 1) * 600], &base[y * 600..(y + 1) * 600]);
            }
        }
    }
}
