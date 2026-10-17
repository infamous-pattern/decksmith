//! Device-addressed profile APIs. The virtual lab and saved-device workspace
//! have separate endpoints; neither replaces Control1 or dispatches live actions.
use crate::{
    device_profiles::{Catalog, Controls, Enrollment, Store, random_device_id},
    pages::{Pages, TouchState},
};
use decksmith_core::{
    device_context::{DeviceContext, DeviceId},
    device_model::{ControlAddress, DeviceDescriptor, DeviceModel},
};
use decksmith_device::{
    VirtualDeck,
    registry::{Candidate, Discovery},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
const LIMIT: usize = 1_048_576;

pub(crate) struct Renderer {
    pages: Pages,
    descriptor: DeviceDescriptor,
}
pub(crate) struct Frame {
    pub(crate) keys: Vec<u8>,
    pub(crate) touch: Option<Vec<u8>>,
}
impl Renderer {
    pub(crate) fn new(bytes: &[u8], model: DeviceModel) -> Result<Self> {
        Ok(Self {
            pages: Pages::model_renderer(bytes, model)?,
            descriptor: DeviceDescriptor::for_model(model),
        })
    }
    fn keys(&mut self, page: u8, state: TouchState) -> Result<Vec<u8>> {
        let full = self.pages.preview_keys(page, state)?;
        let (width, height) = self.descriptor.geometry().key_pixels;
        let keys = if (width, height) == (120, 120) {
            full
        } else {
            let mut bytes = Vec::with_capacity(
                self.descriptor.key_rgb_bytes() * usize::from(self.descriptor.key_count()),
            );
            for key in full.as_chunks::<{ 120 * 120 * 3 }>().0 {
                let image = image::RgbImage::from_raw(120, 120, key.to_vec())
                    .ok_or("invalid_device_frame")?;
                bytes.extend(
                    image::imageops::resize(
                        &image,
                        u32::from(width),
                        u32::from(height),
                        image::imageops::FilterType::Lanczos3,
                    )
                    .into_raw(),
                );
            }
            bytes
        };
        Ok(keys)
    }
    pub(crate) fn frame(&mut self, page: u8, brightness: u8) -> Result<Frame> {
        self.frame_state(
            page,
            TouchState {
                brightness: Some(brightness),
                ..Default::default()
            },
        )
    }
    pub(crate) fn touch_frame(&mut self, page: u8, state: TouchState) -> Result<Vec<u8>> {
        if self.descriptor.capabilities().touch.is_none() {
            return Err("device_has_no_touch");
        }
        self.pages.preview_touch(page, state)
    }
    pub(crate) fn frame_state(&mut self, page: u8, state: TouchState) -> Result<Frame> {
        let keys = self.keys(page, state.clone())?;
        let touch = if self.descriptor.capabilities().touch.is_some() {
            Some(self.pages.preview_touch(page, state)?)
        } else {
            None
        };
        Ok(Frame { keys, touch })
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Token {
    runtime: DeviceId,
    revision: u64,
    connection: u64,
    binding: u64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    VirtualLab,
    SavedWorkspace,
    ConnectedWorkspace,
    PermanentService,
}
impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::VirtualLab => "virtual-lab",
            Self::SavedWorkspace => "saved-workspace",
            Self::ConnectedWorkspace => "connected-workspace",
            Self::PermanentService => "permanent-service",
        }
    }
}
struct Host {
    mode: Mode,
    discovery: Discovery,
    discovery_revision: u64,
    discovery_available: bool,
    import_plus: Option<Vec<u8>>,
    store: Store,
    catalog: Catalog,
    controls: Controls,
    runtime: DeviceId,
    renderers: BTreeMap<DeviceId, Renderer>,
    preview: Option<(DeviceId, String, Renderer)>,
    live: Option<crate::live_profiles::Link>,
    native: bool,
    cancelled: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}
impl Host {
    fn new(store: Store) -> Result<Self> {
        let catalog = store.load()?;
        let controls = Controls::from_catalog(&catalog)?;
        let mut renderers = BTreeMap::new();
        for item in catalog.summaries() {
            renderers.insert(
                item.id,
                Renderer::new(catalog.profile(item.id)?.document().bytes(), item.model)?,
            );
        }
        Ok(Self {
            mode: Mode::VirtualLab,
            discovery: Discovery::new(Vec::new())?,
            discovery_revision: 0,
            discovery_available: false,
            import_plus: None,
            store,
            catalog,
            controls,
            runtime: random_device_id()?,
            renderers,
            preview: None,
            live: None,
            native: false,
            cancelled: Arc::new(AtomicBool::new(false)),
            stopping: Arc::new(AtomicBool::new(false)),
        })
    }
    fn context(&self, id: DeviceId) -> Result<DeviceContext> {
        if let Some(live) = &self.live {
            return live
                .view()
                .contexts
                .get(&id)
                .copied()
                .ok_or("device_not_ready");
        }
        self.controls.manager().context(id, ControlAddress::Key(0))
    }
    fn token(&self, id: DeviceId) -> Result<Token> {
        let profile = self.catalog.profile(id)?;
        let statuses = self.live.as_ref().map_or_else(
            || self.controls.manager().statuses(),
            |link| link.view().sessions.clone(),
        );
        let status = statuses
            .into_iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")?;
        Ok(Token {
            runtime: self.runtime,
            revision: profile.revision,
            connection: status.connection,
            binding: status.binding,
        })
    }
    fn check_token(&self, id: DeviceId, token: &Token) -> Result<()> {
        let now = self.token(id)?;
        if now.runtime != token.runtime
            || now.revision != token.revision
            || now.connection != token.connection
            || now.binding != token.binding
        {
            return Err("stale_profile_request");
        }
        Ok(())
    }
    fn check(&self, id: DeviceId, token: &Token) -> Result<DeviceContext> {
        self.check_token(id, token)?;
        self.context(id)
    }
    fn devices(&self) -> Result<String> {
        let associations = self.catalog.associations(&self.discovery);
        let candidates: Vec<_> = self
            .discovery
            .candidates()
            .iter()
            .map(|c| {
                let association = associations
                    .iter()
                    .find(|(slot, _)| *slot == c.slot())
                    .map(|(_, a)| a);
                serde_json::json!({"slot":c.slot(),"model":c.model(),"association":association})
            })
            .collect();
        let sessions = self.live.as_ref().map_or_else(
            || self.controls.manager().statuses(),
            |link| link.view().sessions.clone(),
        );
        Ok(serde_json::json!({"api_version":2,"mode":self.mode.name(),"runtime":self.runtime,"generation":self.catalog.generation,"selected":self.catalog.selected,"devices":self.catalog.summaries(),"sessions":sessions,"history":self.store.history_status()?,"discovery":{"available":self.discovery_available,"revision":self.discovery_revision,"candidates":candidates},"actions_available":self.mode == Mode::PermanentService,"live_profiles":self.live.is_some(),"native_actions":self.native,"legacy_import_prepared":self.import_plus.is_some()}).to_string())
    }
    fn profile(&self, id: DeviceId) -> Result<String> {
        let p = self.catalog.profile(id)?;
        // RawValue preserves unavailable-provider integer precision in the wire response.
        let layout: Box<serde_json::value::RawValue> =
            serde_json::from_slice(p.document().bytes()).map_err(|_| "invalid_config")?;
        #[derive(Serialize)]
        struct ProfileReply {
            id: DeviceId,
            token: Token,
            saved_page: u8,
            layout: Box<serde_json::value::RawValue>,
            name: String,
            brightness: u8,
        }
        let summary = self
            .catalog
            .summaries()
            .into_iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")?;
        serde_json::to_string(&ProfileReply {
            id,
            token: self.token(id)?,
            saved_page: p.saved_page,
            layout,
            name: summary.name.into(),
            brightness: summary.brightness,
        })
        .map_err(|_| "profile_reply_failed")
    }
    fn render(&mut self, id: DeviceId) -> Result<()> {
        let context = self.context(id)?;
        let page = if let Some(live) = &self.live {
            live.view()
                .sessions
                .iter()
                .find(|s| s.id == id)
                .and_then(|s| s.page)
                .ok_or("device_not_ready")?
        } else {
            self.controls.manager().active_page(id)?
        };
        let brightness = self
            .controls
            .manager()
            .statuses()
            .into_iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")?
            .brightness;
        let frame = self
            .renderers
            .get_mut(&id)
            .ok_or("device_profile_missing")?
            .frame(page, brightness)?;
        self.controls
            .paint(context, &frame.keys, frame.touch.as_deref())
    }
    fn test_control(
        &self,
        id: DeviceId,
        token: Token,
        raw: &str,
        page: u8,
        slot: u8,
    ) -> Result<String> {
        let context = self.check(id, &token)?;
        let live = self.live.as_ref().ok_or("live_controls_unavailable")?;
        live.test_control(context, raw.into(), page, slot, self.cancelled.clone())?;
        Ok("{}".into())
    }
    fn archive(&mut self, expected: u64) -> Result<String> {
        serde_json::to_string(&self.store.archive_sibling(expected, 8)?)
            .map_err(|_| "invalid_profile_retention")
    }
    fn metadata(
        &mut self,
        id: DeviceId,
        token: Token,
        name: &str,
        brightness: u8,
    ) -> Result<String> {
        let raw = String::from_utf8(self.catalog.profile(id)?.document().bytes().to_vec())
            .map_err(|_| "invalid_config")?;
        let page = if let Some(live) = &self.live {
            live.view()
                .sessions
                .iter()
                .find(|s| s.id == id)
                .and_then(|s| s.page)
                .ok_or("device_not_ready")?
        } else {
            self.controls.manager().active_page(id)?
        };
        self.save(id, token, &raw, page, name, brightness)
    }
    #[allow(clippy::too_many_arguments)]
    fn save_connected(
        &mut self,
        id: DeviceId,
        token: Token,
        bytes: &str,
        page: u8,
        name: &str,
        brightness: u8,
    ) -> Result<String> {
        let context = self.check(id, &token)?;
        let link = self
            .live
            .as_ref()
            .ok_or("live_workspace_unavailable")?
            .clone();
        let view = link.view();
        let base = self.store.load()?;
        if !base.same_associations(&view.catalog)
            || base.summaries().iter().any(|s| {
                base.profile(s.id).map(|p| p.revision)
                    != view.catalog.profile(s.id).map(|p| p.revision)
            })
        {
            return Err("live_profile_reload_required");
        }
        let projected =
            base.project_profile(id, token.revision, bytes.as_bytes(), page, name, brightness)?;
        let plan = crate::saved_native::Plan::from_catalog(&projected)?;
        let old_plan = crate::saved_native::Plan::from_catalog(&view.catalog)?;
        let mut bindings: Vec<_> = view
            .bindings
            .iter()
            .filter(|s| plan.media.contains(&s.selector))
            .cloned()
            .collect();
        let protected: std::collections::BTreeSet<_> = old_plan
            .pages
            .iter()
            .filter(|(partner, _)| **partner != id)
            .flat_map(|(_, pages)| pages.iter().flat_map(|p| p.media.iter().cloned()))
            .collect();
        let new: Vec<_> = plan
            .media
            .iter()
            .filter(|s| !bindings.iter().any(|b| &b.selector == *s) && !protected.contains(*s))
            .cloned()
            .collect();
        if self.native && self.mode != Mode::PermanentService && !new.is_empty() {
            bindings.extend(
                crate::shared_native_media::NativeBackend::connect()?.resolve_media(&new, None)?,
            );
        }
        let prepared = crate::device_execution::PreparedEdit::new(&projected, id, bindings)?;
        if self.cancelled.load(Ordering::Acquire) || self.stopping.load(Ordering::Acquire) {
            return Err("profile_request_cancelled");
        }
        let ticket = link.reserve(context, self.cancelled.clone())?;
        // Isolated acceptance can deliberately slow disk preparation while the
        // partner's actor/effects continue. Never read this switch in native mode.
        if !self.native
            && std::env::var("DECKSMITH_ISOLATED_TEST").as_deref() == Ok("1")
            && std::env::var("DECKSMITH_LIVE_PROFILE_TEST_DELAY").as_deref() == Ok("1")
        {
            std::thread::sleep(Duration::from_millis(250));
        }
        if self.cancelled.load(Ordering::Acquire) || self.stopping.load(Ordering::Acquire) {
            let _ = link.cancel(ticket);
            return Err("profile_request_cancelled");
        }
        self.catalog = match self.store.publish_projected(projected) {
            Ok(catalog) => catalog,
            Err(code) => {
                if code != "profile_commit_uncertain" {
                    let _ = link.cancel(ticket);
                }
                return Err(code);
            }
        };
        // Once published, finish the known user-requested application even if
        // its reply deadline expires. Caller receives uncertainty, never retries.
        link.apply(ticket, self.catalog.clone(), prepared)
            .map_err(|_| "profile_saved_apply_failed")?;
        self.preview = None;
        self.profile(id)
    }
    fn select(&mut self, id: DeviceId, expected: u64) -> Result<String> {
        if self.mode == Mode::PermanentService {
            self.store.ensure_daily_capacity()?;
        }
        self.store.select(expected, id)?;
        self.catalog = self.store.load()?;
        if let Some(live) = &self.live {
            live.select(self.catalog.clone(), self.cancelled.clone())
                .map_err(|_| "profile_selected_apply_uncertain")?;
        }
        self.devices()
    }
    fn save(
        &mut self,
        id: DeviceId,
        token: Token,
        bytes: &str,
        page: u8,
        name: &str,
        brightness: u8,
    ) -> Result<String> {
        self.check_token(id, &token)?;
        if self.mode == Mode::PermanentService {
            self.store.ensure_daily_capacity()?;
        }
        if matches!(self.mode, Mode::ConnectedWorkspace | Mode::PermanentService) {
            return self.save_connected(id, token, bytes, page, name, brightness);
        }
        if self.mode == Mode::SavedWorkspace
            && self
                .controls
                .manager()
                .statuses()
                .iter()
                .any(|s| s.connected)
        {
            return Err("workspace_device_is_live");
        }
        if self.mode == Mode::VirtualLab {
            self.check(id, &token)?;
        }
        let binding = self.token(id)?.binding;
        let model = self.catalog.profile(id)?.document().model();
        let renderer = Renderer::new(bytes.as_bytes(), model)?;
        self.store
            .save_profile(id, token.revision, bytes.as_bytes(), page, name, brightness)?;
        self.catalog = self.store.load()?;
        // Publication can succeed even if subsequent application/rendering fails.
        // Return an explicit uncertain/apply error; callers retain drafts and must reload.
        self.controls
            .apply_saved_profile(&self.catalog, id, binding)
            .map_err(|_| "profile_saved_apply_failed")?;
        self.renderers.insert(id, renderer);
        if self.mode == Mode::VirtualLab {
            self.controls.poll_round();
            self.render(id).map_err(|_| "profile_saved_render_failed")?;
        }
        self.profile(id)
    }
    fn page(&mut self, id: DeviceId, token: Token, page: u8) -> Result<String> {
        if let Some(live) = &self.live {
            let context = self.check(id, &token)?;
            live.page(context, page, self.cancelled.clone())?;
        } else if self.mode == Mode::SavedWorkspace {
            self.check_token(id, &token)?;
            self.controls.browse_saved_page(id, token.binding, page)?;
        } else {
            let context = self.check(id, &token)?;
            self.controls.navigate(context, page)?;
            self.controls.poll_round();
            self.render(id)?;
        }
        self.profile(id)
    }
    fn set_discovery(&mut self, discovery: Discovery, available: bool) -> Result<()> {
        if self.discovery_available != available
            || self.discovery.candidates() != discovery.candidates()
        {
            self.discovery_revision = self
                .discovery_revision
                .checked_add(1)
                .ok_or("discovery_revision_exhausted")?;
        }
        self.discovery = discovery;
        self.discovery_available = available;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn enroll(
        &mut self,
        runtime: DeviceId,
        revision: u64,
        generation: u64,
        slot: u8,
        name: &str,
        brightness: u8,
        import: bool,
    ) -> Result<String> {
        if self.mode != Mode::SavedWorkspace
            || !self.discovery_available
            || runtime != self.runtime
            || revision != self.discovery_revision
        {
            return Err("stale_device_discovery");
        }
        let candidate = self
            .discovery
            .candidates()
            .iter()
            .find(|c| c.slot() == slot)
            .ok_or("unknown_candidate")?;
        let legacy = if import {
            if candidate.model() != DeviceModel::Plus {
                return Err("mk2_requires_own_defaults");
            }
            Some(
                self.import_plus
                    .as_deref()
                    .ok_or("legacy_import_not_prepared")?,
            )
        } else {
            None
        };
        let id = random_device_id()?;
        self.store.enroll(
            generation,
            &self.discovery,
            slot,
            Enrollment::new(id, name, brightness, legacy),
        )?;
        self.catalog = self.store.load()?;
        self.controls
            .enroll_saved_profile(&self.catalog, id)
            .map_err(|_| "profile_enrolled_reload_required")?;
        self.renderers.insert(
            id,
            Renderer::new(
                self.catalog.profile(id)?.document().bytes(),
                candidate.model(),
            )?,
        );
        self.profile(id)
    }
    fn preview(&mut self, id: DeviceId, bytes: &str, page: u8, touch: bool) -> Result<Vec<u8>> {
        let model = self.catalog.profile(id)?.document().model();
        if bytes.len() > LIMIT {
            return Err("layout_too_large");
        }
        if self
            .preview
            .as_ref()
            .is_none_or(|(owner, raw, _)| *owner != id || raw != bytes)
        {
            let renderer = Renderer::new(bytes.as_bytes(), model)?;
            self.preview = Some((id, bytes.into(), renderer));
        }
        let brightness = self
            .catalog
            .summaries()
            .iter()
            .find(|s| s.id == id)
            .ok_or("unknown_device")?
            .brightness;
        let renderer = &mut self.preview.as_mut().ok_or("preview_missing")?.2;
        let state = TouchState {
            brightness: Some(brightness),
            ..Default::default()
        };
        // Avoid rendering the other surface on each preview request.
        if touch {
            if model != DeviceModel::Plus {
                return Err("device_has_no_touch");
            }
            renderer.pages.preview_touch(page, state)
        } else {
            renderer.keys(page, state)
        }
    }
}
enum Operation {
    Devices,
    Enroll(DeviceId, u64, u64, u8, String, u8, bool),
    Archive(u64),
    Metadata(DeviceId, Token, String, u8),
    Profile(DeviceId),
    Select(DeviceId, u64),
    Save(DeviceId, Token, String, u8, String, u8),
    Page(DeviceId, Token, u8),
    Preview(DeviceId, String, u8, bool),
    Previous(DeviceId),
    Test(DeviceId, Token, String, u8, u8),
}
enum Reply {
    Text(String),
    Bytes(Vec<u8>),
}
struct Request {
    operation: Operation,
    cancelled: Arc<AtomicBool>,
    answer: async_channel::Sender<Result<Reply>>,
}
struct Api {
    queue: SyncSender<Request>,
}
impl Api {
    async fn send(&self, operation: Operation) -> zbus::fdo::Result<Reply> {
        self.send_until(operation, Duration::from_secs(5)).await
    }
    async fn send_until(
        &self,
        operation: Operation,
        deadline: Duration,
    ) -> zbus::fdo::Result<Reply> {
        let (answer, receive) = async_channel::bounded(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        self.queue
            .try_send(Request {
                operation,
                cancelled: cancelled.clone(),
                answer,
            })
            .map_err(|_| zbus::fdo::Error::LimitsExceeded("Profile queue full".into()))?;
        // Waiting must yield to the zbus executor, which also transmits replies.
        // Blocking recv_timeout here can starve concurrent D-Bus calls.
        let reply =
            futures_lite::future::or(async { receive.recv().await.map_err(|_| ()) }, async {
                async_io::Timer::after(deadline).await;
                Err(())
            })
            .await;
        match reply {
            Ok(value) => value.map_err(|e| zbus::fdo::Error::Failed(e.into())),
            Err(_) => {
                cancelled.store(true, Ordering::Release);
                Err(zbus::fdo::Error::Failed(
                    "profile_request_uncertain_reload_before_retry".into(),
                ))
            }
        }
    }
    async fn text(&self, op: Operation) -> zbus::fdo::Result<String> {
        match self.send(op).await? {
            Reply::Text(s) => Ok(s),
            _ => Err(zbus::fdo::Error::Failed("invalid_reply".into())),
        }
    }
}
fn id(raw: &str) -> zbus::fdo::Result<DeviceId> {
    DeviceId::parse(raw).map_err(|e| zbus::fdo::Error::InvalidArgs(e.into()))
}
fn token(raw: &str) -> zbus::fdo::Result<Token> {
    if raw.len() > 512 {
        return Err(zbus::fdo::Error::InvalidArgs("invalid_token".into()));
    }
    serde_json::from_str(raw).map_err(|_| zbus::fdo::Error::InvalidArgs("invalid_token".into()))
}
#[zbus::interface(name = "cc.senecal.Decksmith.ProfileLab2")]
impl Api {
    async fn list_devices(&self) -> zbus::fdo::Result<String> {
        self.text(Operation::Devices).await
    }
    async fn get_profile(&self, device: &str) -> zbus::fdo::Result<String> {
        self.text(Operation::Profile(id(device)?)).await
    }
    async fn get_previous_profile(&self, device: &str) -> zbus::fdo::Result<String> {
        self.text(Operation::Previous(id(device)?)).await
    }
    async fn test_control(
        &self,
        device: &str,
        revision: &str,
        raw: &str,
        page: u8,
        slot: u8,
    ) -> zbus::fdo::Result<String> {
        if raw.len() > LIMIT {
            return Err(zbus::fdo::Error::InvalidArgs("layout_too_large".into()));
        }
        self.text(Operation::Test(
            id(device)?,
            token(revision)?,
            raw.into(),
            page,
            slot,
        ))
        .await
    }
    async fn archive_history(&self, generation: u64) -> zbus::fdo::Result<String> {
        self.text(Operation::Archive(generation)).await
    }
    async fn save_metadata(
        &self,
        device: &str,
        revision: &str,
        name: &str,
        brightness: u8,
    ) -> zbus::fdo::Result<String> {
        if name.len() > 256 {
            return Err(zbus::fdo::Error::InvalidArgs(
                "profile_request_too_large".into(),
            ));
        }
        self.text(Operation::Metadata(
            id(device)?,
            token(revision)?,
            name.into(),
            brightness,
        ))
        .await
    }
    async fn select_device(&self, device: &str, generation: u64) -> zbus::fdo::Result<String> {
        self.text(Operation::Select(id(device)?, generation)).await
    }
    async fn save_profile(
        &self,
        device: &str,
        revision: &str,
        layout: &str,
        page: u8,
        name: &str,
        brightness: u8,
    ) -> zbus::fdo::Result<String> {
        if layout.len() > LIMIT || name.len() > 256 {
            return Err(zbus::fdo::Error::InvalidArgs(
                "profile_request_too_large".into(),
            ));
        }
        self.text(Operation::Save(
            id(device)?,
            token(revision)?,
            layout.into(),
            page,
            name.into(),
            brightness,
        ))
        .await
    }
    async fn show_page(&self, device: &str, revision: &str, page: u8) -> zbus::fdo::Result<String> {
        self.text(Operation::Page(id(device)?, token(revision)?, page))
            .await
    }
    async fn preview(
        &self,
        device: &str,
        layout: &str,
        page: u8,
        touch: bool,
    ) -> zbus::fdo::Result<Vec<u8>> {
        if layout.len() > LIMIT {
            return Err(zbus::fdo::Error::InvalidArgs("layout_too_large".into()));
        }
        match self
            .send(Operation::Preview(id(device)?, layout.into(), page, touch))
            .await?
        {
            Reply::Bytes(b) => Ok(b),
            _ => Err(zbus::fdo::Error::Failed("invalid_reply".into())),
        }
    }
}
struct WorkspaceApi(Api);
#[zbus::interface(name = "cc.senecal.Decksmith.DeviceWorkspace2")]
impl WorkspaceApi {
    async fn list_devices(&self) -> zbus::fdo::Result<String> {
        self.0.list_devices().await
    }
    async fn get_profile(&self, device: &str) -> zbus::fdo::Result<String> {
        self.0.get_profile(device).await
    }
    async fn get_previous_profile(&self, device: &str) -> zbus::fdo::Result<String> {
        self.0.get_previous_profile(device).await
    }
    async fn test_control(
        &self,
        device: &str,
        revision: &str,
        raw: &str,
        page: u8,
        slot: u8,
    ) -> zbus::fdo::Result<String> {
        self.0.test_control(device, revision, raw, page, slot).await
    }
    async fn select_device(&self, device: &str, generation: u64) -> zbus::fdo::Result<String> {
        self.0.select_device(device, generation).await
    }
    async fn save_metadata(
        &self,
        device: &str,
        revision: &str,
        name: &str,
        brightness: u8,
    ) -> zbus::fdo::Result<String> {
        self.0
            .save_metadata(device, revision, name, brightness)
            .await
    }
    async fn save_profile(
        &self,
        device: &str,
        revision: &str,
        layout: &str,
        page: u8,
        name: &str,
        brightness: u8,
    ) -> zbus::fdo::Result<String> {
        self.0
            .save_profile(device, revision, layout, page, name, brightness)
            .await
    }
    async fn show_page(&self, device: &str, revision: &str, page: u8) -> zbus::fdo::Result<String> {
        self.0.show_page(device, revision, page).await
    }
    async fn preview(
        &self,
        device: &str,
        layout: &str,
        page: u8,
        touch: bool,
    ) -> zbus::fdo::Result<Vec<u8>> {
        self.0.preview(device, layout, page, touch).await
    }
    async fn archive_history(&self, generation: u64) -> zbus::fdo::Result<String> {
        self.0.archive_history(generation).await
    }
    #[allow(clippy::too_many_arguments)]
    async fn enroll_device(
        &self,
        runtime: &str,
        discovery: u64,
        generation: u64,
        slot: u8,
        name: &str,
        brightness: u8,
        import_plus: bool,
    ) -> zbus::fdo::Result<String> {
        if name.len() > 256 {
            return Err(zbus::fdo::Error::InvalidArgs(
                "profile_request_too_large".into(),
            ));
        }
        self.0
            .text(Operation::Enroll(
                id(runtime)?,
                discovery,
                generation,
                slot,
                name.into(),
                brightness,
                import_plus,
            ))
            .await
    }
}
fn execute(host: &mut Host, op: Operation) -> Result<Reply> {
    match op {
        Operation::Devices => host.devices().map(Reply::Text),
        Operation::Previous(id) => String::from_utf8(host.store.previous_profile_layout(id)?)
            .map(Reply::Text)
            .map_err(|_| "invalid_profile_history"),
        Operation::Test(id, t, raw, p, s) => host.test_control(id, t, &raw, p, s).map(Reply::Text),
        Operation::Enroll(r, d, g, s, n, b, i) => {
            host.enroll(r, d, g, s, &n, b, i).map(Reply::Text)
        }
        Operation::Archive(g) => host.archive(g).map(Reply::Text),
        Operation::Metadata(id, t, n, b) => host.metadata(id, t, &n, b).map(Reply::Text),
        Operation::Profile(id) => host.profile(id).map(Reply::Text),
        Operation::Select(id, g) => host.select(id, g).map(Reply::Text),
        Operation::Save(id, t, raw, p, n, b) => host.save(id, t, &raw, p, &n, b).map(Reply::Text),
        Operation::Page(id, t, p) => host.page(id, t, p).map(Reply::Text),
        Operation::Preview(id, raw, p, t) => host.preview(id, &raw, p, t).map(Reply::Bytes),
    }
}
pub(crate) fn start_connected(
    store: Store,
    link: crate::live_profiles::Link,
    native: bool,
    stop: Arc<AtomicBool>,
    permanent: bool,
) -> Result<(zbus::Connection, std::thread::JoinHandle<()>)> {
    let (queue, incoming) = mpsc::sync_channel::<Request>(8);
    // No name replacement or service auto-activation; this cannot adopt V1 or
    // the saved-only workspace. Claim the endpoint before loading a live store.
    let connection = futures_lite::future::block_on(
        zbus::connection::Builder::session()
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name("cc.senecal.Decksmith.ConnectedWorkspace")
            .serve_at(
                "/cc/senecal/Decksmith/ConnectedWorkspace",
                WorkspaceApi(Api { queue }),
            )
            .build(),
    )
    .map_err(|_| "bus_registration_failed")?;
    let (ready_send, ready_receive) = mpsc::sync_channel(1);
    let ending = stop.clone();
    let thread = std::thread::Builder::new()
        .name("decksmith-live-profile-disk".into())
        .spawn(move || {
            let mut host = match Host::new(store) {
                Ok(host) => host,
                Err(code) => {
                    let _ = ready_send.send(Err(code));
                    return;
                }
            };
            host.mode = if permanent {
                Mode::PermanentService
            } else {
                Mode::ConnectedWorkspace
            };
            host.runtime = link.view().runtime;
            host.live = Some(link);
            host.native = native;
            host.stopping = stop.clone();
            host.renderers.clear();
            let _ = ready_send.send(Ok(()));
            while !stop.load(Ordering::Acquire) {
                match incoming.recv_timeout(Duration::from_millis(50)) {
                    Ok(request) if !request.cancelled.load(Ordering::Acquire) => {
                        host.cancelled = request.cancelled;
                        let reply = execute(&mut host, request.operation);
                        let _ = request.answer.try_send(reply);
                    }
                    Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(_) => break,
                }
            }
        })
        .map_err(|_| "live_profile_worker_spawn_failed")?;
    match ready_receive.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {}
        Ok(Err(code)) => {
            ending.store(true, Ordering::Release);
            let _ = thread.join();
            return Err(code);
        }
        Err(_) => {
            ending.store(true, Ordering::Release);
            return Err("live_profile_startup_timeout");
        }
    }
    Ok((connection, thread))
}
pub fn lab(args: &[String]) -> Result<()> {
    let [_, destination, flag, seconds] = args else {
        return Err("profile_lab_arguments");
    };
    let seconds = seconds
        .parse::<u64>()
        .map_err(|_| "profile_lab_arguments")?;
    if flag != "--seconds"
        || !(1..=3600).contains(&seconds)
        || std::env::var("DECKSMITH_ISOLATED_TEST").as_deref() != Ok("1")
    {
        return Err("profile_lab_isolated_only");
    }
    let path = Path::new(destination);
    if !path.is_absolute() {
        return Err("profile_lab_absolute_path_required");
    }
    // Exclusive fresh destination: never adopts/migrates an existing user store.
    let store = Store::create(path)?;
    let discovery = Discovery::new(vec![
        Candidate::new(0, DeviceModel::Plus, Some("virtual-lab-plus")),
        Candidate::new(1, DeviceModel::Mk2, Some("virtual-lab-mk2")),
    ])?;
    let plus = random_device_id()?;
    let mk2 = random_device_id()?;
    let generation = store.enroll(
        0,
        &discovery,
        0,
        Enrollment::new(
            plus,
            "Stream Deck + · Virtual",
            50,
            Some(include_bytes!("../../../config/audio.json")),
        ),
    )?;
    store.enroll(
        generation,
        &discovery,
        1,
        Enrollment::new(mk2, "Stream Deck MK.2 · Virtual", 50, None),
    )?;
    let mut host = Host::new(store)?;
    let revision = host.controls.discover(discovery)?;
    host.controls
        .attach(revision, 0, Box::new(VirtualDeck::new(DeviceModel::Plus)))?;
    host.controls
        .attach(revision, 1, Box::new(VirtualDeck::new(DeviceModel::Mk2)))?;
    host.controls.poll_round();
    host.render(plus)?;
    host.render(mk2)?;
    let (queue, incoming) = mpsc::sync_channel::<Request>(8);
    let connection = futures_lite::future::block_on(
        zbus::connection::Builder::session()
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name("cc.senecal.Decksmith.ProfileLab")
            .serve_at("/cc/senecal/Decksmith/ProfileLab", Api { queue })
            .build(),
    )
    .map_err(|_| "bus_registration_failed")?;
    let stopping = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stopping.clone())
            .map_err(|_| "profile_lab_signal_failed")?;
    }
    let end = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < end && !stopping.load(Ordering::Acquire) {
        match incoming.recv_timeout(Duration::from_millis(250)) {
            Ok(r) => {
                if !r.cancelled.load(Ordering::Acquire) {
                    let reply = execute(&mut host, r.operation);
                    let _ = r.answer.try_send(reply);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }
    }
    host.controls.stop();
    drop(connection);
    Ok(())
}

/// Opt-in persistent saved-profile service. Read-only HID discovery; no HID
/// owner, handle, reports, Control1 calls or desktop/audio actions are allowed.
#[cfg(feature = "hardware")]
pub fn workspace(args: &[String]) -> Result<()> {
    let (destination, import_path) = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["--device-workspace", destination, "--run"] => (destination.to_string(), None),
        [
            "--device-workspace",
            destination,
            "--import-plus",
            import,
            "--run",
        ] => (destination.to_string(), Some(import.to_string())),
        _ => return Err("device_workspace_arguments"),
    };
    let path = Path::new(&destination);
    if !path.is_absolute() {
        return Err("device_workspace_absolute_path_required");
    }
    // Reserve the endpoint before touching any destination. No second service
    // can silently adopt another store behind an editor's outstanding tokens.
    let (queue, incoming) = mpsc::sync_channel::<Request>(8);
    let connection = futures_lite::future::block_on(
        zbus::connection::Builder::session()
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name("cc.senecal.Decksmith.DeviceWorkspace")
            .serve_at(
                "/cc/senecal/Decksmith/DeviceWorkspace",
                WorkspaceApi(Api { queue }),
            )
            .build(),
    )
    .map_err(|_| "bus_registration_failed")?;
    let import = import_path
        .map(|p| {
            let path = Path::new(&p);
            if !path.is_absolute() {
                return Err("legacy_import_absolute_path_required");
            }
            let metadata =
                std::fs::symlink_metadata(path).map_err(|_| "legacy_import_unavailable")?;
            if !metadata.is_file() || metadata.len() > LIMIT as u64 {
                return Err("legacy_import_invalid_file");
            }
            // Bounded even if the caller-selected file grows between stat and read.
            use std::io::Read;
            use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
            let mut bytes = Vec::new();
            let file = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(path)
                .map_err(|_| "legacy_import_unavailable")?;
            let opened = file.metadata().map_err(|_| "legacy_import_unavailable")?;
            if !opened.is_file() || opened.dev() != metadata.dev() || opened.ino() != metadata.ino()
            {
                return Err("legacy_import_invalid_file");
            }
            file.take((LIMIT + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| "legacy_import_unavailable")?;
            if bytes.len() > LIMIT {
                return Err("layout_too_large");
            }
            let document = crate::layout_document::Document::for_model(&bytes, DeviceModel::Plus)?;
            if document.version() != 1 {
                return Err("profile_import_requires_legacy_plus");
            }
            Ok(bytes)
        })
        .transpose()?;
    let store = match std::fs::symlink_metadata(path) {
        Ok(_) => Store::open(path)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Store::create(path)?,
        Err(_) => return Err("profile_store_io"),
    };
    let mut host = Host::new(store)?;
    host.mode = Mode::SavedWorkspace;
    host.import_plus = import;
    let stopping = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stopping.clone())
            .map_err(|_| "device_workspace_signal_failed")?;
    }
    let mut refresh_at = Instant::now();
    while !stopping.load(Ordering::Acquire) {
        if Instant::now() >= refresh_at {
            refresh_discovery(&mut host)?;
            refresh_at = Instant::now() + Duration::from_secs(5);
        }
        match incoming.recv_timeout(Duration::from_millis(250)) {
            Ok(request) if !request.cancelled.load(Ordering::Acquire) => {
                // Refresh against a new hardware enumeration at the enrollment
                // boundary. Snapshot/order changes reject the old request.
                if matches!(request.operation, Operation::Enroll(..)) {
                    refresh_discovery(&mut host)?;
                }
                let _ = request
                    .answer
                    .try_send(execute(&mut host, request.operation));
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }
    }
    drop(connection);
    Ok(())
}
#[cfg(feature = "hardware")]
fn refresh_discovery(host: &mut Host) -> Result<()> {
    match decksmith_device::hardware::Inventory::discover() {
        Ok(inventory) => host.set_discovery(inventory.snapshot(), true),
        Err(_) => host.set_discovery(Discovery::new(Vec::new())?, false),
    }
}

/// Fresh, isolated native acceptance endpoint. Uses fake identities only and
/// never enumerates or opens hardware. Production workspace cannot enter this.
#[cfg(feature = "hardware")]
pub fn workspace_lab(args: &[String]) -> Result<()> {
    let [_, destination, flag, seconds] = args else {
        return Err("workspace_lab_arguments");
    };
    let seconds = seconds
        .parse::<u64>()
        .map_err(|_| "workspace_lab_arguments")?;
    if flag != "--seconds"
        || !(1..=3600).contains(&seconds)
        || std::env::var("DECKSMITH_ISOLATED_TEST").as_deref() != Ok("1")
    {
        return Err("workspace_lab_isolated_only");
    }
    let path = Path::new(destination);
    if !path.is_absolute() {
        return Err("workspace_lab_absolute_path_required");
    }
    let mut host = Host::new(Store::create(path)?)?;
    host.mode = Mode::SavedWorkspace;
    host.import_plus = Some(include_bytes!("../../../config/audio.json").to_vec());
    host.set_discovery(
        Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("workspace-fixture-plus")),
            Candidate::new(1, DeviceModel::Mk2, Some("workspace-fixture-mk2")),
        ])?,
        true,
    )?;
    let (queue, incoming) = mpsc::sync_channel::<Request>(8);
    let connection = futures_lite::future::block_on(
        zbus::connection::Builder::session()
            .allow_name_replacements(false)
            .replace_existing_names(false)
            .name("cc.senecal.Decksmith.DeviceWorkspace")
            .serve_at(
                "/cc/senecal/Decksmith/DeviceWorkspace",
                WorkspaceApi(Api { queue }),
            )
            .build(),
    )
    .map_err(|_| "bus_registration_failed")?;
    let stopping = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, stopping.clone())
            .map_err(|_| "workspace_lab_signal_failed")?;
    }
    let end = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < end && !stopping.load(Ordering::Acquire) {
        match incoming.recv_timeout(Duration::from_millis(250)) {
            Ok(r) if !r.cancelled.load(Ordering::Acquire) => {
                let _ = r.answer.try_send(execute(&mut host, r.operation));
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }
    }
    drop(connection);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use decksmith_device::{DeckDevice, DeviceError};
    use std::{cell::RefCell, rc::Rc};
    struct Probe {
        deck: Rc<RefCell<VirtualDeck>>,
        fail: Rc<RefCell<bool>>,
    }
    impl DeckDevice for Probe {
        fn descriptor(&self) -> DeviceDescriptor {
            self.deck.borrow().descriptor()
        }
        fn set_key_image(&mut self, i: u8, b: &[u8]) -> std::result::Result<(), DeviceError> {
            if *self.fail.borrow() {
                return Err(DeviceError::Transport);
            }
            self.deck.borrow_mut().set_key_image(i, b)
        }
        fn set_touch_image(&mut self, b: &[u8]) -> std::result::Result<(), DeviceError> {
            self.deck.borrow_mut().set_touch_image(b)
        }
        fn set_brightness(&mut self, p: u8) -> std::result::Result<(), DeviceError> {
            self.deck.borrow_mut().set_brightness(p)
        }
        fn poll_event(
            &mut self,
        ) -> std::result::Result<Option<decksmith_core::InputEvent>, DeviceError> {
            self.deck.borrow_mut().poll_event()
        }
    }
    struct Pair {
        root: std::path::PathBuf,
        host: Host,
        plus: DeviceId,
        mk2: DeviceId,
        a: Rc<RefCell<VirtualDeck>>,
        b: Rc<RefCell<VirtualDeck>>,
        fail: Rc<RefCell<bool>>,
    }
    impl Drop for Pair {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    fn pair() -> Pair {
        let root = std::env::temp_dir().join(format!(
            "decksmith-profile-control-{}",
            random_device_id().unwrap()
        ));
        let store = Store::create(&root).unwrap();
        let plus = random_device_id().unwrap();
        let mk2 = random_device_id().unwrap();
        let d = Discovery::new(vec![
            Candidate::new(0, DeviceModel::Plus, Some("fixture-plus")),
            Candidate::new(1, DeviceModel::Mk2, Some("fixture-mk2")),
        ])
        .unwrap();
        let g = store
            .enroll(
                0,
                &d,
                0,
                Enrollment::new(
                    plus,
                    "Plus",
                    50,
                    Some(include_bytes!("../../../config/audio.json")),
                ),
            )
            .unwrap();
        store
            .enroll(g, &d, 1, Enrollment::new(mk2, "MK.2", 40, None))
            .unwrap();
        let mut host = Host::new(store).unwrap();
        let rev = host.controls.discover(d).unwrap();
        let a = Rc::new(RefCell::new(VirtualDeck::new(DeviceModel::Plus)));
        let b = Rc::new(RefCell::new(VirtualDeck::new(DeviceModel::Mk2)));
        let fail = Rc::new(RefCell::new(false));
        host.controls
            .attach(
                rev,
                0,
                Box::new(Probe {
                    deck: a.clone(),
                    fail: fail.clone(),
                }),
            )
            .unwrap();
        host.controls
            .attach(
                rev,
                1,
                Box::new(Probe {
                    deck: b.clone(),
                    fail: Rc::new(RefCell::new(false)),
                }),
            )
            .unwrap();
        host.controls.poll_round();
        host.render(plus).unwrap();
        host.render(mk2).unwrap();
        Pair {
            root,
            host,
            plus,
            mk2,
            a,
            b,
            fail,
        }
    }
    fn snapshot(deck: &VirtualDeck) -> Vec<Vec<u8>> {
        (0..deck.descriptor().key_count())
            .map(|i| deck.key_image(i).unwrap().to_vec())
            .chain(std::iter::once(deck.touch_image().to_vec()))
            .collect()
    }
    #[test]
    fn profile_reply_wait_yields_and_queue_and_timeout_stay_bounded() {
        futures_lite::future::block_on(async {
            let (queue, incoming) = mpsc::sync_channel(8);
            let api = Api { queue };
            let mut waiting = std::pin::pin!(api.send(Operation::Devices));
            // A D-Bus handler must yield before its service replies; it cannot
            // block the same executor that has to transmit that reply.
            assert!(
                futures_lite::future::poll_once(&mut waiting)
                    .await
                    .is_none()
            );
            let request = incoming.try_recv().unwrap();
            request
                .answer
                .try_send(Ok(Reply::Text("ready".into())))
                .unwrap();
            assert!(matches!(waiting.await, Ok(Reply::Text(s)) if s == "ready"));
            assert!(
                api.send_until(Operation::Devices, Duration::ZERO)
                    .await
                    .is_err()
            );
            let expired = incoming.try_recv().unwrap();
            assert!(expired.cancelled.load(Ordering::Acquire));
            assert!(
                expired
                    .answer
                    .try_send(Ok(Reply::Text("late".into())))
                    .is_err()
            );
            for _ in 0..8 {
                let (answer, _) = async_channel::bounded(1);
                api.queue
                    .try_send(Request {
                        operation: Operation::Devices,
                        cancelled: Arc::new(AtomicBool::new(false)),
                        answer,
                    })
                    .unwrap();
            }
            assert!(matches!(
                api.send(Operation::Devices).await,
                Err(zbus::fdo::Error::LimitsExceeded(_))
            ));
        });
    }
    fn saved_workspace() -> (std::path::PathBuf, Host) {
        let root = std::env::temp_dir().join(format!(
            "decksmith-workspace-{}",
            random_device_id().unwrap()
        ));
        let mut host = Host::new(Store::create(&root).unwrap()).unwrap();
        host.mode = Mode::SavedWorkspace;
        host.set_discovery(
            Discovery::new(vec![
                Candidate::new(0, DeviceModel::Plus, Some("saved-plus")),
                Candidate::new(1, DeviceModel::Mk2, Some("saved-mk2")),
            ])
            .unwrap(),
            true,
        )
        .unwrap();
        (root, host)
    }
    fn add(host: &mut Host, slot: u8, import: bool) -> Result<DeviceId> {
        let reply = host.enroll(
            host.runtime,
            host.discovery_revision,
            host.catalog.generation,
            slot,
            "Desk 🎵",
            54,
            import,
        )?;
        let value: serde_json::Value = serde_json::from_str(&reply).unwrap();
        DeviceId::parse(value["id"].as_str().unwrap())
    }
    #[test]
    fn saved_workspace_enroll_edit_restart_preserves_ids_selection_and_other_owner() {
        let (root, mut h) = saved_workspace();
        let plus = add(&mut h, 0, false).unwrap();
        let mk2 = add(&mut h, 1, false).unwrap();
        assert_eq!(h.controls.manager().statuses().len(), 2);
        assert!(h.controls.manager().statuses().iter().all(|s| !s.connected));
        let plus_before = h.catalog.profile(plus).unwrap().document().bytes().to_vec();
        let token = h.token(mk2).unwrap();
        let mut raw: serde_json::Value =
            serde_json::from_slice(h.catalog.profile(mk2).unwrap().document().bytes()).unwrap();
        raw["pages"][0]["keys"][14]["label"] = "Fifteen 🎵".into();
        h.save(mk2, token.clone(), &raw.to_string(), 1, "MK.2 🎵", 61)
            .unwrap();
        assert_eq!(
            h.catalog.profile(plus).unwrap().document().bytes(),
            plus_before
        );
        assert_eq!(h.catalog.profile(plus).unwrap().revision, 1);
        assert!(
            h.save(mk2, token, &raw.to_string(), 0, "stale", 50)
                .is_err()
        );
        h.select(mk2, h.catalog.generation).unwrap();
        let before_restart = h.token(mk2).unwrap();
        drop(h);
        let mut reopened = Host::new(Store::open(&root).unwrap()).unwrap();
        reopened.mode = Mode::SavedWorkspace;
        assert_eq!(reopened.catalog.selected, Some(mk2));
        assert_eq!(reopened.catalog.profile(mk2).unwrap().saved_page, 1);
        assert_eq!(reopened.controls.manager().active_page(mk2).unwrap(), 1);
        assert!(reopened.page(mk2, before_restart, 0).is_err());
        reopened.page(mk2, reopened.token(mk2).unwrap(), 2).unwrap();
        assert_eq!(reopened.controls.manager().active_page(plus).unwrap(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn discovery_generation_runtime_change_failure_and_ambiguous_identity_reject_enrollment() {
        let (root, mut h) = saved_workspace();
        let r = h.discovery_revision;
        // Same snapshot leaves an editor's explicit choice usable.
        h.set_discovery(
            Discovery::new(h.discovery.candidates().to_vec()).unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(h.discovery_revision, r);
        let wrong = random_device_id().unwrap();
        assert!(h.enroll(wrong, r, 0, 0, "Wrong", 50, false).is_err());
        h.set_discovery(
            Discovery::new(vec![
                Candidate::new(0, DeviceModel::Mk2, Some("saved-mk2")),
                Candidate::new(1, DeviceModel::Plus, Some("saved-plus")),
            ])
            .unwrap(),
            true,
        )
        .unwrap();
        assert!(h.enroll(h.runtime, r, 0, 0, "Old slot", 50, false).is_err());
        h.set_discovery(Discovery::new(Vec::new()).unwrap(), false)
            .unwrap();
        assert!(add(&mut h, 0, false).is_err());
        h.set_discovery(
            Discovery::new(vec![
                Candidate::new(0, DeviceModel::Mk2, Some("duplicate")),
                Candidate::new(1, DeviceModel::Mk2, Some("duplicate")),
            ])
            .unwrap(),
            true,
        )
        .unwrap();
        assert!(add(&mut h, 0, false).is_err());
        h.set_discovery(
            Discovery::new(vec![Candidate::new(0, DeviceModel::Mk2, None)]).unwrap(),
            true,
        )
        .unwrap();
        assert!(add(&mut h, 0, false).is_err());
        assert_eq!(h.store.load().unwrap().generation, 0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn explicit_import_retains_exact_legacy_and_mk2_never_copies_plus() {
        let (root, mut h) = saved_workspace();
        assert!(add(&mut h, 0, true).is_err());
        let bytes = include_bytes!("../../../config/audio.json");
        h.import_plus = Some(bytes.to_vec());
        let plus = add(&mut h, 0, true).unwrap();
        assert!(add(&mut h, 1, true).is_err());
        let mk2 = add(&mut h, 1, false).unwrap();
        assert_eq!(
            h.catalog.profile(mk2).unwrap().document().bytes(),
            crate::layout_document::Document::mk2_defaults()
                .unwrap()
                .bytes()
        );
        // Original import retained byte-for-byte in its private generation.
        let path = root.join("generations");
        let files: Vec<_> = std::fs::read_dir(path)
            .unwrap()
            .flat_map(|d| std::fs::read_dir(d.unwrap().path()).unwrap())
            .collect();
        assert!(
            files
                .iter()
                .any(|f| std::fs::read(f.as_ref().unwrap().path()).is_ok_and(|b| b == bytes))
        );
        assert_eq!(h.catalog.profile(plus).unwrap().document().version(), 2);
        let status = h.devices().unwrap();
        for private in ["saved-plus", "saved-mk2"] {
            assert!(!status.contains(private));
        }
        assert!(status.contains("saved-workspace"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn workspace_cannot_save_through_accidentally_live_adapters() {
        let mut p = pair();
        p.host.mode = Mode::SavedWorkspace;
        let before = snapshot(&p.a.borrow());
        let token = p.host.token(p.plus).unwrap();
        assert_eq!(
            p.host.metadata(p.plus, token.clone(), "Wrong", 75),
            Err("workspace_device_is_live")
        );
        assert_eq!(
            p.host.page(p.plus, token, 1),
            Err("workspace_device_is_live")
        );
        assert_eq!(snapshot(&p.a.borrow()), before);
        assert_eq!(p.host.catalog.profile(p.plus).unwrap().revision, 1);
    }
    #[test]
    fn branded_starters_render_at_both_physical_sizes_with_home_only_website() {
        use decksmith_device::DeckDevice;
        for (model, raw, slot) in [
            (
                DeviceModel::Plus,
                include_bytes!("../../../config/plus-default.json").as_slice(),
                4usize,
            ),
            (
                DeviceModel::Mk2,
                include_bytes!("../../../config/mk2-default.json").as_slice(),
                10usize,
            ),
        ] {
            let document = crate::layout_document::Document::parse(raw)
                .unwrap()
                .upgraded()
                .unwrap();
            let mut renderer = Renderer::new(document.bytes(), model).unwrap();
            let frame = renderer.frame(0, 50).unwrap();
            let descriptor = DeviceDescriptor::for_model(model);
            let stride = descriptor.key_rgb_bytes();
            let (width, height) = descriptor.geometry().key_pixels;
            assert_eq!(
                frame.keys.len(),
                stride * usize::from(descriptor.key_count())
            );
            let mut deck = VirtualDeck::new(model);
            for (i, key) in frame.keys.chunks_exact(stride).enumerate() {
                deck.set_key_image(i as u8, key).unwrap();
                assert_eq!(deck.key_image(i as u8).unwrap(), key);
            }
            if let Some(touch) = frame.touch {
                deck.set_touch_image(&touch).unwrap();
            }
            let config = crate::pages::Config::parse(document.bytes()).unwrap();
            let controls = config.model_controls();
            assert_eq!(
                controls.resolve(
                    0,
                    &decksmith_core::RawEvent::Key {
                        index: slot as u8,
                        pressed: false
                    }
                ),
                Some(crate::pages::Action::OpenWebsite {
                    url: "https://github.com/infamous-pattern/decksmith".into()
                })
            );
            for page in 1..config.topology().0 {
                assert!(!matches!(
                    controls.resolve(
                        page,
                        &decksmith_core::RawEvent::Key {
                            index: slot as u8,
                            pressed: false
                        }
                    ),
                    Some(crate::pages::Action::OpenWebsite { .. })
                ));
            }
            if let Ok(path) = std::env::var("DECKSMITH_STARTER_ARTIFACTS") {
                let name = if model == DeviceModel::Plus {
                    "plus"
                } else {
                    "mk2"
                };
                for (i, key) in frame.keys.chunks_exact(stride).enumerate() {
                    image::RgbImage::from_raw(u32::from(width), u32::from(height), key.to_vec())
                        .unwrap()
                        .save(format!("{path}/{name}-key-{i}.png"))
                        .unwrap();
                }
            }
        }
    }

    #[test]
    fn model_renderer_matches_legacy_plus_and_all_fifteen_mk2_surfaces() {
        let mut p = pair();
        let raw = p
            .host
            .catalog
            .profile(p.plus)
            .unwrap()
            .document()
            .bytes()
            .to_vec();
        let expected = Pages::parse(include_bytes!("../../../config/audio.json"))
            .unwrap()
            .preview_keys(0, Default::default())
            .unwrap();
        let actual = p
            .host
            .preview(p.plus, std::str::from_utf8(&raw).unwrap(), 0, false)
            .unwrap();
        assert_eq!(expected, actual);
        assert_eq!(
            p.host
                .preview(p.plus, std::str::from_utf8(&raw).unwrap(), 0, true)
                .unwrap()
                .len(),
            800 * 100 * 3
        );
        for i in 0..15 {
            assert_eq!(p.b.borrow().key_image(i).unwrap().len(), 72 * 72 * 3);
        }
        assert!(p.b.borrow().touch_image().is_empty());
        let raw = std::str::from_utf8(p.host.catalog.profile(p.mk2).unwrap().document().bytes())
            .unwrap()
            .to_owned();
        assert_eq!(
            p.host.preview(p.mk2, &raw, 0, true),
            Err("device_has_no_touch")
        );
        assert!(p.host.preview(p.plus, &raw, 0, false).is_err());
    }
    #[test]
    fn device_addressed_save_and_navigation_leave_other_frame_context_unchanged() {
        let mut p = pair();
        let before = snapshot(&p.a.borrow());
        let other = p.host.context(p.plus).unwrap();
        let token = p.host.token(p.mk2).unwrap();
        let mut layout: serde_json::Value =
            serde_json::from_slice(p.host.catalog.profile(p.mk2).unwrap().document().bytes())
                .unwrap();
        layout["pages"][0]["keys"][14]["label"] = "🎵 Fifteen".into();
        p.host
            .save(p.mk2, token.clone(), &layout.to_string(), 0, "MK.2", 63)
            .unwrap();
        assert_eq!(snapshot(&p.a.borrow()), before);
        assert_eq!(p.host.context(p.plus).unwrap(), other);
        assert_eq!(p.b.borrow().brightness, 63);
        assert!(p.host.page(p.mk2, token, 1).is_err());
        p.host.page(p.mk2, p.host.token(p.mk2).unwrap(), 1).unwrap();
        assert_eq!(snapshot(&p.a.borrow()), before);
        assert_eq!(p.host.controls.manager().active_page(p.mk2).unwrap(), 1);
        let reopened = Store::open(&p.root).unwrap().load().unwrap();
        assert_eq!(reopened.profile(p.mk2).unwrap().revision, 2);
        assert_eq!(reopened.profile(p.plus).unwrap().revision, 1);
    }
    #[test]
    fn preview_invalid_page_and_wrong_model_do_not_paint_or_publish() {
        let mut p = pair();
        let before = snapshot(&p.b.borrow());
        let catalog_generation = p.host.catalog.generation;
        let raw = String::from_utf8(
            p.host
                .catalog
                .profile(p.plus)
                .unwrap()
                .document()
                .bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(p.host.preview(p.plus, &raw, 16, false).is_err());
        assert!(
            p.host
                .save(p.mk2, p.host.token(p.mk2).unwrap(), &raw, 0, "MK.2", 40)
                .is_err()
        );
        assert_eq!(p.host.store.load().unwrap().generation, catalog_generation);
        assert_eq!(snapshot(&p.b.borrow()), before);
        p.host.preview(p.plus, &raw, 0, false).unwrap();
        assert_eq!(snapshot(&p.b.borrow()), before);
    }
    #[test]
    fn stale_runtime_and_lock_reject_before_disk_and_malformed_frames_before_writes() {
        let mut p = pair();
        let catalog_generation = p.host.catalog.generation;
        let before = snapshot(&p.a.borrow());
        let mut token = p.host.token(p.plus).unwrap();
        token.runtime = random_device_id().unwrap();
        assert!(p.host.page(p.plus, token, 1).is_err());
        let context = p.host.context(p.plus).unwrap();
        assert_eq!(
            p.host.controls.paint(context, &[0; 3], None),
            Err("invalid_device_frame")
        );
        assert_eq!(snapshot(&p.a.borrow()), before);
        p.host.controls.gate(true, false).unwrap();
        let raw = String::from_utf8(
            p.host
                .catalog
                .profile(p.plus)
                .unwrap()
                .document()
                .bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(
            p.host
                .save(p.plus, p.host.token(p.plus).unwrap(), &raw, 0, "Plus", 50)
                .is_err()
        );
        assert_eq!(p.host.store.load().unwrap().generation, catalog_generation);
    }
    #[test]
    fn failed_owner_render_closes_only_that_session_and_stop_blanks_remaining() {
        let mut p = pair();
        let before = snapshot(&p.b.borrow());
        *p.fail.borrow_mut() = true;
        assert_eq!(p.host.render(p.plus), Err("device_frame_failed"));
        assert!(p.host.context(p.plus).is_err());
        assert!(p.host.context(p.mk2).is_ok());
        assert_eq!(snapshot(&p.b.borrow()), before);
        p.host.controls.stop();
        assert!(snapshot(&p.b.borrow()).iter().flatten().all(|v| *v == 0));
    }
    #[test]
    fn raw_profile_reply_retains_large_unknown_provider_integer() {
        let mut p = pair();
        let mut layout: serde_json::Value =
            serde_json::from_slice(p.host.catalog.profile(p.mk2).unwrap().document().bytes())
                .unwrap();
        layout["pages"][0]["keys"][14] = serde_json::json!({"label":"Keep","action":{"type":"none"},"plugin":{"provider":"missing.provider","action":"custom.future","schema":17,"settings":{"n":"precision-sentinel"}}});
        let raw = layout
            .to_string()
            .replace("\"precision-sentinel\"", "184467440737095516160001");
        let reply = p
            .host
            .save(p.mk2, p.host.token(p.mk2).unwrap(), &raw, 0, "MK.2", 40)
            .unwrap();
        assert!(reply.contains("184467440737095516160001"));
        let status = p.host.devices().unwrap();
        assert!(!status.contains("fixture-plus"));
        assert!(!status.contains("fixture-mk2"));
    }
    #[test]
    fn metadata_keeps_document_current_page_and_other_owner_untouched() {
        let mut p = pair();
        p.host
            .page(p.plus, p.host.token(p.plus).unwrap(), 1)
            .unwrap();
        let doc = p
            .host
            .catalog
            .profile(p.plus)
            .unwrap()
            .document()
            .bytes()
            .to_vec();
        let other = p.host.context(p.mk2).unwrap();
        let frame = snapshot(&p.b.borrow());
        p.host
            .metadata(p.plus, p.host.token(p.plus).unwrap(), "Desk 🎵", 61)
            .unwrap();
        assert_eq!(
            p.host.catalog.profile(p.plus).unwrap().document().bytes(),
            doc
        );
        assert_eq!(p.host.controls.manager().active_page(p.plus).unwrap(), 1);
        assert_eq!(p.host.catalog.profile(p.plus).unwrap().saved_page, 1);
        assert_eq!(p.a.borrow().brightness, 61);
        assert_eq!(p.host.context(p.mk2).unwrap(), other);
        assert_eq!(snapshot(&p.b.borrow()), frame);
    }
    #[test]
    fn archive_does_not_change_live_frames_revisions_or_device_contexts() {
        let mut p = pair();
        while p.host.catalog.generation < 16 {
            p.host
                .store
                .save_preferences_private(p.host.catalog.generation, b"{}")
                .unwrap();
            p.host.catalog = p.host.store.load().unwrap();
        }
        let one = p.host.context(p.plus).unwrap();
        let two = p.host.context(p.mk2).unwrap();
        let frame = snapshot(&p.a.borrow());
        let report = p.host.archive(16).unwrap();
        assert!(report.contains("8"));
        assert_eq!(p.host.context(p.plus).unwrap(), one);
        assert_eq!(p.host.context(p.mk2).unwrap(), two);
        assert_eq!(snapshot(&p.a.borrow()), frame);
        assert_eq!(p.host.token(p.plus).unwrap().revision, 1);
    }
}
