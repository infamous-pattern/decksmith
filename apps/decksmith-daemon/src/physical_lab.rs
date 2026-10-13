//! Explicit bounded hardware qualification. No production files, bus service or
//! desktop/audio/plugin effects; the external handoff guard restores V1 ownership.
use crate::{
    device_profiles::{Controls, Enrollment, Store, random_device_id},
    layout_document::Document,
    layout_store as disk,
    pages::Action,
    profile_control::{Frame, Renderer},
};
use decksmith_core::{
    RawEvent,
    device_context::DeviceId,
    device_model::{DeviceDescriptor, DeviceModel},
};
use decksmith_device::manager::ReceivedInput;
use serde::Serialize;
#[cfg(feature = "hardware")]
use std::time::{Duration, Instant};
use std::{collections::BTreeMap, path::Path};
type Result<T> = std::result::Result<T, &'static str>;

fn layout(model: DeviceModel, pair: bool) -> Result<Document> {
    let count = DeviceDescriptor::for_model(model).key_count();
    let pages=(0..2).map(|page| {
        let keys=(0..count).map(|index| {
            let (label,action)=if pair && index==count-1 {("Next".into(),serde_json::json!({"type":"next_page"}))}
                else if pair && index==count-2 {("Prev".into(),serde_json::json!({"type":"previous_page"}))}
                else if pair && index==count-3 {("Bright".into(),serde_json::json!({"type":"brightness_adjust","percent":5}))}
                else if pair && index==count-4 {("Dim".into(),serde_json::json!({"type":"brightness_adjust","percent":-5}))}
                else {((index+1).to_string(),serde_json::json!({"type":"none"}))};
            serde_json::json!({"label":label,"action":action})
        }).collect::<Vec<_>>();
        serde_json::json!({"name":if page==0 {"Copper A"}else{"Blue B"},"background":if page==0 {[105,55,25]}else{[35,65,105]},"keys":keys})
    }).collect::<Vec<_>>();
    let mut value = serde_json::json!({"version":2,"model":model,"pages":pages});
    if model == DeviceModel::Plus {
        value["dials"]=serde_json::json!((0..4).map(|_|serde_json::json!({"label":"Test brightness","rotation":"brightness","step":1,"press":{"type":"none"}})).collect::<Vec<_>>());
    }
    Document::parse(&serde_json::to_vec(&value).map_err(|_| "invalid_test_layout")?)
}
fn corners(keys: &mut [u8], descriptor: &DeviceDescriptor) {
    let (width, height) = descriptor.geometry().key_pixels;
    let width = usize::from(width);
    let height = usize::from(height);
    for key in keys.chunks_exact_mut(descriptor.key_rgb_bytes()) {
        for y in 0..height {
            for x in 0..width {
                let color = if x < 8 && y < 8 {
                    Some([230, 50, 50])
                } else if x >= width - 8 && y < 8 {
                    Some([40, 210, 80])
                } else if x < 8 && y >= height - 8 {
                    Some([50, 100, 240])
                } else if x >= width - 8 && y >= height - 8 {
                    Some([240, 210, 40])
                } else {
                    None
                };
                if let Some(color) = color {
                    let at = (y * width + x) * 3;
                    key[at..at + 3].copy_from_slice(&color);
                }
            }
        }
    }
}
#[derive(Serialize)]
struct Counts {
    model: DeviceModel,
    down: Vec<u64>,
    up: Vec<u64>,
    dial_turns: u64,
    dial_pushes: u64,
    touch: u64,
    connections: u64,
    disconnects: u64,
}
struct Unit {
    descriptor: DeviceDescriptor,
    renderer: Renderer,
    frame: Frame,
    counts: Counts,
    painted_connection: u64,
    painted_binding: u64,
}
struct Trial {
    controls: Controls,
    units: BTreeMap<DeviceId, Unit>,
}
impl Drop for Trial {
    fn drop(&mut self) {
        self.controls.stop();
    }
}
impl Trial {
    fn paint(&mut self, id: DeviceId) -> Result<()> {
        let context = self
            .controls
            .manager()
            .context(id, decksmith_core::device_model::ControlAddress::Key(0))?;
        let status = self
            .controls
            .manager()
            .statuses()
            .into_iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")?;
        let unit = self.units.get_mut(&id).ok_or("unknown_device")?;
        unit.frame = unit.renderer.frame(
            status.page.ok_or("device_layout_unassigned")?,
            status.brightness,
        )?;
        corners(&mut unit.frame.keys, &unit.descriptor);
        self.controls
            .paint(context, &unit.frame.keys, unit.frame.touch.as_deref())?;
        unit.painted_connection = status.connection;
        unit.painted_binding = status.binding;
        Ok(())
    }
    fn input(&mut self, event: ReceivedInput, pair: bool) -> Result<()> {
        let id = event.context.device;
        let unit = self.units.get_mut(&id).ok_or("unknown_device")?;
        match event.input.event {
            RawEvent::Key { index, pressed } => {
                let counters = if pressed {
                    &mut unit.counts.down
                } else {
                    &mut unit.counts.up
                };
                let c = counters
                    .get_mut(usize::from(index))
                    .ok_or("invalid_test_key")?;
                *c = c.saturating_add(1);
                let len = unit.descriptor.key_rgb_bytes();
                let start = usize::from(index) * len;
                let base = unit
                    .frame
                    .keys
                    .get(start..start + len)
                    .ok_or("test_frame_unavailable")?;
                if pressed {
                    let mut image = base.to_vec();
                    let width = usize::from(unit.descriptor.geometry().key_pixels.0);
                    for y in 10..width - 10 {
                        for x in 10..width - 10 {
                            if x < 12 || x >= width - 12 || y < 12 || y >= width - 12 {
                                let at = (y * width + x) * 3;
                                image[at..at + 3].copy_from_slice(&[40, 240, 100]);
                            }
                        }
                    }
                    self.controls.paint_key(event.context, &image)?;
                } else {
                    self.controls.paint_key(event.context, base)?;
                }
            }
            RawEvent::DialRotate { .. } => {
                unit.counts.dial_turns = unit.counts.dial_turns.saturating_add(1)
            }
            RawEvent::DialPush { .. } => {
                unit.counts.dial_pushes = unit.counts.dial_pushes.saturating_add(1)
            }
            RawEvent::Touch { .. } => unit.counts.touch = unit.counts.touch.saturating_add(1),
        }
        let action = if pair {
            self.controls.resolve(&event)?
        } else {
            match event.input.event {
                RawEvent::Key {
                    index: 13,
                    pressed: false,
                } => Some(Action::BrightnessAdjust { percent: -5 }),
                RawEvent::Key {
                    index: 14,
                    pressed: false,
                } => Some(Action::BrightnessAdjust { percent: 5 }),
                _ => None,
            }
        };
        if let Some(action) = action {
            match action {
                Action::GoToPage { page } => {
                    self.controls.navigate(event.context, page)?;
                }
                Action::BrightnessAdjust { percent } => {
                    let status = self
                        .controls
                        .manager()
                        .statuses()
                        .into_iter()
                        .find(|s| s.id == id)
                        .ok_or("unknown_device")?;
                    let brightness = (i16::from(status.brightness) + percent).clamp(0, 100) as u8;
                    self.controls.brightness(event.context, brightness)?;
                    self.paint(id)?;
                }
                _ => return Err("unsafe_test_action_rejected"),
            }
        }
        Ok(())
    }
    fn report(&self, path: &Path, status: &str) -> Result<()> {
        let bytes=serde_json::to_vec_pretty(&serde_json::json!({"mode":"physical-hardware-lab","status":status,
            "sessions":self.controls.manager().statuses(),"counts":self.units.iter().map(|(id,u)|(id,&u.counts)).collect::<BTreeMap<_,_>>() }))
            .map_err(|_|"physical_report_failed")?;
        let tmp = path.with_extension("tmp");
        disk::write(&tmp, &bytes)?;
        std::fs::rename(tmp, path).map_err(|_| "physical_report_failed")
    }
}

#[cfg(feature = "hardware")]
pub fn run(args: &[String]) -> Result<()> {
    use decksmith_device::hardware::{HardwareOwner, Inventory};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let [_, destination, mode_flag, mode, time_flag, seconds] = args else {
        return Err("physical_lab_arguments");
    };
    let pair = match mode.as_str() {
        "mk2" => false,
        "pair" => true,
        _ => return Err("physical_lab_arguments"),
    };
    let seconds = seconds
        .parse::<u64>()
        .map_err(|_| "physical_lab_arguments")?;
    if mode_flag != "--mode"
        || time_flag != "--seconds"
        || !(1..=600).contains(&seconds)
        || std::env::var("DECKSMITH_PHYSICAL_TEST").as_deref() != Ok("1")
    {
        return Err("physical_lab_explicit_only");
    }
    let path = Path::new(destination);
    if !path.is_absolute() {
        return Err("physical_lab_absolute_path_required");
    }
    let stopping = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stopping.clone())
            .map_err(|_| "physical_lab_signal_failed")?;
    }
    let mut owner = HardwareOwner::acquire().map_err(|_| "physical_ownership_unavailable")?;
    let inventory = Inventory::discover().map_err(|_| "physical_discovery_failed")?;
    let mk2: Vec<_> = inventory
        .discovery()
        .candidates()
        .iter()
        .filter(|c| c.model() == DeviceModel::Mk2)
        .collect();
    let plus: Vec<_> = inventory
        .discovery()
        .candidates()
        .iter()
        .filter(|c| c.model() == DeviceModel::Plus)
        .collect();
    if mk2.len() != 1
        || (pair && (plus.len() != 1 || inventory.discovery().candidates().len() != 2))
    {
        return Err("physical_test_pair_ambiguous");
    }
    let selected = if pair {
        vec![mk2[0], plus[0]]
    } else {
        vec![mk2[0]]
    };
    disk::mkdir(path)?;
    let store = Store::create(&path.join("profiles"))?;
    let mut identities = Vec::new();
    let mut generation = 0;
    for candidate in selected {
        let id = random_device_id()?;
        let model = candidate.model();
        store.enroll(
            generation,
            inventory.discovery(),
            candidate.slot(),
            Enrollment::new(id, "Hardware test", 50, None),
        )?;
        let catalog = store.load()?;
        let document = layout(model, pair)?;
        generation = store.save_profile(
            id,
            catalog.profile(id)?.revision,
            document.bytes(),
            0,
            if model == DeviceModel::Mk2 {
                "MK.2 test"
            } else {
                "Plus test"
            },
            50,
        )?;
        identities.push((id, model, candidate.slot()));
    }
    let catalog = store.load()?;
    let controls = Controls::from_catalog(&catalog)?;
    let mut trial = Trial {
        controls,
        units: BTreeMap::new(),
    };
    for (id, model, _) in &identities {
        let descriptor = DeviceDescriptor::for_model(*model);
        let count = usize::from(descriptor.key_count());
        trial.units.insert(
            *id,
            Unit {
                renderer: Renderer::new(catalog.profile(*id)?.document().bytes(), *model)?,
                descriptor,
                frame: Frame {
                    keys: Vec::new(),
                    touch: None,
                },
                counts: Counts {
                    model: *model,
                    down: vec![0; count],
                    up: vec![0; count],
                    dial_turns: 0,
                    dial_pushes: 0,
                    touch: 0,
                    connections: 0,
                    disconnects: 0,
                },
                painted_connection: 0,
                painted_binding: 0,
            },
        );
    }
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut refresh = Instant::now() - Duration::from_secs(2);
    let mut report = Instant::now() - Duration::from_secs(2);
    let report_path = path.join("status.json");
    let result: Result<()> = (|| {
        while Instant::now() < end && !stopping.load(Ordering::Acquire) {
            let before = trial.controls.manager().statuses();
            if refresh.elapsed() >= Duration::from_secs(1) {
                let current = Inventory::discover().map_err(|_| "physical_discovery_failed")?;
                let revision = trial.controls.discover_hardware(&current)?;
                let associations = catalog.associations(current.discovery());
                for (slot, association) in associations {
                    let decksmith_device::registry::Association::Assigned(id) = association else {
                        continue;
                    };
                    if trial
                        .controls
                        .manager()
                        .statuses()
                        .iter()
                        .any(|s| s.id == id && !s.connected)
                    {
                        trial
                            .controls
                            .attach_physical(revision, &current, slot, &mut owner)?;
                        let unit = trial.units.get_mut(&id).ok_or("unknown_device")?;
                        unit.counts.connections = unit.counts.connections.saturating_add(1);
                    }
                }
                refresh = Instant::now();
            }
            let events = trial.controls.poll_round();
            for status in trial.controls.manager().statuses() {
                if status.input_ready
                    && trial.units.get(&status.id).is_some_and(|u| {
                        u.painted_connection != status.connection
                            || u.painted_binding != status.binding
                    })
                    && let Err(code) = trial.paint(status.id)
                    && code != "device_frame_failed"
                {
                    return Err(code);
                }
            }
            for event in events {
                if !trial.controls.manager().current(&event.context) {
                    continue;
                }
                if let Err(code) = trial.input(event, pair)
                    && !matches!(
                        code,
                        "device_write_failed" | "device_frame_failed" | "device_brightness_failed"
                    )
                {
                    return Err(code);
                }
            }
            for s in trial.controls.manager().statuses() {
                if !s.connected && before.iter().any(|old| old.id == s.id && old.connected) {
                    let c = &mut trial
                        .units
                        .get_mut(&s.id)
                        .ok_or("unknown_device")?
                        .counts
                        .disconnects;
                    *c = c.saturating_add(1);
                }
            }
            if report.elapsed() >= Duration::from_millis(250) {
                trial.report(&report_path, "running")?;
                report = Instant::now();
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    })();
    let failed = trial.controls.stop();
    trial.report(
        &report_path,
        if result.is_ok() && failed.is_empty() {
            "completed"
        } else {
            "failed"
        },
    )?;
    result?;
    if !failed.is_empty() {
        return Err("physical_test_blank_failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_core::InputEvent;
    use decksmith_core::device_model::ControlAddress;
    use decksmith_device::{
        VirtualDeck,
        registry::{Candidate, Discovery},
    };
    #[test]
    fn diagnostic_and_pair_documents_have_only_scoped_navigation_and_brightness() {
        for model in [DeviceModel::Plus, DeviceModel::Mk2] {
            for pair in [false, true] {
                let doc = layout(model, pair).unwrap();
                let mut r = Renderer::new(doc.bytes(), model).unwrap();
                let mut frame = r.frame(0, 50).unwrap();
                corners(&mut frame.keys, &doc.descriptor());
                assert_eq!(
                    frame.keys.len(),
                    doc.descriptor().key_rgb_bytes() * usize::from(doc.descriptor().key_count())
                );
                let config = crate::pages::Config::parse(doc.bytes())
                    .unwrap()
                    .model_controls();
                for index in 0..doc.descriptor().key_count() {
                    let a = config.resolve(
                        0,
                        &RawEvent::Key {
                            index,
                            pressed: false,
                        },
                    );
                    assert!(matches!(
                        a,
                        None | Some(Action::GoToPage { .. })
                            | Some(Action::BrightnessAdjust { .. })
                    ));
                }
                assert_eq!(&frame.keys[..3], &[230, 50, 50]);
            }
        }
    }
    #[test]
    fn paired_test_effects_remain_device_scoped_and_shutdown_blanks_both() {
        let path = std::env::temp_dir().join(format!(
            "physical-lab-fixture-{}",
            random_device_id().unwrap()
        ));
        let store = Store::create(&path).unwrap();
        let discovery = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("test-plus")),
            Candidate::new(1, DeviceModel::Mk2, Some("test-mk2")),
        ])
        .unwrap();
        let ids = [random_device_id().unwrap(), random_device_id().unwrap()];
        let mut g = 0;
        for (index, model) in [DeviceModel::Plus, DeviceModel::Mk2]
            .into_iter()
            .enumerate()
        {
            store
                .enroll(
                    g,
                    &discovery,
                    index as u8,
                    Enrollment::new(ids[index], "Test", 50, None),
                )
                .unwrap();
            g = store
                .save_profile(
                    ids[index],
                    1,
                    layout(model, true).unwrap().bytes(),
                    0,
                    "Test",
                    50,
                )
                .unwrap();
        }
        let catalog = store.load().unwrap();
        let mut trial = Trial {
            controls: Controls::from_catalog(&catalog).unwrap(),
            units: BTreeMap::new(),
        };
        let revision = trial.controls.discover(discovery).unwrap();
        for (index, model) in [DeviceModel::Plus, DeviceModel::Mk2]
            .into_iter()
            .enumerate()
        {
            trial
                .controls
                .attach(revision, index as u8, Box::new(VirtualDeck::new(model)))
                .unwrap();
            let descriptor = DeviceDescriptor::for_model(model);
            let count = usize::from(descriptor.key_count());
            trial.units.insert(
                ids[index],
                Unit {
                    descriptor,
                    renderer: Renderer::new(
                        catalog.profile(ids[index]).unwrap().document().bytes(),
                        model,
                    )
                    .unwrap(),
                    frame: Frame {
                        keys: vec![],
                        touch: None,
                    },
                    counts: Counts {
                        model,
                        down: vec![0; count],
                        up: vec![0; count],
                        dial_turns: 0,
                        dial_pushes: 0,
                        touch: 0,
                        connections: 0,
                        disconnects: 0,
                    },
                    painted_connection: 0,
                    painted_binding: 0,
                },
            );
        }
        trial.controls.poll_round();
        for id in ids {
            trial.paint(id).unwrap();
        }
        let event = |trial: &Trial, id, index| ReceivedInput {
            context: trial
                .controls
                .manager()
                .context(id, ControlAddress::Key(index))
                .unwrap(),
            input: InputEvent {
                timestamp_ms: 1,
                event: RawEvent::Key {
                    index,
                    pressed: false,
                },
            },
            received_at: None,
        };
        trial.input(event(&trial, ids[1], 14), true).unwrap();
        trial.controls.poll_round();
        trial.paint(ids[1]).unwrap();
        assert_eq!(trial.controls.manager().active_page(ids[1]).unwrap(), 1);
        assert_eq!(trial.controls.manager().active_page(ids[0]).unwrap(), 0);
        trial.input(event(&trial, ids[1], 12), true).unwrap();
        let status = trial.controls.manager().statuses();
        assert_eq!(status[0].brightness, 50);
        assert_eq!(status[1].brightness, 55);
        trial.report(&path.join("report.json"), "fixture").unwrap();
        assert!(
            !std::fs::read_to_string(path.join("report.json"))
                .unwrap()
                .contains("test-plus")
        );
        assert!(trial.controls.stop().is_empty());
        drop(trial);
        std::fs::remove_dir_all(path).unwrap();
    }
}
