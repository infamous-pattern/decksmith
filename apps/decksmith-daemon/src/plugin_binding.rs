//! Saved plugin identity is independent of runtime availability and display labels.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub provider: String,
    pub action: String,
    pub schema: u32,
    pub settings: serde_json::Value,
}

impl Binding {
    pub fn supported(&self, dial: bool) -> bool {
        if self.schema == 2 {
            let identity = self.settings["accessoryId"].as_str().unwrap_or("");
            return self.provider == "com.infamous-pattern.openhomeb"
                && identity.len() == 64
                && identity
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                && self.settings.as_object().is_some_and(|s| s.len() == 1)
                && if dial {
                    self.action == "com.infamous-pattern.openhomeb.level"
                } else {
                    matches!(
                        self.action.as_str(),
                        "com.infamous-pattern.openhomeb.toggle"
                            | "com.infamous-pattern.openhomeb.on"
                            | "com.infamous-pattern.openhomeb.off"
                            | "com.infamous-pattern.openhomeb.status"
                    )
                };
        }
        self.provider == "com.infamous-pattern.openhomeb"
            && self.schema == 1
            && self.settings["accessoryId"]
                == "b3d109c968d17f5cc965ddfa89a1fa695a2d60832200b819ad08127ee15634b4"
            && if dial {
                self.action == "com.infamous-pattern.openhomeb.brightness"
                    && self.settings["characteristicType"] == "Brightness"
                    && self.settings["turnOnWhenAdjusting"] == false
            } else {
                self.action == "com.infamous-pattern.openhomeb.set"
                    && self.settings["characteristicType"] == "On"
                    && self.settings["targetValue"].is_boolean()
            }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        fn identity(s: &str) -> bool {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        }
        fn bounded(value: &serde_json::Value, depth: usize) -> bool {
            if depth > 8 {
                return false;
            }
            match value {
                serde_json::Value::Array(items) => {
                    items.len() <= 64 && items.iter().all(|v| bounded(v, depth + 1))
                }
                serde_json::Value::Object(items) => {
                    items.len() <= 64
                        && items
                            .iter()
                            .all(|(k, v)| k.len() <= 128 && bounded(v, depth + 1))
                }
                _ => true,
            }
        }
        if !identity(&self.provider)
            || !identity(&self.action)
            || self.schema == 0
            || !self.settings.is_object()
            || !bounded(&self.settings, 0)
            || serde_json::to_vec(&self.settings)
                .map_err(|_| "invalid_plugin_binding")?
                .len()
                > 4096
        {
            return Err("invalid_plugin_binding");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovered_assignments_are_scoped_to_valid_identity_and_surface() {
        let mut binding = Binding {
            provider: "com.infamous-pattern.openhomeb".into(),
            action: "com.infamous-pattern.openhomeb.toggle".into(),
            schema: 2,
            settings: serde_json::json!({"accessoryId":"a".repeat(64)}),
        };
        assert!(binding.supported(false));
        assert!(!binding.supported(true));
        binding.action = "com.infamous-pattern.openhomeb.level".into();
        assert!(binding.supported(true));
        assert!(!binding.supported(false));
        binding.settings["targetValue"] = serde_json::json!(true);
        assert!(!binding.supported(true));
        binding.settings = serde_json::json!({"accessoryId":"../other"});
        assert!(!binding.supported(true));
    }
}
