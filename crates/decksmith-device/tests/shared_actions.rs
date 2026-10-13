//! Shared lifecycle fixtures fed by the real model-aware manager/VirtualDeck.
//! No physical opener, helpers, audio commands, desktop bus or installed config.
use decksmith_core::{
    InputEvent, RawEvent,
    device_context::DeviceId,
    device_model::{ControlAddress, DeviceDescriptor, DeviceModel},
    microphone_holds::{DesiredMic, HoldToken, LEASE_MS, MAX_TARGETS, MicrophoneHolds},
    shared_dispatch::{
        ActionScope, Dispatcher, Effect, MAX_INFLIGHT, MAX_PENDING, PER_DEVICE_PENDING, TargetId,
    },
};
use decksmith_device::{
    DeckDevice, DeviceError, VirtualDeck,
    manager::{DeviceManager, SessionLayout},
    registry::{Candidate, Discovery, Registry},
};
use std::{cell::RefCell, rc::Rc, time::Instant};
fn id(n: u8) -> DeviceId {
    DeviceId::from_bytes([n; 16]).unwrap()
}
fn target(n: u8) -> TargetId {
    TargetId::new(&format!("canonical-input-{n}")).unwrap()
}
#[derive(Clone)]
struct Probe(Rc<RefCell<VirtualDeck>>);
impl DeckDevice for Probe {
    fn descriptor(&self) -> DeviceDescriptor {
        self.0.borrow().descriptor()
    }
    fn set_key_image(&mut self, i: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        self.0.borrow_mut().set_key_image(i, rgb)
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        self.0.borrow_mut().set_touch_image(rgb)
    }
    fn set_brightness(&mut self, n: u8) -> Result<(), DeviceError> {
        self.0.borrow_mut().set_brightness(n)
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        self.0.borrow_mut().poll_event()
    }
}
struct Pair {
    manager: DeviceManager,
    probes: [Probe; 2],
    runtime: DeviceId,
    foreground: u64,
}
impl Pair {
    fn new(models: [DeviceModel; 2]) -> Self {
        let discovery = Discovery::new(vec![
            Candidate::new(0, models[0], Some("fixture-A")),
            Candidate::new(1, models[1], Some("fixture-B")),
        ])
        .unwrap();
        let mut registry = Registry::default();
        registry
            .associate(0, id(1), "First", 50, &discovery, 0)
            .unwrap();
        registry
            .associate(1, id(2), "Second", 60, &discovery, 1)
            .unwrap();
        let mut manager = DeviceManager::new(registry).unwrap();
        let revision = manager.reconcile(discovery).unwrap();
        let probes = models.map(|m| Probe(Rc::new(RefCell::new(VirtualDeck::new(m)))));
        for (i, model) in models.iter().enumerate() {
            manager
                .configure(
                    id(i as u8 + 1),
                    0,
                    SessionLayout::new(*model, 3, 0).unwrap(),
                )
                .unwrap();
            manager
                .attach(revision, i as u8, Box::new(probes[i].clone()))
                .unwrap();
        }
        manager.poll_round();
        Self {
            manager,
            probes,
            runtime: id(99),
            foreground: 1,
        }
    }
    fn scope(&self, device: u8, control: ControlAddress) -> ActionScope {
        ActionScope {
            runtime: self.runtime,
            foreground: self.foreground,
            device: self.manager.context(id(device), control).unwrap(),
        }
    }
    fn current(&self, s: &ActionScope) -> bool {
        s.runtime == self.runtime
            && s.foreground == self.foreground
            && self.manager.current(&s.device)
    }
    fn holds(&self) -> MicrophoneHolds {
        MicrophoneHolds::new(self.runtime, &[id(1), id(2)]).unwrap()
    }
    fn queue(&self) -> Dispatcher {
        Dispatcher::new(self.runtime, &[id(1), id(2)]).unwrap()
    }
}
fn token(scope: ActionScope, press: u64) -> HoldToken {
    HoldToken { scope, press }
}

#[test]
fn raw_virtual_edges_create_cross_device_holds_and_last_release_mutes() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut manager = pair.manager;
    let mut holds = MicrophoneHolds::new(pair.runtime, &[id(1), id(2)]).unwrap();
    let mut tokens = Vec::new();
    for pressed in [true, false] {
        for (i, probe) in pair.probes.iter().enumerate() {
            probe
                .0
                .borrow_mut()
                .inject(InputEvent {
                    timestamp_ms: u64::from(!pressed),
                    event: RawEvent::Key { index: 0, pressed },
                })
                .unwrap();
            for received in manager.poll_round() {
                let scope = ActionScope {
                    runtime: pair.runtime,
                    foreground: 1,
                    device: received.context,
                };
                let current = |s: &ActionScope| {
                    s.runtime == pair.runtime && s.foreground == 1 && manager.current(&s.device)
                };
                if pressed {
                    let t = token(scope, i as u64 + 1);
                    holds.acquire(t, target(0), 0, current).unwrap();
                    tokens.push(t);
                } else {
                    holds.release(tokens[i], 100, current).unwrap();
                }
            }
            if pressed {
                assert_eq!(holds.updates()[0].desired, DesiredMic::Live);
            } else if i == 0 {
                assert_eq!(holds.held(), 1);
                assert_eq!(holds.updates()[0].desired, DesiredMic::Live);
            }
        }
    }
    assert_eq!(holds.held(), 0);
    assert_eq!(holds.updates()[0].desired, DesiredMic::Muted);
}
#[test]
fn healthy_device_heartbeat_cannot_refresh_expired_other_owner_or_reopen_it() {
    let pair = Pair::new([DeviceModel::Mk2, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    let a = token(pair.scope(1, ControlAddress::Key(0)), 1);
    let b = token(pair.scope(2, ControlAddress::Key(0)), 1);
    for t in [a, b] {
        holds.acquire(t, target(0), 0, |s| pair.current(s)).unwrap();
    }
    let live = holds.updates().remove(0);
    holds
        .acknowledge(&live, true, 0, |s| pair.current(s))
        .unwrap();
    holds.heartbeat(id(2), 400, |s| pair.current(s)).unwrap();
    holds.maintain(LEASE_MS, |s| pair.current(s)).unwrap();
    assert_eq!(holds.held(), 1);
    assert!(holds.updates().is_empty());
    holds.heartbeat(id(1), 501, |s| pair.current(s)).unwrap();
    assert_eq!(holds.held(), 1);
    assert_eq!(
        holds.acquire(a, target(0), 502, |s| pair.current(s)),
        Err("hold_requires_fresh_press")
    );
    holds.maintain(900, |s| pair.current(s)).unwrap();
    assert_eq!(holds.held(), 0);
    assert_eq!(holds.updates()[0].desired, DesiredMic::Muted);
}
#[test]
fn device_disconnect_page_lock_and_foreground_changes_cancel_only_valid_scope() {
    for mode in 0..4 {
        let mut pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
        let mut holds = pair.holds();
        let a = token(pair.scope(1, ControlAddress::DialPush(0)), 1);
        let b = token(pair.scope(2, ControlAddress::Key(0)), 1);
        holds.acquire(a, target(0), 0, |s| pair.current(s)).unwrap();
        holds.acquire(b, target(0), 0, |s| pair.current(s)).unwrap();
        match mode {
            0 => pair.manager.disconnect(id(1)).unwrap(),
            1 => {
                pair.manager.page(id(1), 1, 1).unwrap();
            }
            2 => pair.manager.gate(true, false).unwrap(),
            _ => pair.foreground += 1,
        }
        holds.maintain(10, |s| pair.current(s)).unwrap();
        assert_eq!(holds.held(), if mode < 2 { 1 } else { 0 });
        if mode < 2 {
            holds.release(b, 11, |s| pair.current(s)).unwrap();
        }
        assert_eq!(holds.updates()[0].desired, DesiredMic::Muted);
        assert!(!holds.release(a, 12, |s| pair.current(s)).unwrap());
    }
}
#[test]
fn explicit_mute_or_guardian_failure_never_replays_cancelled_holds() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    let a = token(pair.scope(1, ControlAddress::Key(0)), 1);
    let b = token(pair.scope(2, ControlAddress::Key(0)), 1);
    holds.acquire(a, target(0), 0, |s| pair.current(s)).unwrap();
    holds.acquire(b, target(1), 0, |s| pair.current(s)).unwrap();
    let live = holds
        .updates()
        .into_iter()
        .find(|u| u.target == target(0))
        .unwrap();
    holds
        .acknowledge(&live, false, 0, |s| pair.current(s))
        .unwrap();
    assert_eq!(holds.held(), 1);
    assert_eq!(
        holds.acquire(a, target(0), 1, |s| pair.current(s)),
        Err("hold_requires_fresh_press")
    );
    assert!(
        holds
            .updates()
            .iter()
            .any(|u| u.target == target(1) && u.desired == DesiredMic::Live)
    );
    holds.cancel_target(&target(1)).unwrap();
    holds.heartbeat(id(2), 2, |s| pair.current(s)).unwrap();
    assert_eq!(holds.held(), 0);
    assert_eq!(
        holds.acquire(b, target(1), 3, |s| pair.current(s)),
        Err("hold_requires_fresh_press")
    );
    assert!(
        holds
            .updates()
            .iter()
            .all(|u| u.desired == DesiredMic::Muted)
    );
    holds
        .acquire(token(a.scope, 2), target(0), 4, |s| pair.current(s))
        .unwrap();
    assert_eq!(holds.held(), 1);
}
#[test]
fn late_live_and_mute_acknowledgements_cannot_change_newer_desired_state() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    let a = token(pair.scope(1, ControlAddress::Key(0)), 1);
    holds.acquire(a, target(0), 0, |s| pair.current(s)).unwrap();
    let live = holds.updates().remove(0);
    holds.release(a, 1, |s| pair.current(s)).unwrap();
    let muted = holds.updates().remove(0);
    assert!(!holds.accepts_update(&live));
    assert!(
        !holds
            .acknowledge(&live, true, 1, |s| pair.current(s))
            .unwrap()
    );
    holds
        .acquire(token(a.scope, 2), target(0), 2, |s| pair.current(s))
        .unwrap();
    assert!(
        !holds
            .acknowledge(&muted, true, 2, |s| pair.current(s))
            .unwrap()
    );
    assert_eq!(holds.updates()[0].desired, DesiredMic::Live);
    holds.stop().unwrap();
    assert_eq!(holds.updates()[0].desired, DesiredMic::Muted);
    assert!(
        holds
            .acquire(token(a.scope, 3), target(0), 3, |s| pair.current(s))
            .is_err()
    );
}
#[test]
fn ordinary_queue_saturation_does_not_delay_microphone_release() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    let mut queue = pair.queue();
    let a = token(pair.scope(1, ControlAddress::Key(0)), 1);
    holds.acquire(a, target(0), 0, |s| pair.current(s)).unwrap();
    for device in 1..=2 {
        for _ in 0..PER_DEVICE_PENDING {
            queue
                .submit(
                    pair.scope(device, ControlAddress::Key(1)),
                    target(1),
                    Effect::Activate,
                    None,
                    |s| pair.current(s),
                )
                .unwrap();
        }
    }
    assert_eq!(queue.pending(), MAX_PENDING);
    assert_eq!(
        queue.submit(a.scope, target(2), Effect::Activate, None, |s| pair
            .current(s)),
        Err("action_queue_full")
    );
    holds.release(a, 1, |s| pair.current(s)).unwrap();
    assert_eq!(holds.updates()[0].desired, DesiredMic::Muted);
    assert_eq!(queue.pending(), MAX_PENDING);
}
#[test]
fn target_budget_and_backwards_clock_fail_closed_without_reopening_old_downs() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    for n in 0..MAX_TARGETS {
        holds
            .acquire(
                token(pair.scope(1, ControlAddress::Key(n as u8)), 1),
                target(n as u8),
                0,
                |s| pair.current(s),
            )
            .unwrap();
    }
    let extra = token(pair.scope(2, ControlAddress::Key(0)), 1);
    assert_eq!(
        holds.acquire(extra, target(9), 1, |s| pair.current(s)),
        Err("hold_target_limit")
    );
    holds.cancel_target(&target(0)).unwrap();
    let mute = holds
        .updates()
        .into_iter()
        .find(|u| u.target == target(0))
        .unwrap();
    holds
        .acknowledge(&mute, true, 1, |s| pair.current(s))
        .unwrap();
    assert_eq!(
        holds.acquire(extra, target(9), 2, |s| pair.current(s)),
        Err("hold_requires_fresh_press")
    );
    holds
        .acquire(token(extra.scope, 2), target(9), 3, |s| pair.current(s))
        .unwrap();
    assert_eq!(
        holds.maintain(2, |s| pair.current(s)),
        Err("hold_time_reversed")
    );
    assert_eq!(holds.held(), 0);
    assert!(
        holds
            .updates()
            .iter()
            .all(|u| u.desired == DesiredMic::Muted)
    );
}
#[test]
fn same_target_fifo_and_start_order_survive_multiple_devices() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut q = pair.queue();
    let a = pair.scope(1, ControlAddress::Key(0));
    let b = pair.scope(2, ControlAddress::Key(0));
    let first = q
        .submit(a, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    let second = q
        .submit(b, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    assert_eq!(q.select(|s| pair.current(s)).unwrap().id, first);
    assert!(q.select(|s| pair.current(s)).is_none());
    q.begin(first, |s| pair.current(s)).unwrap();
    assert_eq!(
        q.begin(first, |s| pair.current(s)).err(),
        Some("action_already_started")
    );
    q.finish(first, Ok(()), |s| pair.current(s)).unwrap();
    assert_eq!(q.select(|s| pair.current(s)).unwrap().id, second);
    q.begin(second, |s| pair.current(s)).unwrap();
    q.finish(second, Ok(()), |s| pair.current(s)).unwrap();
    let x = q
        .submit(a, target(1), Effect::Activate, None, |s| pair.current(s))
        .unwrap();
    q.submit(a, target(2), Effect::Activate, None, |s| pair.current(s))
        .unwrap();
    assert_eq!(q.select(|s| pair.current(s)).unwrap().id, x);
    assert!(q.select(|s| pair.current(s)).is_none());
    q.begin(x, |s| pair.current(s)).unwrap();
    assert!(q.select(|s| pair.current(s)).is_some());
}
#[test]
fn coalescing_keeps_direction_context_receipt_and_cross_device_toggle_barriers() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = pair.scope(1, ControlAddress::DialRotation(0));
    let b = pair.scope(2, ControlAddress::Key(0));
    let mut q = pair.queue();
    let receipt = Instant::now();
    let first = q
        .submit(a, target(0), Effect::Adjust(1), Some(receipt), |s| {
            pair.current(s)
        })
        .unwrap();
    for _ in 1..100 {
        assert_eq!(
            q.submit(a, target(0), Effect::Adjust(1), None, |s| pair.current(s))
                .unwrap(),
            first
        );
    }
    assert_eq!(q.pending(), 1);
    let selected = q.select(|s| pair.current(s)).unwrap();
    assert_eq!(selected.effect, Effect::Adjust(100));
    assert_eq!(selected.submissions, 100);
    assert_eq!(selected.received_at, Some(receipt));
    q.begin(first, |s| pair.current(s)).unwrap();
    q.finish(first, Ok(()), |s| pair.current(s)).unwrap();
    let left = q
        .submit(a, target(0), Effect::Adjust(1), None, |s| pair.current(s))
        .unwrap();
    let toggle = q
        .submit(b, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    let right = q
        .submit(a, target(0), Effect::Adjust(1), None, |s| pair.current(s))
        .unwrap();
    assert_ne!(left, right);
    for expected in [left, toggle, right] {
        assert_eq!(q.select(|s| pair.current(s)).unwrap().id, expected);
        q.begin(expected, |s| pair.current(s)).unwrap();
        q.finish(expected, Ok(()), |s| pair.current(s)).unwrap();
    }
    q.submit(a, target(0), Effect::Adjust(1), None, |s| pair.current(s))
        .unwrap();
    q.submit(a, target(0), Effect::Adjust(-1), None, |s| pair.current(s))
        .unwrap();
    assert_eq!(q.pending(), 2); // Reversals retain clamped-boundary semantics.
}
#[test]
fn stale_started_effect_retains_target_reservation_but_suppresses_feedback() {
    let mut pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = pair.scope(1, ControlAddress::Key(0));
    let b = pair.scope(2, ControlAddress::Key(0));
    let mut q = pair.queue();
    let old = q
        .submit(a, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    q.select(|s| pair.current(s)).unwrap();
    q.begin(old, |s| pair.current(s)).unwrap();
    let other = q
        .submit(b, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    q.submit(b, target(1), Effect::Activate, None, |s| pair.current(s))
        .unwrap();
    pair.manager.disconnect(id(1)).unwrap();
    q.cancel_stale(|s| pair.current(s));
    assert_eq!(q.inflight(), 1);
    assert!(q.select(|s| pair.current(s)).is_none()); // B keeps its own FIFO.
    assert!(
        q.finish(old, Ok(()), |s| pair.current(s))
            .unwrap()
            .is_none()
    );
    assert_eq!(q.select(|s| pair.current(s)).unwrap().id, other);
    assert_eq!(
        q.finish(old, Ok(()), |s| pair.current(s)).err(),
        Some("unknown_action_request")
    );
}
#[test]
fn selection_start_completion_and_provider_failures_each_keep_full_scope() {
    let mut pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = pair.scope(1, ControlAddress::Key(0));
    let b = pair.scope(2, ControlAddress::Key(0));
    let mut q = pair.queue();
    let old = q
        .submit(a, target(0), Effect::Activate, None, |s| pair.current(s))
        .unwrap();
    q.select(|s| pair.current(s)).unwrap();
    pair.manager.page(id(1), 1, 1).unwrap();
    assert_eq!(
        q.begin(old, |s| pair.current(s)).err(),
        Some("stale_action_scope")
    );
    let fresh = q
        .submit(b, target(1), Effect::Activate, None, |s| pair.current(s))
        .unwrap();
    q.select(|s| pair.current(s)).unwrap();
    q.begin(fresh, |s| pair.current(s)).unwrap();
    let feedback = q
        .finish(fresh, Err("provider_unavailable"), |s| pair.current(s))
        .unwrap()
        .unwrap();
    assert_eq!(feedback.scope, b);
    assert_eq!(feedback.id, fresh);
    assert!(feedback.result.is_err());
    let working = q
        .submit(b, target(2), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    assert_eq!(q.select(|s| pair.current(s)).unwrap().id, working);
    q.begin(working, |s| pair.current(s)).unwrap();
    assert!(
        q.finish(working, Ok(()), |s| pair.current(s))
            .unwrap()
            .unwrap()
            .result
            .is_ok()
    );
    let selected = q
        .submit(b, target(2), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    q.select(|s| pair.current(s)).unwrap();
    pair.foreground += 1;
    assert_eq!(
        q.begin(selected, |s| pair.current(s)).err(),
        Some("stale_action_scope")
    );
}
#[test]
fn stop_worker_termination_and_allocation_caps_never_retry_effects() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = pair.scope(1, ControlAddress::Key(0));
    let mut q = pair.queue();
    let old = q
        .submit(a, target(0), Effect::Activate, None, |s| pair.current(s))
        .unwrap();
    q.select(|s| pair.current(s)).unwrap();
    q.begin(old, |s| pair.current(s)).unwrap();
    q.stop();
    assert_eq!(q.inflight(), 1);
    assert!(q.select(|s| pair.current(s)).is_none());
    assert!(
        q.submit(a, target(1), Effect::Activate, None, |s| pair.current(s))
            .is_err()
    );
    q.worker_terminated(old).unwrap();
    assert_eq!(q.inflight(), 0);
    assert!(q.worker_terminated(old).is_err());
    let devices = (1..=8).map(id).collect::<Vec<_>>();
    let mut q = Dispatcher::new(pair.runtime, &devices).unwrap();
    for device in &devices {
        let mut scope = a;
        scope.device.device = *device;
        q.submit(
            scope,
            TargetId::new(&format!("target-{device}")).unwrap(),
            Effect::Activate,
            None,
            |_| true,
        )
        .unwrap();
    }
    for _ in 0..MAX_INFLIGHT {
        assert!(q.select(|_| true).is_some());
    }
    assert!(q.select(|_| true).is_none());
    assert_eq!(q.inflight(), MAX_INFLIGHT);
    assert!(Dispatcher::new(pair.runtime, &[id(1), id(1)]).is_err());
    assert!(Dispatcher::new(pair.runtime, &(1..=9).map(id).collect::<Vec<_>>()).is_err());
}
#[test]
fn startup_nonce_unsupported_controls_and_invalid_effects_are_rejected() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = pair.scope(1, ControlAddress::Key(0));
    let mut q = pair.queue();
    let mut wrong = a;
    wrong.runtime = id(77);
    assert!(
        q.submit(wrong, target(0), Effect::Toggle, None, |_| true)
            .is_err()
    );
    wrong = a;
    wrong.device.device = id(2);
    wrong.device.control = ControlAddress::DialPush(0);
    assert!(
        q.submit(wrong, target(0), Effect::Toggle, None, |s| pair.current(s))
            .is_err()
    );
    for effect in [Effect::Adjust(0), Effect::Adjust(-101), Effect::Adjust(101)] {
        assert!(
            q.submit(a, target(0), effect, None, |s| pair.current(s))
                .is_err()
        );
    }
    for invalid in ["".into(), "\nUnsafe".into(), "x".repeat(513)] {
        assert!(TargetId::new(&invalid).is_err());
    }
    assert_eq!(q.pending(), 0);
    assert!(!format!("{:?}", target(0)).contains("canonical"));
}

#[test]
fn guardian_boundary_checks_current_owners_before_write_and_after_completion() {
    let mut pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    let a = token(pair.scope(1, ControlAddress::Key(0)), 1);
    let b = token(pair.scope(2, ControlAddress::Key(0)), 1);
    holds.acquire(a, target(0), 0, |s| pair.current(s)).unwrap();
    holds.acquire(b, target(0), 0, |s| pair.current(s)).unwrap();
    let live = holds.updates().remove(0);
    pair.manager.disconnect(id(1)).unwrap();
    assert!(holds.prepare_update(&live, 1, |s| pair.current(s)).unwrap()); // B still consents.
    pair.manager.gate(true, false).unwrap();
    assert!(!holds.prepare_update(&live, 2, |s| pair.current(s)).unwrap());
    assert!(
        !holds
            .acknowledge(&live, true, 3, |s| pair.current(s))
            .unwrap()
    );
    let mute = holds.updates().remove(0);
    assert_eq!(mute.desired, DesiredMic::Muted);
    assert!(holds.prepare_update(&mute, 4, |s| pair.current(s)).unwrap());
}
#[test]
fn routed_effects_change_the_selected_target_and_reversals_preserve_limits() {
    use std::collections::BTreeMap;
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut q = pair.queue();
    let a = pair.scope(1, ControlAddress::Key(1));
    let b = pair.scope(2, ControlAddress::Key(1));
    let mut levels = BTreeMap::from([(target(0), 100i16), (target(1), 50i16)]);
    q.submit(a, target(0), Effect::Adjust(1), None, |s| pair.current(s))
        .unwrap();
    q.submit(a, target(0), Effect::Adjust(-1), None, |s| pair.current(s))
        .unwrap();
    q.submit(b, target(1), Effect::Adjust(7), None, |s| pair.current(s))
        .unwrap();
    let mut replies = Vec::new();
    while let Some(selected) = q.select(|s| pair.current(s)) {
        let executing = q.begin(selected.id, |s| pair.current(s)).unwrap();
        let Effect::Adjust(delta) = executing.effect else {
            panic!("fixture scalar adjustment");
        };
        let level = levels.get_mut(&executing.target).unwrap();
        *level = (*level + delta).clamp(0, 100);
        replies.push(
            q.finish(executing.id, Ok(()), |s| pair.current(s))
                .unwrap()
                .unwrap(),
        );
    }
    assert_eq!(levels[&target(0)], 99);
    assert_eq!(levels[&target(1)], 57);
    assert_eq!(
        replies
            .iter()
            .filter(|r| r.scope.device.device == id(1))
            .count(),
        2
    );
    assert_eq!(
        replies
            .iter()
            .filter(|r| r.scope.device.device == id(2))
            .count(),
        1
    );
}

#[test]
fn coordinator_and_dispatch_replacements_reject_reused_counter_callbacks() {
    let pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let a = pair.scope(1, ControlAddress::Key(0));
    let mut first = pair.holds();
    first
        .acquire(token(a, 1), target(0), 0, |s| pair.current(s))
        .unwrap();
    let old_update = first.updates().remove(0);
    first.stop().unwrap();
    let mut next = pair.holds();
    next.acquire(token(a, 1), target(0), 0, |s| pair.current(s))
        .unwrap();
    assert_eq!(old_update.revision, next.updates()[0].revision);
    assert!(
        !next
            .prepare_update(&old_update, 1, |s| pair.current(s))
            .unwrap()
    );
    assert!(
        !next
            .acknowledge(&old_update, false, 2, |s| pair.current(s))
            .unwrap()
    );
    assert_eq!(next.held(), 1);
    let mut old_queue = pair.queue();
    let old_id = old_queue
        .submit(a, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    old_queue.select(|s| pair.current(s));
    old_queue.begin(old_id, |s| pair.current(s)).unwrap();
    old_queue.stop();
    let mut new_queue = pair.queue();
    let new_id = new_queue
        .submit(a, target(0), Effect::Toggle, None, |s| pair.current(s))
        .unwrap();
    new_queue.select(|s| pair.current(s));
    new_queue.begin(new_id, |s| pair.current(s)).unwrap();
    assert_ne!(old_id, new_id);
    assert_eq!(
        new_queue.finish(old_id, Ok(()), |s| pair.current(s)).err(),
        Some("unknown_action_request")
    );
    assert_eq!(new_queue.inflight(), 1);
    assert!(
        new_queue
            .finish(new_id, Ok(()), |s| pair.current(s))
            .unwrap()
            .is_some()
    );
}

#[test]
fn reconnect_hold_cannot_be_released_by_the_previous_connection_token() {
    let mut pair = Pair::new([DeviceModel::Plus, DeviceModel::Mk2]);
    let mut holds = pair.holds();
    let old = token(pair.scope(1, ControlAddress::Key(0)), 1);
    holds
        .acquire(old, target(0), 0, |s| pair.current(s))
        .unwrap();
    pair.manager.disconnect(id(1)).unwrap();
    holds.maintain(1, |s| pair.current(s)).unwrap();
    let discovery = Discovery::new(vec![
        Candidate::new(0, DeviceModel::Mk2, Some("fixture-B")),
        Candidate::new(1, DeviceModel::Plus, Some("fixture-A")),
    ])
    .unwrap();
    let revision = pair.manager.reconcile(discovery).unwrap();
    pair.manager
        .attach(revision, 1, Box::new(pair.probes[0].clone()))
        .unwrap();
    pair.manager.poll_round();
    let fresh = token(pair.scope(1, ControlAddress::Key(0)), 1);
    assert_ne!(old.scope.device.connection, fresh.scope.device.connection);
    holds
        .acquire(fresh, target(0), 2, |s| pair.current(s))
        .unwrap();
    assert!(!holds.release(old, 3, |s| pair.current(s)).unwrap());
    assert_eq!(holds.held(), 1);
    assert_eq!(holds.updates()[0].desired, DesiredMic::Live);
    assert!(holds.release(fresh, 4, |s| pair.current(s)).unwrap());
    assert_eq!(holds.updates()[0].desired, DesiredMic::Muted);
}
