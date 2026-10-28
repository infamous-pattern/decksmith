//! Daily multi-device host. Physical identity, saved profiles, providers and all
//! teardown are owned here; no diagnostic layout or timed lab is adopted.
use crate::{
    device_execution::Runtime,
    device_profiles::{Catalog, Enrollment, Store, random_device_id},
};
use decksmith_core::{
    device_context::DeviceId,
    device_model::{ControlAddress, DeviceModel},
};
use decksmith_device::{
    DeckDevice, DeviceError,
    registry::{Association, Candidate, Discovery, UnassignedReason},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
fn config_root() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .ok_or("config_home_unavailable")?;
    if !base.is_absolute() {
        return Err("invalid_config_home");
    }
    Ok(base.join("decksmith"))
}
fn store_at(root: &Path) -> Result<Store> {
    crate::layout_store::directory(root)?;
    let path = root.join("devices");
    if path.try_exists().map_err(|_| "profile_store_io")? {
        Store::open(&path)
    } else {
        Store::create(&path)
    }
}
fn legacy_at(root: &Path, default: &Path) -> Result<(Vec<u8>, u8)> {
    let settings = root.join("control-panel.json");
    let (layout, brightness) = if settings.try_exists().map_err(|_| "settings_read_failed")? {
        let raw = crate::layout_store::read(&settings, 4096)?;
        let s: crate::control::Settings =
            serde_json::from_slice(&raw).map_err(|_| "invalid_settings")?;
        (s.layout, s.brightness.unwrap_or(50))
    } else {
        ("audio".into(), 50)
    };
    if brightness > 100 {
        return Err("invalid_settings");
    }
    let path = match layout.as_str() {
        "custom" => root.join("layout.json"),
        "audio" => default.into(),
        "navigation" => crate::runtime_paths::config("navigation.json"),
        _ => return Err("unknown_layout"),
    };
    let raw = crate::layout_store::read(&path, crate::layout_document::LIMIT)?;
    crate::layout_document::Document::for_model(&raw, DeviceModel::Plus)?;
    Ok((raw, brightness))
}
// Legacy files are consulted only when a uniquely identified, new Plus needs
// the one-time import. Existing device profiles remain authoritative thereafter.
fn enroll_discovered(
    store: &Store,
    discovery: &Discovery,
    root: &Path,
    default: &Path,
) -> Result<Catalog> {
    let catalog = store.load()?;
    let needs_import = !catalog
        .summaries()
        .iter()
        .any(|s| s.model == DeviceModel::Plus)
        && discovery
            .candidates()
            .iter()
            .filter(|c| c.model() == DeviceModel::Plus)
            .count()
            == 1
        && catalog
            .associations(discovery)
            .iter()
            .any(|(_, a)| matches!(a, Association::Unassigned(UnassignedReason::NewDevice)));
    let (legacy, brightness) = if needs_import {
        legacy_at(root, default)?
    } else {
        (Vec::new(), 50)
    };
    enroll_initial(store, discovery, &legacy, brightness)
}
/// Explicit host enrollment policy: only unique, known supported identities.
/// Never guess which of two new Plus units should inherit a legacy layout.
fn enroll_initial(
    store: &Store,
    discovery: &Discovery,
    legacy: &[u8],
    brightness: u8,
) -> Result<Catalog> {
    let initial = store.load()?;
    let has_plus = initial
        .summaries()
        .iter()
        .any(|s| s.model == DeviceModel::Plus);
    let plus_count = discovery
        .candidates()
        .iter()
        .filter(|c| c.model() == DeviceModel::Plus)
        .count();
    for c in discovery.candidates() {
        let catalog = store.load()?;
        if catalog.summaries().len() >= decksmith_device::registry::MAX_DEVICES {
            break;
        }
        if !matches!(
            catalog
                .associations(discovery)
                .into_iter()
                .find(|(slot, _)| *slot == c.slot())
                .map(|(_, a)| a),
            Some(Association::Unassigned(UnassignedReason::NewDevice))
        ) {
            continue;
        }
        if c.model() == DeviceModel::Plus && !has_plus && plus_count != 1 {
            continue;
        }
        store.ensure_daily_capacity()?;
        let name = match c.model() {
            DeviceModel::Plus => "Stream Deck +",
            DeviceModel::Mk2 => "Stream Deck MK.2",
        };
        let import = (c.model() == DeviceModel::Plus && !has_plus).then_some(legacy);
        let id = random_device_id()?;
        store.enroll(
            catalog.generation,
            discovery,
            c.slot(),
            Enrollment::new(
                id,
                name,
                if c.model() == DeviceModel::Plus {
                    brightness
                } else {
                    50
                },
                import,
            ),
        )?;
        if import.is_some() && initial.selected.is_none() {
            store.select(store.load()?.generation, id)?;
        }
    }
    store.load()
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    page: u8,
    brightness: u8,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedState {
    version: u8,
    devices: BTreeMap<DeviceId, Selection>,
}
impl Default for SavedState {
    fn default() -> Self {
        Self {
            version: 1,
            devices: BTreeMap::new(),
        }
    }
}
fn read_state(root: &Path) -> Result<SavedState> {
    let path = root.join("device-state.json");
    if !path.try_exists().map_err(|_| "device_state_read_failed")? {
        return Ok(SavedState::default());
    }
    let bytes = crate::layout_store::read(&path, 8192)?;
    let value: SavedState = serde_json::from_slice(&bytes).map_err(|_| "invalid_device_state")?;
    if value.version != 1
        || value.devices.len() > 8
        || value.devices.values().any(|v| v.brightness > 100)
    {
        return Err("invalid_device_state");
    }
    Ok(value)
}
fn restore_state(store: &Store, state: &SavedState) -> Result<Catalog> {
    for (id, s) in &state.devices {
        let c = store.load()?;
        let Ok(p) = c.profile(*id) else {
            continue;
        };
        let summary = c
            .summaries()
            .into_iter()
            .find(|r| r.id == *id)
            .ok_or("unknown_device")?;
        let page = if s.page < p.document().topology().0 {
            s.page
        } else {
            p.saved_page
        };
        if p.saved_page != page || summary.brightness != s.brightness {
            store.ensure_daily_capacity()?;
            store.save_profile(
                *id,
                p.revision,
                p.document().bytes(),
                page,
                summary.name,
                s.brightness,
            )?;
        }
    }
    store.load()
}
fn session_state(runtime: &Runtime) -> SavedState {
    SavedState {
        version: 1,
        devices: runtime
            .statuses()
            .iter()
            .filter_map(|s| {
                s.page.map(|page| {
                    (
                        s.id,
                        Selection {
                            page,
                            brightness: s.brightness,
                        },
                    )
                })
            })
            .collect(),
    }
}
struct StateWriter {
    sender: Option<SyncSender<SavedState>>,
    completed: Receiver<(SavedState, bool)>,
    thread: Option<JoinHandle<()>>,
    last: SavedState,
}
impl StateWriter {
    fn start(root: PathBuf, initial: SavedState) -> Result<Self> {
        let (send, receive) = mpsc::sync_channel::<SavedState>(1);
        let (completion_send, completed) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("decksmith-device-state".into())
            .spawn(move || {
                while let Ok(mut state) = receive.recv() {
                    while let Ok(next) = receive.try_recv() {
                        state = next;
                    }
                    let result = (|| {
                        let bytes =
                            serde_json::to_vec(&state).map_err(|_| "device_state_write_failed")?;
                        let temporary =
                            root.join(format!("device-state-{}.tmp", random_device_id()?));
                        crate::layout_store::write(&temporary, &bytes)?;
                        std::fs::rename(&temporary, root.join("device-state.json"))
                            .map_err(|_| "device_state_write_failed")?;
                        crate::layout_store::sync(&root)
                    })();
                    if let Err(code) = result {
                        tracing::warn!(error_code = code, "Device page state could not be saved");
                    }
                    let _ = completion_send.send((state, result.is_ok()));
                }
            })
            .map_err(|_| "device_state_worker_failed")?;
        Ok(Self {
            sender: Some(send),
            completed,
            thread: Some(thread),
            last: initial,
        })
    }
    fn finish(&mut self, state: SavedState) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(state);
        }
    }
    fn publish(&mut self, state: SavedState) {
        while let Ok((saved, success)) = self.completed.try_recv() {
            if success {
                self.last = saved;
            }
        }
        if state != self.last
            && let Some(sender) = &self.sender
        {
            let _ = sender.try_send(state);
        }
    }
}
impl Drop for StateWriter {
    fn drop(&mut self) {
        self.sender = None;
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
#[derive(Deserialize)]
struct PageSpec {
    #[serde(default)]
    application: Option<String>,
    #[serde(default)]
    default: bool,
}
#[derive(Deserialize)]
struct PageSpecs {
    pages: Vec<PageSpec>,
}
fn app_page(bytes: &[u8], application: &str) -> Result<Option<u8>> {
    if application == crate::foreground::STUDIO {
        return Ok(None);
    }
    let doc: PageSpecs = serde_json::from_slice(bytes).map_err(|_| "invalid_config")?;
    Ok(Some(
        doc.pages
            .iter()
            .position(|p| p.application.as_deref() == Some(application))
            .or_else(|| doc.pages.iter().position(|p| p.default))
            .unwrap_or(0) as u8,
    ))
}
#[derive(Default)]
struct ForegroundPolicy {
    revision: Option<u64>,
    application: Option<String>,
}
impl ForegroundPolicy {
    fn observe(
        &mut self,
        snapshot: &crate::foreground::Snapshot,
        locked: bool,
    ) -> Option<(u64, Option<String>)> {
        if locked || !snapshot.available || self.revision == Some(snapshot.revision) {
            return None;
        }
        self.revision = Some(snapshot.revision);
        let changed = snapshot.application.is_some() && snapshot.application != self.application;
        if snapshot.application.is_some() {
            self.application = snapshot.application.clone();
        }
        Some((
            snapshot.revision.saturating_add(1),
            if changed {
                snapshot.application.clone()
            } else {
                None
            },
        ))
    }
}
struct NoEffects;
impl crate::shared_worker::Backend for NoEffects {
    fn execute(
        &mut self,
        _: &crate::pages::Action,
        fence: &crate::shared_worker::Fence,
    ) -> Result<()> {
        if fence.accepts() {
            Ok(())
        } else {
            Err("stale_action_scope")
        }
    }
}

#[cfg(feature = "hardware")]
enum IoCommand {
    Scan,
    Open {
        sequence: u64,
        revision: u64,
        slot: u8,
        id: DeviceId,
        sleep_epoch: u64,
    },
}
#[cfg(feature = "hardware")]
enum IoReply {
    Snapshot(u64, Discovery),
    Opened {
        revision: u64,
        discovery: Discovery,
        slot: u8,
        id: DeviceId,
        sleep_epoch: u64,
        device: std::result::Result<Box<dyn DeckDevice + Send>, DeviceError>,
    },
    Unavailable,
}
#[cfg(feature = "hardware")]
struct HardwareIo {
    sender: Option<SyncSender<IoCommand>>,
    receive: Receiver<IoReply>,
    thread: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    pending: bool,
}
#[cfg(feature = "hardware")]
impl HardwareIo {
    fn start() -> Result<Self> {
        let mut owner =
            decksmith_device::hardware::HardwareOwner::acquire().map_err(|_| "device_busy")?;
        let (sender, commands) = mpsc::sync_channel(1);
        let (send, receive) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let ending = stop.clone();
        let thread = std::thread::Builder::new()
            .name("decksmith-hardware-discovery".into())
            .spawn(move || {
                let mut cached = decksmith_device::hardware::InventoryCache::default();
                let mut sequence = 0u64;
                while !ending.load(Ordering::Acquire) {
                    let Ok(command) = commands.recv_timeout(Duration::from_millis(100)) else {
                        continue;
                    };
                    let reply = match command {
                        IoCommand::Scan => match cached.refresh() {
                            Ok(inventory) => {
                                sequence = sequence.saturating_add(1);
                                let discovery = inventory.snapshot();
                                IoReply::Snapshot(sequence, discovery)
                            }
                            Err(_) => IoReply::Unavailable,
                        },
                        IoCommand::Open {
                            sequence: expected,
                            revision,
                            slot,
                            id,
                            sleep_epoch,
                        } => {
                            if expected != sequence {
                                IoReply::Unavailable
                            } else if let Some(inventory) = cached.current() {
                                IoReply::Opened {
                                    revision,
                                    discovery: inventory.snapshot(),
                                    slot,
                                    id,
                                    sleep_epoch,
                                    device: owner
                                        .open(inventory, slot)
                                        .map(|d| Box::new(d) as Box<dyn DeckDevice + Send>),
                                }
                            } else {
                                IoReply::Unavailable
                            }
                        }
                    };
                    if send.try_send(reply).is_err() {
                        break;
                    }
                }
            })
            .map_err(|_| "discovery_worker_failed")?;
        Ok(Self {
            sender: Some(sender),
            receive,
            thread: Some(thread),
            stop,
            pending: false,
        })
    }
    fn request(&mut self, command: IoCommand) {
        if !self.pending
            && self
                .sender
                .as_ref()
                .is_some_and(|s| s.try_send(command).is_ok())
        {
            self.pending = true;
        }
    }
    fn poll(&mut self) -> Result<Option<IoReply>> {
        match self.receive.try_recv() {
            Ok(reply) => {
                self.pending = false;
                Ok(Some(reply))
            }
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Err("discovery_worker_failed"),
        }
    }
}
#[cfg(feature = "hardware")]
impl Drop for HardwareIo {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.sender = None;
        if let Some(t) = self.thread.take() {
            let end = Instant::now() + Duration::from_secs(2);
            while !t.is_finished() && Instant::now() < end {
                std::thread::sleep(Duration::from_millis(10));
            }
            if t.is_finished() {
                let _ = t.join();
            } else {
                tracing::error!("Hardware discovery did not retire before process shutdown");
            }
        }
    }
}

pub(crate) fn run(default: &Path, virtual_root: Option<&Path>, seconds: Option<u64>) -> Result<()> {
    let root = virtual_root
        .map(Path::to_path_buf)
        .map_or_else(config_root, Ok)?;
    if !root.is_absolute() {
        return Err("absolute_config_root_required");
    }
    if !root
        .try_exists()
        .map_err(|_| "config_directory_unavailable")?
    {
        crate::layout_store::mkdir(&root)?;
    }
    crate::layout_store::directory(&root)?;
    let physical = virtual_root.is_none();
    #[cfg(feature = "hardware")]
    let mut hardware = if physical {
        Some(HardwareIo::start()?)
    } else {
        None
    };
    let store = store_at(&root)?;
    let saved = read_state(&root)?;
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stop.clone()).map_err(|_| "signal_setup_failed")?;
    }
    let gate = crate::session_lock::Gate::new(if physical {
        match std::fs::read(root.join("auto-lock.json")) {
            Ok(bytes) => serde_json::from_slice::<bool>(&bytes).unwrap_or(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(_) => true,
        }
    } else {
        true
    });
    let foreground = crate::foreground::Foreground::default();
    if physical {
        gate.monitor();
        foreground.monitor();
    } else {
        gate.observe(Some(false), "Virtual");
    }
    let _public = crate::permanent_control::start(gate.clone(), foreground.clone())?;
    let end = seconds.map(|s| Instant::now() + Duration::from_secs(s));
    let virtual_discovery = Discovery::new(vec![
        Candidate::new(0, DeviceModel::Plus, Some("virtual-service-plus")),
        Candidate::new(1, DeviceModel::Mk2, Some("virtual-service-mk2")),
    ])?;
    let mut catalog = store.load()?;
    if !physical {
        catalog = enroll_discovered(&store, &virtual_discovery, &root, default)?;
    }
    #[cfg(feature = "hardware")]
    while catalog.summaries().is_empty()
        && !stop.load(Ordering::Acquire)
        && end.is_none_or(|t| Instant::now() < t)
    {
        let io = hardware.as_mut().ok_or("hardware_unavailable")?;
        io.request(IoCommand::Scan);
        if let Some(IoReply::Snapshot(_, discovery)) = io.poll()? {
            catalog = enroll_discovered(&store, &discovery, &root, default)?;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if catalog.summaries().is_empty() {
        return Ok(());
    }
    // Discover newly connected units on each startup without rebinding known IDs.
    #[cfg(feature = "hardware")]
    if let Some(io) = &mut hardware {
        io.request(IoCommand::Scan);
        let until = Instant::now() + Duration::from_secs(2);
        while Instant::now() < until {
            if let Some(IoReply::Snapshot(_, d)) = io.poll()? {
                enroll_discovered(&store, &d, &root, default)?;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    catalog = restore_state(&store, &saved)?;
    let mut runtime = if physical {
        let mut r = Runtime::start_saved_native(
            catalog,
            crate::shared_native_media::NativeBackend::connect_daily()?,
            vec![],
        )?;
        r.enable_dynamic_media();
        r.enable_homebridge()?;
        r
    } else {
        Runtime::start(catalog, vec![Box::new(NoEffects)], None)?
    };
    let mut profile = crate::live_profiles::Service::start_permanent(
        Store::open(&root.join("devices"))?,
        &runtime,
        physical,
    )?;
    let mut writer = StateWriter::start(root.clone(), saved)?;
    let mut sleep = if physical {
        Some(crate::sleep_lifecycle::Observer::start()?)
    } else {
        None
    };
    let mut resume = crate::resume::Monitor::default();
    resume.poll();
    if !physical {
        let revision =
            runtime.discover(Discovery::new(virtual_discovery.candidates().to_vec())?)?;
        for (slot, a) in runtime.catalog().associations(&virtual_discovery) {
            if let Association::Assigned(id) = a {
                let model = runtime.catalog().profile(id)?.document().model();
                runtime.attach(
                    revision,
                    &virtual_discovery,
                    slot,
                    Box::new(decksmith_device::VirtualDeck::new(model)),
                )?;
            }
        }
    }
    let mut next_scan = Instant::now();
    let mut next_state = Instant::now();
    let mut foreground_policy = ForegroundPolicy::default();
    let mut last_sleep_failure_epoch = None;
    let mut preparation_failures = 0;
    let mut wanted_pages = BTreeMap::<DeviceId, u8>::new();
    #[cfg(feature = "hardware")]
    let mut inventory: Option<(u64, u64, Discovery)> = None;
    let mut retry = BTreeMap::<DeviceId, Instant>::new();
    let mut plan = crate::saved_native::Plan::from_catalog(runtime.catalog())?;
    let mut plan_generation = runtime.catalog().generation;
    let mut system = if physical {
        Some(crate::shared_native_state::SystemObserver::start(
            plan.system.clone(),
        )?)
    } else {
        None
    };
    let mut plugin = if physical && plan.pages.values().flatten().any(|p| p.homebridge) {
        Some(crate::plugin_runtime::Observer::start()?)
    } else {
        None
    };
    let result = (|| {
        while !stop.load(Ordering::Acquire) && end.is_none_or(|t| Instant::now() < t) {
            if !physical {
                gate.observe(Some(false), "Virtual");
            }
            let host_lock = gate.snapshot();
            runtime.host_lock(host_lock.locked, host_lock.epoch)?;
            let mut sleep_epoch = 0;
            if let Some(sleep) = &mut sleep {
                if resume.poll() {
                    sleep.resumed();
                }
                let snapshot = sleep.snapshot();
                if snapshot.preparation_failures > preparation_failures {
                    tracing::warn!(
                        error_code = snapshot
                            .last_preparation_error
                            .unwrap_or("sleep_preparation_failed"),
                        count = snapshot.preparation_failures,
                        "Sleep preparation was not acknowledged before the deadline"
                    );
                    preparation_failures = snapshot.preparation_failures;
                }
                sleep_epoch = snapshot.epoch;
                runtime.host_sleep(snapshot.blocked, snapshot.epoch)?;
                let failures = runtime.sleep_errors();
                if !failures.is_empty() && last_sleep_failure_epoch != Some(snapshot.epoch) {
                    tracing::warn!(
                        count = failures.len(),
                        "Device sleep preparation could not confirm clean retirement"
                    );
                    last_sleep_failure_epoch = Some(snapshot.epoch);
                }
                if snapshot.preparing && runtime.sleep_ready() {
                    sleep.acknowledge(snapshot.epoch, true)?;
                }
            }
            #[cfg(feature = "hardware")]
            if let Some(io) = &mut hardware {
                if let Some(reply) = io.poll()? {
                    match reply {
                        IoReply::Snapshot(sequence, discovery) => {
                            let revision = runtime
                                .discover(Discovery::new(discovery.candidates().to_vec())?)?;
                            inventory = Some((sequence, revision, discovery));
                        }
                        IoReply::Opened {
                            revision,
                            discovery,
                            slot,
                            id,
                            sleep_epoch: issued,
                            device,
                        } => {
                            if issued == sleep_epoch && runtime.ready_to_attach(id) {
                                match device {
                                    Ok(device) => {
                                        if let Err(code) =
                                            runtime.attach(revision, &discovery, slot, device)
                                        {
                                            tracing::warn!(
                                                error_code = code,
                                                "Device attachment deferred"
                                            );
                                            retry.insert(
                                                id,
                                                Instant::now() + Duration::from_secs(1),
                                            );
                                        }
                                    }
                                    Err(_) => {
                                        retry.insert(id, Instant::now() + Duration::from_secs(1));
                                    }
                                }
                            }
                        }
                        IoReply::Unavailable => {
                            inventory = None;
                        }
                    }
                }
                if !io.pending {
                    if Instant::now() >= next_scan {
                        io.request(IoCommand::Scan);
                        next_scan = Instant::now() + Duration::from_secs(1);
                    } else if let Some((sequence, revision, d)) = &inventory {
                        for (slot, a) in runtime.catalog().associations(d) {
                            if let Association::Assigned(id) = a
                                && runtime
                                    .statuses()
                                    .iter()
                                    .any(|s| s.id == id && !s.connected)
                                && runtime.ready_to_attach(id)
                                && retry.get(&id).is_none_or(|t| Instant::now() >= *t)
                            {
                                io.request(IoCommand::Open {
                                    sequence: *sequence,
                                    revision: *revision,
                                    slot,
                                    id,
                                    sleep_epoch,
                                });
                                break;
                            }
                        }
                    }
                }
            }
            let f = foreground.snapshot();
            if let Some((revision, application)) = foreground_policy.observe(&f, host_lock.locked) {
                runtime.foreground(revision)?;
                if let Some(app) = application {
                    wanted_pages.clear();
                    for summary in runtime.catalog().summaries() {
                        if let Some(page) = app_page(
                            runtime.catalog().profile(summary.id)?.document().bytes(),
                            &app,
                        )? {
                            wanted_pages.insert(summary.id, page);
                        }
                    }
                }
            }
            if runtime.catalog().generation != plan_generation {
                plan = crate::saved_native::Plan::from_catalog(runtime.catalog())?;
                if let Some(system) = &mut system {
                    system.configure(plan.system.clone())?;
                }
                if physical
                    && plugin.is_none()
                    && plan.pages.values().flatten().any(|p| p.homebridge)
                {
                    plugin = Some(crate::plugin_runtime::Observer::start()?);
                }
                if !plan.pages.values().flatten().any(|p| p.homebridge) {
                    plugin = None;
                }
                plan_generation = runtime.catalog().generation;
            }
            if let Some(system) = &mut system
                && let Some(states) = system.snapshot()
            {
                runtime.update_system(states);
            }
            if let Some(plugin) = &mut plugin
                && plugin.snapshot().is_some()
            {
                runtime.refresh_plugin_display();
            }
            for reply in runtime.tick() {
                if let Err(code) = reply.result {
                    tracing::warn!(error_code = code, "Assigned action unavailable");
                }
            }
            let pending: Vec<_> = wanted_pages.iter().map(|(id, page)| (*id, *page)).collect();
            for (id, page) in pending {
                if runtime
                    .statuses()
                    .iter()
                    .any(|s| s.id == id && s.page == Some(page))
                {
                    wanted_pages.remove(&id);
                } else if let Ok(context) = runtime.context(id, ControlAddress::Key(0)) {
                    runtime.navigate(context, page)?;
                    wanted_pages.remove(&id);
                }
            }
            profile.poll(&mut runtime);
            profile.publish(&runtime);
            if Instant::now() >= next_state {
                writer.publish(session_state(&runtime));
                next_state = Instant::now() + Duration::from_secs(1);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    })();
    let final_state = session_state(&runtime);
    profile.stop_accepting();
    let stopped = runtime.stop();
    drop(profile);
    if stopped.iter().any(|(_, r)| r.is_err()) {
        tracing::warn!("Some disconnected devices could not confirm blanking");
    }
    writer.finish(final_state);
    drop(sleep);
    drop(system);
    drop(plugin);
    drop(writer);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = std::env::temp_dir()
                .join(format!("decksmith-daily-{}", random_device_id().unwrap()));
            crate::layout_store::mkdir(&p).unwrap();
            Self(p)
        }
        fn store(&self) -> Store {
            Store::create(&self.0.join("devices")).unwrap()
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn discovery() -> Discovery {
        Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("fixture-plus")),
            Candidate::new(1, DeviceModel::Mk2, Some("fixture-mk2")),
        ])
        .unwrap()
    }
    fn legacy() -> Vec<u8> {
        let mut v: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        v["pages"][0]["keys"][0]["label"] = "My preserved 🎵".into();
        serde_json::to_vec(&v).unwrap()
    }
    #[test]
    fn migrated_profiles_do_not_depend_on_obsolete_legacy_files() {
        let t = Temp::new();
        let store = t.store();
        let c = enroll_initial(&store, &discovery(), &legacy(), 55).unwrap();
        crate::layout_store::write(&t.0.join("control-panel.json"), b"broken legacy").unwrap();
        let next =
            enroll_discovered(&store, &discovery(), &t.0, Path::new("/missing-default")).unwrap();
        assert_eq!(next.generation, c.generation);
        for row in c.summaries() {
            assert_eq!(
                next.profile(row.id).unwrap().document().bytes(),
                c.profile(row.id).unwrap().document().bytes()
            );
        }
        // An actual first Plus import still fails safely rather than overwriting
        // a malformed old layout with guessed defaults.
        let fresh = Temp::new();
        let fresh_store = fresh.store();
        crate::layout_store::write(&fresh.0.join("control-panel.json"), b"broken legacy").unwrap();
        assert_eq!(
            enroll_discovered(
                &fresh_store,
                &discovery(),
                &fresh.0,
                Path::new("/missing-default")
            )
            .err(),
            Some("invalid_settings")
        );
        assert!(fresh_store.load().unwrap().summaries().is_empty());
    }
    #[test]
    #[cfg(feature = "hardware")]
    fn lost_discovery_worker_surfaces_failure_instead_of_latching_pending() {
        let (sender, _) = mpsc::sync_channel(1);
        let (reply, receive) = mpsc::sync_channel(1);
        let mut io = HardwareIo {
            sender: Some(sender),
            receive,
            thread: None,
            stop: Arc::new(AtomicBool::new(false)),
            pending: true,
        };
        assert!(io.poll().unwrap().is_none());
        drop(reply);
        assert_eq!(io.poll().err(), Some("discovery_worker_failed"));
    }
    #[test]
    fn lock_and_same_app_reappearance_do_not_reset_manual_pages() {
        let mut p = ForegroundPolicy::default();
        let mut s = crate::foreground::Snapshot {
            available: true,
            application: Some("music.desktop".into()),
            revision: 1,
        };
        assert_eq!(
            p.observe(&s, false),
            Some((2, Some("music.desktop".into())))
        );
        s.revision = 2;
        s.application = None;
        assert_eq!(p.observe(&s, true), None);
        s.revision = 3;
        s.application = Some("music.desktop".into());
        assert_eq!(p.observe(&s, false), Some((4, None)));
        assert_eq!(p.observe(&s, false), None); // unrelated saves/selection do nothing
        s.revision = 4;
        s.application = Some("other.desktop".into());
        assert_eq!(
            p.observe(&s, false),
            Some((5, Some("other.desktop".into())))
        );
    }
    #[test]
    fn enrolls_unique_pair_once_and_preserves_legacy_labels_and_independent_defaults() {
        let t = Temp::new();
        let store = t.store();
        let raw = legacy();
        let c = enroll_initial(&store, &discovery(), &raw, 55).unwrap();
        assert_eq!(c.summaries().len(), 2);
        let plus = c
            .summaries()
            .into_iter()
            .find(|s| s.model == DeviceModel::Plus)
            .unwrap()
            .id;
        let mk = c
            .summaries()
            .into_iter()
            .find(|s| s.model == DeviceModel::Mk2)
            .unwrap()
            .id;
        let p: serde_json::Value =
            serde_json::from_slice(c.profile(plus).unwrap().document().bytes()).unwrap();
        assert_eq!(p["pages"][0]["keys"][0]["label"], "My preserved 🎵");
        assert_eq!(p["model"], "plus");
        assert_eq!(
            c.profile(mk).unwrap().document().descriptor().key_count(),
            15
        );
        assert!(
            !String::from_utf8_lossy(c.profile(mk).unwrap().document().bytes())
                .contains("My preserved")
        );
        let reversed = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Mk2, Some("fixture-mk2")),
            Candidate::new(1, DeviceModel::Plus, Some("fixture-plus")),
        ])
        .unwrap();
        let again = enroll_initial(&store, &reversed, &raw, 20).unwrap();
        assert_eq!(again.generation, c.generation);
        assert_eq!(
            again.summaries().iter().map(|s| s.id).collect::<Vec<_>>(),
            c.summaries().iter().map(|s| s.id).collect::<Vec<_>>()
        );
    }
    #[test]
    fn ambiguous_or_missing_identity_is_not_adopted_and_two_new_plus_cannot_steal_legacy() {
        let t = Temp::new();
        let store = t.store();
        let missing = Discovery::new(vec![Candidate::new(0, DeviceModel::Plus, None)]).unwrap();
        assert!(
            enroll_initial(&store, &missing, &legacy(), 50)
                .unwrap()
                .summaries()
                .is_empty()
        );
        let twice = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("a")),
            Candidate::new(1, DeviceModel::Plus, Some("b")),
        ])
        .unwrap();
        assert!(
            enroll_initial(&store, &twice, &legacy(), 50)
                .unwrap()
                .summaries()
                .is_empty()
        );
    }
    #[test]
    fn saved_page_and_brightness_restore_without_rewriting_layout_or_other_device() {
        let t = Temp::new();
        let store = t.store();
        let c = enroll_initial(&store, &discovery(), &legacy(), 50).unwrap();
        let id = c
            .summaries()
            .into_iter()
            .find(|s| s.model == DeviceModel::Plus)
            .unwrap()
            .id;
        let old = c.profile(id).unwrap().document().bytes().to_vec();
        let state = SavedState {
            version: 1,
            devices: [(
                id,
                Selection {
                    page: 1,
                    brightness: 37,
                },
            )]
            .into_iter()
            .collect(),
        };
        let next = restore_state(&store, &state).unwrap();
        assert_eq!(next.profile(id).unwrap().saved_page, 1);
        assert_eq!(next.profile(id).unwrap().document().bytes(), old);
        assert_eq!(
            next.summaries()
                .iter()
                .find(|s| s.id == id)
                .unwrap()
                .brightness,
            37
        );
        assert_eq!(
            restore_state(&store, &state).unwrap().generation,
            next.generation
        );
        let mut writer = StateWriter::start(t.0.clone(), SavedState::default()).unwrap();
        writer.publish(state.clone());
        writer.finish(state.clone());
        drop(writer);
        assert!(read_state(&t.0).unwrap() == state);
    }
    #[test]
    fn state_writer_retries_unchanged_state_after_write_failure() {
        let t = Temp::new();
        let root = t.0.join("temporarily-unavailable");
        let state = SavedState {
            version: 1,
            devices: [(
                random_device_id().unwrap(),
                Selection {
                    page: 1,
                    brightness: 37,
                },
            )]
            .into_iter()
            .collect(),
        };
        let mut writer = StateWriter::start(root.clone(), SavedState::default()).unwrap();
        writer.publish(state.clone());
        let (_, success) = writer
            .completed
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        assert!(!success);
        crate::layout_store::mkdir(&root).unwrap();
        writer.publish(state.clone());
        let (saved, success) = writer
            .completed
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        assert!(success);
        assert!(saved == state);
        assert!(read_state(&root).unwrap() == state);
    }
    #[test]
    fn daily_archival_preserves_previous_layout_through_many_selection_generations() {
        let t = Temp::new();
        let store = t.store();
        let mut c = enroll_initial(&store, &discovery(), &legacy(), 50).unwrap();
        let id = c
            .summaries()
            .into_iter()
            .find(|s| s.model == DeviceModel::Plus)
            .unwrap()
            .id;
        let old = c.profile(id).unwrap().document().bytes().to_vec();
        let mut value: serde_json::Value = serde_json::from_slice(&old).unwrap();
        value["pages"][0]["keys"][0]["label"] = "Changed ✅".into();
        let new = serde_json::to_vec(&value).unwrap();
        let row = c.summaries().into_iter().find(|s| s.id == id).unwrap();
        store
            .save_profile(id, c.profile(id).unwrap().revision, &new, 0, row.name, 50)
            .unwrap();
        for n in 0..36 {
            store.ensure_daily_capacity().unwrap();
            c = store.load().unwrap();
            let selected = c.summaries()[n % 2].id;
            store.select(c.generation, selected).unwrap();
        }
        assert!(store.history_status().unwrap().0 < 16);
        assert_eq!(store.previous_profile_layout(id).unwrap(), old);
        assert_eq!(
            store
                .load()
                .unwrap()
                .profile(id)
                .unwrap()
                .document()
                .bytes(),
            new
        );
    }
    #[test]
    fn foreground_routes_each_layout_and_studio_does_not_override_manual_page() {
        let mut v: serde_json::Value = serde_json::from_slice(&legacy()).unwrap();
        v["pages"][0]["default"] = false.into();
        v["pages"][1]["default"] = true.into();
        v["pages"][0]["application"] = "editor.desktop".into();
        let bytes = serde_json::to_vec(&v).unwrap();
        assert_eq!(app_page(&bytes, "editor.desktop").unwrap(), Some(0));
        assert_eq!(app_page(&bytes, "unknown.desktop").unwrap(), Some(1));
        assert_eq!(app_page(&bytes, crate::foreground::STUDIO).unwrap(), None);
    }
}
