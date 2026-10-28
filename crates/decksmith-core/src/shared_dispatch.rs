//! Bounded, deterministic scheduling only. The host supplies current fences and
//! performs I/O elsewhere; selecting/beginning a job never executes an effect.
use crate::device_context::{DeviceContext, DeviceId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{collections::VecDeque, sync::Arc, time::Instant};

pub const MAX_DEVICES: usize = 8;
pub const MAX_PENDING: usize = 32;
pub const PER_DEVICE_PENDING: usize = 16;
pub const MAX_INFLIGHT: usize = 4;
type Result<T> = std::result::Result<T, &'static str>;
static DISPATCH_EPOCH: AtomicU64 = AtomicU64::new(0);

/// Host-canonicalized resource key, never a display label. The host must resolve
/// aliases (e.g. default microphone) before comparing keys. This structural
/// check does not validate a real audio target or grant plugin permissions.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetId(Arc<str>);
impl TargetId {
    pub fn new(key: &str) -> Result<Self> {
        if key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
            return Err("invalid_target_key");
        }
        Ok(Self(key.into()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for TargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TargetId([private])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionScope {
    /// Random host-startup nonce supplied by the host. Tests use
    /// explicit fixtures. Never derive from serials, PID or a device ID.
    pub runtime: DeviceId,
    pub device: DeviceContext,
    pub foreground: u64,
}
impl ActionScope {
    pub fn structurally_valid(&self) -> bool {
        self.device.supervisor != 0 && self.device.connection != 0 && self.device.binding != 0
    }
}

/// Internal scheduling classifications, not saved action IDs or a plugin wire
/// contract. The host compiles validated built-in settings into these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Adjust(i16),
    Toggle,
    Set(bool),
    Activate,
}
impl Effect {
    fn valid(self) -> bool {
        !matches!(self, Self::Adjust(0))
            && !matches!(self, Self::Adjust(n) if !(-100..=100).contains(&n))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestId {
    runtime: DeviceId,
    dispatcher: u64,
    sequence: u64,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: RequestId,
    pub scope: ActionScope,
    pub target: TargetId,
    pub effect: Effect,
    pub received_at: Option<Instant>,
    pub submissions: u16,
    last_submission: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Selected,
    Started,
}
struct Active {
    job: Job,
    phase: Phase,
}
struct Owner {
    id: DeviceId,
    jobs: VecDeque<Job>,
}
pub struct Completion {
    pub id: RequestId,
    pub scope: ActionScope,
    pub result: Result<()>,
}
pub struct Dispatcher {
    epoch: u64,
    runtime: DeviceId,
    owners: Vec<Owner>,
    active: Vec<Active>,
    next_id: u64,
    cursor: usize,
    stopped: bool,
    cancelled: u64,
}
impl Dispatcher {
    pub fn new(runtime: DeviceId, devices: &[DeviceId]) -> Result<Self> {
        if devices.len() > MAX_DEVICES
            || devices
                .iter()
                .enumerate()
                .any(|(i, id)| devices[..i].contains(id))
        {
            return Err("invalid_dispatch_devices");
        }
        let epoch = DISPATCH_EPOCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_add(1)
            })
            .map_err(|_| "dispatcher_epoch_exhausted")?
            + 1;
        Ok(Self {
            epoch,
            runtime,
            owners: devices
                .iter()
                .map(|id| Owner {
                    id: *id,
                    jobs: VecDeque::new(),
                })
                .collect(),
            active: Vec::new(),
            next_id: 0,
            cursor: 0,
            stopped: false,
            cancelled: 0,
        })
    }
    /// Bounded inventory for the host's immutable payload side table. Includes
    /// started reservations so cancellation cannot discard an in-flight writer.
    pub fn request_ids(&self) -> Vec<RequestId> {
        self.owners
            .iter()
            .flat_map(|o| o.jobs.iter().map(|j| j.id))
            .chain(self.active.iter().map(|a| a.job.id))
            .collect()
    }
    pub fn pending(&self) -> usize {
        self.owners.iter().map(|o| o.jobs.len()).sum()
    }
    pub fn inflight(&self) -> usize {
        self.active.len()
    }
    pub fn cancelled(&self) -> u64 {
        self.cancelled
    }
    fn allowed(&self, scope: &ActionScope, current: &impl Fn(&ActionScope) -> bool) -> bool {
        !self.stopped
            && scope.runtime == self.runtime
            && scope.structurally_valid()
            && current(scope)
    }
    pub fn submit(
        &mut self,
        scope: ActionScope,
        target: TargetId,
        effect: Effect,
        received_at: Option<Instant>,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<RequestId> {
        if !self.allowed(&scope, &current) {
            return Err("stale_action_scope");
        }
        if !effect.valid() {
            return Err("invalid_action_effect");
        }
        self.cancel_stale(&current);
        let index = self
            .owners
            .iter()
            .position(|o| o.id == scope.device.device)
            .ok_or("unknown_action_device")?;
        let id = self.next_id.checked_add(1).ok_or("request_id_exhausted")?;
        if let Some(tail) = self.owners[index].jobs.back() {
            let intervening = self
                .owners
                .iter()
                .flat_map(|o| &o.jobs)
                .any(|j| j.target == target && j.last_submission > tail.last_submission)
                || self.active.iter().any(|a| {
                    a.job.target == target && a.job.last_submission > tail.last_submission
                });
            if !intervening
                && tail.scope == scope
                && tail.target == target
                && let (Effect::Adjust(old), Effect::Adjust(new)) = (tail.effect, effect)
                && old.signum() == new.signum()
                && let Some(total) = old.checked_add(new).filter(|n| (-100..=100).contains(n))
            {
                let tail = self.owners[index].jobs.back_mut().expect("inspected tail");
                tail.effect = Effect::Adjust(total);
                tail.last_submission = id;
                tail.submissions += 1;
                self.next_id = id;
                return Ok(tail.id);
            }
        }
        if self.pending() >= MAX_PENDING || self.owners[index].jobs.len() >= PER_DEVICE_PENDING {
            return Err("action_queue_full");
        }
        self.owners[index].jobs.push_back(Job {
            id: RequestId {
                runtime: self.runtime,
                dispatcher: self.epoch,
                sequence: id,
            },
            scope,
            target,
            effect,
            received_at,
            submissions: 1,
            last_submission: id,
        });
        self.next_id = id;
        Ok(RequestId {
            runtime: self.runtime,
            dispatcher: self.epoch,
            sequence: id,
        })
    }
    /// Pending work is cancelled independently. Started effects retain their
    /// target reservation until the worker reports completion/termination; page
    /// changes must not let a conflicting effect overlap a still-running write.
    pub fn cancel_stale(&mut self, current: impl Fn(&ActionScope) -> bool) {
        for owner in &mut self.owners {
            let before = owner.jobs.len();
            owner
                .jobs
                .retain(|j| !self.stopped && j.scope.runtime == self.runtime && current(&j.scope));
            self.cancelled = self
                .cancelled
                .saturating_add((before - owner.jobs.len()) as u64);
        }
        let before = self.active.len();
        self.active.retain(|a| {
            a.phase == Phase::Started
                || (!self.stopped && a.job.scope.runtime == self.runtime && current(&a.job.scope))
        });
        self.cancelled = self
            .cancelled
            .saturating_add((before - self.active.len()) as u64);
    }
    /// Fair owner selection plus cross-owner target FIFO. A queued toggle/launch
    /// is never coalesced or retried, and newer target work cannot pass an older
    /// request hidden behind another request in an owner's queue.
    pub fn select(&mut self, current: impl Fn(&ActionScope) -> bool) -> Option<Job> {
        self.cancel_stale(&current);
        if self.stopped || self.active.len() >= MAX_INFLIGHT {
            return None;
        }
        let count = self.owners.len();
        for offset in 0..count {
            let index = (self.cursor + offset) % count;
            let Some(head) = self.owners[index].jobs.front() else {
                continue;
            };
            if self.active.iter().any(|a| {
                a.phase == Phase::Selected && a.job.scope.device.device == head.scope.device.device
            }) {
                continue;
            }
            if self.active.iter().any(|a| a.job.target == head.target)
                || self
                    .owners
                    .iter()
                    .flat_map(|o| &o.jobs)
                    .any(|j| j.target == head.target && j.id < head.id)
            {
                continue;
            }
            let job = self.owners[index].jobs.pop_front()?;
            self.active.push(Active {
                job: job.clone(),
                phase: Phase::Selected,
            });
            self.cursor = (index + 1) % count;
            return Some(job);
        }
        None
    }
    /// Recheck immediately before handing the effect to a backend. Calling this
    /// twice does not execute twice. A host must still check the fence at the
    /// external-write boundary if that backend waits or performs multiple writes.
    pub fn begin(&mut self, id: RequestId, current: impl Fn(&ActionScope) -> bool) -> Result<Job> {
        let index = self
            .active
            .iter()
            .position(|a| a.job.id == id)
            .ok_or("unknown_action_request")?;
        if self.active[index].phase != Phase::Selected {
            return Err("action_already_started");
        }
        if !self.allowed(&self.active[index].job.scope, &current) {
            self.active.remove(index);
            self.cancelled = self.cancelled.saturating_add(1);
            return Err("stale_action_scope");
        }
        self.active[index].phase = Phase::Started;
        Ok(self.active[index].job.clone())
    }
    /// Always release this request's reservation; publish feedback only while
    /// its full scope remains current. Never undo/retry an already issued effect.
    pub fn finish(
        &mut self,
        id: RequestId,
        result: Result<()>,
        current: impl Fn(&ActionScope) -> bool,
    ) -> Result<Option<Completion>> {
        let index = self
            .active
            .iter()
            .position(|a| a.job.id == id)
            .ok_or("unknown_action_request")?;
        if self.active[index].phase != Phase::Started {
            return Err("action_not_started");
        }
        let job = self.active.remove(index).job;
        Ok(self.allowed(&job.scope, &current).then_some(Completion {
            id,
            scope: job.scope,
            result,
        }))
    }
    /// Call only after a failed worker's external writes have terminated. This
    /// drops its reservation without retrying an uncertain external effect.
    pub fn worker_terminated(&mut self, id: RequestId) -> Result<()> {
        let index = self
            .active
            .iter()
            .position(|a| a.job.id == id)
            .ok_or("unknown_action_request")?;
        self.active.remove(index);
        self.cancelled = self.cancelled.saturating_add(1);
        Ok(())
    }
    pub fn stop(&mut self) {
        self.stopped = true;
        self.cancel_stale(|_| false);
    }
}
