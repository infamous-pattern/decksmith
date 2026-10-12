//! Saved action identity projection; never an execution or permission boundary.
use crate::{pages::Action, plugin_binding::Binding};

pub const BUILTIN_PROVIDER: &str = "decksmith.builtin";

/// Legacy names map explicitly to stable IDs. Labels and translated UI strings
/// are deliberately absent; settings and execution semantics remain unchanged.
pub fn builtin(action: &Action, player: Option<&str>) -> Result<Binding, &'static str> {
    let id = match action {
        Action::None => "none",
        Action::System { .. } => "system",
        Action::GoToPage { .. } => "go_to_page",
        Action::NextPage => "next_page",
        Action::PreviousPage => "previous_page",
        Action::OpenApplication { .. } => "open_application",
        Action::OpenWebsite { .. } => "open_website",
        Action::AudioAdjust { .. } => "audio_adjust",
        Action::AudioMute { .. } => "audio_mute",
        Action::AudioSelect { .. } => "audio_select",
        Action::PushToTalk { .. } => "push_to_talk",
        Action::VolumeUp => "volume_up",
        Action::VolumeDown => "volume_down",
        Action::MuteToggle => "mute_toggle",
        Action::MediaPlayPause => "media_play_pause",
        Action::MediaNext => "media_next",
        Action::MediaPrevious => "media_previous",
        Action::VolumeAdjust { .. } => "volume_adjust",
        Action::BrightnessAdjust { .. } => "brightness_adjust",
        Action::Plugin { .. } | Action::MediaTarget { .. } => {
            return Err("runtime_action_not_saved");
        }
    };
    let mut settings = serde_json::to_value(action).map_err(|_| "invalid_action")?;
    settings
        .as_object_mut()
        .ok_or("invalid_action")?
        .remove("type");
    if let Some(player) = player {
        if !matches!(
            action,
            Action::MediaPlayPause | Action::MediaNext | Action::MediaPrevious
        ) {
            return Err("invalid_media_player");
        }
        settings["media_player"] = player.into();
    }
    let binding = Binding {
        provider: BUILTIN_PROVIDER.into(),
        action: id.into(),
        schema: 1,
        settings,
    };
    binding.validate()?;
    Ok(binding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn saved_builtin_variants_project_settings_without_changing_the_legacy_action() {
        for source in [
            json!({"type":"none"}),
            json!({"type":"system","command":"lock"}),
            json!({"type":"go_to_page","page":1}),
            json!({"type":"next_page"}),
            json!({"type":"previous_page"}),
            json!({"type":"open_application","desktop_id":"demo.desktop"}),
            json!({"type":"open_website","url":"https://example.com"}),
            json!({"type":"audio_adjust","target":"input:mic","percent":2}),
            json!({"type":"audio_mute","target":"input:mic"}),
            json!({"type":"audio_select","target":"output:speakers"}),
            json!({"type":"push_to_talk","target":"input:mic"}),
            json!({"type":"volume_up"}),
            json!({"type":"volume_down"}),
            json!({"type":"mute_toggle"}),
            json!({"type":"media_play_pause"}),
            json!({"type":"media_next"}),
            json!({"type":"media_previous"}),
            json!({"type":"volume_adjust","percent":3}),
            json!({"type":"brightness_adjust","percent":4}),
        ] {
            let action: Action = serde_json::from_value(source.clone()).unwrap();
            let binding = builtin(&action, None).unwrap();
            assert_eq!(binding.provider, BUILTIN_PROVIDER);
            assert_eq!(binding.schema, 1);
            assert_eq!(binding.action, source["type"].as_str().unwrap());
            let mut reconstructed = binding.settings;
            reconstructed["type"] = binding.action.into();
            assert_eq!(reconstructed, source);
            assert_eq!(serde_json::to_value(action).unwrap(), source);
        }
    }
    #[test]
    fn runtime_actions_and_player_on_unrelated_action_are_rejected() {
        assert!(
            builtin(
                &Action::MediaTarget {
                    player: "test".into(),
                    command: "next".into()
                },
                None
            )
            .is_err()
        );
        assert!(builtin(&Action::None, Some("test")).is_err());
    }
}
