//! Explicit, timed native-HID acceptance of the connected runtime. Effects are
//! memory-only diagnostics; no real audio, microphone, desktop bus or providers.
use crate::{
    device_execution::Runtime,
    device_profiles::{Catalog, Enrollment, Store, random_device_id},
    layout_document::Document,
    pages::{Action, TouchState},
    shared_worker::{Backend, Fence},
};
use decksmith_core::{
    InputEvent, RawEvent,
    device_context::DeviceId,
    device_model::{DeviceDescriptor, DeviceModel},
};
use decksmith_device::{DeckDevice, DeviceError, registry::Discovery};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};
type Result<T> = std::result::Result<T, &'static str>;
const TARGET: &str = "output:connected-test-level";

#[derive(Clone, Copy, Serialize)]
struct Level {
    percent: u8,
    muted: bool,
    writes: u64,
}
impl Default for Level {
    fn default() -> Self {
        Self {
            percent: 50,
            muted: false,
            writes: 0,
        }
    }
}
struct DiagnosticBackend(Arc<Mutex<Level>>);
impl Backend for DiagnosticBackend {
    fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
        let mut level = self.0.lock().map_err(|_| "diagnostic_state_failed")?;
        if !fence.accepts() {
            return Err("stale_action_scope");
        }
        match action {
            Action::AudioAdjust { target, percent } if target == TARGET => {
                level.percent = (i16::from(level.percent) + percent).clamp(0, 100) as u8;
            }
            Action::AudioMute { target } if target == TARGET => level.muted = !level.muted,
            _ => return Err("diagnostic_target_unavailable"),
        }
        level.writes = level.writes.saturating_add(1);
        Ok(())
    }
}

fn layout(model: DeviceModel) -> Result<Document> {
    let count = DeviceDescriptor::for_model(model).key_count();
    let pages = (0..2).map(|page| {
        let keys = (0..count).map(|index| {
            let (label, action) = match index {
                0 => ("Test Down".to_string(), serde_json::json!({"type":"audio_adjust","target":TARGET,"percent":-1})),
                1 => ("Test Up".into(), serde_json::json!({"type":"audio_adjust","target":TARGET,"percent":1})),
                2 => ("Test Mute".into(), serde_json::json!({"type":"audio_mute","target":TARGET})),
                i if i == count-4 => ("Dim".into(), serde_json::json!({"type":"brightness_adjust","percent":-5})),
                i if i == count-3 => ("Bright".into(), serde_json::json!({"type":"brightness_adjust","percent":5})),
                i if i == count-2 => ("Prev".into(), serde_json::json!({"type":"previous_page"})),
                i if i == count-1 => ("Next".into(), serde_json::json!({"type":"next_page"})),
                _ => (format!("Key {}", index+1), serde_json::json!({"type":"none"})),
            };
            serde_json::json!({"label":label,"action":action})
        }).collect::<Vec<_>>();
        serde_json::json!({"name":if page==0 {"Copper A"} else {"Blue B"},"background":if page==0 {[105,55,25]} else {[35,65,105]},"keys":keys})
    }).collect::<Vec<_>>();
    let mut value = serde_json::json!({"version":2,"model":model,"pages":pages});
    if model == DeviceModel::Plus {
        value["dials"] = serde_json::json!([
            {"label":"Test brightness","rotation":"brightness","step":1,"press":{"type":"none"}},
            {"label":"Shared test level","rotation":"volume","step":1,"audio_target":TARGET,"press":{"type":"mute_toggle"}},
            {"label":"Shared test level","rotation":"volume","step":1,"audio_target":TARGET,"press":{"type":"mute_toggle"}},
            {"label":"Shared test level","rotation":"volume","step":1,"audio_target":TARGET,"press":{"type":"mute_toggle"}}
        ]);
    }
    Document::parse(&serde_json::to_vec(&value).map_err(|_| "invalid_test_layout")?)
}

fn prepare(path: &Path, discovery: &Discovery) -> Result<Catalog> {
    let candidates = discovery.candidates();
    if candidates.len() != 2
        || [DeviceModel::Plus, DeviceModel::Mk2]
            .iter()
            .any(|model| candidates.iter().filter(|c| c.model() == *model).count() != 1)
    {
        return Err("physical_test_pair_ambiguous");
    }
    // The caller supplies a fresh destination; Store verifies private identities
    // and refuses missing/duplicate association instead of guessing a slot.
    crate::layout_store::mkdir(path)?;
    let store = Store::create(&path.join("profiles"))?;
    let mut generation = 0;
    for candidate in candidates {
        let id = random_device_id()?;
        store.enroll(
            generation,
            discovery,
            candidate.slot(),
            Enrollment::new(id, "Connected test", 50, None),
        )?;
        let catalog = store.load()?;
        let document = layout(candidate.model())?;
        generation = store.save_profile(
            id,
            catalog.profile(id)?.revision,
            document.bytes(),
            0,
            "Connected test",
            50,
        )?;
    }
    store.load()
}

#[derive(Default, Clone, Serialize)]
struct Counts {
    keys: u64,
    dial_turns: u64,
    dial_pushes: u64,
    touch: u64,
    key_writes: u64,
    touch_writes: u64,
    connections: u64,
    feedback_ok: u64,
    feedback_failed: u64,
    last_feedback_error: Option<&'static str>,
    blank_completed: u64,
}
struct Observed {
    device: Box<dyn DeckDevice + Send>,
    counts: Arc<Mutex<Counts>>,
}
impl DeckDevice for Observed {
    fn descriptor(&self) -> DeviceDescriptor {
        self.device.descriptor()
    }
    fn set_key_image(&mut self, index: u8, bytes: &[u8]) -> std::result::Result<(), DeviceError> {
        self.device.set_key_image(index, bytes)?;
        self.counts.lock().unwrap().key_writes += 1;
        Ok(())
    }
    fn set_touch_image(&mut self, bytes: &[u8]) -> std::result::Result<(), DeviceError> {
        self.device.set_touch_image(bytes)?;
        self.counts.lock().unwrap().touch_writes += 1;
        Ok(())
    }
    fn set_brightness(&mut self, percent: u8) -> std::result::Result<(), DeviceError> {
        self.device.set_brightness(percent)
    }
    fn poll_event(&mut self) -> std::result::Result<Option<InputEvent>, DeviceError> {
        self.poll_event_nonblocking()
    }
    fn poll_event_nonblocking(&mut self) -> std::result::Result<Option<InputEvent>, DeviceError> {
        let event = self.device.poll_event_nonblocking()?;
        if let Some(event) = &event {
            let mut c = self.counts.lock().unwrap();
            match event.event {
                RawEvent::Key { .. } => c.keys += 1,
                RawEvent::DialRotate { .. } => c.dial_turns += 1,
                RawEvent::DialPush { .. } => c.dial_pushes += 1,
                RawEvent::Touch { .. } => c.touch += 1,
            }
        }
        Ok(event)
    }
    fn event_received_at(&self) -> Option<Instant> {
        self.device.event_received_at()
    }
    fn blank(&mut self) -> std::result::Result<(), DeviceError> {
        self.device.blank()?;
        self.counts.lock().unwrap().blank_completed += 1;
        Ok(())
    }
}
struct Trial {
    runtime: Runtime,
    level: Arc<Mutex<Level>>,
    counts: BTreeMap<DeviceId, Arc<Mutex<Counts>>>,
    last_observation: u64,
}
impl Trial {
    fn start(catalog: Catalog) -> Result<Self> {
        let level = Arc::new(Mutex::new(Level::default()));
        let counts = catalog
            .summaries()
            .iter()
            .map(|s| (s.id, Arc::new(Mutex::new(Counts::default()))))
            .collect();
        let runtime = Runtime::start(
            catalog,
            vec![
                Box::new(DiagnosticBackend(level.clone())),
                Box::new(DiagnosticBackend(level.clone())),
            ],
            None,
        )?;
        Ok(Self {
            runtime,
            level,
            counts,
            last_observation: u64::MAX,
        })
    }
    fn attach(
        &mut self,
        revision: u64,
        discovery: &Discovery,
        slot: u8,
        id: DeviceId,
        device: Box<dyn DeckDevice + Send>,
    ) -> Result<()> {
        let counts = self.counts.get(&id).ok_or("unknown_device")?.clone();
        self.runtime.attach(
            revision,
            discovery,
            slot,
            Box::new(Observed {
                device,
                counts: counts.clone(),
            }),
        )?;
        counts.lock().unwrap().connections += 1;
        Ok(())
    }
    fn tick(&mut self) -> Result<()> {
        for reply in self.runtime.tick() {
            let counts = self
                .counts
                .get(&reply.scope.device.device)
                .ok_or("unknown_device")?;
            let mut c = counts.lock().unwrap();
            match reply.result {
                Ok(()) => c.feedback_ok += 1,
                Err(error) => {
                    c.feedback_failed += 1;
                    c.last_feedback_error = Some(error);
                }
            }
        }
        let level = *self.level.lock().map_err(|_| "diagnostic_state_failed")?;
        if self.last_observation != level.writes {
            let state = TouchState {
                targets: vec![(
                    TARGET.into(),
                    Some(crate::audio::State {
                        percent: u16::from(level.percent),
                        muted: level.muted,
                        active: None,
                        icon: crate::audio_icon::Kind::Speaker,
                    }),
                )],
                ..Default::default()
            };
            for id in self.counts.keys() {
                self.runtime.update_touch(*id, state.clone())?;
            }
            self.last_observation = level.writes;
        }
        Ok(())
    }
    fn report(&self, path: &Path, status: &str) -> Result<()> {
        let units = self.runtime.statuses().into_iter().map(|s| serde_json::json!({"model":s.model,"connected":s.connected,"input_ready":s.input_ready,"display_ready":s.display_ready,"page":s.page,"brightness":s.brightness,"error":s.error,"counts":self.counts[&s.id].lock().unwrap().clone()})).collect::<Vec<_>>();
        write_report(path, &serde_json::to_vec_pretty(&serde_json::json!({"status":status,"mode":"connected-physical-diagnostic","native_audio":false,"native_microphone":false,"shared_test_level":*self.level.lock().unwrap(),"devices":units})).map_err(|_| "test_status_failed")?)
    }
}

fn write_report(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    crate::layout_store::write(&tmp, bytes)?;
    std::fs::rename(tmp, path).map_err(|_| "physical_report_failed")
}

#[cfg(feature = "hardware")]
pub fn run(args: &[String]) -> Result<()> {
    use decksmith_device::{
        hardware::{HardwareOwner, Inventory},
        registry::Association,
    };
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };
    let [_, destination, flag, seconds] = args else {
        return Err("connected_physical_arguments");
    };
    let seconds = seconds
        .parse::<u64>()
        .map_err(|_| "connected_physical_arguments")?;
    if flag != "--seconds"
        || !(20..=600).contains(&seconds)
        || std::env::var("DECKSMITH_PHYSICAL_TEST").as_deref() != Ok("1")
    {
        return Err("physical_lab_explicit_only");
    }
    let path = Path::new(destination);
    if !path.is_absolute() || path.exists() {
        return Err("fresh_absolute_destination_required");
    }
    let stopping = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stopping.clone())
            .map_err(|_| "physical_lab_signal_failed")?;
    }
    let mut owner = HardwareOwner::acquire().map_err(|_| "physical_ownership_unavailable")?;
    let inventory = Inventory::discover().map_err(|_| "physical_discovery_failed")?;
    let catalog = prepare(path, inventory.discovery())?;
    let mut trial = Trial::start(Store::open(&path.join("profiles"))?.load()?)?;
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut refresh = Instant::now() - Duration::from_secs(2);
    let mut report = refresh;
    let result = (|| {
        while Instant::now() < end && !stopping.load(Ordering::Acquire) {
            if refresh.elapsed() >= Duration::from_secs(1) {
                let current = Inventory::discover().map_err(|_| "physical_discovery_failed")?;
                let revision = trial.runtime.discover(current.snapshot())?;
                for (slot, association) in catalog.associations(current.discovery()) {
                    let Association::Assigned(id) = association else {
                        continue;
                    };
                    if trial
                        .runtime
                        .statuses()
                        .iter()
                        .any(|s| s.id == id && !s.connected)
                    {
                        // A retiring I/O owner retains its physical claim until
                        // cleanup ends; do not replace it with a second writer.
                        match owner.open(&current, slot) {
                            Ok(device) => trial.attach(
                                revision,
                                current.discovery(),
                                slot,
                                id,
                                Box::new(device),
                            )?,
                            Err(DeviceError::DeviceUnavailable) => {}
                            Err(_) => return Err("physical_open_failed"),
                        }
                    }
                }
                refresh = Instant::now();
            }
            trial.tick()?;
            if report.elapsed() >= Duration::from_millis(250) {
                trial.report(&path.join("status.json"), "running")?;
                report = Instant::now();
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    })();
    let blank = trial.runtime.stop();
    let blank_failed = blank.iter().any(|(_, r)| r.is_err());
    write_report(
        &path.join("blanking.json"),
        &serde_json::to_vec_pretty(
            &blank
                .iter()
                .map(|(_, r)| format!("{r:?}"))
                .collect::<Vec<_>>(),
        )
        .map_err(|_| "test_status_failed")?,
    )?;
    trial.report(
        &path.join("status.json"),
        if result.is_ok() && !blank_failed {
            "completed"
        } else {
            "failed"
        },
    )?;
    result?;
    if blank_failed {
        return Err("physical_test_blank_failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_core::device_model::ControlAddress;
    use decksmith_device::{VirtualDeck, registry::Candidate};
    use std::time::Duration;
    fn pump(trial: &mut Trial, predicate: impl Fn(&Trial) -> bool) {
        let end = Instant::now() + Duration::from_secs(5);
        while Instant::now() < end {
            trial.tick().unwrap();
            if predicate(trial) {
                return;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("connected physical fixture deadline");
    }
    #[test]
    fn hardware_diagnostic_documents_limit_actions_to_memory_target_and_own_device() {
        for model in [DeviceModel::Plus, DeviceModel::Mk2] {
            let document = layout(model).unwrap();
            assert_eq!(document.model(), model);
            let value: serde_json::Value = serde_json::from_slice(document.bytes()).unwrap();
            for page in value["pages"].as_array().unwrap() {
                for key in page["keys"].as_array().unwrap() {
                    let action = &key["action"];
                    assert!(matches!(
                        action["type"].as_str().unwrap(),
                        "audio_adjust"
                            | "audio_mute"
                            | "brightness_adjust"
                            | "previous_page"
                            | "next_page"
                            | "none"
                    ));
                    if action["target"].is_string() {
                        assert_eq!(action["target"], TARGET);
                    }
                }
            }
        }
    }
    #[test]
    fn physical_trial_uses_same_runtime_on_virtual_pair_and_joins_blank_writers() {
        let path = std::env::temp_dir().join(format!(
            "decksmith-connected-physical-{}",
            random_device_id().unwrap()
        ));
        let discovery = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("lab-plus")),
            Candidate::new(1, DeviceModel::Mk2, Some("lab-mk2")),
        ])
        .unwrap();
        let catalog = prepare(&path, &discovery).unwrap();
        let mut trial =
            Trial::start(Store::open(&path.join("profiles")).unwrap().load().unwrap()).unwrap();
        let revision = trial
            .runtime
            .discover(Discovery::new(discovery.candidates().to_vec()).unwrap())
            .unwrap();
        for (slot, association) in catalog.associations(&discovery) {
            let decksmith_device::registry::Association::Assigned(id) = association else {
                panic!()
            };
            let model = discovery.candidates()[usize::from(slot)].model();
            trial
                .attach(
                    revision,
                    &discovery,
                    slot,
                    id,
                    Box::new(VirtualDeck::new(model)),
                )
                .unwrap();
        }
        pump(&mut trial, |t| {
            t.runtime
                .statuses()
                .iter()
                .all(|s| s.display_ready && s.input_ready)
        });
        let mk2 = trial
            .runtime
            .statuses()
            .iter()
            .find(|s| s.model == DeviceModel::Mk2)
            .unwrap()
            .id;
        let c = trial.runtime.context(mk2, ControlAddress::Key(0)).unwrap();
        trial.runtime.navigate(c, 1).unwrap();
        pump(&mut trial, |t| {
            t.runtime
                .statuses()
                .iter()
                .all(|s| s.display_ready && s.input_ready)
        });
        assert!(
            trial
                .runtime
                .statuses()
                .iter()
                .all(|s| s.page == Some(if s.id == mk2 { 1 } else { 0 }))
        );
        let blank = trial.runtime.stop();
        assert_eq!(blank.len(), 2);
        assert!(blank.iter().all(|(_, r)| r.is_ok()));
        assert!(
            trial
                .counts
                .values()
                .all(|c| c.lock().unwrap().blank_completed == 1)
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}
