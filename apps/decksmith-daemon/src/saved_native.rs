//! Bounded provider requirements from immutable saved documents. No discovery,
//! artwork decoding, changed labels, migration or native effects here.
use crate::{device_profiles::Catalog, pages::Action};
use decksmith_core::device_context::DeviceId;
use std::collections::{BTreeMap, BTreeSet};
pub const DEFAULT_MEDIA: &str = "automatic";
pub fn route_levels(
    subscriptions: &BTreeMap<DeviceId, BTreeSet<String>>,
    levels: &crate::meter::Levels,
) -> BTreeMap<DeviceId, crate::meter::Levels> {
    subscriptions
        .iter()
        .map(|(id, targets)| {
            (
                *id,
                targets
                    .iter()
                    .map(|target| (target.clone(), levels.get(target).copied().flatten()))
                    .collect(),
            )
        })
        .collect()
}
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Page {
    pub audio: BTreeSet<String>,
    pub media: BTreeSet<String>,
    pub key_audio: BTreeSet<String>,
    pub meters: BTreeSet<String>,
}
impl Page {
    pub fn action(&mut self, action: Action) {
        match normalize(action) {
            Action::AudioAdjust { target, .. }
            | Action::AudioMute { target }
            | Action::AudioSelect { target }
            | Action::PushToTalk { target } => {
                self.audio.insert(target);
            }
            Action::MediaTarget { player, .. } => {
                self.media.insert(player);
            }
            _ => {}
        }
    }
}
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Plan {
    pub pages: BTreeMap<DeviceId, Vec<Page>>,
    pub audio: Vec<String>,
    pub media: Vec<String>,
}
impl Plan {
    /// Only connected touch-capable devices' current pages request peaks. Keys,
    /// offline pages and MK.2 profiles never create monitor streams.
    pub fn active_meters(
        &self,
        sessions: &[decksmith_device::manager::SessionStatus],
        blocked: bool,
    ) -> BTreeMap<DeviceId, BTreeSet<String>> {
        if blocked {
            return BTreeMap::new();
        }
        sessions
            .iter()
            .filter(|s| s.connected && s.model == decksmith_core::device_model::DeviceModel::Plus)
            .filter_map(|s| {
                self.pages
                    .get(&s.id)?
                    .get(usize::from(s.page?))
                    .map(|p| (s.id, p.meters.clone()))
            })
            .collect()
    }
    pub fn from_catalog(catalog: &Catalog) -> Result<Self, &'static str> {
        let mut plan = Self::default();
        let mut audio = BTreeSet::new();
        let mut media = BTreeSet::new();
        for summary in catalog.summaries() {
            let config =
                crate::pages::Config::parse(catalog.profile(summary.id)?.document().bytes())?;
            let pages = config.model_controls().provider_pages();
            for page in &pages {
                audio.extend(page.audio.iter().cloned());
                media.extend(page.media.iter().cloned());
            }
            plan.pages.insert(summary.id, pages);
        }
        if audio.len() > 32 || media.len() > 8 {
            return Err("native_provider_budget_exceeded");
        }
        plan.audio = audio.into_iter().collect();
        plan.media = media.into_iter().collect();
        Ok(plan)
    }
}
pub fn normalize(action: Action) -> Action {
    match action {
        Action::VolumeUp => Action::AudioAdjust {
            target: "default_output".into(),
            percent: 5,
        },
        Action::VolumeDown => Action::AudioAdjust {
            target: "default_output".into(),
            percent: -5,
        },
        Action::VolumeAdjust { percent } => Action::AudioAdjust {
            target: "default_output".into(),
            percent,
        },
        Action::MuteToggle => Action::AudioMute {
            target: "default_output".into(),
        },
        Action::MediaPlayPause => Action::MediaTarget {
            player: DEFAULT_MEDIA.into(),
            command: "play_pause".into(),
        },
        Action::MediaNext => Action::MediaTarget {
            player: DEFAULT_MEDIA.into(),
            command: "next".into(),
        },
        Action::MediaPrevious => Action::MediaTarget {
            player: DEFAULT_MEDIA.into(),
            command: "previous".into(),
        },
        action => action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effective_saved_assignments_include_keys_inherited_dials_and_page_overrides_without_edits() {
        let mut value: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap();
        value["pages"][0]["keys"][0] =
            serde_json::json!({"label":"🎵 My own label","action":{"type":"media_play_pause"}});
        value["pages"][0]["keys"][1] = serde_json::json!({"label":"Custom music","action":{"type":"media_next"},"media_player":"org.mpris.MediaPlayer2.brave"});
        value["pages"][0]["keys"][2] = serde_json::json!({"label":"Event sounds","action":{"type":"audio_adjust","target":"system","percent":1}});
        value["dials"] = serde_json::json!([
            {"label":"Browser","rotation":"volume","step":1,"audio_target":"app:application.process.binary=brave","press":{"type":"mute_toggle"}},
            {"label":"Brightness","rotation":"brightness","step":1,"press":{"type":"none"}},
            {"label":"Saved output","rotation":"volume","step":1,"audio_target":"output:fixture","press":{"type":"mute_toggle"}},
            {"label":"No rotation","rotation":"none","step":1,"press":{"type":"none"}}
        ]);
        value["pages"][1]["dial_overrides"] = serde_json::json!([{ "label":"MPZ 📻","rotation":"volume","step":2,"audio_target":"app:application.name=MPZ","press":{"type":"mute_toggle"}},null,null,null]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let config = crate::pages::Config::parse(&bytes).unwrap();
        let pages = config.model_controls().provider_pages();
        assert!(pages[0].audio.contains("system"));
        assert!(
            pages[0]
                .audio
                .contains("app:application.process.binary=brave")
        );
        assert!(pages[0].media.contains(DEFAULT_MEDIA));
        assert!(pages[0].media.contains("org.mpris.MediaPlayer2.brave"));
        assert!(pages[1].audio.contains("app:application.name=MPZ"));
        assert!(
            !pages[1]
                .audio
                .contains("app:application.process.binary=brave")
        );
        assert!(pages[1].audio.contains("output:fixture"));
        assert_eq!(
            pages[0].meters,
            [
                "app:application.process.binary=brave".into(),
                "output:fixture".into()
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            pages[1].meters,
            ["app:application.name=MPZ".into(), "output:fixture".into()]
                .into_iter()
                .collect()
        );
        assert!(pages[0].key_audio.is_empty());
        assert_eq!(
            crate::layout_document::Document::parse(&bytes)
                .unwrap()
                .bytes(),
            bytes
        );
    }
    #[test]
    fn saved_audio_compilation_keeps_app_system_and_legacy_output_separate() {
        for target in [
            "system",
            "default_output",
            "app:application.name=Music",
            "output:fixture",
        ] {
            assert!(
                crate::shared_worker::Compiled::saved_audio(Action::AudioMute {
                    target: target.into()
                })
                .is_ok()
            );
        }
        for target in ["output:@DEFAULT_SINK@", "output:77", "app:unexpected=Music"] {
            assert!(
                crate::shared_worker::Compiled::saved_audio(Action::AudioMute {
                    target: target.into()
                })
                .is_err()
            );
        }
    }
    #[test]
    fn active_meters_are_device_and_page_scoped_and_missing_is_not_silence() {
        use decksmith_core::device_model::DeviceModel;
        use decksmith_device::manager::SessionStatus;
        let ids = [1, 2, 3].map(|n| DeviceId::from_bytes([n; 16]).unwrap());
        let a = Page {
            meters: ["output:A".into()].into_iter().collect(),
            ..Default::default()
        };
        let b = Page {
            meters: ["output:B".into()].into_iter().collect(),
            ..Default::default()
        };
        let plan = Plan {
            pages: [
                (ids[0], vec![a.clone(), b.clone()]),
                (ids[1], vec![a.clone()]),
                (ids[2], vec![b]),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let mut statuses: Vec<_> = ids
            .into_iter()
            .enumerate()
            .map(|(i, id)| SessionStatus {
                id,
                model: if i == 2 {
                    DeviceModel::Mk2
                } else {
                    DeviceModel::Plus
                },
                connected: true,
                input_ready: true,
                display_ready: true,
                connection: 1,
                binding: 1,
                page: Some(0),
                brightness: 50,
                error: None,
            })
            .collect();
        let wanted = plan.active_meters(&statuses, false);
        assert_eq!(wanted.len(), 2);
        let levels = [("output:A".into(), Some(0)), ("output:B".into(), Some(99))]
            .into_iter()
            .collect();
        let routed = route_levels(&wanted, &levels);
        assert_eq!(routed[&ids[0]], routed[&ids[1]]);
        assert_eq!(routed[&ids[0]].get("output:A"), Some(&Some(0)));
        statuses[0].page = Some(1);
        let routed = route_levels(
            &plan.active_meters(&statuses, false),
            &[("output:A".into(), Some(73))].into_iter().collect(),
        );
        assert_eq!(routed[&ids[0]].get("output:B"), Some(&None));
        assert_eq!(routed[&ids[1]].get("output:A"), Some(&Some(73)));
        statuses[0].connected = false;
        assert_eq!(plan.active_meters(&statuses, false).len(), 1);
        assert!(plan.active_meters(&statuses, true).is_empty());
    }
}
