//! Bounded per-device I/O ownership. Input and latest display slots are separate;
//! one transport may wait without blocking the supervisor or another transport.
use crate::{DeckDevice, DeviceError};
use decksmith_core::{InputEvent, device_context::DeviceId, device_model::DeviceDescriptor};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
const INPUT_LIMIT: usize = 128;
struct State {
    desired: Vec<Option<Arc<[u8]>>>,
    sent: Vec<Option<Arc<[u8]>>>,
    brightness: Option<u8>,
    input: VecDeque<(InputEvent, Option<Instant>)>,
    error: Option<DeviceError>,
    stop: bool,
    full_frame: bool,
    writing: bool,
    cursor: usize,
    last_read: Instant,
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
    alive: AtomicBool,
    epoch: AtomicU64,
    drained: AtomicU64,
}
#[derive(Clone)]
pub struct Lease(Arc<Shared>);
impl Lease {
    pub fn epoch(&self) -> u64 {
        self.0.epoch.load(Ordering::Acquire)
    }
    pub fn accepts(&self, epoch: u64) -> bool {
        self.0.alive.load(Ordering::Acquire) && self.epoch() == epoch
    }
    /// Microphone hold leases renew only while this I/O owner is progressing.
    pub fn recent(&self, age: Duration) -> bool {
        self.0.state.lock().unwrap().last_read.elapsed() <= age
    }
    pub fn ready(&self) -> bool {
        let s = self.0.state.lock().unwrap();
        self.0.alive.load(Ordering::Acquire)
            && s.full_frame
            && !s.writing
            && s.brightness.is_none()
            && s.desired == s.sent
    }
}
pub struct Device {
    descriptor: DeviceDescriptor,
    shared: Arc<Shared>,
    receipt: Option<Instant>,
}
impl Device {
    pub fn lease(&self) -> Lease {
        Lease(self.shared.clone())
    }
    fn state(&self) -> Result<std::sync::MutexGuard<'_, State>, DeviceError> {
        let s = self.shared.state.lock().unwrap();
        if let Some(error) = s.error {
            return Err(error);
        }
        if s.stop {
            return Err(DeviceError::Disconnected);
        }
        Ok(s)
    }
    fn shutdown(&self) {
        let mut s = self.shared.state.lock().unwrap();
        s.stop = true;
        self.shared.alive.store(false, Ordering::Release);
        self.shared.epoch.fetch_add(1, Ordering::AcqRel);
        s.input.clear();
        s.desired.fill(None);
        drop(s);
        self.shared.wake.notify_one();
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.shutdown();
    }
}
impl DeckDevice for Device {
    fn descriptor(&self) -> DeviceDescriptor {
        self.descriptor.clone()
    }
    fn set_key_image(&mut self, index: u8, rgb: &[u8]) -> Result<(), DeviceError> {
        if index >= self.descriptor.key_count() {
            return Err(DeviceError::InvalidKey);
        }
        if rgb.len() != self.descriptor.key_rgb_bytes() {
            return Err(DeviceError::InvalidKeyImage);
        }
        self.state()?.desired[usize::from(index)] = Some(Arc::from(rgb));
        self.shared.wake.notify_one();
        Ok(())
    }
    fn set_touch_image(&mut self, rgb: &[u8]) -> Result<(), DeviceError> {
        let surface = self
            .descriptor
            .capabilities()
            .touch
            .ok_or(DeviceError::InvalidTouchImage)?;
        if rgb.len() != surface.rgb_bytes() {
            return Err(DeviceError::InvalidTouchImage);
        }
        self.state()?.desired[usize::from(self.descriptor.key_count())] = Some(Arc::from(rgb));
        self.shared.wake.notify_one();
        Ok(())
    }
    fn queue_frame(&mut self, keys: &[u8], touch: Option<&[u8]>) -> Result<(), DeviceError> {
        if keys.len() != self.descriptor.key_rgb_bytes() * usize::from(self.descriptor.key_count())
            || match (self.descriptor.capabilities().touch, touch) {
                (Some(t), Some(b)) => t.rgb_bytes() != b.len(),
                (None, None) => false,
                _ => true,
            }
        {
            return Err(DeviceError::InvalidKeyImage);
        }
        // Validate and prepare every surface before changing shared desired state.
        let mut frames: Vec<Option<Arc<[u8]>>> = keys
            .chunks_exact(self.descriptor.key_rgb_bytes())
            .map(|b| Some(Arc::from(b)))
            .collect();
        if let Some(touch) = touch {
            frames.push(Some(Arc::from(touch)))
        }
        let mut s = self.state()?;
        s.desired = frames;
        s.full_frame = true;
        drop(s);
        self.shared.wake.notify_one();
        Ok(())
    }
    fn set_brightness(&mut self, percent: u8) -> Result<(), DeviceError> {
        if percent > 100 {
            return Err(DeviceError::InvalidBrightness);
        }
        self.state()?.brightness = Some(percent);
        self.shared.wake.notify_one();
        Ok(())
    }
    fn poll_event(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        self.poll_event_nonblocking()
    }
    fn poll_event_nonblocking(&mut self) -> Result<Option<InputEvent>, DeviceError> {
        let next = self.state()?.input.pop_front();
        self.receipt = next.as_ref().and_then(|(_, stamp)| *stamp);
        Ok(next.map(|(event, _)| event))
    }
    fn event_received_at(&self) -> Option<Instant> {
        self.receipt
    }
    fn input_drained(&self) -> bool {
        self.shared.drained.load(Ordering::Acquire) == self.shared.epoch.load(Ordering::Acquire)
    }
    fn display_ready(&self) -> bool {
        self.lease().ready()
    }
    fn invalidate_outputs(&mut self) {
        let mut s = self.shared.state.lock().unwrap();
        self.shared.epoch.fetch_add(1, Ordering::AcqRel);
        s.desired.fill(None);
        s.sent.fill(None);
        s.full_frame = false;
        // Keep queued edges for the manager's held-control drain. Only an actual
        // empty adapter read acknowledges this new epoch, not an empty mailbox.
        drop(s);
        self.shared.wake.notify_one();
    }
    fn blank(&mut self) -> Result<(), DeviceError> {
        self.shutdown();
        Ok(())
    }
}
struct Owner {
    shared: Arc<Shared>,
    thread: JoinHandle<Result<(), DeviceError>>,
}
#[derive(Default)]
pub struct Pool {
    owners: BTreeMap<DeviceId, Owner>,
}
impl Pool {
    /// Includes retiring owners: a stuck writer cannot grow the thread budget
    /// or be replaced by another writer for the same durable device identity.
    pub fn spawn(
        &mut self,
        id: DeviceId,
        mut device: Box<dyn DeckDevice + Send>,
    ) -> Result<Device, DeviceError> {
        self.reap();
        if self.owners.len() >= crate::registry::MAX_DEVICES || self.owners.contains_key(&id) {
            return Err(DeviceError::DeviceBusy);
        }
        let descriptor = device.descriptor();
        let surfaces = usize::from(descriptor.key_count())
            + usize::from(descriptor.capabilities().touch.is_some());
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                desired: vec![None; surfaces],
                sent: vec![None; surfaces],
                brightness: None,
                input: VecDeque::with_capacity(INPUT_LIMIT),
                error: None,
                stop: false,
                full_frame: false,
                writing: false,
                cursor: 0,
                last_read: Instant::now(),
            }),
            wake: Condvar::new(),
            alive: AtomicBool::new(true),
            epoch: AtomicU64::new(1),
            drained: AtomicU64::new(0),
        });
        let owner = shared.clone();
        let model = descriptor.clone();
        let thread = std::thread::Builder::new()
            .name("decksmith-device-io".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(&owner, &model, &mut *device)
                }));
                // In-flight writes cannot be revoked. Blank only after this sole
                // owner's write has ended, never through a concurrent replacement.
                if result.is_err() {
                    fail(&owner, DeviceError::Transport);
                }
                let blank =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| device.blank()))
                        .unwrap_or(Err(DeviceError::Transport));
                if let Err(error) = blank {
                    fail(&owner, error);
                }
                owner.alive.store(false, Ordering::Release);
                match result {
                    Ok(Ok(())) => blank,
                    Ok(Err(e)) => Err(e),
                    Err(_) => {
                        fail(&owner, DeviceError::Transport);
                        Err(DeviceError::Transport)
                    }
                }
            })
            .map_err(|_| DeviceError::Transport)?;
        self.owners.insert(
            id,
            Owner {
                shared: shared.clone(),
                thread,
            },
        );
        Ok(Device {
            descriptor,
            shared,
            receipt: None,
        })
    }
    pub fn reap(&mut self) -> Vec<(DeviceId, Result<(), DeviceError>)> {
        let finished: Vec<_> = self
            .owners
            .iter()
            .filter(|(_, o)| o.thread.is_finished())
            .map(|(id, _)| *id)
            .collect();
        finished
            .into_iter()
            .map(|id| {
                let o = self.owners.remove(&id).unwrap();
                (id, o.thread.join().unwrap_or(Err(DeviceError::Transport)))
            })
            .collect()
    }
    pub fn stop(&self) {
        for o in self.owners.values() {
            let mut s = o.shared.state.lock().unwrap();
            s.stop = true;
            o.shared.alive.store(false, Ordering::Release);
            o.shared.epoch.fetch_add(1, Ordering::AcqRel);
            drop(s);
            o.shared.wake.notify_one();
        }
    }
    /// Call after signalling every owner. The supplied adapter must bound its
    /// I/O; Rust cannot forcibly terminate an arbitrary blocked driver thread.
    pub fn join(&mut self) -> Vec<(DeviceId, Result<(), DeviceError>)> {
        self.stop();
        std::mem::take(&mut self.owners)
            .into_iter()
            .map(|(id, o)| (id, o.thread.join().unwrap_or(Err(DeviceError::Transport))))
            .collect()
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        self.join();
    }
}
fn fail(shared: &Shared, error: DeviceError) {
    let mut s = shared.state.lock().unwrap();
    s.error = Some(error);
    s.stop = true;
    shared.alive.store(false, Ordering::Release);
    shared.epoch.fetch_add(1, Ordering::AcqRel);
    s.input.clear();
    shared.wake.notify_one();
}
fn run(
    shared: &Shared,
    descriptor: &DeviceDescriptor,
    device: &mut dyn DeckDevice,
) -> Result<(), DeviceError> {
    loop {
        if shared.state.lock().unwrap().stop {
            return Ok(());
        }
        let epoch = shared.epoch.load(Ordering::Acquire);
        let input = match device.poll_event_nonblocking() {
            Ok(v) => v,
            Err(e) => {
                fail(shared, e);
                return Err(e);
            }
        };
        let receipt = device.event_received_at();
        let active = input.is_some();
        {
            let mut s = shared.state.lock().unwrap();
            if s.stop {
                return Ok(());
            }
            s.last_read = Instant::now();
            if let Some(input) = input {
                if s.input.len() == INPUT_LIMIT {
                    drop(s);
                    fail(shared, DeviceError::QueueFull);
                    return Err(DeviceError::QueueFull);
                }
                s.input.push_back((input, receipt));
            } else if shared.epoch.load(Ordering::Acquire) == epoch {
                shared.drained.store(epoch, Ordering::Release)
            }
        }
        enum Output {
            Brightness(u8),
            Image(usize, Arc<[u8]>),
        }
        let output = {
            let mut s = shared.state.lock().unwrap();
            if s.stop {
                return Ok(());
            }
            let out = if let Some(b) = s.brightness.take() {
                Some(Output::Brightness(b))
            } else {
                let count = s.desired.len();
                let found = (0..count)
                    .map(|n| (s.cursor + n) % count)
                    .find(|i| s.desired[*i].is_some() && s.desired[*i] != s.sent[*i]);
                found.map(|i| {
                    s.cursor = (i + 1) % count;
                    Output::Image(i, s.desired[i].as_ref().unwrap().clone())
                })
            };
            s.writing = out.is_some();
            out.map(|o| (shared.epoch.load(Ordering::Acquire), o))
        };
        if let Some((epoch, output)) = output {
            // A superseding fence can cancel selected work, but not an issued
            // USB call. Completion is never credited to a later generation.
            if shared.epoch.load(Ordering::Acquire) == epoch && shared.alive.load(Ordering::Acquire)
            {
                let result = match &output {
                    Output::Brightness(b) => device.set_brightness(*b),
                    Output::Image(i, b) => {
                        if *i < usize::from(descriptor.key_count()) {
                            device.set_key_image(*i as u8, b)
                        } else {
                            device.set_touch_image(b)
                        }
                    }
                };
                if let Err(e) = result {
                    fail(shared, e);
                    return Err(e);
                }
                let mut s = shared.state.lock().unwrap();
                if shared.epoch.load(Ordering::Acquire) == epoch {
                    if let Output::Image(i, bytes) = output {
                        s.sent[i] = Some(bytes)
                    }
                } else {
                    s.sent.fill(None)
                }
                s.writing = false;
            } else {
                shared.state.lock().unwrap().writing = false;
            }
        } else {
            let s = shared.state.lock().unwrap();
            if !s.stop {
                let _ = shared
                    .wake
                    .wait_timeout(
                        s,
                        if active {
                            Duration::from_millis(5)
                        } else {
                            Duration::from_millis(20)
                        },
                    )
                    .unwrap();
            }
        }
    }
}
