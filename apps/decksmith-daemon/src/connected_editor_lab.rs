//! Fresh isolated live editor acceptance. Real runtime/actors, VirtualDecks and
//! simulated effects only. Neither HID nor native audio/media is opened here.
use crate::{
    device_execution::Runtime,
    device_profiles::{Enrollment, Store, random_device_id},
    pages::Action,
    shared_worker::{Backend, Fence},
};
use decksmith_core::{
    InputEvent, RawEvent,
    device_model::{DeviceDescriptor, DeviceModel},
};
use decksmith_device::{
    DeckDevice, DeviceError, VirtualDeck,
    registry::{Candidate, Discovery},
};
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
struct Probe(Arc<Mutex<VirtualDeck>>);
impl DeckDevice for Probe {
    fn descriptor(&self) -> DeviceDescriptor {
        self.0.lock().unwrap().descriptor()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> std::result::Result<(), DeviceError> {
        self.0.lock().unwrap().set_key_image(index, rgb)
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> std::result::Result<(), DeviceError> {
        self.0.lock().unwrap().set_touch_image(rgb)
    }
    fn set_brightness(&mut self, value: u8) -> std::result::Result<(), DeviceError> {
        self.0.lock().unwrap().set_brightness(value)
    }
    fn poll_event(&mut self) -> std::result::Result<Option<InputEvent>, DeviceError> {
        self.0.lock().unwrap().poll_event()
    }
}
struct Effects(Arc<Mutex<u64>>);
impl Backend for Effects {
    fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
        if !fence.accepts() {
            return Err("stale_action_scope");
        }
        if !matches!(
            action,
            Action::AudioAdjust { .. } | Action::AudioMute { .. }
        ) {
            return Err("fixture_action_unsupported");
        }
        *self.0.lock().unwrap() += 1;
        Ok(())
    }
}
pub fn run(args: &[String]) -> Result<()> {
    let [_, destination, flag, seconds] = args else {
        return Err("connected_editor_lab_arguments");
    };
    let seconds = seconds
        .parse::<u64>()
        .map_err(|_| "connected_editor_lab_arguments")?;
    if flag != "--seconds"
        || !(1..=600).contains(&seconds)
        || std::env::var("DECKSMITH_ISOLATED_TEST").as_deref() != Ok("1")
        || !std::env::var("DBUS_SESSION_BUS_ADDRESS")
            .is_ok_and(|a| a.starts_with("unix:path=/tmp/dbus-"))
    {
        return Err("connected_editor_lab_isolated_only");
    }
    let path = Path::new(destination);
    if !path.is_absolute() {
        return Err("connected_editor_lab_absolute_path_required");
    }
    let store = Store::create(path)?;
    let discovery = Discovery::new(vec![
        Candidate::new(0, DeviceModel::Plus, Some("editor-fixture-plus")),
        Candidate::new(1, DeviceModel::Mk2, Some("editor-fixture-mk2")),
    ])?;
    let plus = random_device_id()?;
    let mk2 = random_device_id()?;
    let generation = store.enroll(
        0,
        &discovery,
        0,
        Enrollment::new(
            plus,
            "Stream Deck + · Live test",
            50,
            Some(include_bytes!("../../../config/audio.json")),
        ),
    )?;
    store.enroll(
        generation,
        &discovery,
        1,
        Enrollment::new(mk2, "Stream Deck MK.2 · Live test", 50, None),
    )?;
    let catalog = store.load()?;
    let mut value: serde_json::Value =
        serde_json::from_slice(catalog.profile(mk2)?.document().bytes())
            .map_err(|_| "invalid_config")?;
    for page in value["pages"].as_array_mut().unwrap() {
        page["keys"][0] = serde_json::json!({"label":"Peer progress","action":{"type":"audio_adjust","target":"output:editor-fixture-peer","percent":1}});
    }
    store.save_profile(
        mk2,
        1,
        &serde_json::to_vec(&value).unwrap(),
        0,
        "Stream Deck MK.2 · Live test",
        50,
    )?;
    let effects = Arc::new(Mutex::new(0));
    let mut runtime = Runtime::start(
        store.load()?,
        vec![Box::new(Effects(effects.clone()))],
        None,
    )?;
    let revision = runtime.discover(Discovery::new(discovery.candidates().to_vec())?)?;
    let a = Arc::new(Mutex::new(VirtualDeck::new(DeviceModel::Plus)));
    let b = Arc::new(Mutex::new(VirtualDeck::new(DeviceModel::Mk2)));
    runtime.attach(revision, &discovery, 0, Box::new(Probe(a.clone())))?;
    runtime.attach(revision, &discovery, 1, Box::new(Probe(b.clone())))?;
    let ready = Instant::now() + Duration::from_secs(3);
    while Instant::now() < ready
        && runtime
            .statuses()
            .iter()
            .any(|s| !s.input_ready || !s.display_ready)
    {
        runtime.tick();
        std::thread::sleep(Duration::from_millis(2));
    }
    if runtime
        .statuses()
        .iter()
        .any(|s| !s.input_ready || !s.display_ready)
    {
        return Err("connected_editor_lab_not_ready");
    }
    let mut service = crate::live_profiles::Service::start(store, &runtime, false)?;
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stop.clone())
            .map_err(|_| "connected_editor_lab_signal_failed")?;
    }
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut next = Instant::now();
    let mut peer_while_pending = 0;
    let mut time = 0;
    while Instant::now() < end && !stop.load(Ordering::Acquire) {
        service.poll(&mut runtime);
        let feedback = runtime.tick();
        if runtime
            .statuses()
            .iter()
            .any(|s| s.id == plus && s.error.as_deref() == Some("profile_update_pending"))
        {
            peer_while_pending += feedback
                .iter()
                .filter(|r| r.scope.device.device == mk2 && r.result.is_ok())
                .count();
        }
        service.publish(&runtime);
        if Instant::now() >= next {
            let mut deck = b.lock().unwrap();
            for pressed in [true, false] {
                time += 1;
                deck.inject(InputEvent {
                    timestamp_ms: time,
                    event: RawEvent::Key { index: 0, pressed },
                })
                .map_err(|_| "fixture_input_invalid")?;
            }
            next = Instant::now() + Duration::from_millis(50);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    drop(service);
    let blank = runtime.stop();
    let result = serde_json::json!({"status":if blank.iter().all(|(_,r)|r.is_ok()){"passed"}else{"failed"},"blanked":blank.len(),"simulated_effects":*effects.lock().unwrap(),"peer_effects_while_other_save_pending":peer_while_pending,"physical_hid":false,"native_audio":false});
    crate::layout_store::write(
        &path.join("live-editor-result.json"),
        &serde_json::to_vec_pretty(&result).unwrap(),
    )?;
    if result["status"] != "passed" {
        return Err("connected_editor_lab_blanking_failed");
    }
    Ok(())
}
