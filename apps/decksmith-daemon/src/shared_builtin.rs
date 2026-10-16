//! Ordinary built-ins use the existing adapters with an explicit per-effect
//! grant after discovery. A lost reply never causes an automatic replay.
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
        crate::shared_worker::Compiled::builtin(action.clone())?;
        if !current() {
            return Err("stale_action_scope");
        }
        if self.helper.is_none() {
            self.helper = Some(Connection::start_helper("shared_builtin.py", &[])?);
        }
        let result = conversation(action, current, |request| {
            self.helper.as_mut().unwrap().exchange(request)
        });
        if result.is_err() {
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
    let mut request = serde_json::to_value(action).map_err(|_| "invalid_shared_builtin_action")?;
    // Night Light can update its schedule record and multiple settings. This
    // fixed grant budget also bounds a malformed helper's request stream.
    for _ in 0..10 {
        let bytes = exchange(&request).map_err(|_| "native_builtin_effect_uncertain")?;
        match serde_json::from_slice::<Response>(&bytes)
            .map_err(|_| "native_builtin_effect_uncertain")?
        {
            Response::Ready {} => {
                request = serde_json::json!({"decision": if current() {"apply"} else {"abort"}})
            }
            Response::Ok {} => return Ok(()),
            Response::Error { code } => {
                return Err(match code.as_str() {
                    "stale" => "stale_action_scope",
                    "unavailable" => "builtin_unavailable",
                    "unsupported" => "unsupported_shared_builtin_action",
                    _ => "native_builtin_effect_uncertain",
                });
            }
        }
    }
    Err("native_builtin_effect_uncertain")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_revoked_after_discovery_aborts_each_effect() {
        let current = std::cell::Cell::new(true);
        let mut requests = Vec::new();
        let action = Action::System {
            command: "dnd".into(),
        };
        let result = conversation(
            &action,
            || current.get(),
            |r| {
                requests.push(r.clone());
                Ok(match requests.len() {
                    1 => {
                        current.set(false);
                        br#"{"status":"ready"}"#.to_vec()
                    }
                    _ => br#"{"status":"error","code":"stale"}"#.to_vec(),
                })
            },
        );
        assert_eq!(result, Err("stale_action_scope"));
        assert_eq!(requests[1], serde_json::json!({"decision":"abort"}));
    }
    #[test]
    fn timeout_malformed_and_lost_ack_are_uncertain_without_replay() {
        let action = Action::OpenWebsite {
            url: "https://example.com/".into(),
        };
        for reply in [
            Err("audio_target_timeout"),
            Ok(b"broken".to_vec()),
            Ok(br#"{"status":"ok","extra":1}"#.to_vec()),
        ] {
            let mut calls = 0;
            assert_eq!(
                conversation(
                    &action,
                    || true,
                    |_| {
                        calls += 1;
                        reply.clone()
                    }
                ),
                Err("native_builtin_effect_uncertain")
            );
            assert_eq!(calls, 1);
        }
        let mut calls = 0;
        assert_eq!(
            conversation(
                &action,
                || true,
                |_| {
                    calls += 1;
                    Ok(br#"{"status":"ready"}"#.to_vec())
                }
            ),
            Err("native_builtin_effect_uncertain")
        );
        assert_eq!(calls, 10);
    }
}
