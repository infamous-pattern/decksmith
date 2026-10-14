//! Guarded app/System sounds writer. One host-owned helper; explicit handshake
//! after discovery, before each write. Nothing starts in normal V1 startup.
use crate::{audio_target::Connection, pages::Action};
type Result<T> = std::result::Result<T, &'static str>;
#[derive(serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum Response {
    Ready {},
    Ok {},
    Error { code: String },
}
#[derive(Default)]
pub struct Client {
    helper: Option<Connection>,
}
impl Client {
    pub fn execute(&mut self, action: &Action, current: impl Fn() -> bool) -> Result<()> {
        if !current() {
            return Err("stale_action_scope");
        }
        if self.helper.is_none() {
            self.helper = Some(Connection::start("serve-shared")?);
        }
        let result = conversation(action, current, |request| {
            self.helper.as_mut().unwrap().exchange(request)
        });
        if matches!(result, Err("native_audio_effect_uncertain")) {
            // Destroy its entire owned process group; never replay a lost reply.
            self.helper = None;
        }
        result
    }
}
fn conversation(
    action: &Action,
    current: impl Fn() -> bool,
    mut exchange: impl FnMut(&serde_json::Value) -> Result<Vec<u8>>,
) -> Result<()> {
    let mut request = serde_json::to_value(action).map_err(|_| "invalid_shared_native_action")?;
    // At most one event-role write plus sixteen live streams. A malformed or
    // stalled helper cannot solicit unlimited grants or keep a writer alive.
    for _ in 0..18 {
        let bytes = exchange(&request).map_err(|_| "native_audio_effect_uncertain")?;
        match serde_json::from_slice::<Response>(&bytes)
            .map_err(|_| "native_audio_effect_uncertain")?
        {
            Response::Ready {} => {
                request = serde_json::json!({"decision":if current() {"apply"} else {"abort"}});
            }
            Response::Ok {} => return Ok(()),
            Response::Error { code } => {
                return Err(match code.as_str() {
                    "unavailable" => "audio_target_unavailable",
                    "unsupported" => "audio_target_unsupported",
                    "stale" => "stale_action_scope",
                    "failed" => "audio_target_failed",
                    // A timed-out/malformed transaction cannot prove a write
                    // did not happen; stop this target for the native session.
                    _ => "native_audio_effect_uncertain",
                });
            }
        }
    }
    Err("native_audio_effect_uncertain")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_is_checked_after_every_ready_and_abort_is_not_replayed() {
        let action = Action::AudioMute {
            target: "app:application.name=Fixture".into(),
        };
        let calls = std::cell::Cell::new(0);
        let mut requests = Vec::new();
        let result = conversation(
            &action,
            || {
                let n = calls.get();
                calls.set(n + 1);
                n == 0
            },
            |r| {
                requests.push(r.clone());
                Ok(match requests.len() {
                    1 | 2 => br#"{"status":"ready"}"#.to_vec(),
                    _ => br#"{"status":"error","code":"stale"}"#.to_vec(),
                })
            },
        );
        assert_eq!(result, Err("stale_action_scope"));
        assert_eq!(requests[1], serde_json::json!({"decision":"apply"}));
        assert_eq!(requests[2], serde_json::json!({"decision":"abort"}));
    }
    #[test]
    fn lost_or_invalid_acknowledgement_is_uncertain_and_never_retried() {
        let action = Action::AudioMute {
            target: "system".into(),
        };
        for response in [
            br#"{"status":"error","code":"uncertain"}"#.as_slice(),
            b"broken",
            br#"{"status":"ok","extra":true}"#,
        ] {
            let mut calls = 0;
            assert_eq!(
                conversation(
                    &action,
                    || true,
                    |_| {
                        calls += 1;
                        Ok(response.to_vec())
                    }
                ),
                Err("native_audio_effect_uncertain")
            );
            assert_eq!(calls, 1);
        }
    }
}
