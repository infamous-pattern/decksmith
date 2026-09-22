//! Separate microphone guardians: releases bypass the ordinary action queue.
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
struct Desired {
    slots: BTreeMap<u8, String>,
    touched: Instant,
    stop: bool,
    failures: BTreeSet<String>,
}
pub struct Controller {
    desired: Arc<Mutex<Desired>>,
    thread: Option<JoinHandle<()>>,
}
impl Controller {
    pub fn start() -> Self {
        let desired = Arc::new(Mutex::new(Desired {
            slots: BTreeMap::new(),
            touched: Instant::now(),
            stop: false,
            failures: BTreeSet::new(),
        }));
        let shared = desired.clone();
        let thread = std::thread::spawn(move || {
            let mut reported = BTreeSet::new();
            let mut guards: BTreeMap<String, Option<Child>> = BTreeMap::new();
            loop {
                let (targets, stop) = {
                    let mut state = shared.lock().unwrap();
                    if state.touched.elapsed() >= Duration::from_millis(500) {
                        state.slots.clear();
                    }
                    (
                        if state.touched.elapsed() < Duration::from_millis(500) && !state.stop {
                            state.slots.values().cloned().collect::<BTreeSet<_>>()
                        } else {
                            BTreeSet::new()
                        },
                        state.stop,
                    )
                };
                let removed: Vec<_> = guards
                    .keys()
                    .filter(|t| !targets.contains(*t))
                    .cloned()
                    .collect();
                for target in removed {
                    reported.remove(&target);
                    if let Some(Some(mut child)) = guards.remove(&target) {
                        child.stdin.take();
                        reap(child);
                    }
                }
                if stop {
                    break;
                }
                for target in targets {
                    let guard = guards.entry(target.clone()).or_insert_with(|| {
                        Command::new("/usr/bin/python3")
                            .arg(crate::runtime_paths::helper("ptt_guard.py"))
                            .arg(&target)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                            .ok()
                    });
                    let failed = match guard {
                        Some(child) => match child.try_wait() {
                            Ok(None) => child
                                .stdin
                                .as_mut()
                                .is_none_or(|input| input.write_all(b"hold\n").is_err()),
                            _ => true,
                        },
                        None => true,
                    };
                    if failed && reported.insert(target.clone()) {
                        shared.lock().unwrap().failures.insert(target);
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        Self {
            desired,
            thread: Some(thread),
        }
    }
    #[cfg(test)]
    pub fn fixture() -> Self {
        Self {
            desired: Arc::new(Mutex::new(Desired {
                slots: BTreeMap::new(),
                touched: Instant::now(),
                stop: false,
                failures: BTreeSet::new(),
            })),
            thread: None,
        }
    }
    #[cfg(test)]
    pub fn held_count(&self) -> usize {
        self.desired.lock().unwrap().slots.len()
    }
    pub fn take_failures(&self) -> Vec<String> {
        let mut desired = self.desired.lock().unwrap();
        std::mem::take(&mut desired.failures).into_iter().collect()
    }
    pub fn tick(&self) {
        self.desired.lock().unwrap().touched = Instant::now();
    }
    pub fn press(&self, slot: u8, target: String) {
        self.desired
            .lock()
            .unwrap()
            .slots
            .entry(slot)
            .or_insert(target);
    }
    pub fn release(&self, slot: u8) -> bool {
        self.desired.lock().unwrap().slots.remove(&slot).is_some()
    }
    pub fn cancel(&self) {
        self.desired.lock().unwrap().slots.clear();
    }
}
fn reap(mut child: Child) {
    // Reap outside the HID thread; the guardian has bounded audio commands.
    let deadline = Instant::now() + Duration::from_secs(3);
    while child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }
    let _ = child.wait();
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.desired.lock().unwrap().stop = true;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn invalid_target_guard_reports_failure_once_without_audio_access() {
        let controller = super::Controller::start();
        controller.press(0, "invalid-target".into());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            controller.tick();
            let errors = controller.take_failures();
            if !errors.is_empty() {
                assert_eq!(errors, vec!["invalid-target"]);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        for _ in 0..3 {
            controller.tick();
            std::thread::sleep(std::time::Duration::from_millis(100));
            assert!(controller.take_failures().is_empty());
        }
        controller.release(0);
    }

    use super::*;
    #[test]
    #[ignore = "requires a disposable virtual microphone specified by DECKSMITH_PTT_TEST_INPUT"]
    fn live_virtual_mic_hold_release_overlap_cancel_and_shutdown() {
        let target = std::env::var("DECKSMITH_PTT_TEST_INPUT").unwrap();
        assert!(target.starts_with("input:decksmith_ptt_test"));
        let controller = Controller::start();
        let wait = |muted: bool, controller: Option<&Controller>| {
            let deadline = Instant::now() + Duration::from_secs(4);
            loop {
                if let Some(c) = controller {
                    c.tick();
                }
                if crate::audio_target::Client::default()
                    .read(std::slice::from_ref(&target))
                    .first()
                    .copied()
                    .flatten()
                    .is_some_and(|s| s.muted == muted)
                {
                    break;
                }
                assert!(Instant::now() < deadline, "microphone state did not settle");
                std::thread::sleep(Duration::from_millis(50));
            }
        };
        controller.press(0, target.clone());
        wait(false, Some(&controller));
        controller.press(8, target.clone());
        controller.release(0);
        for _ in 0..4 {
            controller.tick();
            std::thread::sleep(Duration::from_millis(100));
        }
        wait(false, Some(&controller));
        controller.release(8);
        wait(true, Some(&controller));
        controller.press(0, target.clone());
        wait(false, Some(&controller));
        controller.cancel();
        wait(true, Some(&controller));
        controller.press(0, target.clone());
        wait(false, Some(&controller));
        // A stalled HID owner loses the lease and cannot reopen without another press.
        std::thread::sleep(Duration::from_millis(1200));
        wait(true, None);
        controller.tick();
        assert_eq!(controller.held_count(), 0);
        controller.press(0, target.clone());
        wait(false, Some(&controller));
        drop(controller);
        wait(true, None);
    }
}
