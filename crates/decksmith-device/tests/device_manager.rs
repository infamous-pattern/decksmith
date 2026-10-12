//! Isolated adapter fixtures: no HID enumeration, audio, desktop bus or service.
use decksmith_core::{
    InputEvent, RawEvent,
    device_context::DeviceId,
    device_model::{ControlAddress, DeviceDescriptor, DeviceModel},
};
use decksmith_device::{
    DeckDevice, DeviceError, VirtualDeck,
    manager::{DeviceManager, SessionLayout},
    registry::{
        Association, Candidate, Discovery, MAX_DEVICES, MAX_DISCOVERED, PhysicalIdentity,
        REGISTRY_BYTES, Registry, UnassignedReason,
    },
};
use std::{cell::RefCell, rc::Rc};

fn id(n: u8) -> DeviceId {
    DeviceId::from_bytes([n; 16]).unwrap()
}
fn snapshot(items: &[(u8, DeviceModel, Option<&str>)]) -> Discovery {
    Discovery::new(
        items
            .iter()
            .map(|(slot, model, serial)| Candidate::new(*slot, *model, *serial))
            .collect(),
    )
    .unwrap()
}
fn registry(models: &[DeviceModel]) -> Registry {
    let mut registry = Registry::default();
    for (index, model) in models.iter().enumerate() {
        let discovery = snapshot(&[(index as u8, *model, Some(&format!("fixture-{index}")))]);
        registry
            .associate(
                registry.revision(),
                id(index as u8 + 1),
                &format!("🎛️ Desk {index}"),
                30 + index as u8 * 20,
                &discovery,
                index as u8,
            )
            .unwrap();
    }
    registry
}
#[derive(Clone)]
struct Probe(Rc<RefCell<State>>);
struct State {
    deck: VirtualDeck,
    fail_read: bool,
    fail_write: bool,
    blanked: bool,
    invalid_adapter_input: std::collections::VecDeque<InputEvent>,
}
impl Probe {
    fn new(model: DeviceModel) -> Self {
        Self(Rc::new(RefCell::new(State {
            deck: VirtualDeck::new(model),
            fail_read: false,
            fail_write: false,
            blanked: false,
            invalid_adapter_input: std::collections::VecDeque::new(),
        })))
    }
    fn inject(&self, time: u64, event: RawEvent) {
        self.0
            .borrow_mut()
            .deck
            .inject(InputEvent {
                timestamp_ms: time,
                event,
            })
            .unwrap();
    }
    fn key(&self, time: u64, pressed: bool) {
        self.inject(time, RawEvent::Key { index: 0, pressed });
    }
    fn brightness(&self) -> u8 {
        self.0.borrow().deck.brightness
    }
    fn pixel(&self) -> u8 {
        self.0.borrow().deck.key_image(0).unwrap()[0]
    }
}
impl DeckDevice for Probe {
    fn descriptor(&self) -> DeviceDescriptor {
        self.0.borrow().deck.descriptor()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        let mut s = self.0.borrow_mut();
        if s.fail_write {
            Err(DeviceError::Transport)
        } else {
            s.deck.set_key_image(index, rgb)
        }
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        self.0.borrow_mut().deck.set_touch_image(rgb)
    }
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        self.0.borrow_mut().deck.set_brightness(percent)
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        let mut s = self.0.borrow_mut();
        if s.fail_read {
            Err(DeviceError::Transport)
        } else if let Some(event) = s.invalid_adapter_input.pop_front() {
            Ok(Some(event))
        } else {
            s.deck.poll_event()
        }
    }
    fn blank(&mut self) -> Result<(), DeviceError> {
        self.0.borrow_mut().blanked = true;
        self.0.borrow_mut().deck.blank()
    }
}
fn pair(models: [DeviceModel; 2]) -> (DeviceManager, [Probe; 2]) {
    let mut manager = DeviceManager::new(registry(&models)).unwrap();
    let discovery = snapshot(&[
        (0, models[0], Some("fixture-0")),
        (1, models[1], Some("fixture-1")),
    ]);
    let revision = manager.reconcile(discovery).unwrap();
    let probes = [Probe::new(models[0]), Probe::new(models[1])];
    for (index, model) in models.iter().enumerate() {
        manager
            .configure(
                id(index as u8 + 1),
                0,
                SessionLayout::new(*model, 3, 0).unwrap(),
            )
            .unwrap();
        manager
            .attach(revision, index as u8, Box::new(probes[index].clone()))
            .unwrap();
    }
    assert!(manager.poll_round().is_empty()); // Explicit initial no-ready-input drain.
    (manager, probes)
}

#[test]
fn ids_are_canonical_bounded_and_serde_validated() {
    let original = id(17);
    let text = original.to_string();
    assert_eq!(DeviceId::parse(&text).unwrap(), original);
    assert_eq!(
        serde_json::from_str::<DeviceId>(&format!("\"{text}\"")).unwrap(),
        original
    );
    assert_eq!(
        serde_json::to_string(&original).unwrap(),
        format!("\"{text}\"")
    );
    for invalid in [
        "",
        "00000000000000000000000000000000",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "../profile",
        "1111111111111111111111111111111",
        "🎛️111111111111111111111111111111",
    ] {
        assert!(DeviceId::parse(invalid).is_err());
        assert!(
            serde_json::from_str::<DeviceId>(&serde_json::to_string(invalid).unwrap()).is_err()
        );
    }
}
#[test]
fn explicit_enrollment_and_private_roundtrip_keep_public_serials_out() {
    let mut r = Registry::default();
    let discovery = snapshot(&[(0, DeviceModel::Plus, Some("private-SERIAL"))]);
    let candidate = &discovery.candidates()[0];
    assert_eq!(
        r.resolve(&discovery, candidate),
        Association::Unassigned(UnassignedReason::NewDevice)
    );
    r.associate(0, id(1), "🎛️ Production Desk", 73, &discovery, 0)
        .unwrap();
    let bytes = r.encode_private().unwrap();
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .contains("private-SERIAL")
    );
    let loaded = Registry::decode(&bytes).unwrap();
    assert_eq!(loaded.encode_private().unwrap(), bytes);
    assert_eq!(
        loaded.resolve(&discovery, candidate),
        Association::Assigned(id(1))
    );
    assert_eq!(loaded.get(id(1)).unwrap().name(), "🎛️ Production Desk");
    let public = serde_json::to_string(&loaded.summaries()).unwrap();
    assert!(!public.contains("SERIAL"));
    assert!(!format!("{loaded:?}").contains("SERIAL"));
    assert!(r.associate(0, id(2), "Other", 50, &discovery, 0).is_err());
    assert_eq!(r.encode_private().unwrap(), bytes);
}
#[test]
fn ambiguous_missing_invalid_and_new_identity_never_guess_existing_device() {
    let r = registry(&[DeviceModel::Mk2]);
    for (items, reason) in [
        (
            vec![(0, DeviceModel::Mk2, None)],
            UnassignedReason::MissingIdentity,
        ),
        (
            vec![(0, DeviceModel::Mk2, Some("bad serial"))],
            UnassignedReason::MissingIdentity,
        ),
        (
            vec![
                (0, DeviceModel::Mk2, Some("fixture-0")),
                (1, DeviceModel::Mk2, Some("fixture-0")),
            ],
            UnassignedReason::AmbiguousIdentity,
        ),
        (
            vec![(0, DeviceModel::Plus, Some("fixture-0"))],
            UnassignedReason::NewDevice,
        ),
    ] {
        let d = snapshot(&items);
        for c in d.candidates() {
            assert_eq!(r.resolve(&d, c), Association::Unassigned(reason));
        }
    }
    assert!(PhysicalIdentity::new(DeviceModel::Plus, &"x".repeat(65)).is_err());
    assert!(PhysicalIdentity::new(DeviceModel::Plus, "../device").is_err());
    let d = snapshot(&[(0, DeviceModel::Mk2, Some("fixture-0"))]);
    let foreign = Candidate::new(1, DeviceModel::Mk2, Some("fixture-0"));
    assert!(matches!(
        r.resolve(&d, &foreign),
        Association::Unassigned(_)
    ));
}
#[test]
fn registry_and_discovery_limits_and_stale_edits_are_atomic() {
    let mut r = Registry::default();
    for n in 0..MAX_DEVICES {
        let d = snapshot(&[(0, DeviceModel::Mk2, Some(&format!("fixture-{n}")))]);
        r.associate(r.revision(), id(n as u8 + 1), "Desk", 50, &d, 0)
            .unwrap();
    }
    let bytes = r.encode_private().unwrap();
    let d = snapshot(&[(0, DeviceModel::Mk2, Some("extra"))]);
    assert_eq!(
        r.associate(r.revision(), id(99), "More", 50, &d, 0),
        Err("device_registry_full")
    );
    assert_eq!(r.encode_private().unwrap(), bytes);
    assert!(
        Discovery::new(
            (0..=MAX_DISCOVERED)
                .map(|n| Candidate::new(n as u8, DeviceModel::Mk2, None))
                .collect()
        )
        .is_err()
    );
    assert!(
        Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, None),
            Candidate::new(0, DeviceModel::Mk2, None)
        ])
        .is_err()
    );
    assert!(Registry::decode(&vec![b' '; REGISTRY_BYTES + 1]).is_err());
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["version"] = 99.into();
    assert!(Registry::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    value["version"] = 1.into();
    value["devices"][0]["brightness"] = 101.into();
    assert!(Registry::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    value["devices"][0]["brightness"] = 50.into();
    value["devices"][1] = value["devices"][0].clone();
    assert!(Registry::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut r = Registry::default();
    for (name, brightness) in [("", 50), ("\nUnsafe", 50), ("A", 101)] {
        assert!(r.associate(0, id(1), name, brightness, &d, 0).is_err());
    }
    assert_eq!(r.revision(), 0);
}
#[test]
fn mixed_and_identical_pairs_route_independent_pages_frames_and_brightness() {
    for models in [
        [DeviceModel::Plus, DeviceModel::Mk2],
        [DeviceModel::Mk2, DeviceModel::Mk2],
    ] {
        let (mut manager, probes) = pair(models);
        let a = manager.context(id(1), ControlAddress::Key(0)).unwrap();
        let b = manager.context(id(2), ControlAddress::Key(0)).unwrap();
        assert_eq!(a.connection, b.connection);
        assert_eq!(a.binding, b.binding);
        assert_ne!(a.device, b.device);
        manager
            .write_key(&a, &vec![11; probes[0].descriptor().key_rgb_bytes()])
            .unwrap();
        manager
            .write_key(&b, &vec![22; probes[1].descriptor().key_rgb_bytes()])
            .unwrap();
        manager.page(id(1), 1, 2).unwrap();
        assert!(!manager.current(&a));
        assert!(manager.current(&b));
        assert!(manager.poll_round().is_empty());
        let status = manager.statuses();
        assert_eq!(status[0].page, Some(2));
        assert_eq!(status[1].page, Some(0));
        assert_eq!(probes[0].pixel(), 11);
        assert_eq!(probes[1].pixel(), 22);
        assert_eq!(probes[0].brightness(), 30);
        assert_eq!(probes[1].brightness(), 50);
        assert!(manager.context(id(2), ControlAddress::DialPush(0)).is_err());
        assert!(manager.page(id(1), 1, 0).is_err());
        assert!(manager.page(id(1), 2, 3).is_err());
    }
}
#[test]
fn reconnect_order_retains_association_page_and_invalidates_only_one_generation() {
    let (mut manager, probes) = pair([DeviceModel::Mk2, DeviceModel::Mk2]);
    manager.page(id(1), 1, 2).unwrap();
    manager.poll_round();
    let old = manager.context(id(1), ControlAddress::Key(0)).unwrap();
    let other = manager.context(id(2), ControlAddress::Key(0)).unwrap();
    manager.disconnect(id(1)).unwrap();
    assert!(!manager.current(&old));
    assert!(manager.current(&other));
    let revision = manager
        .reconcile(snapshot(&[
            (0, DeviceModel::Mk2, Some("fixture-1")),
            (1, DeviceModel::Mk2, Some("fixture-0")),
        ]))
        .unwrap();
    assert!(
        manager
            .attach(revision - 1, 1, Box::new(probes[0].clone()))
            .is_err()
    );
    assert!(
        manager
            .attach(revision, 0, Box::new(probes[0].clone()))
            .is_err()
    ); // Already owned unit, not first free slot.
    manager
        .attach(revision, 1, Box::new(probes[0].clone()))
        .unwrap();
    manager.poll_round();
    let restored = manager.context(id(1), ControlAddress::Key(0)).unwrap();
    assert!(restored.connection > old.connection);
    assert!(!manager.current(&old));
    assert!(manager.current(&other));
    assert_eq!(manager.statuses()[0].page, Some(2));
    assert_eq!(probes[0].brightness(), 30);
}
#[test]
fn discovery_ambiguity_fences_affected_owner_and_never_auto_attaches_new_units() {
    let (mut manager, probes) = pair([DeviceModel::Mk2, DeviceModel::Mk2]);
    let other = manager.context(id(2), ControlAddress::Key(0)).unwrap();
    let revision = manager
        .reconcile(snapshot(&[
            (0, DeviceModel::Mk2, Some("fixture-0")),
            (1, DeviceModel::Mk2, Some("fixture-0")),
            (2, DeviceModel::Mk2, Some("fixture-1")),
            (3, DeviceModel::Plus, None),
        ]))
        .unwrap();
    assert!(!manager.statuses()[0].connected);
    assert!(manager.current(&other));
    assert_eq!(
        manager.attach(revision, 0, Box::new(probes[0].clone())),
        Err("device_unassigned")
    );
    assert_eq!(
        manager.attach(revision, 3, Box::new(Probe::new(DeviceModel::Plus))),
        Err("device_unassigned")
    );
    assert_eq!(manager.statuses().len(), 2);
}
#[test]
fn noisy_unit_has_bounded_fair_polling_and_transport_failure_is_isolated() {
    let (mut manager, probes) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    for time in 1..100 {
        probes[0].inject(time, RawEvent::DialRotate { index: 0, ticks: 1 });
    }
    probes[1].key(1, true);
    let events = manager.poll_round();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.received_at.is_none()));
    assert!(events.iter().any(|e| e.context.device == id(2)));
    probes[0].0.borrow_mut().fail_read = true;
    probes[1].key(2, false);
    let events = manager.poll_round();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].context.device, id(2));
    let status = manager.statuses();
    assert!(!status[0].connected);
    assert!(status[0].error.is_some());
    assert!(status[1].input_ready);
}
#[test]
fn lock_resume_and_binding_fences_require_release_then_a_fresh_down() {
    let (mut manager, probes) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    probes[0].key(1, true);
    probes[1].key(1, true);
    let old = manager.poll_round();
    assert_eq!(old.len(), 2);
    manager.gate(true, false).unwrap();
    assert!(old.iter().all(|e| !manager.current(&e.context)));
    assert!(manager.page(id(1), 1, 1).is_err());
    assert_eq!(
        manager.configure(
            id(1),
            1,
            SessionLayout::new(DeviceModel::Plus, 3, 0).unwrap()
        ),
        Err("devices_blocked")
    );
    probes[0].key(2, true);
    probes[1].key(2, false);
    assert!(manager.poll_round().is_empty());
    manager.gate(false, false).unwrap();
    assert!(manager.poll_round().is_empty());
    probes[0].key(3, true);
    probes[1].key(3, true);
    let events = manager.poll_round();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].context.device, id(2));
    probes[0].key(4, false);
    assert!(manager.poll_round().is_empty());
    probes[0].key(5, true);
    assert_eq!(manager.poll_round().len(), 1);
    manager.page(id(1), 1, 1).unwrap();
    probes[0].key(6, false);
    assert!(manager.poll_round().is_empty());
    probes[0].key(7, true);
    assert_eq!(manager.poll_round().len(), 1);
    manager.gate(false, true).unwrap();
    probes[0].inject(
        8,
        RawEvent::DialRotate {
            index: 0,
            ticks: 20,
        },
    );
    manager.gate(false, false).unwrap();
    assert!(manager.poll_round().is_empty()); // No queued pre-resume rotation replay.
    probes[0].inject(9, RawEvent::DialRotate { index: 0, ticks: 1 });
    assert_eq!(manager.poll_round().len(), 1);
    assert_eq!(manager.statuses()[0].page, Some(1));
}
#[test]
fn long_initial_drain_does_not_starve_other_device_or_dispatch_stale_inputs() {
    let (mut manager, probes) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    manager.disconnect(id(1)).unwrap();
    for time in 1..100 {
        probes[0].inject(time, RawEvent::DialRotate { index: 0, ticks: 1 });
    }
    let revision = manager
        .reconcile(snapshot(&[
            (0, DeviceModel::Plus, Some("fixture-0")),
            (1, DeviceModel::Mk2, Some("fixture-1")),
        ]))
        .unwrap();
    manager
        .attach(revision, 0, Box::new(probes[0].clone()))
        .unwrap();
    for time in 1..=4 {
        probes[1].key(time, time % 2 == 1);
        let events = manager.poll_round();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].context.device, id(2));
    }
    assert!(manager.statuses()[0].input_ready);
    probes[0].inject(100, RawEvent::DialRotate { index: 0, ticks: 1 });
    assert_eq!(manager.poll_round().len(), 1);
}
#[test]
fn invalid_feedback_and_write_failures_do_not_touch_another_owner() {
    let (mut manager, probes) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = manager.context(id(1), ControlAddress::Key(0)).unwrap();
    let b = manager.context(id(2), ControlAddress::Key(0)).unwrap();
    assert_eq!(manager.write_key(&a, &[1]), Err("invalid_key_image"));
    assert!(manager.current(&a));
    probes[0].0.borrow_mut().fail_write = true;
    assert_eq!(
        manager.write_key(&a, &vec![1; 120 * 120 * 3]),
        Err("device_write_failed")
    );
    assert!(!manager.current(&a));
    assert!(manager.current(&b));
    manager.write_key(&b, &vec![22; 72 * 72 * 3]).unwrap();
    assert_eq!(probes[1].pixel(), 22);
    assert_eq!(
        manager.write_key(&a, &vec![1; 120 * 120 * 3]),
        Err("stale_device_context")
    );
}
#[test]
fn model_mismatch_empty_manager_and_best_effort_pair_blanking() {
    let mut empty = DeviceManager::new(Registry::default()).unwrap();
    assert!(empty.poll_round().is_empty());
    assert!(empty.stop().is_empty());
    let mut m = DeviceManager::new(registry(&[DeviceModel::Mk2])).unwrap();
    let revision = m
        .reconcile(snapshot(&[(0, DeviceModel::Mk2, Some("fixture-0"))]))
        .unwrap();
    assert_eq!(
        m.attach(revision, 0, Box::new(Probe::new(DeviceModel::Plus))),
        Err("device_model_mismatch")
    );
    assert_eq!(
        m.configure(
            id(1),
            0,
            SessionLayout::new(DeviceModel::Plus, 3, 0).unwrap()
        ),
        Err("layout_model_mismatch")
    );
    for (pages, default) in [(0, 0), (17, 0), (3, 3)] {
        assert!(SessionLayout::new(DeviceModel::Mk2, pages, default).is_err());
    }
    let (mut manager, probes) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    for (index, p) in probes.iter().enumerate() {
        let c = manager
            .context(id(index as u8 + 1), ControlAddress::Key(0))
            .unwrap();
        manager
            .write_key(&c, &vec![44; p.descriptor().key_rgb_bytes()])
            .unwrap();
    }
    probes[0].0.borrow_mut().deck.disconnect();
    assert_eq!(manager.stop(), vec![id(1)]);
    assert!(probes.iter().all(|p| p.0.borrow().blanked));
    assert_eq!(probes[1].pixel(), 0);
    assert!(
        manager
            .statuses()
            .iter()
            .all(|s| !s.connected && !s.input_ready)
    );
    assert_eq!(probes[1].brightness(), 50);
}

#[test]
fn malformed_adapter_reports_and_reversed_time_close_only_their_owner() {
    use decksmith_core::TouchGesture;
    for (model, event) in [
        (
            DeviceModel::Mk2,
            RawEvent::Key {
                index: 15,
                pressed: true,
            },
        ),
        (
            DeviceModel::Mk2,
            RawEvent::DialPush {
                index: 0,
                pressed: true,
            },
        ),
        (
            DeviceModel::Mk2,
            RawEvent::Touch {
                x: 0,
                y: 0,
                gesture: TouchGesture::Tap,
            },
        ),
        (
            DeviceModel::Plus,
            RawEvent::DialRotate { index: 0, ticks: 0 },
        ),
        (
            DeviceModel::Plus,
            RawEvent::Touch {
                x: 800,
                y: 0,
                gesture: TouchGesture::Tap,
            },
        ),
        (
            DeviceModel::Plus,
            RawEvent::Key {
                index: 0,
                pressed: false,
            },
        ), // Earlier than accepted input below.
    ] {
        let (mut manager, probes) = pair([model, DeviceModel::Mk2]);
        probes[0].key(10, true);
        assert_eq!(manager.poll_round().len(), 1);
        probes[0]
            .0
            .borrow_mut()
            .invalid_adapter_input
            .push_back(InputEvent {
                timestamp_ms: if matches!(
                    event,
                    RawEvent::Key {
                        index: 0,
                        pressed: false
                    }
                ) {
                    9
                } else {
                    11
                },
                event,
            });
        probes[1].key(11, true);
        let events = manager.poll_round();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].context.device, id(2));
        let status = manager.statuses();
        assert!(!status[0].connected);
        assert_eq!(status[0].error.as_deref(), Some("invalid device report"));
        assert!(status[1].input_ready);
    }
}

#[test]
fn replacing_a_supervisor_cannot_accept_an_old_local_callback() {
    let (mut previous, _) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    let old = previous.context(id(1), ControlAddress::Key(0)).unwrap();
    previous.stop();
    let (next, _) = pair([DeviceModel::Plus, DeviceModel::Mk2]);
    let current = next.context(id(1), ControlAddress::Key(0)).unwrap();
    assert_eq!(old.device, current.device);
    assert_eq!(old.connection, current.connection);
    assert_eq!(old.binding, current.binding);
    assert_ne!(old.supervisor, current.supervisor);
    assert!(!next.current(&old));
}
