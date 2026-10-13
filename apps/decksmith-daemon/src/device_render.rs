//! One bounded sleeping render owner per saved device. No transport or global
//! service access; results and pending state are latest-wins and context fenced.
use crate::{
    pages::TouchState,
    profile_control::{Frame, Renderer},
    shared_worker::Authority,
};
use decksmith_core::{
    device_context::DeviceId, device_model::DeviceModel, shared_dispatch::ActionScope,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
};
type Result<T> = std::result::Result<T, &'static str>;
struct Job {
    seq: u64,
    scope: ActionScope,
    model: DeviceModel,
    layout: Arc<[u8]>,
    page: u8,
    state: TouchState,
    full: bool,
}
pub enum Update {
    Full(Frame),
    Touch(Vec<u8>),
}
pub struct Reply {
    pub scope: ActionScope,
    pub frame: Result<Update>,
}
struct State {
    seq: u64,
    pending: Option<Job>,
    reply: Option<Reply>,
    stop: bool,
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}
struct Owner {
    shared: Arc<Shared>,
    thread: JoinHandle<()>,
}
pub struct Pool {
    owners: BTreeMap<DeviceId, Owner>,
    authority: Authority,
}
impl Pool {
    pub fn start(devices: &[DeviceId], authority: Authority) -> Result<Self> {
        if devices.is_empty()
            || devices.len() > 8
            || devices
                .iter()
                .enumerate()
                .any(|(i, id)| devices[..i].contains(id))
        {
            return Err("invalid_render_roster");
        }
        let mut pool = Self {
            owners: BTreeMap::new(),
            authority,
        };
        for id in devices {
            let shared = Arc::new(Shared {
                state: Mutex::new(State {
                    seq: 0,
                    pending: None,
                    reply: None,
                    stop: false,
                }),
                wake: Condvar::new(),
            });
            let owner = shared.clone();
            let current = pool.authority.clone();
            let thread = std::thread::Builder::new()
                .name("decksmith-device-render".into())
                .spawn(move || run(owner, current))
                .map_err(|_| "render_thread_failed")?;
            pool.owners.insert(*id, Owner { shared, thread });
        }
        Ok(pool)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn request(
        &self,
        scope: ActionScope,
        model: DeviceModel,
        layout: Arc<[u8]>,
        page: u8,
        state: TouchState,
        full: bool,
    ) -> Result<()> {
        if !(self.authority)(&scope) {
            return Err("stale_render_context");
        }
        if layout.len() > crate::layout_document::LIMIT
            || state.targets.len() > 32
            || state.levels.len() > 32
            || state.feedback.len() > 32
        {
            return Err("render_state_too_large");
        }
        let owner = self
            .owners
            .get(&scope.device.device)
            .ok_or("unknown_render_device")?;
        let mut s = owner.shared.state.lock().unwrap();
        if s.stop {
            return Err("render_stopped");
        }
        s.seq = s.seq.checked_add(1).ok_or("render_revision_exhausted")?;
        s.pending = Some(Job {
            seq: s.seq,
            scope,
            model,
            layout,
            page,
            state,
            full,
        });
        s.reply = None;
        drop(s);
        owner.shared.wake.notify_one();
        Ok(())
    }
    pub fn poll(&self) -> Vec<Reply> {
        self.owners
            .values()
            .filter_map(|o| o.shared.state.lock().unwrap().reply.take())
            .filter(|r| (self.authority)(&r.scope))
            .collect()
    }
    pub fn invalidate(&self, id: DeviceId) {
        if let Some(o) = self.owners.get(&id) {
            let mut s = o.shared.state.lock().unwrap();
            s.seq = s.seq.saturating_add(1);
            s.pending = None;
            s.reply = None;
        }
    }
    pub fn stop(&self) {
        for o in self.owners.values() {
            let mut s = o.shared.state.lock().unwrap();
            s.stop = true;
            s.pending = None;
            s.reply = None;
            drop(s);
            o.shared.wake.notify_one();
        }
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        self.stop();
        for (_, o) in std::mem::take(&mut self.owners) {
            let _ = o.thread.join();
        }
    }
}
fn run(owner: Arc<Shared>, current: Authority) {
    let mut cache: Option<(Arc<[u8]>, DeviceModel, Renderer)> = None;
    loop {
        let job = {
            let mut s = owner.state.lock().unwrap();
            while s.pending.is_none() && !s.stop {
                s = owner.wake.wait(s).unwrap();
            }
            if s.stop {
                return;
            }
            s.pending.take().unwrap()
        };
        if !current(&job.scope) {
            continue;
        }
        let frame = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if cache
                .as_ref()
                .is_none_or(|(raw, model, _)| !Arc::ptr_eq(raw, &job.layout) || *model != job.model)
            {
                cache = Some((
                    job.layout.clone(),
                    job.model,
                    Renderer::new(&job.layout, job.model)?,
                ));
            }
            if !current(&job.scope) {
                return Err("stale_render_context");
            }
            if job.full {
                cache
                    .as_mut()
                    .unwrap()
                    .2
                    .frame_state(job.page, job.state)
                    .map(Update::Full)
            } else {
                cache
                    .as_mut()
                    .unwrap()
                    .2
                    .touch_frame(job.page, job.state)
                    .map(Update::Touch)
            }
        }))
        .unwrap_or(Err("render_failed"));
        let mut s = owner.state.lock().unwrap();
        if !s.stop && s.seq == job.seq && current(&job.scope) {
            s.reply = Some(Reply {
                scope: job.scope,
                frame,
            });
        }
    }
}
