use decksmith_core::{
    InputEvent,
    device_context::DeviceId,
    device_model::{DeviceDescriptor, DeviceModel},
};
use decksmith_device::{DeckDevice, DeviceError, actor::Pool};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
#[derive(Default)]
struct Data {
    input: VecDeque<InputEvent>,
    writes: Vec<(u8, u8)>,
    blank: usize,
    fail: bool,
}
#[derive(Clone)]
struct Probe {
    model: DeviceModel,
    data: Arc<Mutex<Data>>,
    stall: Arc<(Mutex<bool>, Condvar)>,
    entered: Arc<AtomicBool>,
}
impl Probe {
    fn new(model: DeviceModel) -> Self {
        Self {
            model,
            data: Arc::default(),
            stall: Arc::default(),
            entered: Arc::default(),
        }
    }
    fn release(&self) {
        *self.stall.0.lock().unwrap() = false;
        self.stall.1.notify_all();
    }
}
impl DeckDevice for Probe {
    fn descriptor(&self) -> DeviceDescriptor {
        DeviceDescriptor::for_model(self.model)
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        let mut s = self.stall.0.lock().unwrap();
        if *s {
            self.entered.store(true, Ordering::Release)
        }
        while *s {
            s = self.stall.1.wait(s).unwrap();
        }
        drop(s);
        let mut d = self.data.lock().unwrap();
        if d.fail {
            return Err(DeviceError::Transport);
        }
        d.writes.push((index, rgb[0]));
        Ok(())
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        self.data.lock().unwrap().writes.push((255, rgb[0]));
        Ok(())
    }
    fn set_brightness(&mut self, _: u8) -> Result<(), DeviceError> {
        Ok(())
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        Ok(self.data.lock().unwrap().input.pop_front())
    }
    fn blank(&mut self) -> Result<(), DeviceError> {
        self.data.lock().unwrap().blank += 1;
        Ok(())
    }
}
struct Unblock(Probe);
impl Drop for Unblock {
    fn drop(&mut self) {
        self.0.release();
    }
}
fn id(n: u8) -> DeviceId {
    DeviceId::from_bytes([n; 16]).unwrap()
}
fn until(mut f: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(3);
    while !f() {
        assert!(Instant::now() < end, "fixture deadline");
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn frame(d: &mut impl DeckDevice, value: u8) {
    let desc = d.descriptor();
    d.queue_frame(
        &vec![value; desc.key_rgb_bytes() * usize::from(desc.key_count())],
        desc.capabilities()
            .touch
            .map(|t| vec![value; t.rgb_bytes()])
            .as_deref(),
    )
    .unwrap();
}
#[test]
fn blocked_unit_cannot_block_another_or_claim_queued_frame_completed() {
    let a = Probe::new(DeviceModel::Mk2);
    let b = Probe::new(DeviceModel::Plus);
    *a.stall.0.lock().unwrap() = true;
    let mut pool = Pool::default();
    let _unblock = Unblock(a.clone());
    let mut da = pool.spawn(id(1), Box::new(a.clone())).unwrap();
    let mut db = pool.spawn(id(2), Box::new(b.clone())).unwrap();
    frame(&mut da, 1);
    frame(&mut db, 2);
    until(|| a.entered.load(Ordering::Acquire));
    until(|| db.display_ready());
    assert!(!da.display_ready());
    assert!(b.data.lock().unwrap().writes.len() >= 9);
    assert!(pool.spawn(id(1), Box::new(a.clone())).is_err());
    drop(da);
    assert!(pool.spawn(id(1), Box::new(a.clone())).is_err());
    a.release();
    until(|| !pool.reap().is_empty());
    assert_eq!(a.data.lock().unwrap().blank, 1);
    assert!(db.display_ready());
    drop(db);
    assert!(pool.join().iter().all(|(_, r)| r.is_ok()));
}
#[test]
fn latest_frame_supersedes_fenced_write_and_actual_poll_acknowledges_new_drain() {
    let p = Probe::new(DeviceModel::Mk2);
    let mut pool = Pool::default();
    let _unblock = Unblock(p.clone());
    let mut d = pool.spawn(id(1), Box::new(p.clone())).unwrap();
    until(|| d.input_drained());
    frame(&mut d, 1);
    until(|| d.display_ready());
    p.data.lock().unwrap().writes.clear();
    *p.stall.0.lock().unwrap() = true;
    frame(&mut d, 2);
    until(|| p.entered.load(Ordering::Acquire));
    let lease = d.lease();
    let old = lease.epoch();
    d.invalidate_outputs();
    assert!(!lease.accepts(old));
    assert!(!d.input_drained());
    frame(&mut d, 3);
    frame(&mut d, 4);
    assert!(!d.display_ready());
    p.release();
    until(|| d.display_ready() && d.input_drained());
    let writes = p.data.lock().unwrap().writes.clone();
    assert_eq!(writes.last().map(|(_, value)| *value), Some(4));
    assert!(!writes.iter().any(|(_, v)| *v == 3));
    assert_eq!(writes.iter().filter(|(_, v)| *v == 4).count(), 15);
}
#[test]
fn malformed_frame_and_failed_transport_are_owner_local() {
    let p = Probe::new(DeviceModel::Mk2);
    let q = Probe::new(DeviceModel::Plus);
    let mut pool = Pool::default();
    let mut d = pool.spawn(id(1), Box::new(p.clone())).unwrap();
    let mut other = pool.spawn(id(2), Box::new(q.clone())).unwrap();
    assert!(d.queue_frame(&[0], None).is_err());
    assert!(p.data.lock().unwrap().writes.is_empty());
    frame(&mut other, 1);
    until(|| other.display_ready());
    p.data.lock().unwrap().fail = true;
    frame(&mut d, 2);
    until(|| d.poll_event().is_err());
    assert!(other.display_ready());
    assert_eq!(d.poll_event(), Err(DeviceError::Transport));
}

#[test]
fn retiring_writers_count_against_fixed_pool_budget() {
    let probes: Vec<_> = (0..8).map(|_| Probe::new(DeviceModel::Mk2)).collect();
    let mut pool = Pool::default();
    let _unblock: Vec<_> = probes.iter().cloned().map(Unblock).collect();
    let mut clients = Vec::new();
    for (n, p) in probes.iter().enumerate() {
        *p.stall.0.lock().unwrap() = true;
        let mut d = pool.spawn(id(n as u8 + 1), Box::new(p.clone())).unwrap();
        frame(&mut d, 1);
        clients.push(d);
    }
    until(|| probes.iter().all(|p| p.entered.load(Ordering::Acquire)));
    drop(clients);
    assert!(
        pool.spawn(id(9), Box::new(Probe::new(DeviceModel::Mk2)))
            .is_err()
    );
    for p in &probes {
        p.release();
    }
    let mut reaped = 0;
    until(|| {
        reaped += pool.reap().len();
        reaped == 8
    });
    assert!(
        pool.spawn(id(9), Box::new(Probe::new(DeviceModel::Mk2)))
            .is_ok()
    );
}

#[test]
fn existing_hold_releases_during_redraw_but_hidden_new_down_cannot_arm() {
    use decksmith_core::RawEvent;
    use decksmith_device::{
        manager::{DeviceManager, SessionLayout},
        registry::{Candidate, Discovery, Registry},
    };
    struct ManualReady {
        probe: Probe,
        ready: Arc<AtomicBool>,
    }
    impl DeckDevice for ManualReady {
        fn descriptor(&self) -> DeviceDescriptor {
            self.probe.descriptor()
        }
        fn set_key_image(&mut self, i: u8, b: &[u8]) -> Result<(), DeviceError> {
            self.probe.set_key_image(i, b)
        }
        fn set_touch_image(&mut self, b: &[u8]) -> Result<(), DeviceError> {
            self.probe.set_touch_image(b)
        }
        fn set_brightness(&mut self, b: u8) -> Result<(), DeviceError> {
            self.probe.set_brightness(b)
        }
        fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
            self.probe.poll_event()
        }
        fn display_ready(&self) -> bool {
            self.ready.load(Ordering::Acquire)
        }
    }
    let p = Probe::new(DeviceModel::Mk2);
    let ready = Arc::new(AtomicBool::new(true));
    let discovery = Discovery::new(vec![Candidate::new(
        0,
        DeviceModel::Mk2,
        Some("ready-fixture"),
    )])
    .unwrap();
    let mut registry = Registry::default();
    registry
        .associate(0, id(1), "Ready", 50, &discovery, 0)
        .unwrap();
    let mut manager = DeviceManager::new(registry).unwrap();
    manager
        .configure(
            id(1),
            0,
            SessionLayout::new(DeviceModel::Mk2, 1, 0).unwrap(),
        )
        .unwrap();
    let r = manager.reconcile(discovery).unwrap();
    manager
        .attach(
            r,
            0,
            Box::new(ManualReady {
                probe: p.clone(),
                ready: ready.clone(),
            }),
        )
        .unwrap();
    manager.poll_round();
    let inject = |time, pressed| {
        p.data.lock().unwrap().input.push_back(InputEvent {
            timestamp_ms: time,
            event: RawEvent::Key { index: 0, pressed },
        })
    };
    inject(1, true);
    assert_eq!(manager.poll_round().len(), 1);
    ready.store(false, Ordering::Release);
    inject(2, false);
    assert_eq!(manager.poll_round().len(), 1);
    inject(3, true);
    assert!(manager.poll_round().is_empty());
    ready.store(true, Ordering::Release);
    inject(4, false);
    assert!(manager.poll_round().is_empty());
    inject(5, true);
    assert_eq!(manager.poll_round().len(), 1);
    inject(6, false);
    assert_eq!(manager.poll_round().len(), 1);
}
