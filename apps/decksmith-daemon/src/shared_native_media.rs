//! Guarded MPRIS session adapter for the opt-in connected native path.
//! A selection is pinned before input: no discovery or player switching on keys.
use crate::{
    pages::Action,
    shared_worker::{Backend, Fence, NamedAudioBackend},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
type Result<T> = std::result::Result<T, &'static str>;
const PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
fn dbus_instance_suffix(suffix: &str) -> bool {
    let Some((bus, connection)) = suffix.strip_prefix('_').and_then(|s| s.split_once('_')) else {
        return false;
    };
    !bus.is_empty()
        && !connection.is_empty()
        && bus.bytes().all(|b| b.is_ascii_digit())
        && connection.bytes().all(|b| b.is_ascii_digit())
}
fn family(name: &str) -> &str {
    name.rsplit_once(".instance")
        .filter(|(_, suffix)| {
            !suffix.is_empty()
                && (suffix.bytes().all(|b| b.is_ascii_digit()) || dbus_instance_suffix(suffix))
        })
        .map_or(name, |(prefix, _)| prefix)
}
fn selector_family(selector: &str) -> &str {
    // Retain layouts saved before Firefox's D-Bus-derived suffix was recognized.
    selector
        .rsplit_once(".instance")
        .filter(|(_, suffix)| dbus_instance_suffix(suffix))
        .map_or(selector, |(prefix, _)| prefix)
}
fn matches_selector(name: &str, selector: &str) -> bool {
    selector == crate::saved_native::DEFAULT_MEDIA
        || name == selector
        || family(name) == selector_family(selector)
}
fn previous_selection() -> Option<Session> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Saved {
        name: String,
        owner: String,
    }
    let path = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?)
        .join("decksmith-media-player.json");
    let saved: Saved =
        serde_json::from_slice(&crate::layout_store::read(&path, 4096).ok()?).ok()?;
    let pin = Session {
        selector: crate::saved_native::DEFAULT_MEDIA.into(),
        name: saved.name,
        owner: saved.owner,
    };
    pin.validate().ok()?;
    Some(pin)
}
struct Candidate {
    name: String,
    owner: String,
    status: String,
}
fn selection(
    selector: &str,
    candidates: &[Candidate],
    preferred: Option<&Session>,
) -> Option<Session> {
    candidates
        .iter()
        .filter(|c| matches_selector(&c.name, selector))
        .min_by_key(|c| {
            (
                preferred.is_none_or(|p| p.name != c.name || p.owner != c.owner),
                match c.status.as_str() {
                    "Playing" => 0,
                    "Paused" => 1,
                    _ => 2,
                },
                c.name.as_str(),
            )
        })
        .map(|c| Session {
            selector: selector.into(),
            name: c.name.clone(),
            owner: c.owner.clone(),
        })
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub selector: String,
    pub name: String,
    pub owner: String,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MediaSession([private])")
    }
}
impl Session {
    pub fn validate(&self) -> Result<()> {
        if !(crate::launch::valid_player(&self.selector)
            || self.selector == crate::saved_native::DEFAULT_MEDIA)
            || !crate::launch::valid_player(&self.name)
            || !matches_selector(&self.name, &self.selector)
            || self.owner.len() > 255
            || zbus::names::UniqueName::try_from(self.owner.as_str()).is_err()
        {
            return Err("invalid_native_media_session");
        }
        Ok(())
    }
}
trait Bus {
    fn owner(&self, name: &str) -> Result<String>;
    fn text(&self, owner: &str, property: &str) -> Result<String>;
    fn flag(&self, owner: &str, property: &str) -> Result<bool>;
    fn invoke(&self, owner: &str, method: &str) -> Result<()>;
}
fn execute(
    bus: &impl Bus,
    session: &Session,
    command: &str,
    current: impl Fn() -> bool,
) -> Result<()> {
    session.validate()?;
    let method = match command {
        "play_pause" => "PlayPause",
        "next" => "Next",
        "previous" => "Previous",
        _ => return Err("invalid_shared_media_action"),
    };
    if !current() {
        return Err("stale_action_scope");
    }
    if bus.owner(&session.name)? != session.owner {
        return Err("native_media_session_changed");
    }
    let status = bus.text(&session.owner, "PlaybackStatus")?;
    if !matches!(status.as_str(), "Playing" | "Paused" | "Stopped") {
        return Err("invalid_native_media_state");
    }
    let capability = match command {
        "next" => "CanGoNext",
        "previous" => "CanGoPrevious",
        _ if status == "Playing" => "CanPause",
        _ => "CanPlay",
    };
    if !bus.flag(&session.owner, "CanControl")? || !bus.flag(&session.owner, capability)? {
        return Err("media_unsupported");
    }
    // Capabilities may wait. Revalidate the well-known name's exact owner, then
    // check the device fence immediately before the only external write.
    if bus.owner(&session.name)? != session.owner {
        return Err("native_media_session_changed");
    }
    if !current() {
        return Err("stale_action_scope");
    }
    bus.invoke(&session.owner, method)
}
struct NativeBus(zbus::Connection);
// The flags path bypasses zbus's ordinary connection method_timeout wrapper.
// Dropping a timed-out future cannot undo a request that has already been sent.
fn bounded_call<B, R>(proxy: &zbus::Proxy<'_>, method: &str, body: &B) -> zbus::Result<Option<R>>
where
    B: serde::Serialize + zbus::DynamicType,
    R: for<'de> zbus::wire::DynamicDeserialize<'de>,
{
    futures_lite::future::block_on(futures_lite::future::race(
        proxy.call_with_flags(method, zbus::proxy::MethodFlags::NO_AUTO_START, body),
        async {
            async_io::Timer::after(Duration::from_millis(200)).await;
            Err(zbus::Error::Failure("bounded native media timeout".into()))
        },
    ))
}
impl NativeBus {
    fn connect() -> Result<Self> {
        futures_lite::future::block_on(
            zbus::connection::Builder::session()
                .method_timeout(Duration::from_millis(200))
                .build(),
        )
        .map(Self)
        .map_err(|_| "media_unavailable")
    }
    fn proxy<'a>(&'a self, owner: &'a str, interface: &'a str) -> Result<zbus::Proxy<'a>> {
        futures_lite::future::block_on(zbus::Proxy::new(&self.0, owner, PATH, interface))
            .map_err(|_| "media_unavailable")
    }
    fn property(&self, owner: &str, property: &str) -> Result<zbus::OwnedVariant> {
        bounded_call(
            &self.proxy(owner, "org.freedesktop.DBus.Properties")?,
            "Get",
            &(PLAYER, property),
        )
        .map_err(|_| "media_unavailable")?
        .ok_or("media_unavailable")
    }
}
impl Bus for NativeBus {
    fn owner(&self, name: &str) -> Result<String> {
        futures_lite::future::block_on(async {
            zbus::Proxy::new(
                &self.0,
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
            )
            .await
            .map_err(|_| "media_unavailable")?
            .call("GetNameOwner", &(name,))
            .await
            .map_err(|_| "media_unavailable")
        })
    }
    fn text(&self, owner: &str, property: &str) -> Result<String> {
        let value = String::try_from(self.property(owner, property)?)
            .map_err(|_| "invalid_native_media_state")?;
        if value.len() > 32 {
            return Err("invalid_native_media_state");
        }
        Ok(value)
    }
    fn flag(&self, owner: &str, property: &str) -> Result<bool> {
        bool::try_from(self.property(owner, property)?).map_err(|_| "invalid_native_media_state")
    }
    fn invoke(&self, owner: &str, method: &str) -> Result<()> {
        bounded_call::<_, ()>(&self.proxy(owner, PLAYER)?, method, &())
            .map_err(|_| "native_media_effect_uncertain")?
            .ok_or("native_media_effect_uncertain")
    }
}

/// One supplied executor for this initial native slice. Named audio and media
/// are off the I/O/render owners. No input mute or alias/default resolution.
pub struct NativeBackend {
    audio: NamedAudioBackend,
    saved_audio: crate::shared_native_audio::Client,
    builtin: crate::shared_builtin::Client,
    blocked_builtin: bool,
    bus: Option<NativeBus>,
    daily: bool,
    preferred: Option<Session>,
    blocked_audio: BTreeSet<String>,
    blocked_media: BTreeSet<String>,
    uncertain: Arc<AtomicBool>,
}
impl NativeBackend {
    pub fn connect() -> Result<Self> {
        Ok(Self {
            audio: NamedAudioBackend,
            saved_audio: crate::shared_native_audio::Client::default(),
            builtin: crate::shared_builtin::Client::default(),
            blocked_builtin: false,
            bus: Some(NativeBus::connect()?),
            daily: false,
            preferred: None,
            blocked_audio: BTreeSet::new(),
            blocked_media: BTreeSet::new(),
            uncertain: Arc::new(AtomicBool::new(false)),
        })
    }
    /// Permanent service startup must not depend on an active player or even
    /// the optional media bus. Discovery happens only on the executor's action.
    pub fn connect_daily() -> Result<Self> {
        Ok(Self {
            audio: NamedAudioBackend,
            saved_audio: crate::shared_native_audio::Client::default(),
            builtin: crate::shared_builtin::Client::default(),
            blocked_builtin: false,
            bus: None,
            daily: true,
            preferred: previous_selection(),
            blocked_audio: BTreeSet::new(),
            blocked_media: BTreeSet::new(),
            uncertain: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn health(&self) -> Arc<AtomicBool> {
        self.uncertain.clone()
    }
    /// Resolve on the setup owner, never in HID/render/coordinator ticks. The
    /// returned pins remain stable through pause; absent explicit apps stay absent.
    pub fn resolve_media(
        &self,
        selectors: &[String],
        preferred: Option<&Session>,
    ) -> Result<Vec<Session>> {
        if selectors.len() > 8
            || selectors
                .iter()
                .any(|s| s != crate::saved_native::DEFAULT_MEDIA && !crate::launch::valid_player(s))
        {
            return Err("invalid_native_media_bindings");
        }
        if selectors.is_empty() {
            return Ok(Vec::new());
        }
        let Some(bus) = &self.bus else {
            return Ok(Vec::new());
        };
        let previous = preferred.cloned().or_else(previous_selection);
        let proxy = futures_lite::future::block_on(zbus::Proxy::new(
            &bus.0,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
        ))
        .map_err(|_| "media_unavailable")?;
        let names: Vec<String> = futures_lite::future::block_on(proxy.call("ListNames", &()))
            .map_err(|_| "media_unavailable")?;
        let names: Vec<_> = names
            .into_iter()
            .filter(|n| crate::launch::valid_player(n))
            .collect();
        if names.len() > 8 {
            return Err("native_media_inventory_too_large");
        }
        let mut candidates = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_millis(900);
        for name in names {
            if std::time::Instant::now() >= deadline {
                break;
            }
            if !selectors
                .iter()
                .any(|selector| matches_selector(&name, selector))
            {
                continue;
            }
            let Ok(owner) = bus.owner(&name) else {
                continue;
            };
            let Ok(status) = bus.text(&owner, "PlaybackStatus") else {
                continue;
            };
            if !matches!(status.as_str(), "Playing" | "Paused" | "Stopped")
                || bus.flag(&owner, "CanControl") != Ok(true)
                || bus.owner(&name).as_ref() != Ok(&owner)
            {
                continue;
            }
            candidates.push(Candidate {
                name,
                owner,
                status,
            });
        }
        Ok(selectors
            .iter()
            .filter_map(|s| selection(s, &candidates, previous.as_ref()))
            .collect())
    }
}
impl Backend for NativeBackend {
    fn execute(&mut self, action: &Action, fence: &Fence) -> Result<()> {
        match action {
            Action::OpenApplication { .. } | Action::OpenWebsite { .. } | Action::System { .. } => {
                if self.blocked_builtin {
                    return Err("native_builtin_effect_blocked");
                }
                let result = self.builtin.execute(action, || fence.accepts());
                if result == Err("native_builtin_effect_uncertain") {
                    self.blocked_builtin = true;
                    self.uncertain.store(true, Ordering::Release);
                }
                result
            }
            Action::AudioSelect { .. } => {
                if self.blocked_audio.contains("selection") {
                    return Err("native_audio_effect_blocked");
                }
                let result = self.saved_audio.execute(action, || fence.accepts());
                if result == Err("native_audio_effect_uncertain") {
                    self.blocked_audio.insert("selection".into());
                    self.uncertain.store(true, Ordering::Release);
                }
                result
            }
            Action::AudioAdjust { target, .. } | Action::AudioMute { target } => {
                if self.blocked_audio.len() >= 8 || self.blocked_audio.contains(target) {
                    return Err("native_audio_effect_blocked");
                }
                let result = if matches!(target.as_str(), "system" | "default_output")
                    || target.starts_with("app:")
                {
                    self.saved_audio.execute(action, || fence.accepts())
                } else {
                    self.audio.execute(action, fence)
                };
                if matches!(
                    result,
                    Err("shared_audio_timeout" | "native_audio_effect_uncertain")
                ) {
                    self.uncertain.store(true, Ordering::Release);
                    self.blocked_audio.insert(target.clone());
                }
                result
            }
            Action::MediaTarget { player, command } => {
                if self.bus.is_none() {
                    self.bus = Some(NativeBus::connect()?);
                }
                let session = if let Some(session) = fence.media_session() {
                    session.clone()
                } else if self.daily {
                    self.resolve_media(std::slice::from_ref(player), self.preferred.as_ref())?
                        .into_iter()
                        .next()
                        .ok_or("media_unavailable")?
                } else {
                    return Err("native_media_requires_pinned_session");
                };
                if player != &session.selector {
                    return Err("native_media_payload_mismatch");
                }
                if self.blocked_media.len() >= 8 || self.blocked_media.contains(&session.owner) {
                    return Err("native_media_effect_blocked");
                }
                let result = execute(self.bus.as_ref().unwrap(), &session, command, || {
                    fence.accepts()
                });
                if result == Err("native_media_effect_uncertain") {
                    self.uncertain.store(true, Ordering::Release);
                    self.blocked_media.insert(session.owner.clone());
                }
                if result.is_ok() && player == crate::saved_native::DEFAULT_MEDIA {
                    self.preferred = Some(session);
                }
                result
            }
            _ => Err("unsupported_shared_native_action"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    struct Fake {
        owner: RefCell<String>,
        writes: RefCell<Vec<(String, String)>>,
        alive: Cell<bool>,
        replace: Cell<bool>,
        revoke: Cell<bool>,
        supported: Cell<bool>,
    }
    impl Bus for Fake {
        fn owner(&self, _: &str) -> Result<String> {
            Ok(self.owner.borrow().clone())
        }
        fn text(&self, _: &str, _: &str) -> Result<String> {
            Ok("Playing".into())
        }
        fn flag(&self, _: &str, _: &str) -> Result<bool> {
            if self.replace.get() {
                *self.owner.borrow_mut() = ":1.99".into();
            }
            if self.revoke.get() {
                self.alive.set(false);
            }
            Ok(self.supported.get())
        }
        fn invoke(&self, owner: &str, method: &str) -> Result<()> {
            self.writes.borrow_mut().push((owner.into(), method.into()));
            Ok(())
        }
    }
    fn fixture() -> (Fake, Session) {
        (
            Fake {
                owner: RefCell::new(":1.22".into()),
                writes: RefCell::new(Vec::new()),
                alive: Cell::new(true),
                replace: Cell::new(false),
                revoke: Cell::new(false),
                supported: Cell::new(true),
            },
            Session {
                selector: "org.mpris.MediaPlayer2.fixture".into(),
                name: "org.mpris.MediaPlayer2.fixture".into(),
                owner: ":1.22".into(),
            },
        )
    }
    #[test]
    fn exact_owner_dispatch_rejects_replacement_unsupported_and_late_scope_without_fallback() {
        let (bus, s) = fixture();
        execute(&bus, &s, "play_pause", || bus.alive.get()).unwrap();
        assert_eq!(
            *bus.writes.borrow(),
            vec![(":1.22".into(), "PlayPause".into())]
        );
        for mode in 0..3 {
            let (bus, s) = fixture();
            match mode {
                0 => bus.replace.set(true),
                1 => bus.revoke.set(true),
                _ => bus.supported.set(false),
            };
            assert!(execute(&bus, &s, "next", || bus.alive.get()).is_err());
            assert!(bus.writes.borrow().is_empty());
        }
    }
    #[test]
    fn pinned_media_requires_exact_valid_name_and_unique_owner() {
        let (_, mut s) = fixture();
        s.owner = "org.mpris.MediaPlayer2.other".into();
        assert!(s.validate().is_err());
        s.owner = ":1.22".into();
        s.selector = "org.mpris.MediaPlayer2.other".into();
        assert!(s.validate().is_err());
    }
    #[test]
    fn family_and_default_resolution_keep_explicit_missing_and_retained_paused_player() {
        let candidates = vec![
            Candidate {
                name: "org.mpris.MediaPlayer2.brave.instance7".into(),
                owner: ":1.22".into(),
                status: "Paused".into(),
            },
            Candidate {
                name: "org.mpris.MediaPlayer2.mpz".into(),
                owner: ":1.23".into(),
                status: "Playing".into(),
            },
        ];
        let brave = selection("org.mpris.MediaPlayer2.brave", &candidates, None).unwrap();
        brave.validate().unwrap();
        assert_eq!(brave.owner, ":1.22");
        assert!(selection("org.mpris.MediaPlayer2.missing", &candidates, None).is_none());
        assert_eq!(
            selection(
                crate::saved_native::DEFAULT_MEDIA,
                &candidates,
                Some(&brave)
            )
            .unwrap()
            .owner,
            ":1.22"
        );
        assert_eq!(
            selection(crate::saved_native::DEFAULT_MEDIA, &candidates, None)
                .unwrap()
                .owner,
            ":1.23"
        );
        let mut stale = brave;
        stale.owner = ":1.99".into();
        assert_eq!(
            selection(
                crate::saved_native::DEFAULT_MEDIA,
                &candidates,
                Some(&stale)
            )
            .unwrap()
            .owner,
            ":1.23"
        );
    }
    #[test]
    fn firefox_saved_choices_survive_restart_without_cross_app_fallback() {
        let candidates = vec![
            Candidate {
                name: "org.mpris.MediaPlayer2.firefox.instance_1_955".into(),
                owner: ":1.955".into(),
                status: "Paused".into(),
            },
            Candidate {
                name: "org.mpris.MediaPlayer2.music".into(),
                owner: ":1.22".into(),
                status: "Playing".into(),
            },
        ];
        for selector in [
            "org.mpris.MediaPlayer2.firefox",
            "org.mpris.MediaPlayer2.firefox.instance_1_162",
        ] {
            let selected = selection(selector, &candidates, None).unwrap();
            selected.validate().unwrap();
            assert_eq!(selected.selector, selector);
            assert_eq!(selected.owner, ":1.955");
            let (bus, _) = fixture();
            *bus.owner.borrow_mut() = ":1.955".into();
            execute(&bus, &selected, "play_pause", || true).unwrap();
            assert_eq!(
                *bus.writes.borrow(),
                vec![(":1.955".into(), "PlayPause".into())]
            );
            bus.writes.borrow_mut().clear();
            bus.replace.set(true);
            assert!(execute(&bus, &selected, "play_pause", || true).is_err());
            assert!(bus.writes.borrow().is_empty());
            assert!(selection(selector, &candidates[1..], None).is_none());
        }
    }
    #[test]
    fn instance_suffixes_are_bounded_and_existing_exact_choices_stay_exact() {
        let base = "org.mpris.MediaPlayer2.firefox";
        assert_eq!(family(&format!("{base}.instance_1_162")), base);
        for suffix in ["_1", "_1_2_extra", "__2", "_1_", "_named", "_١_٢"] {
            let name = format!("{base}.instance{suffix}");
            assert_eq!(family(&name), name);
            assert_eq!(selector_family(&name), name);
        }
        let exact = "org.mpris.MediaPlayer2.brave.instance123";
        assert_eq!(family(exact), "org.mpris.MediaPlayer2.brave");
        assert_eq!(selector_family(exact), exact);
        assert!(!matches_selector(
            "org.mpris.MediaPlayer2.brave.instance456",
            exact
        ));
    }
    #[test]
    fn daily_factory_requires_no_media_or_audio_connection_at_startup() {
        let backend = NativeBackend::connect_daily().unwrap();
        assert!(backend.daily && backend.bus.is_none());
        assert!(
            backend
                .resolve_media(&[crate::saved_native::DEFAULT_MEDIA.into()], None)
                .unwrap()
                .is_empty()
        );
        assert!(!backend.health().load(Ordering::Acquire));
    }
}
