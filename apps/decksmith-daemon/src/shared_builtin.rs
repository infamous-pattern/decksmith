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
    let mut apply_attempts = 0;
    // Night Light can update its schedule record and multiple settings. This
    // fixed grant budget also bounds a malformed helper's request stream.
    for _ in 0..10 {
        if request.get("decision").and_then(serde_json::Value::as_str) == Some("apply") {
            apply_attempts += 1;
        }
        let bytes = exchange(&request).map_err(|code| {
            uncertain(
                if code == "audio_target_timeout" {
                    "reply_timeout"
                } else {
                    "transport_failure"
                },
                apply_attempts,
            )
        })?;
        match serde_json::from_slice::<Response>(&bytes)
            .map_err(|_| uncertain("invalid_reply", apply_attempts))?
        {
            Response::Ready {} => {
                let apply = current();
                request = serde_json::json!({"decision": if apply {"apply"} else {"abort"}})
            }
            Response::Ok {} => return Ok(()),
            Response::Error { code } => {
                return Err(match code.as_str() {
                    "stale" => "stale_action_scope",
                    "unavailable" => "builtin_unavailable",
                    "unsupported" => "unsupported_shared_builtin_action",
                    _ => uncertain("helper_error", apply_attempts),
                });
            }
        }
    }
    Err(uncertain("grant_limit", apply_attempts))
}
fn uncertain(reason: &'static str, apply_attempts: usize) -> &'static str {
    // Only fixed reason labels and a bounded counter: never log reply bytes,
    // URLs, desktop IDs, commands, exception text or other user configuration.
    tracing::warn!(
        reason,
        apply_attempts,
        "Built-in helper acknowledgement uncertain; action will not be replayed"
    );
    "native_builtin_effect_uncertain"
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
    #[test]
    fn lost_ack_after_grant_never_replays_the_original_action() {
        let action = Action::OpenWebsite {
            url: "https://example.invalid/private-path".into(),
        };
        for reply in [
            Err("audio_target_timeout"),
            Err("audio_target_failed"),
            Ok(b"child output".to_vec()),
            Ok(br#"{"status":"error","code":"uncertain"}"#.to_vec()),
        ] {
            let mut requests = Vec::new();
            assert_eq!(
                conversation(
                    &action,
                    || true,
                    |request| {
                        requests.push(request.clone());
                        if requests.len() == 1 {
                            Ok(br#"{"status":"ready"}"#.to_vec())
                        } else {
                            reply.clone()
                        }
                    },
                ),
                Err("native_builtin_effect_uncertain")
            );
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0], serde_json::to_value(&action).unwrap());
            assert_eq!(requests[1], serde_json::json!({"decision":"apply"}));
        }
    }
    #[test]
    fn uncertainty_diagnostics_keep_only_reason_and_apply_attempt_count() {
        #[derive(Clone)]
        struct Capture(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let bytes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let writer = Capture(bytes.clone());
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let action = Action::OpenWebsite {
                url: "https://example.invalid/private-path".into(),
            };
            let mut calls = 0;
            assert_eq!(
                conversation(
                    &action,
                    || true,
                    |_| {
                        calls += 1;
                        Ok(if calls == 1 {
                            br#"{"status":"ready"}"#.to_vec()
                        } else {
                            b"private child output".to_vec()
                        })
                    },
                ),
                Err("native_builtin_effect_uncertain")
            );
        });
        let log = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
        assert!(log.contains("invalid_reply"));
        assert!(log.contains("apply_attempts=1"));
        assert!(!log.contains("example.invalid"));
        assert!(!log.contains("private"));
        assert!(!log.contains("child output"));
    }
}
