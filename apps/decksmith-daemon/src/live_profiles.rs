//! Opt-in live profile bridge. One bounded disk/preview/preparation owner; the
//! input coordinator only handles already prepared snapshots and control fences.
use crate::{
    device_execution::{EditTicket, PreparedEdit, Runtime},
    device_profiles::{Catalog, Store},
    shared_native_media::Session,
};
use decksmith_core::device_context::{DeviceContext, DeviceId};
use decksmith_core::device_model::ControlAddress;
use decksmith_device::manager::SessionStatus;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
pub(crate) struct View {
    pub runtime: DeviceId,
    pub catalog: Arc<Catalog>,
    pub sessions: Vec<SessionStatus>,
    pub contexts: BTreeMap<DeviceId, DeviceContext>,
    pub bindings: Vec<Session>,
}
enum Operation {
    Reserve(DeviceContext),
    Cancel(EditTicket),
    Apply(EditTicket, Catalog, Box<PreparedEdit>),
    Select(Catalog),
    Page(DeviceContext, u8),
    Test(DeviceContext, String, u8, u8),
}
enum Reply {
    Ticket(EditTicket),
    Done,
}
struct Request {
    operation: Operation,
    cancelled: Arc<AtomicBool>,
    answer: SyncSender<Result<Reply>>,
}
#[derive(Clone)]
pub(crate) struct Link {
    queue: SyncSender<Request>,
    view: Arc<Mutex<Arc<View>>>,
}
impl Link {
    pub fn view(&self) -> Arc<View> {
        self.view.lock().unwrap().clone()
    }
    fn call(&self, operation: Operation, cancelled: Arc<AtomicBool>) -> Result<Reply> {
        let (send, receive) = mpsc::sync_channel(1);
        let finish = matches!(operation, Operation::Apply(..) | Operation::Cancel(..));
        self.queue
            .try_send(Request {
                operation,
                cancelled: cancelled.clone(),
                answer: send,
            })
            .map_err(|_| "live_profile_queue_busy")?;
        match receive.recv_timeout(Duration::from_secs(2)) {
            Ok(reply) => reply,
            Err(_) => {
                if !finish {
                    cancelled.store(true, Ordering::Release);
                }
                Err("live_profile_apply_uncertain")
            }
        }
    }
    pub fn reserve(
        &self,
        context: DeviceContext,
        cancelled: Arc<AtomicBool>,
    ) -> Result<EditTicket> {
        match self.call(Operation::Reserve(context), cancelled)? {
            Reply::Ticket(t) => Ok(t),
            _ => Err("invalid_live_reply"),
        }
    }
    pub fn cancel(&self, ticket: EditTicket) -> Result<()> {
        self.call(Operation::Cancel(ticket), Arc::new(AtomicBool::new(false)))
            .map(|_| ())
    }
    pub fn apply(
        &self,
        ticket: EditTicket,
        catalog: Catalog,
        prepared: PreparedEdit,
    ) -> Result<()> {
        self.call(
            Operation::Apply(ticket, catalog, Box::new(prepared)),
            Arc::new(AtomicBool::new(false)),
        )
        .map(|_| ())
    }
    pub fn select(&self, catalog: Catalog, cancelled: Arc<AtomicBool>) -> Result<()> {
        self.call(Operation::Select(catalog), cancelled).map(|_| ())
    }
    pub fn test_control(
        &self,
        context: DeviceContext,
        raw: String,
        page: u8,
        slot: u8,
        cancelled: Arc<AtomicBool>,
    ) -> Result<()> {
        self.call(Operation::Test(context, raw, page, slot), cancelled)
            .map(|_| ())
    }
    pub fn page(&self, context: DeviceContext, page: u8, cancelled: Arc<AtomicBool>) -> Result<()> {
        self.call(Operation::Page(context, page), cancelled)
            .map(|_| ())
    }
}
pub(crate) struct Service {
    incoming: Receiver<Request>,
    view: Arc<Mutex<Arc<View>>>,
    connection: Option<zbus::Connection>,
    thread: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    next_publish: Instant,
}
impl Service {
    pub fn start(store: Store, runtime: &Runtime, native: bool) -> Result<Self> {
        Self::start_mode(store, runtime, native, false)
    }
    pub fn start_permanent(store: Store, runtime: &Runtime, native: bool) -> Result<Self> {
        Self::start_mode(store, runtime, native, true)
    }
    fn start_mode(store: Store, runtime: &Runtime, native: bool, permanent: bool) -> Result<Self> {
        let (queue, incoming) = mpsc::sync_channel(8);
        let view = Arc::new(Mutex::new(Arc::new(snapshot(
            runtime,
            Arc::new(runtime.catalog().clone()),
        ))));
        let link = Link {
            queue,
            view: view.clone(),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (connection, thread) =
            crate::profile_control::start_connected(store, link, native, stop.clone(), permanent)?;
        Ok(Self {
            incoming,
            view,
            connection: Some(connection),
            thread: Some(thread),
            stop,
            next_publish: Instant::now(),
        })
    }
    /// At most one prepared operation per coordinator turn. No filesystem,
    /// MPRIS discovery, renderer work or owned-thread joins occur here.
    pub fn poll(&mut self, runtime: &mut Runtime) {
        if let Ok(request) = self.incoming.try_recv() {
            let result = if request.cancelled.load(Ordering::Acquire) {
                Err("profile_request_cancelled")
            } else {
                match request.operation {
                    Operation::Reserve(context) => runtime.reserve_edit(context).map(Reply::Ticket),
                    Operation::Cancel(ticket) => runtime.cancel_edit(ticket).map(|_| Reply::Done),
                    Operation::Apply(ticket, catalog, prepared) => runtime
                        .apply_edit(ticket, catalog, *prepared)
                        .map(|_| Reply::Done),
                    Operation::Select(catalog) => {
                        runtime.apply_selection(catalog).map(|_| Reply::Done)
                    }
                    Operation::Test(context, raw, page, slot) => runtime
                        .test_control(context, &raw, page, slot)
                        .map(|_| Reply::Done),
                    Operation::Page(context, page) => {
                        runtime.navigate(context, page).map(|_| Reply::Done)
                    }
                }
            };
            self.next_publish = Instant::now();
            self.publish(runtime);
            let _ = request.answer.try_send(result);
        }
    }
    /// Revoke requests immediately; never wait for an in-flight disk/preview
    /// task before the coordinator fences effects and blanks physical displays.
    pub fn stop_accepting(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.connection = None;
    }
    pub fn publish(&mut self, runtime: &Runtime) {
        if Instant::now() < self.next_publish {
            return;
        }
        self.next_publish = Instant::now() + Duration::from_millis(50);
        let old = self.view.lock().unwrap().clone();
        let catalog = if old.catalog.generation == runtime.catalog().generation {
            old.catalog.clone()
        } else {
            Arc::new(runtime.catalog().clone())
        };
        let next = snapshot(runtime, catalog);
        if next.sessions != old.sessions
            || next.catalog.generation != old.catalog.generation
            || next.bindings != old.bindings
        {
            *self.view.lock().unwrap() = Arc::new(next);
        }
    }
}
fn snapshot(runtime: &Runtime, catalog: Arc<Catalog>) -> View {
    let sessions = runtime.statuses();
    let contexts = sessions
        .iter()
        .filter_map(|s| {
            runtime
                .context(s.id, ControlAddress::Key(0))
                .ok()
                .map(|c| (s.id, c))
        })
        .collect();
    View {
        runtime: runtime.instance(),
        catalog,
        sessions,
        contexts,
        bindings: runtime.bindings(),
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.stop_accepting();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
