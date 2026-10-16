//! Fresh isolated connected-runtime acceptance. Fake transports and effect
//! adapters only: no HID discovery, native audio/microphone or production bus.
use crate::{
    device_execution::Runtime,
    device_profiles::{Enrollment, Store, random_device_id},
    pages::{Action, TouchState},
    shared_microphone::{Factory, Guardian, Observation},
    shared_worker::{Backend, Fence},
};
use decksmith_core::{
    InputEvent, RawEvent,
    device_context::DeviceId,
    device_model::{ControlAddress, DeviceDescriptor, DeviceModel},
    microphone_holds::DesiredMic,
};
use decksmith_device::{
    DeckDevice, DeviceError, VirtualDeck,
    registry::{Candidate, Discovery},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
struct Data {
    deck: VirtualDeck,
    time: u64,
    writes: u64,
    key_writes: u64,
}
#[derive(Clone)]
struct Probe {
    data: Arc<Mutex<Data>>,
    stall: Arc<(Mutex<bool>, Condvar)>,
    entered: Arc<AtomicBool>,
}
impl Probe {
    fn new(model: DeviceModel) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                deck: VirtualDeck::new(model),
                time: 0,
                writes: 0,
                key_writes: 0,
            })),
            stall: Arc::default(),
            entered: Arc::default(),
        }
    }
    fn inject(&self, event: RawEvent) -> Result<()> {
        let mut d = self.data.lock().unwrap();
        d.time += 1;
        let timestamp_ms = d.time;
        d.deck
            .inject(InputEvent {
                timestamp_ms,
                event,
            })
            .map_err(|_| "fixture_input_invalid")
    }
    fn release(&self) {
        *self.stall.0.lock().unwrap() = false;
        self.stall.1.notify_all();
    }
    fn blanked(&self) -> bool {
        let d = self.data.lock().unwrap();
        (0..d.deck.descriptor().key_count())
            .all(|i| d.deck.key_image(i).unwrap().iter().all(|b| *b == 0))
            && d.deck.touch_image().iter().all(|b| *b == 0)
    }
}
impl DeckDevice for Probe {
    fn descriptor(&self) -> DeviceDescriptor {
        self.data.lock().unwrap().deck.descriptor()
    }
    fn set_key_image(&mut self, i: u8, b: &[u8]) -> std::result::Result<(), DeviceError> {
        let mut s = self.stall.0.lock().unwrap();
        if *s {
            self.entered.store(true, Ordering::Release)
        }
        while *s {
            s = self.stall.1.wait(s).unwrap();
        }
        drop(s);
        let mut d = self.data.lock().unwrap();
        d.writes += 1;
        d.key_writes += 1;
        d.deck.set_key_image(i, b)
    }
    fn set_touch_image(&mut self, b: &[u8]) -> std::result::Result<(), DeviceError> {
        let mut d = self.data.lock().unwrap();
        d.writes += 1;
        d.deck.set_touch_image(b)
    }
    fn set_brightness(&mut self, b: u8) -> std::result::Result<(), DeviceError> {
        self.data.lock().unwrap().deck.set_brightness(b)
    }
    fn poll_event(&mut self) -> std::result::Result<Option<InputEvent>, DeviceError> {
        self.data.lock().unwrap().deck.poll_event()
    }
    fn blank(&mut self) -> std::result::Result<(), DeviceError> {
        self.data.lock().unwrap().deck.blank()
    }
}
struct Audio {
    level: i16,
    writes: usize,
}
struct FakeAudio(
    Arc<Mutex<Audio>>,
    Arc<(Mutex<bool>, Condvar)>,
    Arc<AtomicBool>,
);
impl Backend for FakeAudio {
    fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
        if !fence.accepts() {
            return Err("stale_action_scope");
        }
        let mut wait = self.1.0.lock().unwrap();
        if *wait {
            self.2.store(true, Ordering::Release)
        }
        while *wait {
            wait = self.1.1.wait(wait).unwrap();
        }
        drop(wait);
        let mut audio = self.0.lock().unwrap();
        if !fence.accepts() {
            return Err("stale_action_scope");
        }
        match action {
            Action::AudioAdjust { target, percent } if target == "output:fixture-speakers" => {
                audio.level = (audio.level + percent).clamp(0, 100);
                audio.writes += 1;
                Ok(())
            }
            _ => Err("fixture_target_unavailable"),
        }
    }
}
struct FakeMicrophone {
    muted: Arc<AtomicBool>,
}
struct Guard {
    muted: Arc<AtomicBool>,
    closed: bool,
}
impl Guardian for Guard {
    fn pulse(&mut self) -> Result<()> {
        Ok(())
    }
    fn close(&mut self) {
        self.closed = true;
        self.muted.store(true, Ordering::Release)
    }
    fn poll(&mut self) -> Result<Observation> {
        let muted = self.muted.load(Ordering::Acquire);
        Ok(Observation {
            committed: false,
            live: !muted,
            muted,
            exited: self.closed.then_some(true),
        })
    }
    fn terminate(&mut self) -> bool {
        self.close();
        true
    }
}
impl Factory for FakeMicrophone {
    fn start(&mut self, target: &str, desired: DesiredMic) -> Result<Box<dyn Guardian>> {
        if target != "input:fixture-mic" {
            return Err("fixture_target_unavailable");
        };
        self.muted
            .store(desired == DesiredMic::Muted, Ordering::Release);
        Ok(Box::new(Guard {
            muted: self.muted.clone(),
            closed: false,
        }))
    }
}
fn discovery() -> Result<Discovery> {
    Discovery::new(vec![
        Candidate::new(0, DeviceModel::Plus, Some("execution-fixture-plus")),
        Candidate::new(1, DeviceModel::Mk2, Some("execution-fixture-mk2")),
    ])
}
fn layout(model: DeviceModel) -> Vec<u8> {
    let count = DeviceDescriptor::for_model(model).key_count();
    let pages=(0..2).map(|p|serde_json::json!({"name":if p==0 {"Copper"}else{"Blue"},"background":if p==0 {[105,55,25]}else{[35,65,105]},"keys":(0..count).map(|i|serde_json::json!({"label":if i==0 {"Volume"}else if i==1 {"Hold mic"}else{"Empty"},"action":if i==0 {serde_json::json!({"type":"audio_adjust","target":"output:fixture-speakers","percent":1})}else if i==1 {serde_json::json!({"type":"push_to_talk","target":"input:fixture-mic"})}else{serde_json::json!({"type":"none"})}})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let mut data = serde_json::json!({"version":2,"model":model,"pages":pages});
    if model == DeviceModel::Plus {
        data["dials"]=serde_json::json!((0..4).map(|i|serde_json::json!({"label":"Audio","rotation":if i==0 {"none"}else{"volume"},"step":1,"audio_target":if i==0 {"input:fixture-mic"}else{"output:fixture-speakers"},"press":if i==0 {serde_json::json!({"type":"push_to_talk","target":"input:fixture-mic"})}else{serde_json::json!({"type":"none"})}})).collect::<Vec<_>>());
    }
    serde_json::to_vec(&data).unwrap()
}
struct Trial {
    runtime: Runtime,
    store: Store,
    ids: [DeviceId; 2],
    probes: [Probe; 2],
    audio: Arc<Mutex<Audio>>,
    audio_wait: Arc<(Mutex<bool>, Condvar)>,
    #[cfg(test)]
    audio_entered: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    feedback: usize,
    deadline: Instant,
}
impl Drop for Trial {
    fn drop(&mut self) {
        for p in &self.probes {
            p.release();
        }
        *self.audio_wait.0.lock().unwrap() = false;
        self.audio_wait.1.notify_all();
    }
}
impl Trial {
    fn new(root: &Path) -> Result<Self> {
        let store = Store::create(root)?;
        let ids = [random_device_id()?, random_device_id()?];
        let snapshot = discovery()?;
        for (i, model) in [DeviceModel::Plus, DeviceModel::Mk2]
            .into_iter()
            .enumerate()
        {
            let generation = store.load()?.generation;
            store.enroll(
                generation,
                &snapshot,
                i as u8,
                Enrollment::new(ids[i], "Execution fixture", 50, None),
            )?;
            store.save_profile(ids[i], 1, &layout(model), 0, "Execution fixture", 50)?;
        }
        let audio = Arc::new(Mutex::new(Audio {
            level: 50,
            writes: 0,
        }));
        let muted = Arc::new(AtomicBool::new(true));
        let audio_wait: Arc<(Mutex<bool>, Condvar)> = Arc::default();
        let audio_entered: Arc<AtomicBool> = Arc::default();
        let runtime = Runtime::start(
            store.load()?,
            vec![
                Box::new(FakeAudio(
                    audio.clone(),
                    audio_wait.clone(),
                    audio_entered.clone(),
                )),
                Box::new(FakeAudio(
                    audio.clone(),
                    audio_wait.clone(),
                    audio_entered.clone(),
                )),
            ],
            Some(Box::new(FakeMicrophone {
                muted: muted.clone(),
            })),
        )?;
        let mut t = Self {
            runtime,
            store,
            ids,
            probes: [Probe::new(DeviceModel::Plus), Probe::new(DeviceModel::Mk2)],
            audio,
            audio_wait,
            #[cfg(test)]
            audio_entered,
            muted,
            feedback: 0,
            deadline: Instant::now() + Duration::from_secs(45),
        };
        let revision = t.runtime.discover(snapshot)?;
        let d = discovery()?;
        for i in 0..2 {
            t.runtime
                .attach(revision, &d, i as u8, Box::new(t.probes[i].clone()))?;
        }
        t.runtime.update_touch(
            t.ids[0],
            TouchState {
                targets: vec![(
                    "output:fixture-speakers".into(),
                    Some(crate::audio::State {
                        percent: 50,
                        muted: false,
                        active: Some(true),
                        icon: Default::default(),
                    }),
                )],
                ..Default::default()
            },
        )?;
        Ok(t)
    }
    fn tick(&mut self) -> Result<()> {
        for f in self.runtime.tick() {
            if f.scope.device.device != self.ids[0] && f.scope.device.device != self.ids[1] {
                return Err("wrong_feedback_owner");
            };
            f.result?;
            self.feedback += 1;
        }
        Ok(())
    }
    fn until(&mut self, mut check: impl FnMut(&Self) -> bool) -> Result<()> {
        let end = (Instant::now() + Duration::from_secs(5)).min(self.deadline);
        while !check(self) {
            if Instant::now() > end {
                return Err("execution_fixture_deadline");
            };
            self.tick()?;
            std::thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }
    fn ready(&mut self) -> Result<()> {
        self.until(|s| {
            s.runtime
                .statuses()
                .iter()
                .all(|s| s.input_ready && s.display_ready)
        })
    }
    fn context(&self, i: usize) -> Result<decksmith_core::device_context::DeviceContext> {
        self.runtime.context(self.ids[i], ControlAddress::Key(0))
    }
    fn key(&self, i: usize, key: u8, pressed: bool) -> Result<()> {
        self.probes[i].inject(RawEvent::Key {
            index: key,
            pressed,
        })
    }
    fn exercise(&mut self) -> Result<()> {
        self.ready()?;
        let writes = self.probes[0].data.lock().unwrap().writes;
        let key_writes = self.probes[0].data.lock().unwrap().key_writes;
        self.runtime.update_levels(
            self.ids[0],
            [("output:fixture-speakers".into(), Some(71))]
                .into_iter()
                .collect(),
        )?;
        self.until(|s| s.probes[0].data.lock().unwrap().writes > writes)?;
        self.ready()?;
        if self.probes[0].data.lock().unwrap().key_writes != key_writes {
            return Err("meter_redrew_keys");
        }
        for i in 0..2 {
            self.key(i, 0, true)?;
            self.key(i, 0, false)?;
        }
        self.until(|s| s.audio.lock().unwrap().level == 52)?;
        for i in 0..2 {
            self.key(i, 1, true)?;
        }
        self.until(|s| s.runtime.held() == 2 && !s.muted.load(Ordering::Acquire))?;
        self.runtime.update_touch(
            self.ids[0],
            TouchState {
                targets: vec![(
                    "output:fixture-speakers".into(),
                    Some(crate::audio::State {
                        percent: 66,
                        muted: false,
                        active: Some(true),
                        icon: Default::default(),
                    }),
                )],
                ..Default::default()
            },
        )?;
        self.key(0, 1, false)?;
        self.until(|s| s.runtime.held() == 1)?;
        if self.muted.load(Ordering::Acquire) {
            return Err("overlap_muted_too_early");
        };
        self.key(1, 1, false)?;
        self.until(|s| s.runtime.held() == 0 && s.muted.load(Ordering::Acquire))?;
        self.ready()?;
        *self.probes[1].stall.0.lock().unwrap() = true;
        self.runtime.navigate(self.context(1)?, 1)?;
        self.until(|s| s.probes[1].entered.load(Ordering::Acquire))?;
        self.key(0, 0, true)?;
        self.key(0, 0, false)?;
        self.until(|s| s.audio.lock().unwrap().level == 53)?;
        if !self
            .runtime
            .statuses()
            .iter()
            .find(|s| s.id == self.ids[0])
            .unwrap()
            .display_ready
        {
            return Err("slow_other_display_blocked_healthy_unit");
        };
        self.runtime.navigate(self.context(1)?, 0)?;
        self.probes[1].release();
        self.ready()?;
        let c = self.context(1)?;
        self.store.save_profile(
            self.ids[1],
            2,
            &layout(DeviceModel::Mk2),
            1,
            "Edited MK.2",
            63,
        )?;
        self.runtime
            .apply_saved(self.store.load()?, self.ids[1], c.binding)?;
        self.ready()?;
        if self
            .runtime
            .statuses()
            .iter()
            .find(|s| s.id == self.ids[0])
            .unwrap()
            .page
            != Some(0)
        {
            return Err("other_page_changed");
        };
        self.runtime.gate(true, false)?;
        self.key(0, 0, true)?;
        self.key(0, 0, false)?;
        for _ in 0..12 {
            self.tick()?;
            std::thread::sleep(Duration::from_millis(2));
        }
        self.runtime.gate(false, false)?;
        self.ready()?;
        if self.audio.lock().unwrap().level != 53 {
            return Err("locked_input_replayed");
        };
        self.runtime.foreground(2)?;
        self.ready()?;
        let feedback_context = self.runtime.context(self.ids[0], ControlAddress::Key(3))?;
        self.runtime
            .key_feedback(feedback_context, &vec![7; 120 * 120 * 3])?;
        self.ready()?;
        self.runtime.disconnect(self.ids[1])?;
        self.key(0, 0, true)?;
        self.key(0, 0, false)?;
        self.until(|s| s.audio.lock().unwrap().level == 54)?;
        let revision = self.runtime.discover(discovery()?)?;
        self.until(|s| s.probes[1].blanked())?;
        // Reap before reopening the same identity; old owner reservation remains valid.
        self.tick()?;
        self.runtime
            .attach(revision, &discovery()?, 1, Box::new(self.probes[1].clone()))?;
        self.ready()?;
        if self
            .runtime
            .statuses()
            .iter()
            .find(|s| s.id == self.ids[1])
            .unwrap()
            .page
            != Some(1)
        {
            return Err("reconnect_page_lost");
        };
        let failures = self.runtime.stop();
        if failures.iter().any(|(_, r)| r.is_err()) || self.probes.iter().any(|p| !p.blanked()) {
            return Err("execution_stop_blank_failed");
        };
        Ok(())
    }
}
#[cfg(feature = "hardware")]
pub fn run(args: &[String]) -> Result<()> {
    let [_, root, flag, seconds] = args else {
        return Err("execution_lab_arguments");
    };
    let seconds = seconds
        .parse::<u64>()
        .map_err(|_| "execution_lab_arguments")?;
    if flag != "--seconds"
        || !(10..=120).contains(&seconds)
        || std::env::var("DECKSMITH_ISOLATED_TEST").as_deref() != Ok("1")
    {
        return Err("execution_lab_isolated_only");
    }
    let path = PathBuf::from(root);
    if !path.is_absolute() {
        return Err("execution_lab_absolute_path_required");
    }
    let mut t = Trial::new(&path)?;
    t.deadline = Instant::now() + Duration::from_secs(seconds);
    t.exercise()?;
    let report = serde_json::json!({"status":"passed","mode":"isolated-connected-execution","physical_hid":false,"native_effects":false,"effects":t.audio.lock().unwrap().writes,"feedback":t.feedback,"checks":["real_manager_input_routes","strip_only_meter_update","shared_target_effects","overlapping_microphone_holds","release_during_redraw","slow_display_other_device_progress","stale_page_frame_superseded","owned_profile_apply","lock_input_drain","foreground_fence","independent_disconnect_reconnect","all_surfaces_blank_after_join"]});
    crate::layout_store::write(
        &path.join("execution-result.json"),
        &serde_json::to_vec_pretty(&report).unwrap(),
    )?;
    println!("{report}");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_host_gate_cancels_holds_draws_each_model_and_restores_own_pages() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-host-lock-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.runtime.host_lock(false, 10)?;
            t.ready()?;
            t.runtime.navigate(t.context(1)?, 1)?;
            t.ready()?;
            let original: Vec<_> = t
                .probes
                .iter()
                .map(|probe| {
                    probe
                        .data
                        .lock()
                        .unwrap()
                        .deck
                        .key_image(0)
                        .unwrap()
                        .to_vec()
                })
                .collect();
            t.key(0, 1, true)?;
            t.key(1, 1, true)?;
            t.until(|s| s.runtime.held() == 2 && !s.muted.load(Ordering::Acquire))?;
            t.runtime.host_lock(true, 11)?;
            t.until(|s| {
                s.runtime.held() == 0
                    && s.muted.load(Ordering::Acquire)
                    && s.runtime
                        .statuses()
                        .iter()
                        .all(|u| u.display_ready && !u.input_ready)
            })?;
            for (i, probe) in t.probes.iter().enumerate() {
                let d = probe.data.lock().unwrap();
                let first = d.deck.key_image(0).unwrap();
                assert_eq!(
                    first.len(),
                    DeviceDescriptor::for_model([DeviceModel::Plus, DeviceModel::Mk2][i])
                        .key_rgb_bytes()
                );
                assert!(first.iter().any(|b| *b != 0));
                assert_ne!(first, original[i]);
                for key in 1..d.deck.descriptor().key_count() {
                    assert!(d.deck.key_image(key).unwrap().iter().all(|b| *b == 0));
                }
                if i == 0 {
                    assert!(d.deck.touch_image().iter().all(|b| *b == 0));
                }
            }
            // Reconnect while locked must paint Locked, never an ordinary page.
            t.runtime.disconnect(t.ids[1])?;
            t.probes[1] = Probe::new(DeviceModel::Mk2);
            let d = discovery()?;
            let rev = t.runtime.discover(discovery()?)?;
            let end = Instant::now() + Duration::from_secs(5);
            loop {
                match t.runtime.attach(rev, &d, 1, Box::new(t.probes[1].clone())) {
                    Ok(_) => break,
                    Err("device_actor_unavailable") if Instant::now() < end => {
                        t.tick()?;
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    Err(error) => return Err(error),
                }
            }
            t.until(|s| {
                s.runtime
                    .statuses()
                    .iter()
                    .all(|u| u.display_ready && !u.input_ready)
            })?;
            assert!(
                t.probes[1]
                    .data
                    .lock()
                    .unwrap()
                    .deck
                    .key_image(0)
                    .unwrap()
                    .iter()
                    .any(|b| *b != 0)
            );
            t.key(0, 1, false)?;
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            for _ in 0..20 {
                t.tick()?;
                std::thread::sleep(Duration::from_millis(2));
            }
            t.runtime.host_lock(false, 12)?;
            t.ready()?;
            assert_eq!(t.audio.lock().unwrap().level, 50);
            assert!(t.muted.load(Ordering::Acquire));
            assert_eq!(
                t.runtime
                    .statuses()
                    .iter()
                    .find(|u| u.id == t.ids[1])
                    .unwrap()
                    .page,
                Some(1)
            );
            for (i, probe) in t.probes.iter().enumerate() {
                assert_eq!(
                    probe.data.lock().unwrap().deck.key_image(0).unwrap(),
                    original[i]
                );
            }
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn host_unlock_cannot_clear_manual_gate_and_short_lock_epoch_fences_old_context() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-host-epoch-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.runtime.host_lock(false, 10)?;
            t.ready()?;
            let before = t.context(0)?;
            t.runtime.host_lock(false, 12)?;
            t.ready()?;
            assert_ne!(before.lock_epoch, t.context(0)?.lock_epoch);
            assert_eq!(
                t.runtime.key_feedback(before, &vec![7; 120 * 120 * 3]),
                Err("stale_device_context")
            );
            t.runtime.gate(true, false)?;
            t.runtime.host_lock(true, 13)?;
            t.runtime.host_lock(false, 14)?;
            t.until(|s| {
                s.runtime
                    .statuses()
                    .iter()
                    .all(|u| u.display_ready && !u.input_ready)
            })?;
            assert!(t.context(0).is_err());
            assert_eq!(t.runtime.host_lock(false, 13), Err("stale_host_lock_epoch"));
            t.runtime.gate(false, false)?;
            t.ready()?;
            t.key(1, 0, true)?;
            t.key(1, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn host_lock_fences_waiting_action_and_late_usb_frame_without_blocking_partner() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-host-late-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.runtime.host_lock(false, 10)?;
            t.ready()?;
            *t.audio_wait.0.lock().unwrap() = true;
            t.key(1, 0, true)?;
            t.key(1, 0, false)?;
            t.until(|s| s.audio_entered.load(Ordering::Acquire))?;
            *t.probes[1].stall.0.lock().unwrap() = true;
            t.runtime
                .key_feedback(t.context(1)?, &vec![7; 72 * 72 * 3])?;
            t.until(|s| s.probes[1].entered.load(Ordering::Acquire))?;
            t.runtime.host_lock(true, 11)?;
            t.until(|s| {
                s.runtime
                    .statuses()
                    .iter()
                    .any(|u| u.id == s.ids[0] && u.display_ready && !u.input_ready)
            })?;
            assert!(
                t.runtime
                    .statuses()
                    .iter()
                    .any(|u| u.id == t.ids[1] && !u.display_ready)
            );
            *t.audio_wait.0.lock().unwrap() = false;
            t.audio_wait.1.notify_all();
            t.probes[1].release();
            t.until(|s| {
                s.runtime
                    .statuses()
                    .iter()
                    .all(|u| u.display_ready && !u.input_ready)
            })?;
            assert_ne!(
                t.probes[1].data.lock().unwrap().deck.key_image(0).unwrap(),
                vec![7; 72 * 72 * 3]
            );
            t.runtime.host_lock(false, 12)?;
            t.ready()?;
            // A fresh request queues behind the old target reservation. One
            // successful effect proves the revoked waiting request never wrote.
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().writes == 1)?;
            assert_eq!(t.audio.lock().unwrap().level, 51);
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn sleep_settlement_retires_old_handles_and_restores_each_saved_page() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-sleep-pair-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.runtime.host_sleep(false, 10)?;
            t.ready()?;
            t.runtime.navigate(t.context(1)?, 1)?;
            t.ready()?;
            t.key(0, 1, true)?;
            t.key(1, 1, true)?;
            t.until(|s| s.runtime.held() == 2 && !s.muted.load(Ordering::Acquire))?;
            t.runtime.host_sleep(true, 11)?;
            assert!(!t.runtime.ready_to_attach(t.ids[0]));
            t.until(|s| s.runtime.sleep_ready())?;
            assert!(t.probes.iter().all(Probe::blanked));
            assert!(t.muted.load(Ordering::Acquire));
            t.runtime.host_sleep(false, 12)?;
            assert!(!t.runtime.ready_to_attach(t.ids[0]));
            t.until(|s| {
                s.runtime.ready_to_attach(s.ids[0]) && s.runtime.ready_to_attach(s.ids[1])
            })?;
            t.probes = [Probe::new(DeviceModel::Plus), Probe::new(DeviceModel::Mk2)];
            let d = discovery()?;
            let rev = t.runtime.discover(discovery()?)?;
            for i in 0..2 {
                t.runtime
                    .attach(rev, &d, i as u8, Box::new(t.probes[i].clone()))?;
            }
            t.ready()?;
            assert_eq!(
                t.runtime
                    .statuses()
                    .iter()
                    .find(|u| u.id == t.ids[1])
                    .unwrap()
                    .page,
                Some(1)
            );
            assert!(t.runtime.statuses().iter().all(|u| u.connection == 2));
            assert_eq!(t.runtime.held(), 0);
            assert!(t.muted.load(Ordering::Acquire));
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn awake_partner_reopens_while_old_other_owner_is_still_retiring() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-sleep-isolation-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.runtime.host_sleep(false, 10)?;
            t.ready()?;
            *t.probes[1].stall.0.lock().unwrap() = true;
            t.runtime
                .key_feedback(t.context(1)?, &vec![7; 72 * 72 * 3])?;
            t.until(|s| s.probes[1].entered.load(Ordering::Acquire))?;
            t.runtime.host_sleep(true, 11)?;
            assert!(!t.runtime.sleep_ready());
            t.runtime.host_sleep(false, 12)?;
            t.until(|s| s.runtime.ready_to_attach(s.ids[0]))?;
            assert!(!t.runtime.ready_to_attach(t.ids[1]));
            t.probes[0] = Probe::new(DeviceModel::Plus);
            let d = discovery()?;
            let rev = t.runtime.discover(discovery()?)?;
            t.runtime
                .attach(rev, &d, 0, Box::new(t.probes[0].clone()))?;
            t.until(|s| {
                s.runtime
                    .statuses()
                    .iter()
                    .any(|u| u.id == s.ids[0] && u.input_ready && u.display_ready)
            })?;
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            t.probes[1].release();
            t.until(|s| s.runtime.ready_to_attach(s.ids[1]))?;
            t.probes[1] = Probe::new(DeviceModel::Mk2);
            t.runtime
                .attach(rev, &d, 1, Box::new(t.probes[1].clone()))?;
            t.ready()?;
            assert!(t.runtime.sleep_errors().is_empty());
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn editor_test_routes_only_current_saved_non_hold_controls() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-test-action-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            let base = t.context(1)?;
            let raw = String::from_utf8(layout(DeviceModel::Mk2)).unwrap();
            assert_eq!(
                t.runtime.test_control(base, &raw, 0, 1),
                Err("test_push_to_talk_on_device")
            );
            assert_eq!(
                t.runtime.test_control(base, &raw, 1, 0),
                Err("save_and_select_page_first")
            );
            assert_eq!(
                t.runtime.test_control(base, "{}", 0, 0),
                Err("save_and_select_page_first")
            );
            t.runtime.test_control(base, &raw, 0, 0)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            t.runtime.gate(true, false)?;
            assert_eq!(
                t.runtime.test_control(base, &raw, 0, 0),
                Err("device_not_ready")
            );
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn lifecycle_fence_cancels_pending_save_reservation_without_stranding_device() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-sleep-pending-save-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            let ticket = t.runtime.reserve_edit(t.context(0)?)?;
            assert!(t.context(0).is_err());
            t.runtime.host_lock(true, 10)?;
            t.runtime.host_lock(false, 11)?;
            t.ready()?;
            assert_eq!(t.runtime.cancel_edit(ticket), Err("stale_profile_update"));
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn prepared_live_save_cannot_change_a_partners_pinned_player() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-partner-pin-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            let base = t.store.load()?;
            let mut doc: serde_json::Value =
                serde_json::from_slice(base.profile(t.ids[1])?.document().bytes()).unwrap();
            doc["pages"][0]["keys"][2] = serde_json::json!({"label":"Partner media","action":{"type":"media_play_pause"},"media_player":"org.mpris.MediaPlayer2.Fixture"});
            t.store.save_profile(
                t.ids[1],
                base.profile(t.ids[1])?.revision,
                &serde_json::to_vec(&doc).unwrap(),
                0,
                "Partner",
                50,
            )?;
            t.runtime
                .apply_saved(t.store.load()?, t.ids[1], t.context(1)?.binding)?;
            t.ready()?;
            let pin = crate::shared_native_media::Session {
                selector: "org.mpris.MediaPlayer2.Fixture".into(),
                name: "org.mpris.MediaPlayer2.Fixture".into(),
                owner: ":1.44".into(),
            };
            t.runtime.bind_native_media(vec![pin.clone()])?;
            let base = t.store.load()?;
            let projected = base.project_profile(
                t.ids[0],
                base.profile(t.ids[0])?.revision,
                base.profile(t.ids[0])?.document().bytes(),
                0,
                "Own",
                50,
            )?;
            let prepared = crate::device_execution::PreparedEdit::new(
                &projected,
                t.ids[0],
                vec![pin.clone()],
            )?;
            let ticket = t.runtime.reserve_edit(t.context(0)?)?;
            t.runtime
                .apply_edit(ticket, t.store.publish_projected(projected)?, prepared)?;
            t.ready()?;
            let partner = t.context(1)?;
            let base = t.store.load()?;
            let projected = base.project_profile(
                t.ids[0],
                base.profile(t.ids[0])?.revision,
                base.profile(t.ids[0])?.document().bytes(),
                0,
                "Own changed",
                50,
            )?;
            let mut other = pin;
            other.owner = ":1.55".into();
            let prepared =
                crate::device_execution::PreparedEdit::new(&projected, t.ids[0], vec![other])?;
            let ticket = t.runtime.reserve_edit(t.context(0)?)?;
            assert_eq!(
                t.runtime.apply_edit(ticket, projected, prepared),
                Err("partner_media_pin_changed")
            );
            assert_eq!(t.context(1)?, partner);
            t.runtime.cancel_edit(ticket)?;
            t.ready()?;
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn prepared_live_save_cancels_only_edited_hold_and_keeps_partner_actions_and_context() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-live-save-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            t.key(0, 1, true)?;
            t.key(1, 1, true)?;
            t.until(|s| s.runtime.held() == 2)?;
            let partner = t.context(1)?;
            let base = t.store.load()?;
            let mut value: serde_json::Value =
                serde_json::from_slice(base.profile(t.ids[0])?.document().bytes()).unwrap();
            value["pages"][0]["keys"][2] = serde_json::json!({"label":"Saved 🎵","action":{"type":"audio_adjust","target":"app:application.name=New fixture","percent":1}});
            let projected = base.project_profile(
                t.ids[0],
                base.profile(t.ids[0])?.revision,
                &serde_json::to_vec(&value).unwrap(),
                1,
                "Desk 🎵",
                61,
            )?;
            let prepared =
                crate::device_execution::PreparedEdit::new(&projected, t.ids[0], Vec::new())?;
            let ticket = t.runtime.reserve_edit(t.context(0)?)?;
            t.key(1, 0, true)?;
            t.key(1, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51 && s.runtime.held() == 1)?;
            assert_eq!(t.context(1)?, partner);
            assert!(
                !t.runtime
                    .statuses()
                    .iter()
                    .find(|s| s.id == t.ids[0])
                    .unwrap()
                    .input_ready
            );
            let published = t.store.publish_projected(projected)?;
            t.runtime.apply_edit(ticket, published, prepared)?;
            t.ready()?;
            assert_eq!(t.context(1)?, partner);
            let own = t
                .runtime
                .statuses()
                .into_iter()
                .find(|s| s.id == t.ids[0])
                .unwrap();
            assert_eq!(own.page, Some(1));
            assert_eq!(own.brightness, 61);
            assert!(
                t.runtime
                    .catalog()
                    .profile(t.ids[0])?
                    .document()
                    .bytes()
                    .windows("Saved 🎵".len())
                    .any(|b| b == "Saved 🎵".as_bytes())
            );
            t.key(1, 1, false)?;
            t.until(|s| s.runtime.held() == 0 && s.muted.load(Ordering::Acquire))?;
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn cancelled_live_save_restores_only_own_input_and_disconnected_ticket_cannot_apply() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-live-cancel-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            let partner = t.context(1)?;
            let before = t.store.load()?.generation;
            let ticket = t.runtime.reserve_edit(t.context(0)?)?;
            t.runtime.cancel_edit(ticket)?;
            t.ready()?;
            assert_eq!(t.context(1)?, partner);
            assert_eq!(t.store.load()?.generation, before);
            let base = t.store.load()?;
            let projected = base.project_profile(
                t.ids[0],
                base.profile(t.ids[0])?.revision,
                base.profile(t.ids[0])?.document().bytes(),
                0,
                "Saved",
                50,
            )?;
            let prepared =
                crate::device_execution::PreparedEdit::new(&projected, t.ids[0], Vec::new())?;
            let ticket = t.runtime.reserve_edit(t.context(0)?)?;
            t.runtime.disconnect(t.ids[0])?;
            assert_eq!(
                t.runtime.apply_edit(ticket, projected, prepared),
                Err("stale_profile_update")
            );
            assert_eq!(t.context(1)?, partner);
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn stalled_io_cannot_keep_its_microphone_hold_live() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-mic-stall-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            t.key(1, 1, true)?;
            t.until(|s| s.runtime.held() == 1 && !s.muted.load(Ordering::Acquire))?;
            *t.probes[1].stall.0.lock().unwrap() = true;
            let context = t.runtime.context(t.ids[1], ControlAddress::Key(3))?;
            t.runtime.key_feedback(context, &vec![7; 72 * 72 * 3])?;
            t.until(|s| s.probes[1].entered.load(Ordering::Acquire))?;
            t.until(|s| s.runtime.held() == 0 && s.muted.load(Ordering::Acquire))?;
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            t.probes[1].release();
            t.ready()?;
            if !t.muted.load(Ordering::Acquire) {
                return Err("expired_hold_reopened");
            };
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn old_started_action_cannot_write_after_own_page_changes() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-action-fence-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            *t.audio_wait.0.lock().unwrap() = true;
            t.key(1, 0, true)?;
            t.key(1, 0, false)?;
            t.until(|s| s.audio_entered.load(Ordering::Acquire))?;
            t.runtime.navigate(t.context(1)?, 1)?;
            t.key(0, 0, true)?;
            t.key(0, 0, false)?;
            *t.audio_wait.0.lock().unwrap() = false;
            t.audio_wait.1.notify_all();
            t.until(|s| s.audio.lock().unwrap().level == 51)?;
            t.ready()?;
            if t.audio.lock().unwrap().writes != 1 {
                return Err("stale_action_wrote");
            };
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn unavailable_builtin_provider_reports_error_and_never_changes_audio() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-unsupported-action-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let check = (|| -> Result<()> {
            t.ready()?;
            let mut data: serde_json::Value =
                serde_json::from_slice(&layout(DeviceModel::Plus)).unwrap();
            data["pages"][0]["keys"][2]["action"] =
                serde_json::json!({"type":"open_website","url":"https://example.org/"});
            let binding = t.context(0)?.binding;
            t.store.save_profile(
                t.ids[0],
                2,
                &serde_json::to_vec(&data).unwrap(),
                0,
                "Fixture",
                50,
            )?;
            t.runtime.apply_saved(t.store.load()?, t.ids[0], binding)?;
            t.ready()?;
            t.key(0, 2, true)?;
            t.key(0, 2, false)?;
            let end = Instant::now() + Duration::from_secs(3);
            let mut rejected = false;
            while !rejected && Instant::now() < end {
                for f in t.runtime.tick() {
                    if f.result == Err("fixture_target_unavailable") {
                        rejected = true;
                    } else {
                        f.result?;
                    }
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            if !rejected || t.audio.lock().unwrap().writes != 0 {
                return Err("unsupported_action_fell_back");
            };
            Ok(())
        })();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        check.unwrap();
    }
    #[test]
    fn supplied_pair_executes_shared_actions_and_keeps_display_failures_independent() {
        let root = std::env::temp_dir().join(format!(
            "decksmith-connected-execution-{}",
            random_device_id().unwrap()
        ));
        let mut t = Trial::new(&root).unwrap();
        let result = t.exercise();
        drop(t);
        let _ = std::fs::remove_dir_all(root);
        result.unwrap();
    }
}
