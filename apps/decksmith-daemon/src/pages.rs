//! Validated saved pages with local navigation actions only.
use decksmith_core::{RawEvent, TouchGesture};
use decksmith_device::{DeckDevice, DeviceError};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    version: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    theme: Option<crate::theme::Theme>,
    #[serde(default)]
    audio_dial: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dials: Option<[Dial; 4]>,
    pages: Vec<Page>,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Rotation {
    None,
    Volume,
    Brightness,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Dial {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plugin_rotation: Option<crate::plugin_binding::Binding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plugin_press: Option<crate::plugin_binding::Binding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_icon_png: Option<Vec<u8>>,
    #[serde(skip)]
    target_icon_rgba: Option<std::sync::Arc<Vec<u8>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<crate::theme::Appearance>,
    label: String,
    rotation: Rotation,
    step: u8,
    press: Action,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    audio_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    media_player: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Page {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    default: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dial_overrides: Option<[Option<Dial>; 4]>,
    name: String,
    background: [u8; 3],
    keys: [Key; 8],
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Key {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plugin: Option<crate::plugin_binding::Binding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    appearance: Option<crate::theme::Appearance>,
    #[serde(default)]
    artwork: Option<Artwork>,
    label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label_position: Option<LabelPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label_color: Option<LabelColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    background_color: Option<LabelColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label_background: Option<LabelBackground>,
    #[serde(default)]
    follow_page_name: bool,
    action: Action,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    media_player: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_png: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_source: Option<String>,
    #[serde(default)]
    icon_tint: bool,
    #[serde(skip)]
    icon_rgba: Option<Vec<u8>>,
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelBackground {
    Dark,
    Transparent,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelColor {
    Default,
    White,
    Black,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}
impl LabelColor {
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Default => [240, 230, 215],
            Self::White => [255, 255, 255],
            Self::Black => [0, 0, 0],
            Self::Red => [255, 70, 70],
            Self::Orange => [255, 155, 60],
            Self::Yellow => [255, 225, 60],
            Self::Green => [90, 230, 120],
            Self::Blue => [90, 170, 255],
            Self::Purple => [195, 130, 255],
        }
    }
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelPosition {
    Hidden,
    Top,
    Middle,
    Bottom,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Artwork {
    MakersMark,
    ApplicationIcon,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Plugin {
        binding: crate::plugin_binding::Binding,
        ticks: i16,
    },
    System {
        command: String,
    },
    None,
    GoToPage {
        page: u8,
    },
    NextPage,
    PreviousPage,
    OpenApplication {
        desktop_id: String,
    },
    OpenWebsite {
        url: String,
    },
    AudioAdjust {
        target: String,
        percent: i16,
    },
    AudioMute {
        target: String,
    },
    AudioSelect {
        target: String,
    },
    PushToTalk {
        target: String,
    },
    VolumeUp,
    VolumeDown,
    MuteToggle,
    MediaTarget {
        player: String,
        command: String,
    },
    MediaPlayPause,
    MediaNext,
    MediaPrevious,
    VolumeAdjust {
        percent: i16,
    },
    BrightnessAdjust {
        percent: i16,
    },
}

fn media_command(action: &Action) -> Option<&'static str> {
    match action {
        Action::MediaPlayPause => Some("play_pause"),
        Action::MediaNext => Some("next"),
        Action::MediaPrevious => Some("previous"),
        _ => None,
    }
}
fn media_target(action: Action, player: Option<&str>) -> Action {
    if let Some(player) = player
        && let Some(command) = media_command(&action)
    {
        Action::MediaTarget {
            player: player.into(),
            command: command.into(),
        }
    } else {
        action
    }
}

#[derive(Clone, Default)]
pub struct TouchState {
    pub system: crate::system_actions::States,
    pub feedback: Vec<crate::feedback::Notice>,
    pub levels: crate::meter::Levels,
    pub audio: Option<Option<crate::audio::State>>,
    pub brightness: Option<u8>,
    pub targets: Vec<(String, Option<crate::audio::State>)>,
}

#[derive(Clone, PartialEq, Eq)]
struct StripKey {
    plugin: Option<crate::plugin_runtime::State>,
    page: u8,
    audio: Option<crate::audio::State>,
    brightness: Option<u8>,
    targets: Vec<(String, Option<crate::audio::State>)>,
}
pub struct Pages {
    system: crate::system_actions::States,
    health: Vec<crate::feedback::Notice>,
    failure_painted: bool,
    last_failure: Option<(crate::feedback::Notice, std::time::Instant)>,
    strip_cache: std::cell::RefCell<Option<(StripKey, Vec<u8>)>>,
    levels: crate::meter::Levels,
    pub index: u8,
    logo: Option<Vec<u8>>,
    config: Config,
    pending: [Option<Action>; 8],
    dial_armed: [Option<Action>; 4],
    brightness: Option<u8>,
    target_states: Vec<(String, Option<crate::audio::State>)>,
    audio_checked: Option<std::time::Instant>,
    audio_shown: Option<Option<crate::audio::State>>,
    plugin_shown: Option<crate::plugin_runtime::State>,
}
impl Default for Pages {
    fn default() -> Self {
        Self::parse(include_bytes!("../../../config/navigation.json"))
            .expect("bundled configuration is valid")
    }
}
impl Pages {
    pub fn default_page(&self) -> u8 {
        self.config
            .pages
            .iter()
            .position(|p| p.default)
            .unwrap_or(0) as u8
    }
    pub fn application_page(&self, id: &str) -> Option<u8> {
        if id == crate::foreground::STUDIO {
            return None;
        }
        self.config
            .pages
            .iter()
            .position(|p| p.application.as_deref() == Some(id))
            .map(|i| i as u8)
            .or(Some(self.default_page()))
    }

    pub fn json(&self) -> String {
        serde_json::to_string(&self.config).expect("validated configuration serializes")
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > 1048576 {
            return Err("config_too_large");
        }
        let mut config: Config = serde_json::from_slice(bytes).map_err(|_| "invalid_config")?;
        let valid_label = |s: &str| {
            !s.is_empty()
                && s.len() <= 24
                && !s.trim().is_empty()
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b' ')
        };
        if config.version != 1 || config.pages.is_empty() || config.pages.len() > 16 {
            return Err("invalid_config");
        }
        let defaults = config.pages.iter().filter(|p| p.default).count();
        if defaults > 1 {
            return Err("multiple_default_pages");
        }
        if defaults == 0 {
            config.pages[0].default = true;
        }
        let default_page = config.pages.iter().position(|p| p.default).unwrap() as u8;
        {
            let dials = config.dials.iter_mut().flatten().chain(
                config
                    .pages
                    .iter_mut()
                    .filter_map(|p| p.dial_overrides.as_mut())
                    .flatten()
                    .filter_map(|d| d.as_mut()),
            );
            for dial in dials {
                for binding in [&dial.plugin_rotation, &dial.plugin_press]
                    .into_iter()
                    .flatten()
                {
                    binding.validate()?;
                }
                if (dial.plugin_rotation.is_some() && !matches!(dial.rotation, Rotation::None))
                    || (dial.plugin_press.is_some() && !matches!(dial.press, Action::None))
                {
                    return Err("conflicting_plugin_binding");
                }
                if let Some(bytes) = &dial.target_icon_png {
                    if bytes.len() > 8192
                        || image::ImageReader::with_format(
                            std::io::Cursor::new(bytes),
                            image::ImageFormat::Png,
                        )
                        .into_dimensions()
                        .map_err(|_| "invalid_dial_icon")?
                            != (32, 32)
                    {
                        return Err("invalid_dial_icon");
                    }
                    dial.target_icon_rgba = Some(std::sync::Arc::new(
                        image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
                            .map_err(|_| "invalid_dial_icon")?
                            .to_rgba8()
                            .into_raw(),
                    ));
                }
                if dial.media_player.as_ref().is_some_and(|p| {
                    !crate::launch::valid_player(p) || media_command(&dial.press).is_none()
                }) {
                    return Err("invalid_media_player");
                }
                if let Action::PushToTalk { target } = &dial.press
                    && (!target.starts_with("input:")
                        || !crate::audio_target::valid(target)
                        || dial.audio_target.as_ref() != Some(target))
                {
                    return Err("invalid_ptt_target");
                }
                if dial
                    .audio_target
                    .as_ref()
                    .is_some_and(|t| !crate::audio_target::valid(t))
                {
                    return Err("invalid_audio_target");
                }
                if dial.label.trim().is_empty()
                    || dial.label.len() > 24
                    || !dial
                        .label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b' ')
                    || !(1..=10).contains(&dial.step)
                    || !matches!(
                        dial.press,
                        Action::None
                            | Action::PushToTalk { .. }
                            | Action::MuteToggle
                            | Action::NextPage
                            | Action::PreviousPage
                            | Action::MediaPlayPause
                            | Action::MediaNext
                            | Action::MediaPrevious
                    )
                {
                    return Err("invalid_dial");
                }
            }
        }
        let mut assigned = std::collections::HashSet::new();
        for page in &config.pages {
            if let Some(id) = &page.application {
                if !crate::launch::valid_application(id) || id == crate::foreground::STUDIO {
                    return Err("invalid_page_application");
                }
                if !assigned.insert(id) {
                    return Err("duplicate_page_application");
                }
            }
        }
        for page in &config.pages {
            if !valid_label(&page.name) {
                return Err("invalid_config");
            }
            for key in &page.keys {
                if let Some(binding) = &key.plugin {
                    binding.validate()?;
                    if !matches!(key.action, Action::None) {
                        return Err("conflicting_plugin_binding");
                    }
                }
                if matches!(
                    key.action,
                    Action::MediaTarget { .. } | Action::Plugin { .. }
                ) || key.media_player.as_ref().is_some_and(|p| {
                    !crate::launch::valid_player(p) || media_command(&key.action).is_none()
                }) {
                    return Err("invalid_media_player");
                }
                match &key.action {
                    Action::PushToTalk { target }
                        if !target.starts_with("input:") || !crate::audio_target::valid(target) =>
                    {
                        return Err("invalid_ptt_target");
                    }
                    Action::AudioAdjust { target, percent }
                        if !crate::audio_target::valid(target)
                            || *percent == 0
                            || !(-20..=20).contains(percent) =>
                    {
                        return Err("invalid_audio_target");
                    }
                    Action::AudioMute { target } if !crate::audio_target::valid(target) => {
                        return Err("invalid_audio_target");
                    }
                    Action::AudioSelect { target }
                        if !crate::audio_target::valid(target)
                            || !(target.starts_with("output:") || target.starts_with("input:")) =>
                    {
                        return Err("invalid_audio_target");
                    }
                    _ => (),
                }

                if key.label.is_empty()
                    || key.label.len() > 24
                    || !key
                        .label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b' ')
                {
                    return Err("invalid_config");
                }
                if let Action::VolumeAdjust { percent } | Action::BrightnessAdjust { percent } =
                    key.action
                    && (percent == 0 || !(-20..=20).contains(&percent))
                {
                    return Err("invalid_config");
                }
                if let Action::GoToPage { page } = key.action
                    && usize::from(page) >= config.pages.len()
                {
                    return Err("invalid_config");
                }
            }
        }
        let names: Vec<String> = config.pages.iter().map(|page| page.name.clone()).collect();
        for page in &mut config.pages {
            for key in &mut page.keys {
                match &key.action {
                    Action::System { command } if !crate::system_actions::valid(command) => {
                        return Err("invalid_system_action");
                    }
                    Action::OpenApplication { desktop_id }
                        if !crate::launch::valid_application(desktop_id) =>
                    {
                        return Err("invalid_application");
                    }
                    Action::OpenWebsite { url } if !crate::launch::valid_url(url) => {
                        return Err("invalid_url");
                    }
                    _ => (),
                }
                if key
                    .icon_source
                    .as_ref()
                    .is_some_and(|s| s.len() > 160 || !s.is_ascii())
                {
                    return Err("invalid_icon_source");
                }
                if let Some(bytes) = &key.icon_png {
                    if bytes.len() > 65536
                        || image::ImageReader::with_format(
                            std::io::Cursor::new(bytes),
                            image::ImageFormat::Png,
                        )
                        .into_dimensions()
                        .map_err(|_| "invalid_icon")?
                            != (120, 120)
                    {
                        return Err("invalid_icon");
                    }
                    key.icon_rgba = Some(
                        image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
                            .map_err(|_| "invalid_icon")?
                            .to_rgba8()
                            .into_raw(),
                    );
                }
                if key.follow_page_name {
                    let Action::GoToPage { page } = key.action else {
                        return Err("invalid_config");
                    };
                    key.label = names[usize::from(page)].clone();
                }
            }
        }
        let logo = if config
            .pages
            .iter()
            .any(|page| page.keys.iter().any(|key| key.artwork.is_some()))
        {
            let image = image::load_from_memory(include_bytes!(
                "../../../brand/decksmith-makers-mark-brand-sheet.png"
            ))
            .map_err(|_| "brand_image_failed")?;
            Some(
                image
                    .crop_imm(840, 48, 212, 212)
                    .resize_exact(120, 120, image::imageops::FilterType::Lanczos3)
                    .to_rgb8()
                    .into_raw(),
            )
        } else {
            None
        };
        Ok(Self {
            index: default_page,
            logo,
            config,
            pending: std::array::from_fn(|_| None),
            dial_armed: std::array::from_fn(|_| None),
            brightness: None,
            health: Vec::new(),
            failure_painted: false,
            last_failure: None,
            system: Default::default(),
            target_states: Vec::new(),
            audio_checked: None,
            audio_shown: None,
            plugin_shown: None,
            levels: Default::default(),
            strip_cache: Default::default(),
        })
    }
    fn effective_dials(&self) -> Option<[Dial; 4]> {
        let overrides = self.config.pages[usize::from(self.index)]
            .dial_overrides
            .as_ref();
        if self.config.dials.is_none() && overrides.is_none() {
            return None;
        }
        let defaults = self.config.dials.clone().unwrap_or_else(|| {
            std::array::from_fn(|i| {
                let audio = i == 0 && self.config.audio_dial;
                Dial {
                    plugin_rotation: None,
                    plugin_press: None,
                    target_icon_png: None,
                    target_icon_rgba: None,
                    label: if audio {
                        "Volume".into()
                    } else {
                        format!("Dial {}", i + 1)
                    },
                    rotation: if audio {
                        Rotation::Volume
                    } else {
                        Rotation::None
                    },
                    step: 1,
                    press: if audio {
                        Action::MuteToggle
                    } else {
                        Action::None
                    },
                    appearance: None,
                    audio_target: None,
                    media_player: None,
                }
            })
        });
        Some(std::array::from_fn(|i| {
            let mut dial = overrides
                .and_then(|o| o[i].clone())
                .unwrap_or_else(|| defaults[i].clone());
            if dial.audio_target.is_none()
                && (matches!(dial.rotation, Rotation::Volume)
                    || matches!(dial.press, Action::MuteToggle))
            {
                dial.audio_target = Some("system".into());
            }
            dial
        }))
    }
    pub fn target(&mut self, event: &RawEvent) -> Option<Action> {
        let target = match event {
            RawEvent::DialRotate { index, ticks } if *index < 4 && *ticks != 0 => {
                if let Some(dials) = &self.effective_dials() {
                    let dial = &dials[usize::from(*index)];
                    let percent = ticks.saturating_mul(i16::from(dial.step)).clamp(-20, 20);
                    if let Some(binding) = &dial.plugin_rotation {
                        return binding.supported(true).then(|| Action::Plugin {
                            binding: binding.clone(),
                            ticks: percent,
                        });
                    }
                    match dial.rotation {
                        Rotation::None => None,
                        Rotation::Volume => Some(if let Some(target) = &dial.audio_target {
                            Action::AudioAdjust {
                                target: target.clone(),
                                percent,
                            }
                        } else {
                            Action::VolumeAdjust { percent }
                        }),
                        Rotation::Brightness => Some(Action::BrightnessAdjust { percent }),
                    }
                } else if self.config.audio_dial && *index == 0 {
                    Some(Action::VolumeAdjust {
                        percent: (*ticks).clamp(-20, 20),
                    })
                } else {
                    None
                }
            }
            RawEvent::DialPush { index, pressed } if *index < 4 => {
                let slot = usize::from(*index);
                if *pressed {
                    let action = if let Some(dials) = &self.effective_dials() {
                        if matches!(dials[slot].press, Action::MuteToggle)
                            && let Some(target) = &dials[slot].audio_target
                        {
                            Action::AudioMute {
                                target: target.clone(),
                            }
                        } else {
                            media_target(
                                dials[slot].press.clone(),
                                dials[slot].media_player.as_deref(),
                            )
                        }
                    } else if self.config.audio_dial && slot == 0 {
                        Action::MuteToggle
                    } else {
                        Action::None
                    };
                    self.dial_armed[slot] =
                        if matches!(action, Action::None | Action::PushToTalk { .. }) {
                            None
                        } else {
                            Some(action)
                        };
                    None
                } else {
                    self.dial_armed[slot].take()
                }
            }
            RawEvent::Touch {
                x,
                y,
                gesture: TouchGesture::Tap,
            } if *x < 800 && *y < 100 => {
                let slot = usize::from(*x / 200);
                if let Some(dials) = &self.effective_dials() {
                    let dial = &dials[slot];
                    if matches!(dial.rotation, Rotation::Volume)
                        || matches!(dial.press, Action::MuteToggle | Action::PushToTalk { .. })
                    {
                        Some(
                            dial.audio_target
                                .as_ref()
                                .map_or(Action::MuteToggle, |target| Action::AudioMute {
                                    target: target.clone(),
                                }),
                        )
                    } else {
                        None
                    }
                } else if self.config.audio_dial && slot == 0 {
                    Some(Action::MuteToggle)
                } else {
                    None
                }
            }
            RawEvent::Touch {
                gesture: TouchGesture::FlickLeft,
                ..
            } => Some(Action::NextPage),
            RawEvent::Touch {
                gesture: TouchGesture::FlickRight,
                ..
            } => Some(Action::PreviousPage),
            RawEvent::Key { index, pressed } if *index < 8 => {
                let slot = usize::from(*index);
                if *pressed {
                    if let Some(binding) =
                        &self.config.pages[usize::from(self.index)].keys[slot].plugin
                    {
                        self.pending[slot] = binding.supported(false).then(|| Action::Plugin {
                            binding: binding.clone(),
                            ticks: 0,
                        });
                        return None;
                    }
                    self.pending[slot] = match self.config.pages[usize::from(self.index)].keys[slot]
                        .action
                        .clone()
                    {
                        Action::None | Action::PushToTalk { .. } => None,
                        action => Some(media_target(
                            action,
                            self.config.pages[usize::from(self.index)].keys[slot]
                                .media_player
                                .as_deref(),
                        )),
                    };
                    None
                } else {
                    self.pending[slot].take()
                }
            }
            _ => None,
        };
        let count = self.config.pages.len() as u8;
        let target = target.map(|action| match action {
            Action::NextPage => Action::GoToPage {
                page: (self.index + 1) % count,
            },
            Action::PreviousPage => Action::GoToPage {
                page: (self.index + count - 1) % count,
            },
            action => action,
        });
        target.filter(|target| !matches!(target, Action::GoToPage { page } if *page == self.index))
    }
    /// Resolve an editor click without consuming or arming physical held inputs.
    pub fn test_target(&mut self, slot: u8) -> Option<Action> {
        if slot >= 12 {
            return None;
        }
        let keys = self.pending.clone();
        let dials = self.dial_armed.clone();
        let event = |pressed| {
            if slot < 8 {
                RawEvent::Key {
                    index: slot,
                    pressed,
                }
            } else {
                RawEvent::DialPush {
                    index: slot - 8,
                    pressed,
                }
            }
        };
        self.target(&event(true));
        let result = self.target(&event(false));
        self.pending = keys;
        self.dial_armed = dials;
        result
    }
    pub fn ptt_target(&self, slot: u8) -> Option<String> {
        let action = if slot < 8 {
            self.config.pages[usize::from(self.index)].keys[usize::from(slot)]
                .action
                .clone()
        } else {
            self.effective_dials()?
                .get(usize::from(slot - 8))?
                .press
                .clone()
        };
        if let Action::PushToTalk { target } = action {
            Some(target.clone())
        } else {
            None
        }
    }
    pub fn health_checks(&self) -> Vec<crate::feedback::Check> {
        let mut checks = Vec::new();
        for (i, key) in self.config.pages[usize::from(self.index)]
            .keys
            .iter()
            .enumerate()
        {
            let action = media_target(key.action.clone(), key.media_player.as_deref());
            if let Some(check) = crate::feedback::check(self.index, i as u8, &key.label, &action) {
                checks.push(check);
            }
        }
        if let Some(dials) = self.effective_dials() {
            for (i, dial) in dials.iter().enumerate() {
                if matches!(dial.rotation, Rotation::Volume)
                    && let Some(check) = crate::feedback::check(
                        self.index,
                        i as u8 + 8,
                        &dial.label,
                        &Action::AudioAdjust {
                            percent: 1,
                            target: dial.audio_target.clone().unwrap_or_else(|| "system".into()),
                        },
                    )
                {
                    checks.push(check);
                }
                let action = media_target(dial.press.clone(), dial.media_player.as_deref());
                let action = if matches!(action, Action::MuteToggle) {
                    Action::AudioMute {
                        target: dial.audio_target.clone().unwrap_or_else(|| "system".into()),
                    }
                } else {
                    action
                };
                if let Some(check) =
                    crate::feedback::check(self.index, i as u8 + 8, &dial.label, &action)
                    && !checks.contains(&check)
                {
                    checks.push(check);
                }
            }
        }
        checks
    }
    pub fn attention(&self) -> Vec<crate::feedback::Notice> {
        let mut notices: Vec<_> = self
            .health
            .iter()
            .filter(|n| n.attention() && n.check.page == self.index)
            .cloned()
            .collect();
        if let Some((failure, _)) = &self.last_failure
            && failure.check.page == self.index
        {
            notices.insert(0, failure.clone());
        }
        notices
    }
    fn device_feedback(&self) -> Vec<crate::feedback::Notice> {
        let mut notices: Vec<_> = self
            .health
            .iter()
            .filter(|n| n.attention() && n.check.page == self.index)
            .cloned()
            .collect();
        if let Some((failure, at)) = &self.last_failure
            && failure.check.page == self.index
            && at.elapsed() < std::time::Duration::from_secs(5)
        {
            notices.insert(0, failure.clone());
        }
        notices
    }
    pub fn update_health(
        &mut self,
        deck: &mut impl DeckDevice,
        notices: Vec<crate::feedback::Notice>,
    ) -> Result<(), DeviceError> {
        if self.health != notices {
            self.health = notices;
            self.repaint_feedback(deck)?;
        }
        Ok(())
    }
    pub fn expire_feedback(&mut self, deck: &mut impl DeckDevice) -> Result<(), DeviceError> {
        if self.failure_painted
            && self
                .last_failure
                .as_ref()
                .is_some_and(|(_, at)| at.elapsed() >= std::time::Duration::from_secs(5))
        {
            self.failure_painted = false;
            self.repaint_feedback(deck)?;
        }
        Ok(())
    }
    fn repaint_feedback(&self, deck: &mut impl DeckDevice) -> Result<(), DeviceError> {
        let page = &self.config.pages[usize::from(self.index)];
        for (i, key) in page.keys.iter().enumerate() {
            deck.set_key_image(i as u8, &self.key_image(page, key))?;
        }
        deck.set_touch_image(&self.touch_image(self.audio_shown))
    }
    pub fn action_feedback(
        &mut self,
        deck: &mut impl DeckDevice,
        page: u8,
        action: &Action,
        input: &decksmith_core::InputEvent,
        error: Option<&str>,
    ) -> Result<(), DeviceError> {
        if page != self.index {
            return Ok(());
        }
        let slot = match input.event {
            RawEvent::Key { index, .. } => index,
            RawEvent::DialPush { index, .. } | RawEvent::DialRotate { index, .. } => index + 8,
            RawEvent::Touch { x, .. } => (x / 200).min(3) as u8 + 8,
        };
        if slot >= 12 {
            return Ok(());
        }
        if slot < 8
            && media_target(
                self.config.pages[usize::from(page)].keys[usize::from(slot)]
                    .action
                    .clone(),
                self.config.pages[usize::from(page)].keys[usize::from(slot)]
                    .media_player
                    .as_deref(),
            ) != *action
        {
            return Ok(());
        }
        let label = if slot < 8 {
            self.config.pages[usize::from(page)].keys[usize::from(slot)]
                .label
                .clone()
        } else {
            self.effective_dials()
                .map(|d| d[usize::from(slot - 8)].label.clone())
                .unwrap_or_else(|| "Audio".into())
        };
        let check =
            crate::feedback::check(page, slot, &label, action).unwrap_or(crate::feedback::Check {
                page,
                slot,
                label: label.clone(),
                kind: "action".into(),
                target: label.clone(),
                command: String::new(),
            });
        if slot >= 8
            && check.kind != "action"
            && !self.health_checks().iter().any(|c| {
                c.slot == slot
                    && c.kind == check.kind
                    && c.target == check.target
                    && (check.kind != "media" || c.command == check.command)
            })
        {
            return Ok(());
        }
        if let Some(code) = error {
            let name = self
                .health
                .iter()
                .find(|n| n.check == check)
                .map(|n| n.target_name.clone())
                .unwrap_or_else(|| crate::feedback::target_name(&check));
            self.failure_painted = true;
            self.last_failure = Some((
                crate::feedback::failed(check, name, code),
                std::time::Instant::now(),
            ));
        } else if self
            .last_failure
            .as_ref()
            .is_some_and(|(n, _)| n.check == check)
        {
            self.last_failure = None;
        }
        self.repaint_feedback(deck)
    }
    pub fn system_commands(&self) -> Vec<String> {
        let mut commands: Vec<String> = self.config.pages[usize::from(self.index)]
            .keys
            .iter()
            .filter_map(|k| {
                if let Action::System { command } = &k.action {
                    Some(command.clone())
                } else {
                    None
                }
            })
            .collect();
        commands.sort();
        commands.dedup();
        commands
    }
    pub fn apply_system(
        &mut self,
        deck: &mut impl DeckDevice,
        states: crate::system_actions::States,
    ) -> Result<(), DeviceError> {
        if self.system != states {
            self.system = states;
            self.repaint_feedback(deck)?;
        }
        Ok(())
    }
    pub fn audio_targets(&self) -> Vec<String> {
        let mut targets: Vec<String> = self
            .effective_dials()
            .as_ref()
            .map(|dials| {
                dials
                    .iter()
                    .filter_map(|dial| dial.audio_target.clone())
                    .collect()
            })
            .unwrap_or_default();
        for key in &self.config.pages[usize::from(self.index)].keys {
            if let Action::AudioSelect { target } | Action::PushToTalk { target } = &key.action
                && !targets.contains(target)
            {
                targets.push(target.clone());
            }
        }
        targets
    }

    pub fn apply_targets(&mut self, targets: Vec<(String, Option<crate::audio::State>)>) {
        if self.target_states != targets {
            self.target_states = targets;
            self.audio_shown = None;
        }
    }
    pub fn invalidate_audio(&mut self) {
        self.audio_checked = None;
        self.audio_shown = None;
    }
    pub fn audio_due(&mut self) -> bool {
        let wants_audio = self
            .effective_dials()
            .as_ref()
            .map_or(self.config.audio_dial, |dials| {
                dials.iter().any(|dial| {
                    dial.plugin_rotation.is_some()
                        || matches!(dial.rotation, Rotation::Volume)
                        || matches!(dial.press, Action::MuteToggle | Action::PushToTalk { .. })
                })
            });
        let wants_audio = wants_audio
            || !self.system_commands().is_empty()
            || self.config.pages[usize::from(self.index)]
                .keys
                .iter()
                .any(|key| {
                    key.plugin.is_some()
                        || matches!(
                            key.action,
                            Action::AudioSelect { .. } | Action::PushToTalk { .. }
                        )
                });
        if !wants_audio
            || self
                .audio_checked
                .is_some_and(|last| last.elapsed() < std::time::Duration::from_millis(500))
        {
            return false;
        }
        self.audio_checked = Some(std::time::Instant::now());
        true
    }
    pub fn apply_audio(
        &mut self,
        deck: &mut impl DeckDevice,
        state: Option<crate::audio::State>,
    ) -> Result<bool, DeviceError> {
        let plugin = crate::plugin_runtime::state();
        if self.audio_shown == Some(state) && plugin == self.plugin_shown {
            return Ok(false);
        }
        for (index, key) in self.config.pages[usize::from(self.index)]
            .keys
            .iter()
            .enumerate()
        {
            if key.plugin.is_some()
                || matches!(
                    key.action,
                    Action::AudioSelect { .. } | Action::PushToTalk { .. }
                )
            {
                deck.set_key_image(
                    index as u8,
                    &self.key_image(&self.config.pages[usize::from(self.index)], key),
                )?;
            }
        }
        if self.effective_dials().is_some() || self.config.audio_dial {
            deck.set_touch_image(&self.touch_image(Some(state)))?;
        }
        self.audio_shown = Some(state);
        self.plugin_shown = plugin;
        Ok(true)
    }
    pub fn meter_targets(&self) -> Vec<String> {
        let mut targets = self
            .effective_dials()
            .as_ref()
            .map(|dials| {
                dials
                    .iter()
                    .filter(|dial| {
                        matches!(dial.rotation, Rotation::Volume)
                            || (matches!(dial.rotation, Rotation::None)
                                && matches!(
                                    dial.press,
                                    Action::MuteToggle | Action::PushToTalk { .. }
                                ))
                    })
                    .map(|dial| dial.audio_target.clone().unwrap_or_else(|| "system".into()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        targets.sort();
        targets.dedup();
        targets
    }
    pub fn apply_levels(
        &mut self,
        deck: &mut impl DeckDevice,
        levels: crate::meter::Levels,
    ) -> Result<(), DeviceError> {
        if self.levels != levels {
            self.levels = levels;
            deck.set_touch_image(&self.touch_image(self.audio_shown))?;
        }
        Ok(())
    }
    pub fn touch_state(&self) -> TouchState {
        TouchState {
            system: self.system.clone(),
            feedback: self.device_feedback(),
            levels: self.levels.clone(),
            audio: self.audio_shown,
            brightness: self.brightness,
            targets: self.target_states.clone(),
        }
    }

    /// Read-only draft rendering with the same image function used by hardware.
    pub fn preview_keys(&mut self, page: u8, state: TouchState) -> Result<Vec<u8>, &'static str> {
        self.index = page;
        self.health = state.feedback;
        self.last_failure = None;
        self.target_states = state.targets;
        self.system = state.system;
        let page = self
            .config
            .pages
            .get(usize::from(page))
            .ok_or("invalid_page")?;
        Ok(page
            .keys
            .iter()
            .flat_map(|key| self.key_image(page, key))
            .collect())
    }
    pub fn preview_touch(&mut self, page: u8, state: TouchState) -> Result<Vec<u8>, &'static str> {
        if usize::from(page) >= self.config.pages.len() {
            return Err("invalid_page");
        }
        self.index = page;
        self.health = state.feedback;
        self.last_failure = None;
        self.levels = state.levels;
        self.audio_shown = state.audio;
        self.brightness = state.brightness;
        self.target_states = state.targets;
        self.system = state.system;
        Ok(self.touch_image(self.audio_shown))
    }

    fn touch_image(&self, audio: Option<Option<crate::audio::State>>) -> Vec<u8> {
        if self.effective_dials().is_some() {
            return self.dial_strip(audio.flatten());
        }
        let page = &self.config.pages[usize::from(self.index)];
        if self.config.audio_dial
            && let Some(state) = audio
        {
            let label = match state {
                Some(state) => format!(
                    "{} {}% {}",
                    page.name,
                    state.percent,
                    if state.muted { "MUTED" } else { "VOL" }
                ),
                None => "AUDIO UNAVAILABLE".into(),
            };
            return frame(800, 100, &label, [30, 34, 39]);
        }
        frame(800, 100, &page.name, page.background)
    }

    fn dial_strip(&self, audio: Option<crate::audio::State>) -> Vec<u8> {
        let key = StripKey {
            plugin: crate::plugin_runtime::state(),
            page: self.index,
            audio,
            brightness: self.brightness,
            targets: self.target_states.clone(),
        };
        let mut cache = self.strip_cache.borrow_mut();
        if cache.as_ref().is_none_or(|(k, _)| *k != key) {
            *cache = Some((key, self.dial_strip_base(audio)));
        }
        let mut strip = cache.as_ref().unwrap().1.clone();
        if let Some(dials) = &self.effective_dials() {
            for (index, dial) in dials.iter().enumerate() {
                if !(matches!(dial.rotation, Rotation::Volume)
                    || (matches!(dial.rotation, Rotation::None)
                        && matches!(dial.press, Action::MuteToggle | Action::PushToTalk { .. })))
                {
                    continue;
                }
                let target = dial.audio_target.as_deref().unwrap_or("system");
                let state = self.dial_audio(dial, audio);
                let level = state.and_then(|_| self.levels.get(target).copied().flatten());
                let mut panel = vec![0u8; 200 * 100 * 3];
                for y in 72..94 {
                    let start = (y * 800 + index * 200) * 3;
                    panel[y * 600..(y + 1) * 600].copy_from_slice(&strip[start..start + 600]);
                }
                crate::meter::draw(&mut panel, level, state.is_some_and(|s| s.muted));
                for y in 72..94 {
                    let start = (y * 800 + index * 200) * 3;
                    strip[start..start + 600].copy_from_slice(&panel[y * 600..(y + 1) * 600]);
                }
            }
        }
        for index in 0..4 {
            if let Some(notice) = self
                .device_feedback()
                .iter()
                .find(|n| usize::from(n.check.slot) == index + 8)
            {
                let mut panel = vec![0; 200 * 100 * 3];
                for y in 0..100 {
                    let start = (y * 800 + index * 200) * 3;
                    panel[y * 600..(y + 1) * 600].copy_from_slice(&strip[start..start + 600]);
                }
                let color = if notice.status == "failed" {
                    [255, 85, 75]
                } else {
                    [255, 190, 65]
                };
                for y in 34..100 {
                    for x in 0..200 {
                        panel[(y * 200 + x) * 3..(y * 200 + x) * 3 + 3]
                            .copy_from_slice(&[24, 27, 32]);
                    }
                }
                let no_audio =
                    notice.short.eq_ignore_ascii_case("No audio") && notice.status != "failed";
                let mut style =
                    crate::theme::Style::new(self.config.theme.as_ref(), [30, 34, 39], false, true);
                if let Some(dials) = self.effective_dials()
                    && let Some(appearance) = &dials[index].appearance
                {
                    style.apply(appearance);
                }
                crate::key_text::draw_strip_styled(
                    &mut panel,
                    if no_audio { "No Audio" } else { &notice.short },
                    1,
                    color,
                    style.typography(),
                );
                crate::key_text::draw_strip_styled(
                    &mut panel,
                    &notice.next_step,
                    2,
                    [240, 240, 240],
                    crate::key_text::Typography {
                        font: 0,
                        scale: 0.65,
                    },
                );
                if !no_audio {
                    for y in 0..100 {
                        for x in 0..200 {
                            if !(2..198).contains(&x) || !(2..98).contains(&y) {
                                panel[(y * 200 + x) * 3..(y * 200 + x) * 3 + 3]
                                    .copy_from_slice(&color);
                            }
                        }
                    }
                }
                for y in 0..100 {
                    let start = (y * 800 + index * 200) * 3;
                    strip[start..start + 600].copy_from_slice(&panel[y * 600..(y + 1) * 600]);
                }
            }
        }
        strip
    }
    // Resolve one consistent state snapshot for captions and signal availability.
    fn dial_audio(
        &self,
        dial: &Dial,
        default: Option<crate::audio::State>,
    ) -> Option<crate::audio::State> {
        if let Some(target) = &dial.audio_target {
            self.target_states
                .iter()
                .find(|(t, _)| t == target)
                .and_then(|(_, state)| *state)
        } else {
            default
        }
    }
    fn dial_strip_base(&self, audio: Option<crate::audio::State>) -> Vec<u8> {
        let mut strip = [30, 34, 39].repeat(800 * 100);
        if let Some(dials) = &self.effective_dials() {
            for (index, dial) in dials.iter().enumerate() {
                let audio = self.dial_audio(dial, audio);
                let value = if let Some(binding) = &dial.plugin_rotation {
                    if binding.schema == 2 {
                        crate::plugin_runtime::caption(binding, true)
                            .unwrap_or("Unavailable".into())
                    } else {
                        dial.plugin_rotation
                            .as_ref()
                            .filter(|binding| binding.supported(true))
                            .and_then(|_| crate::plugin_runtime::state())
                            .filter(|s| s.available)
                            .map(|s| format!("{}%{}", s.brightness, if s.on { "" } else { " Off" }))
                            .unwrap_or("Unavailable".into())
                    }
                } else {
                    match dial.rotation {
                        Rotation::Volume => audio
                            .map(|state| {
                                if state.muted {
                                    "Muted".into()
                                } else {
                                    if dial
                                        .audio_target
                                        .as_ref()
                                        .is_some_and(|t| t.starts_with("input:"))
                                    {
                                        format!("{}% Live", state.percent)
                                    } else {
                                        format!("{}%", state.percent)
                                    }
                                }
                            })
                            .unwrap_or("--".into()),
                        Rotation::Brightness => self
                            .brightness
                            .map(|n| format!("{n}%"))
                            .unwrap_or("--".into()),
                        Rotation::None => match dial.press {
                            Action::NextPage | Action::PreviousPage => {
                                format!("Page {}", self.index + 1)
                            }
                            Action::MuteToggle | Action::PushToTalk { .. } => audio
                                .map(|s| if s.muted { "Muted" } else { "Live" }.into())
                                .unwrap_or("--".into()),
                            _ => "--".into(),
                        },
                    }
                };
                let mut style =
                    crate::theme::Style::new(self.config.theme.as_ref(), [30, 34, 39], false, true);
                if let Some(appearance) = &dial.appearance {
                    style.apply(appearance);
                }
                let mut panel = style.background.repeat(200 * 100);
                let audio_control = matches!(dial.rotation, Rotation::Volume)
                    || (matches!(dial.rotation, Rotation::None)
                        && matches!(dial.press, Action::MuteToggle | Action::PushToTalk { .. }));
                crate::key_text::draw_strip_title_styled(
                    &mut panel,
                    &dial.label,
                    style.color,
                    style.typography(),
                );
                crate::key_text::draw_strip_styled(
                    &mut panel,
                    &value,
                    1,
                    if audio_control && audio.is_some_and(|state| state.muted) {
                        [255, 75, 85]
                    } else if audio_control
                        && value.ends_with("Live")
                        && (dial.audio_target.as_deref().is_some_and(|target| {
                            target.starts_with("input:") || target == "microphone"
                        }) || matches!(dial.press, Action::PushToTalk { .. }))
                    {
                        [55, 210, 115]
                    } else {
                        style.accent
                    },
                    style.typography(),
                );
                let level = match dial.rotation {
                    Rotation::Volume => audio.map(|state| state.percent.min(100) as u8),
                    Rotation::Brightness => self.brightness,
                    Rotation::None => None,
                };
                if !matches!(dial.rotation, Rotation::None) {
                    crate::level_bar::draw(
                        &mut panel,
                        level,
                        matches!(dial.rotation, Rotation::Volume)
                            && audio.is_some_and(|state| state.muted),
                    );
                }
                if dial
                    .audio_target
                    .as_ref()
                    .is_some_and(|t| t.starts_with("output:") || t.starts_with("input:"))
                {
                    let color = match audio.and_then(|s| s.active) {
                        Some(true) => [55, 210, 115],
                        Some(false) => [100, 105, 115],
                        None => [230, 165, 55],
                    };
                    for y in 94..97 {
                        for x in 90..110 {
                            let i = (y * 200 + x) * 3;
                            panel[i..i + 3].copy_from_slice(&color);
                        }
                    }
                }
                if audio_control {
                    use crate::audio_icon::Kind;
                    let target = dial.audio_target.as_deref().unwrap_or("system");
                    let fallback = if target.starts_with("app:") {
                        Kind::Application
                    } else if target == "microphone"
                        || (target.starts_with("input:") && !target.ends_with(".monitor"))
                    {
                        Kind::Microphone
                    } else {
                        Kind::Unknown
                    };
                    let kind = audio.map(|state| state.icon).unwrap_or(fallback);
                    crate::audio_icon::draw_styled(
                        &mut panel,
                        kind,
                        audio.is_some_and(|state| state.muted),
                        audio.is_some(),
                        style.color,
                        style.background,
                        if target.starts_with("app:") {
                            dial.target_icon_rgba.as_ref().map(|v| v.as_slice())
                        } else {
                            None
                        },
                    );
                }
                if matches!(dial.rotation, Rotation::Brightness) {
                    crate::audio_icon::draw_styled(
                        &mut panel,
                        crate::audio_icon::Kind::Brightness,
                        false,
                        self.brightness.is_some(),
                        style.color,
                        style.background,
                        None,
                    );
                }
                for y in 0..100 {
                    let offset = (y * 800 + index * 200) * 3;
                    strip[offset..offset + 600].copy_from_slice(&panel[y * 600..(y + 1) * 600]);
                }
                if index > 0 {
                    for y in 0..100 {
                        let offset = (y * 800 + index * 200) * 3;
                        strip[offset..offset + 3].copy_from_slice(&[70, 75, 80]);
                    }
                }
            }
        }
        strip
    }
    pub fn update_brightness(
        &mut self,
        deck: &mut impl DeckDevice,
        value: Option<u8>,
    ) -> Result<bool, DeviceError> {
        if self.brightness == value {
            return Ok(false);
        }
        self.brightness = value;
        if self.effective_dials().is_some() {
            deck.set_touch_image(&self.touch_image(self.audio_shown))?;
            return Ok(true);
        }
        Ok(false)
    }
    fn key_image(&self, page: &Page, key: &Key) -> Vec<u8> {
        let live_label = if let Some(binding) = &key.plugin {
            if binding.schema == 2 {
                Some(format!(
                    "{} {}",
                    key.label,
                    crate::plugin_runtime::caption(binding, false).unwrap_or("Unavailable".into())
                ))
            } else {
                None
            }
        } else if let Action::System { command } = &key.action {
            if command == "power_cycle" && key.label == "Power Mode" {
                self.system
                    .get(command)
                    .filter(|s| s.available)
                    .map(|s| format!("Power {}", s.text))
            } else {
                None
            }
        } else {
            None
        };
        let label = live_label.as_deref().unwrap_or(&key.label);
        let has_artwork = match key.artwork {
            Some(Artwork::ApplicationIcon) => key.icon_rgba.is_some(),
            Some(Artwork::MakersMark) => self.logo.is_some(),
            None => false,
        };
        let mut style = crate::theme::Style::new(
            self.config.theme.as_ref(),
            page.background,
            has_artwork,
            false,
        );
        if matches!(key.action, Action::System { .. }) {
            // A restrained default, before user overrides, shared by hardware and preview.
            if self
                .config
                .theme
                .as_ref()
                .and_then(|t| t.appearance.icon_size)
                .is_none()
            {
                style.icon_size = Some(75);
            }
            if self.config.theme.is_none() {
                style.position = LabelPosition::Bottom;
            }
        }
        style.apply(&crate::theme::Appearance {
            label_position: key.label_position,
            label_color: key.label_color,
            background_color: key.background_color,
            label_background: key.label_background,
            ..Default::default()
        });
        if let Some(appearance) = &key.appearance {
            style.apply(appearance);
        }
        let mut rgb = style.background.repeat(120 * 120);
        let source = match key.artwork {
            Some(Artwork::ApplicationIcon) => key
                .icon_rgba
                .as_ref()
                .and_then(|pixels| image::RgbaImage::from_raw(120, 120, pixels.clone())),
            Some(Artwork::MakersMark) => self.logo.as_ref().map(|pixels| {
                image::RgbaImage::from_fn(120, 120, |x, y| {
                    let i = ((y * 120 + x) * 3) as usize;
                    image::Rgba([pixels[i], pixels[i + 1], pixels[i + 2], 255])
                })
            }),
            None => None,
        };
        if let Some(mut icon) = source {
            let library = key
                .icon_source
                .as_ref()
                .is_some_and(|s| s.starts_with("tabler:") || s.starts_with("local:"));
            let edge_label = matches!(style.position, LabelPosition::Top | LabelPosition::Bottom)
                && !label.trim().is_empty();
            if let Some(percent) = style.icon_size.or((library || edge_label).then_some(100)) {
                // Fit to the free icon area, using the actual rendered label bounds.
                let mut top = 15u32;
                let mut bottom = 105u32;
                if style.position != LabelPosition::Hidden && !label.trim().is_empty() {
                    let mut mask = vec![255; 120 * 120 * 3];
                    let anchor = match style.position {
                        LabelPosition::Top => 0,
                        LabelPosition::Bottom => 2,
                        _ => 1,
                    };
                    crate::key_text::draw_styled(
                        &mut mask,
                        label,
                        anchor,
                        true,
                        style.backing,
                        [0; 3],
                        style.typography(),
                    );
                    let rows: Vec<_> = (0..120usize)
                        .filter(|y| mask[y * 360..(y + 1) * 360].iter().any(|v| *v != 255))
                        .collect();
                    if let (Some(first), Some(last)) = (rows.first(), rows.last()) {
                        if matches!(style.position, LabelPosition::Top) {
                            top = (*last as u32 + 7).min(111);
                            bottom = 112;
                        } else {
                            top = 8;
                            bottom = (*first as u32).saturating_sub(6).max(9);
                        }
                    }
                }
                let occupied: Vec<_> = icon
                    .enumerate_pixels()
                    .filter(|(_, _, p)| p[3] > 0)
                    .map(|(x, y, _)| (x, y))
                    .collect();
                if !occupied.is_empty() {
                    let left = occupied.iter().map(|p| p.0).min().unwrap();
                    let right = occupied.iter().map(|p| p.0).max().unwrap();
                    let upper = occupied.iter().map(|p| p.1).min().unwrap();
                    let lower = occupied.iter().map(|p| p.1).max().unwrap();
                    let content = image::imageops::crop_imm(
                        &icon,
                        left,
                        upper,
                        right - left + 1,
                        lower - upper + 1,
                    )
                    .to_image();
                    let factor = (90.0 / content.width() as f32)
                        .min((bottom - top) as f32 / content.height() as f32)
                        * f32::from(percent)
                        / 100.0;
                    let w = (content.width() as f32 * factor).round().max(1.0) as u32;
                    let h = (content.height() as f32 * factor).round().max(1.0) as u32;
                    let resized = image::imageops::resize(
                        &content,
                        w,
                        h,
                        image::imageops::FilterType::Lanczos3,
                    );
                    icon = image::RgbaImage::new(120, 120);
                    image::imageops::overlay(
                        &mut icon,
                        &resized,
                        i64::from((120 - w) / 2),
                        i64::from(top + (bottom - top - h) / 2),
                    );
                }
            }
            for (pixel, source) in rgb.chunks_exact_mut(3).zip(icon.pixels()) {
                for c in 0..3 {
                    let fg =
                        if key.icon_tint && matches!(key.artwork, Some(Artwork::ApplicationIcon)) {
                            style.color[c]
                        } else {
                            source[c]
                        };
                    pixel[c] = ((u32::from(fg) * u32::from(source[3])
                        + u32::from(pixel[c]) * (255 - u32::from(source[3]))
                        + 127)
                        / 255) as u8;
                }
            }
        }
        if style.position != LabelPosition::Hidden {
            let anchor = match style.position {
                LabelPosition::Top => 0,
                LabelPosition::Bottom => 2,
                _ => 1,
            };
            crate::key_text::draw_styled(
                &mut rgb,
                label,
                anchor,
                has_artwork,
                style.backing,
                style.color,
                style.typography(),
            );
        }

        if let Action::System { command } = &key.action {
            let state = self.system.get(command);
            let color = match state {
                Some(s) if s.available && s.active => [90, 230, 120],
                Some(s) if s.available => [150, 150, 150],
                _ => [255, 185, 60],
            };
            for y in 5usize..12 {
                for x in 107usize..114 {
                    let i = (y * 120 + x) * 3;
                    rgb[i..i + 3].copy_from_slice(&color);
                }
            }
        }
        if let Action::AudioSelect { target } | Action::PushToTalk { target } = &key.action {
            let state = self
                .target_states
                .iter()
                .find(|(t, _)| t == target)
                .and_then(|(_, state)| *state);
            let indicator = if matches!(key.action, Action::PushToTalk { .. }) {
                state.map(|s| !s.muted)
            } else {
                state.and_then(|s| s.active)
            };
            let color = match indicator {
                Some(true) => [55, 210, 115],
                Some(false) => [100, 105, 115],
                None => [230, 165, 55],
            };
            for y in 114..120 {
                for x in 0..120 {
                    let i = (y * 120 + x) * 3;
                    rgb[i..i + 3].copy_from_slice(&color);
                }
            }
        }
        if let Some((page_index, slot)) =
            self.config.pages.iter().enumerate().find_map(|(p, page)| {
                page.keys
                    .iter()
                    .position(|k| std::ptr::eq(k, key))
                    .map(|i| (p, i))
            })
            && let Some(notice) = self.device_feedback().iter().find(|n| {
                usize::from(n.check.page) == page_index && usize::from(n.check.slot) == slot
            })
        {
            if notice.status == "failed" {
                crate::feedback::key_failure(&mut rgb, notice);
            } else {
                crate::feedback::badge(&mut rgb, false);
            }
        }
        rgb
    }
    pub fn show(&mut self, deck: &mut impl DeckDevice, target: u8) -> Result<(), DeviceError> {
        crate::plugin_runtime::invalidate();
        if usize::from(target) >= self.config.pages.len() {
            return Err(DeviceError::InvalidReport);
        }
        self.health.clear();
        self.last_failure = None;
        let page = self
            .config
            .pages
            .get(usize::from(target))
            .ok_or(DeviceError::InvalidReport)?;
        for (index, key) in page.keys.iter().enumerate() {
            let rgb = self.key_image(page, key);
            deck.set_key_image(index as u8, &rgb)?;
        }
        let previous = self.index;
        self.index = target;
        let result = deck.set_touch_image(&self.touch_image(self.audio_shown));
        if let Err(error) = result {
            self.index = previous;
            return Err(error);
        }
        self.pending = std::array::from_fn(|_| None);
        self.dial_armed = std::array::from_fn(|_| None);
        self.invalidate_audio();
        Ok(())
    }
}
fn glyph(c: u8) -> [u8; 5] {
    match c {
        b'0' => [7, 5, 5, 5, 7],
        b'1' => [2, 6, 2, 2, 7],
        b'2' => [7, 1, 7, 4, 7],
        b'3' => [7, 1, 7, 1, 7],
        b'4' => [5, 5, 7, 1, 1],
        b'5' => [7, 4, 7, 1, 7],
        b'6' => [7, 4, 7, 5, 7],
        b'7' => [7, 1, 1, 1, 1],
        b'8' => [7, 5, 7, 5, 7],
        b'9' => [7, 5, 7, 1, 7],
        b'%' => [5, 1, 2, 4, 5],
        b'A' => [2, 5, 7, 5, 5],
        b'B' => [6, 5, 6, 5, 6],
        b'C' => [3, 4, 4, 4, 3],
        b'D' => [6, 5, 5, 5, 6],
        b'E' => [7, 4, 6, 4, 7],
        b'F' => [7, 4, 6, 4, 4],
        b'G' => [3, 4, 5, 5, 3],
        b'H' => [5, 5, 7, 5, 5],
        b'I' => [7, 2, 2, 2, 7],
        b'J' => [1, 1, 1, 5, 2],
        b'K' => [5, 5, 6, 5, 5],
        b'L' => [4, 4, 4, 4, 7],
        b'M' => [5, 7, 7, 5, 5],
        b'N' => [5, 7, 7, 7, 5],
        b'O' => [2, 5, 5, 5, 2],
        b'P' => [6, 5, 6, 4, 4],
        b'Q' => [2, 5, 5, 3, 1],
        b'R' => [6, 5, 6, 5, 5],
        b'S' => [3, 4, 2, 1, 6],
        b'T' => [7, 2, 2, 2, 2],
        b'U' => [5, 5, 5, 5, 7],
        b'V' => [5, 5, 5, 5, 2],
        b'W' => [5, 5, 7, 7, 5],
        b'X' => [5, 5, 2, 5, 5],
        b'Y' => [5, 5, 2, 2, 2],
        b'Z' => [7, 1, 2, 4, 7],
        b'a' => [0, 3, 5, 7, 5],
        b'b' => [4, 4, 6, 5, 6],
        b'c' => [0, 0, 3, 4, 3],
        b'd' => [1, 1, 3, 5, 3],
        b'e' => [0, 2, 5, 6, 3],
        b'f' => [1, 2, 7, 2, 2],
        b'g' => [0, 3, 5, 3, 6],
        b'h' => [4, 4, 6, 5, 5],
        b'i' => [2, 0, 2, 2, 2],
        b'j' => [1, 0, 1, 5, 2],
        b'k' => [4, 4, 5, 6, 5],
        b'l' => [6, 2, 2, 2, 3],
        b'm' => [0, 0, 7, 7, 5],
        b'n' => [0, 0, 6, 5, 5],
        b'o' => [0, 0, 2, 5, 2],
        b'p' => [0, 6, 5, 6, 4],
        b'q' => [0, 3, 5, 3, 1],
        b'r' => [0, 0, 5, 6, 4],
        b's' => [0, 3, 4, 2, 6],
        b't' => [2, 7, 2, 2, 1],
        b'u' => [0, 0, 5, 5, 3],
        b'v' => [0, 0, 5, 5, 2],
        b'w' => [0, 0, 5, 7, 7],
        b'x' => [0, 0, 5, 2, 5],
        b'y' => [0, 5, 5, 3, 6],
        b'z' => [0, 0, 7, 2, 7],
        _ => [0; 5],
    }
}
#[cfg(test)]
fn draw_key_label(
    rgb: &mut [u8],
    label: &str,
    position: LabelPosition,
    artwork: bool,
    backing: bool,
    color: [u8; 3],
) {
    if position == LabelPosition::Hidden {
        return;
    }
    let anchor = match position {
        LabelPosition::Top => 0,
        LabelPosition::Bottom => 2,
        _ => 1,
    };
    crate::key_text::draw(rgb, label, anchor, artwork, backing, color);
}
fn frame(width: usize, height: usize, label: &str, bg: [u8; 3]) -> Vec<u8> {
    let mut rgb = bg.repeat(width * height);
    let scale = ((width - 16) / (label.len() * 4 - 1)).min(10);
    let left = (width - (label.len() * 4 - 1) * scale) / 2;
    let top = (height - 5 * scale) / 2;
    for (letter, c) in label.bytes().enumerate() {
        for (row, bits) in glyph(c).iter().enumerate() {
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    for y in top + row * scale..top + (row + 1) * scale {
                        for x in
                            left + (letter * 4 + col) * scale..left + (letter * 4 + col + 1) * scale
                        {
                            rgb[(y * width + x) * 3..(y * width + x) * 3 + 3]
                                .copy_from_slice(&[240, 230, 215]);
                        }
                    }
                }
            }
        }
    }
    rgb
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_plugin_settings_survive_roundtrip_without_execution() {
        let mut raw: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/navigation.json")).unwrap();
        let binding = serde_json::json!({"provider":"missing.provider","action":"future.action","schema":27,"settings":{"custom":[true,"preserve",{"level":39}]}});
        raw["pages"][0]["keys"][0]["action"] = serde_json::json!({"type":"none"});
        raw["pages"][0]["keys"][0]["plugin"] = binding.clone();
        raw["pages"][0]["dial_overrides"] = serde_json::json!([
            {"label":"My Dial","rotation":"none","step":5,"press":{"type":"none"},"plugin_rotation":binding,"plugin_press":binding},null,null,null]);
        let mut pages = Pages::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        let saved: serde_json::Value = serde_json::from_str(&pages.json()).unwrap();
        assert_eq!(
            saved["pages"][0]["keys"][0]["plugin"],
            raw["pages"][0]["keys"][0]["plugin"]
        );
        assert_eq!(
            saved["pages"][0]["dial_overrides"],
            raw["pages"][0]["dial_overrides"]
        );
        assert!(pages.test_target(0).is_none());
        assert!(pages.test_target(8).is_none());
        assert!(
            pages
                .target(&RawEvent::DialRotate { index: 0, ticks: 5 })
                .is_none()
        );
        raw["pages"][0]["keys"][0]["action"] = serde_json::json!({"type":"volume_up"});
        assert!(Pages::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
    }

    #[test]
    fn plugin_keys_fire_on_key_release_and_do_not_arm_from_dial_push() {
        let mut raw: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/navigation.json")).unwrap();
        raw["pages"][0]["keys"][0]["action"] = serde_json::json!({"type":"none"});
        raw["pages"][0]["keys"][0]["plugin"] = serde_json::json!({
            "provider":"com.infamous-pattern.openhomeb", "action":"com.infamous-pattern.openhomeb.set", "schema":1,
            "settings":{"accessoryId":"b3d109c968d17f5cc965ddfa89a1fa695a2d60832200b819ad08127ee15634b4", "characteristicType":"On", "targetValue":true}
        });
        let mut pages = Pages::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert!(matches!(
            pages.test_target(0),
            Some(Action::Plugin { ticks: 0, .. })
        ));
        assert!(
            pages
                .target(&RawEvent::Key {
                    index: 0,
                    pressed: false
                })
                .is_none()
        );
        pages.target(&RawEvent::DialPush {
            index: 0,
            pressed: true,
        });
        pages.target(&RawEvent::DialPush {
            index: 0,
            pressed: false,
        });
        assert!(
            pages
                .target(&RawEvent::Key {
                    index: 0,
                    pressed: false
                })
                .is_none()
        );
        assert!(
            pages
                .target(&RawEvent::Key {
                    index: 0,
                    pressed: true
                })
                .is_none()
        );
        assert!(matches!(
            pages.target(&RawEvent::Key {
                index: 0,
                pressed: false
            }),
            Some(Action::Plugin { ticks: 0, .. })
        ));
        assert!(
            pages
                .target(&RawEvent::Key {
                    index: 0,
                    pressed: false
                })
                .is_none()
        );
    }

    #[test]
    fn oversized_plugin_settings_are_rejected() {
        let mut binding = crate::plugin_binding::Binding {
            provider: "p".into(),
            action: "a".into(),
            schema: 1,
            settings: serde_json::json!({"text":"x".repeat(4096)}),
        };
        assert!(binding.validate().is_err());
        binding.settings = serde_json::json!({"safe":true});
        assert!(binding.validate().is_ok());
        binding.provider = "../other".into();
        assert!(binding.validate().is_err());
    }
    #[test]
    fn explicit_default_migrates_roundtrips_and_routes_unassigned_apps() {
        let mut raw: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/navigation.json")).unwrap();
        let legacy = Pages::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(legacy.default_page(), 0);
        let normalized: serde_json::Value = serde_json::from_str(&legacy.json()).unwrap();
        assert_eq!(normalized["pages"][0]["default"], true);
        raw["pages"][1]["default"] = true.into();
        raw["pages"][0]["application"] = "joplin.desktop".into();
        let pages = Pages::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(pages.default_page(), 1);
        assert_eq!(pages.index, 1);
        assert_eq!(pages.application_page("thunderbird.desktop"), Some(1));
        assert_eq!(pages.application_page("joplin.desktop"), Some(0));
        assert_eq!(
            Pages::parse(pages.json().as_bytes())
                .unwrap()
                .default_page(),
            1
        );
        raw["pages"][0]["default"] = true.into();
        assert!(Pages::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    #[test]
    fn application_assignments_validate_and_roundtrip() {
        let mut raw: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/navigation.json")).unwrap();
        raw["pages"][0]["application"] = "brave-browser.desktop".into();
        let p = Pages::parse(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(p.application_page("brave-browser.desktop"), Some(0));
        assert_eq!(p.application_page("thunderbird.desktop"), Some(0));
        assert_eq!(p.application_page(crate::foreground::STUDIO), None);
        assert_eq!(
            Pages::parse(p.json().as_bytes())
                .unwrap()
                .application_page("brave-browser.desktop"),
            Some(0)
        );
        raw["pages"][1]["application"] = "brave-browser.desktop".into();
        assert!(Pages::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
        raw["pages"][1]["application"] = crate::foreground::STUDIO.into();
        assert!(Pages::parse(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    #[test]
    fn feedback_warnings_recover_and_match_device_pixels_without_mutating_layout() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][0]["action"] = serde_json::json!({"type":"audio_mute","target":"app:application.process.binary=brave"});
        value["dials"] = serde_json::json!([
            {"label":"Brave","rotation":"volume","audio_target":"app:application.process.binary=brave","step":1,"press":{"type":"mute_toggle"}},
            {"label":"Wave mic","rotation":"volume","audio_target":"input:wave","step":1,"press":{"type":"mute_toggle"}},
            {"label":"MPZ","rotation":"none","step":1,"media_player":"org.mpris.MediaPlayer2.mpz","press":{"type":"media_next"}},
            {"label":"Brightness","rotation":"brightness","step":1,"press":{"type":"none"}}
        ]);
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let original = pages.json();
        let mut deck = decksmith_device::VirtualDeck::default();
        pages.show(&mut deck, 0).unwrap();
        let key_before = deck.key_image(0).unwrap().to_vec();
        let strip_before = deck.touch_image().to_vec();
        let notices: Vec<_> = pages
            .health_checks()
            .into_iter()
            .filter(|c| matches!(c.slot, 0 | 8 | 9 | 10))
            .map(|check| crate::feedback::Notice {
                target_name: check.label.clone(),
                status: if check.slot == 10 {
                    "unsupported"
                } else {
                    "missing"
                }
                .into(),
                detail: "Target unavailable".into(),
                hint: "Reconnect or start playback".into(),
                short: if check.slot == 10 {
                    "Unsupported"
                } else if check.slot == 9 {
                    "Missing"
                } else {
                    "No audio"
                }
                .into(),
                next_step: if check.slot == 10 {
                    "Check player"
                } else if check.slot == 9 {
                    "Connect mic"
                } else {
                    "Start app"
                }
                .into(),
                check,
            })
            .collect();
        pages.update_health(&mut deck, notices.clone()).unwrap();
        assert_ne!(key_before, deck.key_image(0).unwrap());
        assert_ne!(strip_before, deck.touch_image());
        let edge = (20 * 800 + 1) * 3;
        assert_eq!(
            &deck.touch_image()[edge..edge + 3],
            &strip_before[edge..edge + 3]
        );
        let missing_edge = (20 * 800 + 201) * 3;
        assert_eq!(
            &deck.touch_image()[missing_edge..missing_edge + 3],
            &[255, 190, 65]
        );
        let mut preview = Pages::parse(original.as_bytes()).unwrap();
        let key_preview = preview.preview_keys(0, pages.touch_state()).unwrap();
        assert_eq!(&key_preview[..43200], deck.key_image(0).unwrap());
        assert_eq!(
            preview.preview_touch(0, pages.touch_state()).unwrap(),
            deck.touch_image()
        );
        if let Ok(path) = std::env::var("DECKSMITH_FEEDBACK_ARTIFACTS") {
            image::RgbImage::from_raw(120, 120, deck.key_image(0).unwrap().to_vec())
                .unwrap()
                .save(format!("{path}/warning-key.png"))
                .unwrap();
            image::RgbImage::from_raw(800, 100, deck.touch_image().to_vec())
                .unwrap()
                .save(format!("{path}/warning-strip.png"))
                .unwrap();
            std::fs::write(
                format!("{path}/notices.json"),
                serde_json::to_vec(&notices).unwrap(),
            )
            .unwrap();
        }
        pages.update_health(&mut deck, vec![]).unwrap();
        assert_eq!(key_before, deck.key_image(0).unwrap());
        assert_eq!(strip_before, deck.touch_image());
        assert_eq!(original, pages.json());
        pages.config.dials.as_mut().unwrap()[2].press = Action::None;
        let stale = Action::MediaTarget {
            player: "org.mpris.MediaPlayer2.mpz".into(),
            command: "next".into(),
        };
        let dial_input = decksmith_core::InputEvent {
            timestamp_ms: 0,
            event: RawEvent::DialPush {
                index: 2,
                pressed: false,
            },
        };
        pages
            .action_feedback(&mut deck, 0, &stale, &dial_input, Some("launch_failed"))
            .unwrap();
        assert!(pages.attention().is_empty());
        let input = decksmith_core::InputEvent {
            timestamp_ms: 0,
            event: RawEvent::Key {
                index: 0,
                pressed: false,
            },
        };
        let action = Action::AudioMute {
            target: "app:application.process.binary=brave".into(),
        };
        pages
            .action_feedback(&mut deck, 1, &action, &input, Some("audio_target_timeout"))
            .unwrap();
        assert!(pages.attention().is_empty());
        pages
            .action_feedback(
                &mut deck,
                0,
                &Action::MediaNext,
                &input,
                Some("launch_failed"),
            )
            .unwrap();
        assert!(pages.attention().is_empty());
        pages
            .action_feedback(&mut deck, 0, &action, &input, Some("audio_target_timeout"))
            .unwrap();
        assert_eq!(pages.attention()[0].status, "failed");
        if let Ok(path) = std::env::var("DECKSMITH_FEEDBACK_ARTIFACTS") {
            image::RgbImage::from_raw(120, 120, deck.key_image(0).unwrap().to_vec())
                .unwrap()
                .save(format!("{path}/failed-key.png"))
                .unwrap();
        }
        assert!(pages.attention()[0].detail.contains("unknown"));
        pages.last_failure.as_mut().unwrap().1 =
            std::time::Instant::now() - std::time::Duration::from_secs(6);
        pages.expire_feedback(&mut deck).unwrap();
        assert_eq!(key_before, deck.key_image(0).unwrap());
        assert_eq!(pages.attention().len(), 1);
        pages
            .action_feedback(&mut deck, 0, &action, &input, None)
            .unwrap();
        assert!(pages.attention().is_empty());
    }
    #[test]
    fn theme_presets_fonts_and_overrides_share_device_pixels_without_action_changes() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        for page in value["pages"].as_array_mut().unwrap() {
            for key in page["keys"].as_array_mut().unwrap() {
                key.as_object_mut().unwrap().remove("artwork");
            }
        }
        value["dials"] = serde_json::json!([
            {"label":"Output","rotation":"volume","step":2,"press":{"type":"mute_toggle"}},
            {"label":"Microphone","rotation":"volume","step":1,"audio_target":"input:mic","press":{"type":"mute_toggle"}},
            {"label":"Apps","rotation":"volume","step":1,"press":{"type":"none"}},
            {"label":"Brightness","rotation":"brightness","step":1,"press":{"type":"none"}}
        ]);
        let actions: Vec<_> = value["pages"][0]["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| k["action"].clone())
            .collect();
        let mut images = Vec::new();
        for preset in ["makers_mark", "dark", "light", "high_contrast"] {
            value["theme"] =
                serde_json::json!({"preset":preset,"appearance":{"font":"serif","size":"large"}});
            let bytes = serde_json::to_vec(&value).unwrap();
            let mut hardware = Pages::parse(&bytes).unwrap();
            let mut deck = decksmith_device::VirtualDeck::default();
            hardware.show(&mut deck, 0).unwrap();
            let mut preview = Pages::parse(&bytes).unwrap();
            let keys = preview.preview_keys(0, hardware.touch_state()).unwrap();
            for i in 0..8 {
                assert_eq!(
                    &keys[i * 43200..(i + 1) * 43200],
                    deck.key_image(i as u8).unwrap()
                );
            }
            assert_eq!(
                preview.preview_touch(0, hardware.touch_state()).unwrap(),
                deck.touch_image()
            );
            let after: serde_json::Value = serde_json::from_str(&hardware.json()).unwrap();
            assert_eq!(
                actions,
                after["pages"][0]["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|k| k["action"].clone())
                    .collect::<Vec<_>>()
            );
            images.push(keys);
        }
        for pair in images.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
        value["pages"][0]["keys"][0]["appearance"] =
            serde_json::json!({"background_color":"red","label_position":"hidden"});
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            &pages.preview_keys(0, TouchState::default()).unwrap()[..3],
            &[255, 70, 70]
        );
        value["pages"][0]["keys"][0]
            .as_object_mut()
            .unwrap()
            .remove("appearance");
        let mut fonts = Vec::new();
        for font in [
            "sans",
            "serif",
            "mono",
            "roboto",
            "open_sans",
            "lato",
            "montserrat",
            "oswald",
            "raleway",
            "poppins",
            "nunito",
            "merriweather",
            "source_sans3",
            "viking_runes",
        ] {
            value["theme"]["appearance"]["font"] = font.into();
            fonts.push(
                Pages::parse(&serde_json::to_vec(&value).unwrap())
                    .unwrap()
                    .preview_keys(0, TouchState::default())
                    .unwrap(),
            );
        }
        assert_ne!(fonts[0], fonts[1]);
        assert_ne!(fonts[1], fonts[2]);
        value["theme"]["appearance"]["action"] = serde_json::json!({"type":"mute_toggle"});
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn transparent_tinted_icons_follow_theme_foreground() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        let image = image::RgbaImage::from_pixel(120, 120, image::Rgba([255, 255, 255, 128]));
        let mut out = std::io::Cursor::new(Vec::new());
        image.write_to(&mut out, image::ImageFormat::Png).unwrap();
        value["theme"] = serde_json::json!({"preset":"light","appearance":{}});
        let key = &mut value["pages"][0]["keys"][0];
        key["artwork"] = "application_icon".into();
        key["icon_png"] = serde_json::json!(out.into_inner());
        key["icon_tint"] = true.into();
        key["label_position"] = "hidden".into();
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let pixels = pages.preview_keys(0, TouchState::default()).unwrap();
        for (i, (fg, bg)) in [24u32, 30, 38]
            .into_iter()
            .zip([236u32, 240, 245])
            .enumerate()
        {
            assert_eq!(
                u32::from(pixels[(60 * 120 + 60) * 3 + i]),
                (fg * 128 + bg * 127 + 127) / 255
            );
        }
    }
    #[test]
    fn library_icons_have_inset_and_match_device_without_mutating_sources() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        let image = image::RgbaImage::from_pixel(120, 120, image::Rgba([255, 255, 255, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        image.write_to(&mut out, image::ImageFormat::Png).unwrap();
        let key = &mut value["pages"][0]["keys"][0];
        key["artwork"] = "application_icon".into();
        key["icon_png"] = serde_json::json!(out.into_inner());
        key["icon_source"] = "tabler:test".into();
        key["label_position"] = "hidden".into();
        key["background_color"] = "black".into();
        let original = value["pages"][0]["keys"][0].clone();
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut deck = decksmith_device::VirtualDeck::default();
        pages.show(&mut deck, 0).unwrap();
        let pixels = pages.preview_keys(0, pages.touch_state()).unwrap();
        assert_eq!(&pixels[..43200], deck.key_image(0).unwrap());
        assert_eq!(&pixels[..3], &[0, 0, 0]);
        let center = (60 * 120 + 60) * 3;
        assert_eq!(&pixels[center..center + 3], &[255, 255, 255]);
        let after: serde_json::Value = serde_json::from_str(&pages.json()).unwrap();
        assert_eq!(
            original["icon_png"],
            after["pages"][0]["keys"][0]["icon_png"]
        );
        assert_eq!(original["action"], after["pages"][0]["keys"][0]["action"]);
    }
    #[test]
    fn system_actions_validate_and_live_state_matches_preview() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][0]["action"] =
            serde_json::json!({"type":"system","command":"dnd"});
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(pages.system_commands(), vec!["dnd"]);
        assert!(matches!(pages.test_target(0),Some(Action::System{command}) if command=="dnd"));
        let mut deck = decksmith_device::VirtualDeck::default();
        pages.show(&mut deck, 0).unwrap();
        let before = deck.key_image(0).unwrap().to_vec();
        pages
            .apply_system(
                &mut deck,
                [(
                    "dnd".into(),
                    crate::system_actions::State {
                        available: true,
                        active: true,
                        text: "On".into(),
                    },
                )]
                .into(),
            )
            .unwrap();
        assert_ne!(before, deck.key_image(0).unwrap());
        let rendered = pages.preview_keys(0, pages.touch_state()).unwrap();
        assert_eq!(&rendered[..43200], deck.key_image(0).unwrap());
        value["pages"][0]["keys"][0]["action"]["command"] = "execute_shell".into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn system_artwork_default_size_and_theme_match_explicit_rendering() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["theme"] = serde_json::json!({"preset":"light","appearance":{}});
        let source = image::RgbaImage::from_pixel(120, 120, image::Rgba([255, 0, 0, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        source.write_to(&mut png, image::ImageFormat::Png).unwrap();
        value["pages"][0]["keys"][0] = serde_json::json!({
            "label":"Lock", "action":{"type":"system","command":"lock"},
            "artwork":"application_icon", "icon_png":png.into_inner(), "icon_tint":true
        });
        let render = |value: &serde_json::Value| {
            let pages = Pages::parse(&serde_json::to_vec(value).unwrap()).unwrap();
            pages.key_image(&pages.config.pages[0], &pages.config.pages[0].keys[0])
        };
        let inherited = render(&value);
        assert_eq!(&inherited[..3], &[236, 240, 245]);
        value["pages"][0]["keys"][0]["appearance"] = serde_json::json!({"icon_size":75});
        assert_eq!(inherited, render(&value));
        value["pages"][0]["keys"][0]["appearance"] = serde_json::json!({"icon_size":100});
        assert_ne!(inherited, render(&value));
        value["pages"][0]["keys"][0]
            .as_object_mut()
            .unwrap()
            .remove("appearance");
        value["theme"]["appearance"] = serde_json::json!({"icon_size":60,"label_position":"top"});
        let themed = render(&value);
        value["pages"][0]["keys"][0]["appearance"] =
            serde_json::json!({"icon_size":60,"label_position":"top"});
        assert_eq!(themed, render(&value));
    }
    #[test]
    fn automatic_artwork_fits_edge_labels_without_explicit_icon_size() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        let source = image::RgbaImage::from_pixel(120, 120, image::Rgba([255, 0, 0, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        source.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let key = &mut value["pages"][0]["keys"][0];
        key["icon_png"] = serde_json::json!(png.into_inner());
        key["artwork"] = "application_icon".into();
        key["label_color"] = "white".into();
        key["background_color"] = "black".into();
        for position in ["top", "bottom"] {
            for backing in ["dark", "transparent"] {
                for label in ["Brave", "Volume Control"] {
                    let key = &mut value["pages"][0]["keys"][0];
                    key["label"] = label.into();
                    key["label_position"] = position.into();
                    key["label_background"] = backing.into();
                    let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
                    let mut deck = decksmith_device::VirtualDeck::default();
                    pages.show(&mut deck, 0).unwrap();
                    let pixels = pages.preview_keys(0, pages.touch_state()).unwrap();
                    assert_eq!(&pixels[..43200], deck.key_image(0).unwrap());
                    let rows = |red: bool| -> Vec<usize> {
                        pixels[..43200]
                            .chunks_exact(360)
                            .enumerate()
                            .filter_map(|(y, row)| {
                                row.chunks_exact(3)
                                    .any(|p| {
                                        if red {
                                            p[0] > 200 && p[1] < 10 && p[2] < 10
                                        } else {
                                            p[0] > 100 && p[1] > 100 && p[2] > 100
                                        }
                                    })
                                    .then_some(y)
                            })
                            .collect()
                    };
                    let art = rows(true);
                    let text = rows(false);
                    assert!(!art.is_empty() && !text.is_empty());
                    if position == "top" {
                        assert!(art[0] >= text.last().unwrap() + 7);
                    } else {
                        assert!(text[0] >= art.last().unwrap() + 7);
                    }
                }
            }
        }
    }
    #[test]
    fn icon_size_steps_fit_label_room_and_preserve_device_preview_parity() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        let source = image::RgbaImage::from_pixel(120, 120, image::Rgba([255, 0, 0, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        source.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let key = &mut value["pages"][0]["keys"][0];
        key["icon_png"] = serde_json::json!(png.into_inner());
        key["artwork"] = "application_icon".into();
        key["label"] = "Volume Control".into();
        key["label_color"] = "white".into();
        key["background_color"] = "black".into();
        key["label_background"] = "transparent".into();
        for position in ["top", "middle", "bottom", "hidden"] {
            let mut prior = 0;
            for size in (10..=100).step_by(5) {
                value["pages"][0]["keys"][0]["appearance"] =
                    serde_json::json!({"icon_size":size,"label_position":position,"size":"large"});
                let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
                let mut deck = decksmith_device::VirtualDeck::default();
                pages.show(&mut deck, 0).unwrap();
                let pixels = pages.preview_keys(0, pages.touch_state()).unwrap();
                assert_eq!(&pixels[..43200], deck.key_image(0).unwrap());
                let red = pixels[..43200]
                    .chunks_exact(3)
                    .filter(|p| p[0] > 200 && p[1] < 10 && p[2] < 10)
                    .count();
                assert!(red >= prior);
                prior = red;
                let saved: serde_json::Value = serde_json::from_str(&pages.json()).unwrap();
                assert_eq!(
                    saved["pages"][0]["keys"][0]["appearance"]["icon_size"],
                    size
                );
            }
        }
        for size in [0, 9, 11, 99, 101] {
            value["pages"][0]["keys"][0]["appearance"]["icon_size"] = size.into();
            assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
    #[test]
    fn page_dial_overrides_resolve_dispatch_targets_rendering_and_roundtrip() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["dials"] = serde_json::json!([
            {"label":"Shared volume","rotation":"volume","step":1,"press":{"type":"mute_toggle"}},
            {"label":"Light","rotation":"brightness","step":1,"press":{"type":"none"}},
            {"label":"Dial 3","rotation":"none","step":1,"press":{"type":"none"}},
            {"label":"Dial 4","rotation":"none","step":1,"press":{"type":"none"}}
        ]);
        value["pages"][1]["dial_overrides"] = serde_json::json!([
            {"label":"Page mic","rotation":"volume","step":3,"audio_target":"input:mic","press":{"type":"push_to_talk","target":"input:mic"}},null,null,null]);
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut deck = decksmith_device::VirtualDeck::default();
        let rotate = RawEvent::DialRotate { index: 0, ticks: 1 };
        assert_eq!(
            pages.target(&rotate),
            Some(Action::AudioAdjust {
                target: "system".into(),
                percent: 1
            })
        );
        pages.show(&mut deck, 1).unwrap();
        assert_eq!(
            pages.target(&rotate),
            Some(Action::AudioAdjust {
                target: "input:mic".into(),
                percent: 3
            })
        );
        assert_eq!(pages.ptt_target(8), Some("input:mic".into()));
        assert_eq!(pages.meter_targets(), vec!["input:mic"]);
        assert!(pages.audio_targets().contains(&"input:mic".into()));
        assert_eq!(
            pages.target(&RawEvent::Touch {
                x: 50,
                y: 50,
                gesture: TouchGesture::Tap
            }),
            Some(Action::AudioMute {
                target: "input:mic".into()
            })
        );
        assert_eq!(
            pages.target(&RawEvent::DialRotate { index: 1, ticks: 1 }),
            Some(Action::BrightnessAdjust { percent: 1 })
        );
        let state = crate::audio::State {
            percent: 62,
            muted: false,
            active: Some(true),
            icon: crate::audio_icon::Kind::Microphone,
        };
        pages.apply_targets(vec![("input:mic".into(), Some(state))]);
        pages.apply_audio(&mut deck, Some(state)).unwrap();
        pages
            .apply_levels(
                &mut deck,
                [("input:mic".into(), Some(80))].into_iter().collect(),
            )
            .unwrap();
        let json = pages.json();
        let mut preview = Pages::parse(json.as_bytes()).unwrap();
        assert_eq!(
            preview.preview_touch(1, pages.touch_state()).unwrap(),
            deck.touch_image()
        );
        assert_eq!(preview.json(), json);
        pages.show(&mut deck, 0).unwrap();
        assert_eq!(pages.ptt_target(8), None);
        assert_eq!(
            pages.target(&rotate),
            Some(Action::AudioAdjust {
                target: "system".into(),
                percent: 1
            })
        );
        value["pages"][1]["dial_overrides"][0]["step"] = 0.into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn application_dial_artwork_roundtrips_and_matches_preview_when_muted() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(32, 32, image::Rgba([20, 120, 200, 255]))
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let dial = serde_json::json!({"label":"Browser","rotation":"volume","step":1,"press":{"type":"mute_toggle"},"audio_target":"app:application.process.binary=brave","target_icon_png":png.into_inner()});
        value["dials"] = serde_json::json!([dial.clone(), dial.clone(), dial.clone(), dial]);
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut deck = decksmith_device::VirtualDeck::default();
        for muted in [false, true, false] {
            let state = crate::audio::State {
                percent: 64,
                muted,
                active: None,
                icon: crate::audio_icon::Kind::Application,
            };
            pages.show(&mut deck, 0).unwrap();
            pages.apply_targets(vec![(
                "app:application.process.binary=brave".into(),
                Some(state),
            )]);
            pages.apply_audio(&mut deck, Some(state)).unwrap();
            let mut preview = Pages::parse(pages.json().as_bytes()).unwrap();
            assert_eq!(
                preview.preview_touch(0, pages.touch_state()).unwrap(),
                deck.touch_image()
            );
            let pixel = (10 * 800 + 10) * 3;
            assert_eq!(&deck.touch_image()[pixel..pixel + 3], &[20, 120, 200]);
        }
        value["dials"][0]["target_icon_png"] = serde_json::json!([1, 2, 3]);
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn preview_pixels_match_physical_pipeline_for_live_and_draft_states() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["dials"] = serde_json::json!([
            {"label":"System Volume","rotation":"volume","step":1,"press":{"type":"mute_toggle"}},
            {"label":"Brave Browser","rotation":"volume","step":1,"audio_target":"app:application.process.binary=brave","press":{"type":"mute_toggle"}},
            {"label":"Microphone","rotation":"volume","step":1,"audio_target":"input:mic","press":{"type":"push_to_talk","target":"input:mic"}},
            {"label":"Brightness","rotation":"brightness","step":1,"press":{"type":"next_page"}}
        ]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let mut hardware = Pages::parse(&bytes).unwrap();
        let mut deck = decksmith_device::VirtualDeck::default();
        for (case, audio) in [
            None,
            Some(crate::audio::State {
                percent: 0,
                muted: false,
                active: None,
                icon: Default::default(),
            }),
            Some(crate::audio::State {
                percent: 73,
                muted: true,
                active: Some(false),
                icon: Default::default(),
            }),
            Some(crate::audio::State {
                percent: 100,
                muted: false,
                active: Some(true),
                icon: Default::default(),
            }),
        ]
        .into_iter()
        .enumerate()
        {
            hardware.show(&mut deck, (case % 2) as u8).unwrap();
            hardware.apply_targets(vec![
                ("app:application.process.binary=brave".into(), audio),
                ("input:mic".into(), audio),
            ]);
            hardware.update_brightness(&mut deck, Some(83)).unwrap();
            hardware.apply_audio(&mut deck, audio).unwrap();
            hardware
                .apply_levels(
                    &mut deck,
                    [
                        ("system".into(), Some([0, 0, 90, 85][case])),
                        (
                            "app:application.process.binary=brave".into(),
                            Some([0, 0, 60, 45][case]),
                        ),
                        ("input:mic".into(), Some([0, 0, 40, 20][case])),
                    ]
                    .into_iter()
                    .collect(),
                )
                .unwrap();
            let mut draft = Pages::parse(&bytes).unwrap();
            let preview = draft
                .preview_touch(hardware.index, hardware.touch_state())
                .unwrap();
            assert_eq!(
                preview,
                deck.touch_image(),
                "all 240000 RGB bytes must match hardware case {case}"
            );
            if let Ok(dir) = std::env::var("DECKSMITH_TOUCH_PREVIEW_DIR") {
                std::fs::create_dir_all(&dir).unwrap();
                image::RgbImage::from_raw(800, 100, preview)
                    .unwrap()
                    .save(std::path::Path::new(&dir).join(format!("touch-{case}.png")))
                    .unwrap();
            }
        }
        let started = std::time::Instant::now();
        for _ in 0..100 {
            hardware.touch_image(hardware.audio_shown);
        }
        eprintln!("100 cached meter frames: {:?}", started.elapsed());
        assert!(
            started.elapsed() < std::time::Duration::from_millis(500),
            "meter updates must reuse static artwork"
        );
        let sent = deck.touch_image().to_vec();
        value["dials"][0]["label"] = "Unsaved label".into();
        let mut draft = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let snapshot = hardware.touch_state();
        let preview = draft.preview_touch(0, snapshot.clone()).unwrap();
        assert_eq!(
            deck.touch_image(),
            sent,
            "draft render must never write hardware"
        );
        let mut future_hardware = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        future_hardware.show(&mut deck, 0).unwrap();
        future_hardware.apply_targets(snapshot.targets);
        future_hardware
            .apply_levels(&mut deck, snapshot.levels)
            .unwrap();
        future_hardware
            .update_brightness(&mut deck, snapshot.brightness)
            .unwrap();
        future_hardware
            .apply_audio(&mut deck, snapshot.audio.flatten())
            .unwrap();
        assert_eq!(preview, deck.touch_image());
        assert!(draft.preview_touch(255, TouchState::default()).is_err());
        for bytes in [
            include_bytes!("../../../config/navigation.json").as_slice(),
            include_bytes!("../../../config/audio.json").as_slice(),
        ] {
            let mut hardware = Pages::parse(bytes).unwrap();
            hardware.show(&mut deck, 1).unwrap();
            let mut preview = Pages::parse(bytes).unwrap();
            assert_eq!(
                preview.preview_touch(1, hardware.touch_state()).unwrap(),
                deck.touch_image()
            );
            hardware.apply_audio(&mut deck, None).unwrap();
            assert_eq!(
                preview.preview_touch(1, hardware.touch_state()).unwrap(),
                deck.touch_image()
            );
        }
    }

    #[test]
    fn explicit_media_player_dispatch_and_roundtrip() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][0]["action"] = serde_json::json!({"type":"media_play_pause"});
        value["pages"][0]["keys"][0]["media_player"] = "org.mpris.MediaPlayer2.mpz".into();
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            pages.target(&RawEvent::Key {
                index: 0,
                pressed: true
            }),
            None
        );
        assert_eq!(
            pages.target(&RawEvent::Key {
                index: 0,
                pressed: false
            }),
            Some(Action::MediaTarget {
                player: "org.mpris.MediaPlayer2.mpz".into(),
                command: "play_pause".into()
            })
        );
        assert_eq!(
            Pages::parse(pages.json().as_bytes()).unwrap().json(),
            pages.json()
        );
        value["pages"][0]["keys"][0]["media_player"] = "bad;command".into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn ptt_named_mic_validation_and_mirrored_feedback() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][0] = serde_json::json!({"label":"Push to Talk","action":{"type":"push_to_talk","target":"input:mic"}});
        value["dials"] = serde_json::json!([
            {"label":"Microphone","rotation":"volume","step":1,"audio_target":"input:mic","press":{"type":"push_to_talk","target":"input:mic"}},
            {"label":"Dial 2","rotation":"none","step":1,"press":{"type":"none"}},
            {"label":"Dial 3","rotation":"none","step":1,"press":{"type":"none"}},
            {"label":"Dial 4","rotation":"none","step":1,"press":{"type":"none"}}
        ]);
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(pages.ptt_target(0), pages.ptt_target(8));
        for (muted, color) in [(false, [55, 210, 115]), (true, [100, 105, 115])] {
            pages.apply_targets(vec![(
                "input:mic".into(),
                Some(crate::audio::State {
                    percent: 75,
                    muted,
                    active: Some(true),
                    icon: Default::default(),
                }),
            )]);
            let page = &pages.config.pages[0];
            let rgb = pages.key_image(page, &page.keys[0]);
            assert_eq!(&rgb[(119 * 120) * 3..(119 * 120) * 3 + 3], &color);
            if let Some(directory) = std::env::var_os("DECKSMITH_PREVIEW_DIR") {
                let directory = std::path::PathBuf::from(directory);
                let name = if muted { "muted" } else { "live" };
                image::RgbImage::from_raw(120, 120, rgb)
                    .unwrap()
                    .save(directory.join(format!("ptt-key-{name}.png")))
                    .unwrap();
                image::RgbImage::from_raw(800, 100, pages.dial_strip(None))
                    .unwrap()
                    .save(directory.join(format!("ptt-strip-{name}.png")))
                    .unwrap();
            }
        }
        value["dials"][0]["press"]["target"] = "input:other".into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value.as_object_mut().unwrap().remove("dials");
        value["pages"][0]["keys"][0]["action"]["target"] = "microphone".into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn touch_sections_and_device_indicators_follow_targets() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["dials"] = serde_json::json!([
            {"label":"Output", "rotation":"volume", "step":1, "press":{"type":"none"}, "audio_target":"output:speakers"},
            {"label":"Mic", "rotation":"volume", "step":1, "press":{"type":"mute_toggle"}, "audio_target":"input:mic"},
            {"label":"Brightness", "rotation":"brightness", "step":1, "press":{"type":"none"}},
            {"label":"Music", "rotation":"volume", "step":1, "press":{"type":"none"}, "audio_target":"app:application.name=Music"}
        ]);
        value["pages"][0]["keys"][0]["action"] =
            serde_json::json!({"type":"audio_select", "target":"output:speakers"});
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        for (x, target) in [
            (0, "output:speakers"),
            (199, "output:speakers"),
            (200, "input:mic"),
            (399, "input:mic"),
            (600, "app:application.name=Music"),
            (799, "app:application.name=Music"),
        ] {
            assert_eq!(
                pages.target(&RawEvent::Touch {
                    x,
                    y: 50,
                    gesture: TouchGesture::Tap
                }),
                Some(Action::AudioMute {
                    target: target.into()
                })
            );
        }
        for (x, y, gesture) in [
            (400, 50, TouchGesture::Tap),
            (599, 50, TouchGesture::Tap),
            (800, 50, TouchGesture::Tap),
            (0, 100, TouchGesture::Tap),
            (0, 50, TouchGesture::LongPress),
        ] {
            assert_eq!(pages.target(&RawEvent::Touch { x, y, gesture }), None);
        }
        assert!(matches!(
            pages.target(&RawEvent::Touch {
                x: 50,
                y: 50,
                gesture: TouchGesture::FlickLeft
            }),
            Some(Action::GoToPage { .. })
        ));
        for (active, color) in [
            (Some(true), [55, 210, 115]),
            (Some(false), [100, 105, 115]),
            (None, [230, 165, 55]),
        ] {
            pages.apply_targets(vec![(
                "output:speakers".into(),
                Some(crate::audio::State {
                    percent: 50,
                    muted: false,
                    active,
                    icon: Default::default(),
                }),
            )]);
            let page = &pages.config.pages[0];
            let rgb = pages.key_image(page, &page.keys[0]);
            assert_eq!(&rgb[(119 * 120 + 60) * 3..(119 * 120 + 60) * 3 + 3], &color);
            let strip = pages.dial_strip(None);
            if let Some(directory) = std::env::var_os("DECKSMITH_PREVIEW_DIR") {
                let directory = std::path::PathBuf::from(directory);
                let name = match active {
                    Some(true) => "active",
                    Some(false) => "inactive",
                    None => "unknown",
                };
                image::RgbImage::from_raw(120, 120, rgb)
                    .unwrap()
                    .save(directory.join(format!("device-{name}.png")))
                    .unwrap();
                image::RgbImage::from_raw(800, 100, strip.clone())
                    .unwrap()
                    .save(directory.join(format!("strip-{name}.png")))
                    .unwrap();
            }

            // A centered 20 px marker replaces the previous 192 px line.
            for x in 0..200 {
                let pixel = &strip[(95 * 800 + x) * 3..(95 * 800 + x) * 3 + 3];
                assert_eq!(pixel == color, (90..110).contains(&x));
            }
        }
        // Device selection keys alone still request feedback, without audio dials.
        value.as_object_mut().unwrap().remove("dials");
        value["audio_dial"] = false.into();
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(pages.audio_targets(), vec!["output:speakers"]);
        assert!(pages.audio_due());
        assert!(!pages.audio_due());
        pages.audio_checked =
            Some(std::time::Instant::now() - std::time::Duration::from_millis(300));
        assert!(!pages.audio_due());
        pages.invalidate_audio();
        assert!(pages.audio_due());
        pages.audio_checked =
            Some(std::time::Instant::now() - std::time::Duration::from_millis(501));
        assert!(pages.audio_due());
    }
    #[test]
    fn linked_page_label_is_resolved_and_invalid_action_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][1]["name"] = "TUESDAYS".into();
        value["pages"][0]["keys"][1]["follow_page_name"] = true.into();
        let pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let saved: serde_json::Value = serde_json::from_str(&pages.json()).unwrap();
        assert_eq!(saved["pages"][0]["keys"][1]["label"], "TUESDAYS");
        value["pages"][0]["keys"][1]["action"] = serde_json::json!({"type":"none"});
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn page_names_and_labels_accept_digits_and_spaces() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["name"] = "PAGE 1".into();
        value["pages"][0]["keys"][1]["label"] = "PAGE 2".into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_ok());
    }
    #[test]
    fn mixed_case_page_names_roundtrip_with_linked_labels_and_render() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/navigation.json")).unwrap();
        value["pages"][1]["name"] = "Joplin Notes 2".into();
        value["pages"][0]["keys"][1]["follow_page_name"] = true.into();
        let pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let restored = Pages::parse(pages.json().as_bytes()).unwrap();
        assert_eq!(restored.config.pages[1].name, "Joplin Notes 2");
        assert_eq!(restored.config.pages[0].keys[1].label, "Joplin Notes 2");
        assert_eq!(restored.config.pages[0].name, "HOME");
        assert_ne!(
            frame(800, 100, "Joplin Notes 2", [0, 0, 0]),
            frame(800, 100, "JOPLIN NOTES 2", [0, 0, 0])
        );
        for name in ["", "   ", "Bad!", "1234567890123456789012345"] {
            value["pages"][1]["name"] = name.into();
            assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
    #[test]
    fn launch_config_and_icon_roundtrip() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][5]["action"] =
            serde_json::json!({"type":"open_website","url":"https://example.com/"});
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(120, 120)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        value["pages"][0]["keys"][5]["icon_png"] = serde_json::json!(bytes.into_inner());
        value["pages"][0]["keys"][5]["artwork"] = "application_icon".into();
        let pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            pages.config.pages[0].keys[5]
                .icon_rgba
                .as_ref()
                .unwrap()
                .len(),
            57600
        );
        assert_eq!(
            Pages::parse(pages.json().as_bytes()).unwrap().json(),
            pages.json()
        );
        value["pages"][0]["keys"][5]["action"] =
            serde_json::json!({"type":"open_application","desktop_id":"/tmp/evil.desktop"});
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn targeted_dial_dispatch_and_levels_are_independent() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["dials"] = serde_json::json!([
            {"label":"Music","rotation":"volume","step":2,"audio_target":"app:application.process.binary=mpz","press":{"type":"mute_toggle"}},
            {"label":"Mic","rotation":"volume","step":1,"audio_target":"microphone","press":{"type":"mute_toggle"}},
            {"label":"System","rotation":"volume","step":1,"press":{"type":"mute_toggle"}},
            {"label":"Light","rotation":"brightness","step":1,"press":{"type":"none"}}
        ]);
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            pages.target(&RawEvent::DialRotate { index: 0, ticks: 2 }),
            Some(Action::AudioAdjust {
                target: "app:application.process.binary=mpz".into(),
                percent: 4
            })
        );
        pages.target(&RawEvent::DialPush {
            index: 1,
            pressed: true,
        });
        assert_eq!(
            pages.target(&RawEvent::DialPush {
                index: 1,
                pressed: false
            }),
            Some(Action::AudioMute {
                target: "microphone".into()
            })
        );
        assert_eq!(
            pages.target(&RawEvent::DialPush {
                index: 1,
                pressed: false
            }),
            None
        );
        let a = crate::audio::State {
            percent: 10,
            muted: false,
            active: None,
            icon: Default::default(),
        };
        let b = crate::audio::State {
            percent: 80,
            muted: true,
            active: None,
            icon: Default::default(),
        };
        pages.apply_targets(vec![
            ("app:application.process.binary=mpz".into(), Some(a)),
            ("microphone".into(), Some(b)),
        ]);
        let first = pages.dial_strip(Some(a));
        pages.apply_targets(vec![
            ("app:application.process.binary=mpz".into(), None),
            ("microphone".into(), Some(b)),
        ]);
        let second = pages.dial_strip(Some(a));
        assert_ne!(first, second);
        for y in 0..100 {
            assert_eq!(
                &first[(y * 800 + 200) * 3..(y * 800 + 400) * 3],
                &second[(y * 800 + 200) * 3..(y * 800 + 400) * 3]
            );
        }
        assert_eq!(
            Pages::parse(pages.json().as_bytes()).unwrap().json(),
            pages.json()
        );
        value["dials"][0]["audio_target"] = "invalid".into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn media_keys_fire_once_on_release() {
        for (kind, expected) in [
            ("media_play_pause", Action::MediaPlayPause),
            ("media_next", Action::MediaNext),
            ("media_previous", Action::MediaPrevious),
        ] {
            let mut value: serde_json::Value =
                serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
            value["pages"][0]["keys"][5]["action"] = serde_json::json!({"type":kind});
            let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
            assert_eq!(
                pages.target(&RawEvent::Key {
                    index: 5,
                    pressed: true
                }),
                None
            );
            assert_eq!(
                pages.target(&RawEvent::Key {
                    index: 5,
                    pressed: false
                }),
                Some(expected)
            );
            assert_eq!(
                pages.target(&RawEvent::Key {
                    index: 5,
                    pressed: false
                }),
                None
            );
        }
    }
    #[test]
    fn per_key_background_preserves_other_keys_and_roundtrips() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][5]["background_color"] = "blue".into();
        let mut pages = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let mut deck = decksmith_device::VirtualDeck::default();
        pages.show(&mut deck, 0).unwrap();
        assert_eq!(&deck.key_image(5).unwrap()[..3], &LabelColor::Blue.rgb());
        assert_eq!(
            &deck.key_image(4).unwrap()[..3],
            &pages.config.pages[0].background
        );
        assert_eq!(
            Pages::parse(pages.json().as_bytes()).unwrap().json(),
            pages.json()
        );
    }
    #[test]
    fn mixed_case_labels_and_overlay_positions_are_preserved() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][5]["label"] = "YouTube".into();
        value["pages"][0]["keys"][5]["label_position"] = "bottom".into();
        let parsed = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(parsed.config.pages[0].keys[5].label, "YouTube");
        assert_ne!(glyph(b'a'), glyph(b'A'));
        for (position, top) in [
            (LabelPosition::Top, 8),
            (LabelPosition::Middle, 52),
            (LabelPosition::Bottom, 97),
        ] {
            let mut rgb = vec![255; 43200];
            draw_key_label(
                &mut rgb,
                "YouTube",
                position,
                true,
                true,
                LabelColor::Default.rgb(),
            );
            assert_eq!(&rgb[..360], &vec![255; 360]);
            assert!(rgb[top * 360..(top + 15) * 360].contains(&240));
        }
        let mut rgb = vec![255; 43200];
        draw_key_label(
            &mut rgb,
            "YouTube",
            LabelPosition::Hidden,
            true,
            true,
            LabelColor::Default.rgb(),
        );
        assert!(rgb.iter().all(|value| *value == 255));
    }
    #[test]
    fn transparent_caption_preserves_image_and_uses_selected_color() {
        let mut rgb = vec![123; 43200];
        draw_key_label(
            &mut rgb,
            "Test",
            LabelPosition::Top,
            true,
            false,
            LabelColor::Yellow.rgb(),
        );
        assert!(
            rgb.chunks_exact(3)
                .all(|pixel| pixel[0] >= 123 && pixel[1] >= 123 && pixel[2] <= 123)
        );
        assert!(
            rgb.chunks_exact(3)
                .any(|pixel| pixel == LabelColor::Yellow.rgb())
        );
        assert!(
            rgb.chunks_exact(3)
                .any(|pixel| pixel != [123, 123, 123] && pixel != LabelColor::Yellow.rgb()),
            "font edges must contain anti-aliased blends"
        );
        let mut dark = vec![123; 43200];
        draw_key_label(
            &mut dark,
            "Test",
            LabelPosition::Top,
            true,
            true,
            LabelColor::White.rgb(),
        );
        assert!(dark.chunks_exact(3).any(|pixel| pixel == [41, 41, 41]));
    }
    #[test]
    fn editable_layout_round_trip_preserves_actions_and_artwork() {
        let pages = Pages::parse(include_bytes!("../../../config/audio.json")).unwrap();
        let first = pages.json();
        assert_eq!(Pages::parse(first.as_bytes()).unwrap().json(), first);
        let mut value: serde_json::Value = serde_json::from_str(&first).unwrap();
        value["pages"][0]["keys"][5]["label"] = "QUIET".into();
        value["pages"][0]["keys"][5]["action"] = serde_json::json!({"type":"mute_toggle"});
        let edited = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(edited.json().contains("QUIET"));
        value["pages"][0]["keys"][5]["action"] =
            serde_json::json!({"type":"run_shell","command":"bad"});
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn configured_dials_dispatch_all_four_and_clamp_extreme_ticks() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["dials"] = serde_json::json!([
            {"label":"Volume","rotation":"volume","step":1,"press":{"type":"mute_toggle"}},
            {"label":"Light","rotation":"brightness","step":2,"press":{"type":"none"}},
            {"label":"Next","rotation":"none","step":1,"press":{"type":"next_page"}},
            {"label":"Previous","rotation":"none","step":1,"press":{"type":"previous_page"}}
        ]);
        let mut p = Pages::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            p.target(&RawEvent::DialRotate {
                index: 1,
                ticks: i16::MAX
            }),
            Some(Action::BrightnessAdjust { percent: 20 })
        );
        assert_eq!(
            p.target(&RawEvent::DialRotate {
                index: 1,
                ticks: i16::MIN
            }),
            Some(Action::BrightnessAdjust { percent: -20 })
        );
        assert_eq!(p.target(&RawEvent::DialRotate { index: 2, ticks: 1 }), None);
        for (index, expected) in [
            (
                0,
                Some(Action::AudioMute {
                    target: "system".into(),
                }),
            ),
            (1, None),
            (2, Some(Action::GoToPage { page: 1 })),
            (3, Some(Action::GoToPage { page: 1 })),
        ] {
            assert_eq!(
                p.target(&RawEvent::DialPush {
                    index,
                    pressed: false
                }),
                None
            );
            assert_eq!(
                p.target(&RawEvent::DialPush {
                    index,
                    pressed: true
                }),
                None
            );
            assert_eq!(
                p.target(&RawEvent::DialPush {
                    index,
                    pressed: false
                }),
                expected
            );
            assert_eq!(
                p.target(&RawEvent::DialPush {
                    index,
                    pressed: false
                }),
                None
            );
        }
        value["dials"][3]["step"] = 0.into();
        assert!(Pages::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn dial_steps_are_signed_bounded_and_push_requires_a_release() {
        let mut p = Pages::parse(include_bytes!("../../../config/audio.json")).unwrap();
        for (ticks, percent) in [(-128, -20), (-2, -2), (1, 1), (127, 20)] {
            assert_eq!(
                p.target(&RawEvent::DialRotate { index: 0, ticks }),
                Some(Action::VolumeAdjust { percent })
            );
        }
        assert_eq!(p.target(&RawEvent::DialRotate { index: 1, ticks: 2 }), None);
        assert_eq!(
            p.target(&RawEvent::DialPush {
                index: 0,
                pressed: false
            }),
            None
        );
        assert_eq!(
            p.target(&RawEvent::DialPush {
                index: 0,
                pressed: true
            }),
            None
        );
        assert_eq!(
            p.target(&RawEvent::DialPush {
                index: 0,
                pressed: false
            }),
            Some(Action::MuteToggle)
        );
        p.target(&RawEvent::DialPush {
            index: 0,
            pressed: true,
        });
        p.show(&mut decksmith_device::VirtualDeck::default(), 1)
            .unwrap();
        assert_eq!(
            p.target(&RawEvent::DialPush {
                index: 0,
                pressed: false
            }),
            None
        );
        let mut nav = Pages::default();
        assert_eq!(
            nav.target(&RawEvent::DialRotate { index: 0, ticks: 1 }),
            None
        );
    }
    #[test]
    fn audio_release_runs_once_and_is_cancelled_by_page_change() {
        let mut p = Pages::parse(include_bytes!("../../../config/audio.json")).unwrap();
        let key = |pressed| RawEvent::Key { index: 4, pressed };
        assert_eq!(p.target(&key(true)), None);
        assert_eq!(p.target(&key(false)), Some(Action::MuteToggle));
        assert_eq!(p.target(&key(false)), None);
        p.target(&key(true));
        p.show(&mut decksmith_device::VirtualDeck::default(), 1)
            .unwrap();
        assert_eq!(p.target(&key(false)), None);
    }
    #[test]
    fn invalid_config_is_rejected_before_hardware() {
        let fixture = include_bytes!("../../../config/navigation.json");
        let base: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        for bad in [
            serde_json::json!(null),
            serde_json::json!({"version":1,"pages":[]}),
        ] {
            assert!(Pages::parse(&serde_json::to_vec(&bad).unwrap()).is_err());
        }
        let mut bad = base.clone();
        bad["pages"][0]["keys"][0]["action"]["page"] = 15.into();
        assert!(Pages::parse(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = base.clone();
        bad["pages"][0]["name"] = "unsupported!".into();
        assert!(Pages::parse(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = base;
        bad["execute"] = "not allowed".into();
        assert!(Pages::parse(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert!(Pages::parse(&vec![b' '; 65537]).is_err());
    }
    #[test]
    fn button_release_dispatches_once_and_page_change_cancels_held_action() {
        let mut p = Pages::default();
        let key = |pressed| RawEvent::Key { index: 1, pressed };
        assert_eq!(p.target(&key(false)), None);
        assert_eq!(p.target(&key(true)), None);
        assert_eq!(p.target(&key(false)), Some(Action::GoToPage { page: 1 }));
        assert_eq!(p.target(&key(false)), None);
        p.target(&key(true));
        let mut deck = decksmith_device::VirtualDeck::default();
        p.show(&mut deck, 1).unwrap();
        assert_eq!(p.target(&key(false)), None);
        assert!(p.show(&mut deck, 255).is_err());
        assert_eq!(p.index, 1);
    }
    #[test]
    fn relative_buttons_wrap_on_release_and_single_page_is_noop() {
        let mut config: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        for (kind, expected) in [("next_page", 1), ("previous_page", 2)] {
            let extra = config["pages"][1].clone();
            config["pages"].as_array_mut().unwrap().truncate(2);
            config["pages"].as_array_mut().unwrap().push(extra);
            config["pages"][0]["keys"][5]["action"] = serde_json::json!({"type":kind});
            let mut pages = Pages::parse(&serde_json::to_vec(&config).unwrap()).unwrap();
            let key = |pressed| RawEvent::Key { index: 5, pressed };
            assert_eq!(pages.target(&key(false)), None);
            assert_eq!(pages.target(&key(true)), None);
            assert_eq!(
                pages.target(&key(false)),
                Some(Action::GoToPage { page: expected })
            );
            assert_eq!(pages.target(&key(false)), None);
        }
        config["pages"].as_array_mut().unwrap().truncate(1);
        config["pages"][0]["keys"][1]["action"] = serde_json::json!({"type":"go_to_page","page":0});
        for kind in ["next_page", "previous_page"] {
            config["pages"][0]["keys"][5]["action"] = serde_json::json!({"type":kind});
            let mut pages = Pages::parse(&serde_json::to_vec(&config).unwrap()).unwrap();
            pages.target(&RawEvent::Key {
                index: 5,
                pressed: true,
            });
            assert_eq!(
                pages.target(&RawEvent::Key {
                    index: 5,
                    pressed: false
                }),
                None
            );
            for gesture in [TouchGesture::FlickLeft, TouchGesture::FlickRight] {
                assert_eq!(
                    pages.target(&RawEvent::Touch {
                        x: 400,
                        y: 50,
                        gesture
                    }),
                    None
                );
            }
        }
    }
    #[test]
    fn navigation_wraps_and_ignores_taps() {
        let mut p = Pages::default();
        let swipe = |gesture| RawEvent::Touch {
            x: 400,
            y: 50,
            gesture,
        };
        assert_eq!(
            p.target(&swipe(TouchGesture::FlickRight)),
            Some(Action::GoToPage { page: 1 })
        );
        assert_eq!(p.target(&swipe(TouchGesture::Tap)), None);
        assert_eq!(
            p.target(&swipe(TouchGesture::FlickLeft)),
            Some(Action::GoToPage { page: 1 })
        );
        let mut deck = decksmith_device::VirtualDeck::default();
        p.show(&mut deck, 1).unwrap();
        assert_eq!(
            p.target(&swipe(TouchGesture::FlickLeft)),
            Some(Action::GoToPage { page: 0 })
        );
        assert_eq!(
            p.target(&swipe(TouchGesture::FlickRight)),
            Some(Action::GoToPage { page: 0 })
        );
        deck.disconnect();
        assert!(p.show(&mut deck, 0).is_err());
        assert_eq!(p.index, 1);
    }
}
