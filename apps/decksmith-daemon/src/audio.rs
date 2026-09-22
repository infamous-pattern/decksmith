//! Fixed WirePlumber actions. Configurations cannot supply executable paths or arguments.
use crate::pages::Action;
use serde::Serialize;
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
fn arguments(action: Action) -> Option<&'static [&'static str]> {
    match action {
        Action::VolumeUp => Some(&[
            "set-volume",
            "--limit",
            "1.0",
            "@DEFAULT_AUDIO_SINK@",
            "5%+",
        ]),
        Action::VolumeDown => Some(&[
            "set-volume",
            "--limit",
            "1.0",
            "@DEFAULT_AUDIO_SINK@",
            "5%-",
        ]),
        Action::MuteToggle => Some(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]),
        _ => None,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct State {
    pub percent: u16,
    pub muted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default)]
    pub icon: crate::audio_icon::Kind,
}
fn parse_state(bytes: &[u8]) -> Result<State, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "audio_invalid_state")?;
    let fields: Vec<_> = text.split_whitespace().collect();
    if fields.len() < 2
        || fields.len() > 3
        || fields[0] != "Volume:"
        || (fields.len() == 3 && fields[2] != "[MUTED]")
    {
        return Err("audio_invalid_state");
    }
    let volume: f64 = fields[1].parse().map_err(|_| "audio_invalid_state")?;
    if !volume.is_finite() || !(0.0..=10.0).contains(&volume) {
        return Err("audio_invalid_state");
    }
    Ok(State {
        percent: (volume * 100.0).round() as u16,
        muted: fields.len() == 3,
        active: None,
        icon: Default::default(),
    })
}
pub fn read() -> Result<State, &'static str> {
    parse_state(&command(&["get-volume", "@DEFAULT_AUDIO_SINK@"], true)?)
}
pub fn execute(action: Action) -> Result<(), &'static str> {
    if let Action::VolumeAdjust { percent } = action {
        if percent == 0 || !(-20..=20).contains(&percent) {
            return Err("invalid_audio_adjustment");
        }
        let amount = format!(
            "{}%{}",
            percent.unsigned_abs(),
            if percent > 0 { "+" } else { "-" }
        );
        command(
            &[
                "set-volume",
                "--limit",
                "1.0",
                "@DEFAULT_AUDIO_SINK@",
                &amount,
            ],
            false,
        )?;
    } else {
        command(arguments(action).ok_or("unsupported_audio_action")?, false)?;
    }
    Ok(())
}
fn command(args: &[&str], capture: bool) -> Result<Vec<u8>, &'static str> {
    let mut child = Command::new("/usr/bin/wpctl")
        .args(args)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(if capture {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "audio_spawn_failed")?;
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err("audio_command_failed");
                }
                let mut bytes = Vec::new();
                if let Some(out) = child.stdout.take() {
                    out.take(257)
                        .read_to_end(&mut bytes)
                        .map_err(|_| "audio_read_failed")?;
                }
                if bytes.len() > 256 {
                    return Err("audio_invalid_state");
                }
                return Ok(bytes);
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(if result.is_err() {
                    "audio_wait_failed"
                } else {
                    "audio_timeout"
                });
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_only_valid_bounded_audio_state() {
        assert_eq!(
            parse_state(b"Volume: 0.70\n").unwrap(),
            State {
                percent: 70,
                muted: false,
                active: None,
                icon: Default::default(),
            }
        );
        assert_eq!(
            parse_state(b"Volume: 1.20 [MUTED]\n").unwrap(),
            State {
                percent: 120,
                muted: true,
                active: None,
                icon: Default::default(),
            }
        );
        for invalid in [
            "Volume: NaN",
            "Volume: -1",
            "Volume: inf",
            "Volume: 0.7 garbage",
            "bad",
        ] {
            assert!(parse_state(invalid.as_bytes()).is_err());
        }
    }
    #[test]
    fn audio_actions_use_fixed_default_output_and_limit() {
        assert_eq!(
            arguments(Action::VolumeUp).unwrap(),
            [
                "set-volume",
                "--limit",
                "1.0",
                "@DEFAULT_AUDIO_SINK@",
                "5%+"
            ]
        );
        assert_eq!(arguments(Action::VolumeDown).unwrap().last(), Some(&"5%-"));
        assert_eq!(
            arguments(Action::MuteToggle).unwrap(),
            ["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]
        );
        assert!(arguments(Action::GoToPage { page: 0 }).is_none());
    }
}
