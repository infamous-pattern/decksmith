//! Desktop launches use GIO with literal arguments, never shell command text.
use crate::pages::Action;
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub fn valid_application(value: &str) -> bool {
    value.len() <= 255
        && value.ends_with(".desktop")
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
pub fn valid_url(value: &str) -> bool {
    let rest = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"));
    value.len() <= 2048
        && !value.chars().any(|c| c.is_control() || c.is_whitespace())
        && rest.is_some_and(|rest| !rest.split('/').next().unwrap_or("").is_empty())
}
pub fn valid_player(value: &str) -> bool {
    value.len() <= 255
        && value
            .strip_prefix("org.mpris.MediaPlayer2.")
            .is_some_and(|rest| {
                rest.split('.').all(|part| {
                    !part.is_empty() && part.as_bytes()[0].is_ascii_alphabetic()
                        || part.starts_with('_')
                })
            })
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
pub fn execute(action: &Action) -> Result<(), &'static str> {
    let (kind, value): (&str, &str) = match action {
        Action::MediaTarget { player, command }
            if valid_player(player)
                && matches!(command.as_str(), "play_pause" | "next" | "previous") =>
        {
            ("media", command)
        }
        Action::MediaPlayPause => ("media", "play_pause"),
        Action::MediaNext => ("media", "next"),
        Action::MediaPrevious => ("media", "previous"),
        Action::OpenApplication { desktop_id } if valid_application(desktop_id) => {
            ("application", desktop_id)
        }
        Action::OpenWebsite { url } if valid_url(url) => ("website", url),
        _ => return Err("invalid_launch"),
    };
    let helper = crate::runtime_paths::helper("launch.py");
    let mut process = Command::new("/usr/bin/python3");
    process.arg(helper).args([kind, value]);
    if let Action::MediaTarget { player, .. } = action {
        process.arg(player);
    }
    let mut child = process
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "launch_failed")?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err(match status.code() {
                        Some(2) => "media_unavailable",
                        Some(3) => "media_unsupported",
                        Some(4) => "application_unavailable",
                        _ => "launch_failed",
                    })
                };
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("launch_timeout");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_commands_and_non_web_schemes() {
        assert!(valid_application("org.gnome.Nautilus.desktop"));
        assert!(!valid_application("/tmp/custom.desktop"));
        assert!(!valid_application("x;touch.desktop"));
        assert!(valid_url("https://example.com/"));
        assert!(!valid_url("file:///tmp/a"));
        assert!(!valid_url("https://"));
        assert!(!valid_url("https://host/\nvalue"));
    }
}
