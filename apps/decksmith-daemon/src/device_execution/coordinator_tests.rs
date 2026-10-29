//! Coordinator diagnostics use private virtual adapters, never native effects.
use super::*;
use crate::device_profiles::{Enrollment, Store};
use decksmith_core::device_model::DeviceModel;
use decksmith_device::{VirtualDeck, registry::Candidate};
struct NoEffects;
impl shared_worker::Backend for NoEffects {
    fn execute(&mut self, _: &Action, _: &shared_worker::Fence) -> Result<()> {
        Err("no_effects_fixture")
    }
}
struct Fixture {
    runtime: Runtime,
    discovery: Discovery,
    ids: [DeviceId; 2],
    root: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "decksmith-coordinator-{}",
            random_device_id().unwrap()
        ));
        let store = Store::create(&root).unwrap();
        let ids = [random_device_id().unwrap(), random_device_id().unwrap()];
        let models = [DeviceModel::Plus, DeviceModel::Mk2];
        let discovery = Discovery::new(
            models
                .into_iter()
                .enumerate()
                .map(|(i, m)| Candidate::new(i as u8, m, Some(&format!("coordinator-{i}"))))
                .collect(),
        )
        .unwrap();
        for (i, id) in ids.into_iter().enumerate() {
            store
                .enroll(
                    store.load().unwrap().generation,
                    &discovery,
                    i as u8,
                    Enrollment::new(id, "Coordinator", 50, None),
                )
                .unwrap();
        }
        let mut runtime = Runtime::start(
            store.load().unwrap(),
            vec![Box::new(NoEffects), Box::new(NoEffects)],
            None,
        )
        .unwrap();
        let revision = runtime
            .discover(Discovery::new(discovery.candidates().to_vec()).unwrap())
            .unwrap();
        for (slot, model) in models.into_iter().enumerate() {
            runtime
                .attach(
                    revision,
                    &discovery,
                    slot as u8,
                    Box::new(VirtualDeck::new(model)),
                )
                .unwrap();
        }
        let mut f = Self {
            runtime,
            discovery,
            ids,
            root,
        };
        f.ready();
        f
    }
    fn ready(&mut self) {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            self.runtime.tick();
            if self
                .runtime
                .statuses()
                .iter()
                .all(|s| s.connected && s.input_ready && s.display_ready)
            {
                return;
            }
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.runtime.stop();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
#[ignore = "read-only synthetic coordinator cost diagnostic; run explicitly"]
fn idle_coordinator_diagnostic() {
    let mut f = Fixture::new();
    for _ in 0..100 {
        f.runtime.tick();
    }
    let ticks = || {
        let raw = std::fs::read_to_string("/proc/thread-self/stat").unwrap();
        let v: Vec<_> = raw[raw.rfind(')').unwrap() + 2..]
            .split_whitespace()
            .collect();
        v[11].parse::<u64>().unwrap() + v[12].parse::<u64>().unwrap()
    };
    for round in 0..3 {
        let cpu = ticks();
        let at = Instant::now();
        for _ in 0..10000 {
            assert!(f.runtime.tick().is_empty());
        }
        println!(
            "coordinator_diagnostic round={round} turns=10000 wall_ms={:.3} thread_cpu_ticks={}",
            at.elapsed().as_secs_f64() * 1000.,
            ticks() - cpu
        );
    }
}

#[test]
fn idle_sync_keeps_actions_live_and_fences_reject_without_waiting_for_refresh() {
    let mut f = Fixture::new();
    let plus = f.ids[0];
    let mk2 = f.ids[1];
    let scope = |r: &Runtime, id| r.scope(r.context(id, ControlAddress::Key(0)).unwrap());
    let first = scope(&f.runtime, plus);
    let partner = scope(&f.runtime, mk2);
    for _ in 0..100 {
        f.runtime.sync();
    }
    assert!(f.runtime.mirror.read().unwrap().accepts(&first));
    // Submission wakes its own provider even after an idle synchronization run.
    let request = f
        .runtime
        .actions
        .submit(
            first,
            Compiled::builtin(Action::OpenWebsite {
                url: "https://example.invalid".into(),
            })
            .unwrap(),
            None,
        )
        .unwrap();
    let until = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(reply) = f.runtime.actions.poll().unwrap() {
            assert_eq!(reply.completion.id, request);
            assert_eq!(reply.completion.result, Err("no_effects_fixture"));
            break;
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(1));
    }
    // A device actor's epoch rejects the old scope before sync sees that fence.
    f.runtime
        .controls
        .fence_input(plus, first.device.binding)
        .unwrap();
    assert!(!f.runtime.mirror.read().unwrap().accepts(&first));
    assert!(f.runtime.mirror.read().unwrap().accepts(&partner));
    // Complete the normal owner transition: raw manager fencing alone does
    // not request the replacement frame (the public Runtime paths do both).
    f.runtime.revoke(plus, "test_fence");
    f.runtime.sync();
    f.ready();
    let before_page = scope(&f.runtime, plus);
    f.runtime.navigate(before_page.device, 1).unwrap();
    assert!(!f.runtime.mirror.read().unwrap().accepts(&before_page));
    assert!(f.runtime.mirror.read().unwrap().accepts(&partner));
    f.ready();
    let before_lock = scope(&f.runtime, plus);
    f.runtime.host_lock(true, 1).unwrap();
    assert!(!f.runtime.mirror.read().unwrap().accepts(&before_lock));
    assert!(!f.runtime.mirror.read().unwrap().accepts(&partner));
    f.runtime.host_lock(false, 2).unwrap();
    f.ready();
    let before_foreground = scope(&f.runtime, plus);
    f.runtime.foreground(2).unwrap();
    assert!(!f.runtime.mirror.read().unwrap().accepts(&before_foreground));
    f.ready();
    let before_disconnect = scope(&f.runtime, plus);
    let partner = scope(&f.runtime, mk2);
    f.runtime.disconnect(plus).unwrap();
    assert!(!f.runtime.mirror.read().unwrap().accepts(&before_disconnect));
    assert!(f.runtime.mirror.read().unwrap().accepts(&partner));
    let until = Instant::now() + Duration::from_secs(2);
    while !f.runtime.ready_to_attach(plus) {
        f.runtime.tick();
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(2));
    }
    let revision = f
        .runtime
        .discover(Discovery::new(f.discovery.candidates().to_vec()).unwrap())
        .unwrap();
    f.runtime
        .attach(
            revision,
            &f.discovery,
            0,
            Box::new(VirtualDeck::new(DeviceModel::Plus)),
        )
        .unwrap();
    f.ready();
    assert!(!f.runtime.mirror.read().unwrap().accepts(&before_disconnect));
    assert!(
        f.runtime
            .mirror
            .read()
            .unwrap()
            .accepts(&scope(&f.runtime, plus))
    );
    assert!(f.runtime.mirror.read().unwrap().accepts(&partner));
}
