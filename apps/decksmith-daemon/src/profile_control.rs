//! Isolated device-addressed editor/render integration. No production migration,
//! HID opener or audio dispatch. The lab endpoint is never Control1.
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

struct Renderer {
    pages: Pages,
    descriptor: DeviceDescriptor,
}
struct Frame {
    keys: Vec<u8>,
    touch: Option<Vec<u8>>,
}
impl Renderer {
    fn new(bytes: &[u8], model: DeviceModel) -> Result<Self> {
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
    fn frame(&mut self, page: u8, brightness: u8) -> Result<Frame> {
        let state = TouchState {
            brightness: Some(brightness),
            ..Default::default()
        };
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
struct Host {
    store: Store,
    catalog: Catalog,
    controls: Controls,
    runtime: DeviceId,
    renderers: BTreeMap<DeviceId, Renderer>,
    preview: Option<(DeviceId, String, Renderer)>,
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
            store,
            catalog,
            controls,
            runtime: random_device_id()?,
            renderers,
            preview: None,
        })
    }
    fn context(&self, id: DeviceId) -> Result<DeviceContext> {
        self.controls.manager().context(id, ControlAddress::Key(0))
    }
    fn token(&self, id: DeviceId) -> Result<Token> {
        let profile = self.catalog.profile(id)?;
        let status = self
            .controls
            .manager()
            .statuses()
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
    fn check(&self, id: DeviceId, token: &Token) -> Result<DeviceContext> {
        let now = self.token(id)?;
        if now.runtime != token.runtime
            || now.revision != token.revision
            || now.connection != token.connection
            || now.binding != token.binding
        {
            return Err("stale_profile_request");
        }
        self.context(id)
    }
    fn devices(&self) -> Result<String> {
        Ok(serde_json::json!({"api_version":2,"mode":"virtual-lab","runtime":self.runtime,"generation":self.catalog.generation,"selected":self.catalog.selected,"devices":self.catalog.summaries(),"sessions":self.controls.manager().statuses()}).to_string())
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
        }
        serde_json::to_string(&ProfileReply {
            id,
            token: self.token(id)?,
            saved_page: p.saved_page,
            layout,
        })
        .map_err(|_| "profile_reply_failed")
    }
    fn render(&mut self, id: DeviceId) -> Result<()> {
        let context = self.context(id)?;
        let page = self.controls.manager().active_page(id)?;
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
    fn select(&mut self, id: DeviceId, expected: u64) -> Result<String> {
        self.store.select(expected, id)?;
        self.catalog = self.store.load()?;
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
        let context = self.check(id, &token)?;
        let model = self.catalog.profile(id)?.document().model();
        let renderer = Renderer::new(bytes.as_bytes(), model)?;
        self.store
            .save_profile(id, token.revision, bytes.as_bytes(), page, name, brightness)?;
        self.catalog = self.store.load()?;
        // Publication can succeed even if subsequent application/rendering fails.
        // Return an explicit uncertain/apply error; callers retain drafts and must reload.
        self.controls
            .apply_saved_profile(&self.catalog, id, context.binding)
            .map_err(|_| "profile_saved_apply_failed")?;
        self.renderers.insert(id, renderer);
        self.controls.poll_round();
        self.render(id).map_err(|_| "profile_saved_render_failed")?;
        self.profile(id)
    }
    fn page(&mut self, id: DeviceId, token: Token, page: u8) -> Result<String> {
        let context = self.check(id, &token)?;
        self.controls.navigate(context, page)?;
        self.controls.poll_round();
        self.render(id)?;
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
    Profile(DeviceId),
    Select(DeviceId, u64),
    Save(DeviceId, Token, String, u8, String, u8),
    Page(DeviceId, Token, u8),
    Preview(DeviceId, String, u8, bool),
}
enum Reply {
    Text(String),
    Bytes(Vec<u8>),
}
struct Request {
    operation: Operation,
    cancelled: Arc<AtomicBool>,
    answer: SyncSender<Result<Reply>>,
}
struct Api {
    queue: SyncSender<Request>,
}
impl Api {
    fn send(&self, operation: Operation) -> zbus::fdo::Result<Reply> {
        let (answer, receive) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        self.queue
            .try_send(Request {
                operation,
                cancelled: cancelled.clone(),
                answer,
            })
            .map_err(|_| zbus::fdo::Error::LimitsExceeded("Profile queue full".into()))?;
        match receive.recv_timeout(Duration::from_secs(5)) {
            Ok(value) => value.map_err(|e| zbus::fdo::Error::Failed(e.into())),
            Err(_) => {
                cancelled.store(true, Ordering::Release);
                Err(zbus::fdo::Error::Failed(
                    "profile_request_uncertain_reload_before_retry".into(),
                ))
            }
        }
    }
    fn text(&self, op: Operation) -> zbus::fdo::Result<String> {
        match self.send(op)? {
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
    fn list_devices(&self) -> zbus::fdo::Result<String> {
        self.text(Operation::Devices)
    }
    fn get_profile(&self, device: &str) -> zbus::fdo::Result<String> {
        self.text(Operation::Profile(id(device)?))
    }
    fn select_device(&self, device: &str, generation: u64) -> zbus::fdo::Result<String> {
        self.text(Operation::Select(id(device)?, generation))
    }
    fn save_profile(
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
    }
    fn show_page(&self, device: &str, revision: &str, page: u8) -> zbus::fdo::Result<String> {
        self.text(Operation::Page(id(device)?, token(revision)?, page))
    }
    fn preview(
        &self,
        device: &str,
        layout: &str,
        page: u8,
        touch: bool,
    ) -> zbus::fdo::Result<Vec<u8>> {
        if layout.len() > LIMIT {
            return Err(zbus::fdo::Error::InvalidArgs("layout_too_large".into()));
        }
        match self.send(Operation::Preview(id(device)?, layout.into(), page, touch))? {
            Reply::Bytes(b) => Ok(b),
            _ => Err(zbus::fdo::Error::Failed("invalid_reply".into())),
        }
    }
}
fn execute(host: &mut Host, op: Operation) -> Result<Reply> {
    match op {
        Operation::Devices => host.devices().map(Reply::Text),
        Operation::Profile(id) => host.profile(id).map(Reply::Text),
        Operation::Select(id, g) => host.select(id, g).map(Reply::Text),
        Operation::Save(id, t, raw, p, n, b) => host.save(id, t, &raw, p, &n, b).map(Reply::Text),
        Operation::Page(id, t, p) => host.page(id, t, p).map(Reply::Text),
        Operation::Preview(id, raw, p, t) => host.preview(id, &raw, p, t).map(Reply::Bytes),
    }
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
    let connection = zbus::blocking::connection::Builder::session()
        .map_err(|_| "bus_unavailable")?
        .name("cc.senecal.Decksmith.ProfileLab")
        .map_err(|_| "bus_name_failed")?
        .serve_at("/cc/senecal/Decksmith/ProfileLab", Api { queue })
        .map_err(|_| "bus_interface_failed")?
        .build()
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
}
