//! Immutable, bounded saved-layout documents, independent of a running device.
use crate::{pages::Config, plugin_binding::Binding};
use decksmith_core::device_model::{DeviceDescriptor, DeviceModel};

pub const LIMIT: usize = 1_048_576;

pub struct Document {
    bytes: Box<[u8]>,
    model: DeviceModel,
    version: u8,
}

impl Document {
    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        let config = Config::parse(bytes)?;
        Ok(Self {
            bytes: bytes.into(),
            model: config.model(),
            version: config.version(),
        })
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn model(&self) -> DeviceModel {
        self.model
    }
    pub fn version(&self) -> u8 {
        self.version
    }
    pub fn descriptor(&self) -> DeviceDescriptor {
        DeviceDescriptor::for_model(self.model)
    }
    pub fn for_model(bytes: &[u8], model: DeviceModel) -> Result<Self, &'static str> {
        let doc = Self::parse(bytes)?;
        if doc.model != model {
            return Err("layout_model_mismatch");
        }
        Ok(doc)
    }
    /// Preserve every original JSON value; add only the explicit V2 metadata.
    /// Original bytes are retained separately by the generation store for rollback.
    pub fn upgraded(&self) -> Result<Self, &'static str> {
        let mut value: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
            serde_json::from_slice(&self.bytes).map_err(|_| "invalid_config")?;
        // Keep nested tokens opaque: reserializing Value would round integers
        // beyond u64 inside an unavailable provider's unknown settings.
        value.insert(
            "version".into(),
            serde_json::value::RawValue::from_string("2".into()).map_err(|_| "invalid_config")?,
        );
        value.insert(
            "model".into(),
            serde_json::value::RawValue::from_string(
                serde_json::to_string(&self.model).map_err(|_| "invalid_config")?,
            )
            .map_err(|_| "invalid_config")?,
        );
        let bytes = serde_json::to_vec(&value).map_err(|_| "invalid_config")?;
        Self::parse(&bytes)
    }
    pub fn mk2_defaults() -> Result<Self, &'static str> {
        Self::for_model(
            include_bytes!("../../../config/mk2-default.json"),
            DeviceModel::Mk2,
        )
    }
    pub fn bindings(&self) -> Result<Vec<Binding>, &'static str> {
        Config::parse(&self.bytes)?.bindings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn sample() -> Value {
        serde_json::from_slice(include_bytes!("../../../config/audio.json")).unwrap()
    }
    fn parse(value: &Value) -> Result<Document, &'static str> {
        Document::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn legacy_upgrade_preserves_custom_values_assets_dials_and_unknown_plugins() {
        let mut value = sample();
        value["pages"][1]["name"] = "👩‍💻 Projects".into();
        value["theme"] =
            json!({"preset":"dark","appearance":{"font":"viking_runes","label_color":"yellow"}});
        let mut image = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            120,
            120,
            image::Rgb([21, 43, 65]),
        ))
        .write_to(&mut image, image::ImageFormat::Png)
        .unwrap();
        value["pages"][0]["keys"][5] = json!({"label":"🎵 My music", "action":{"type":"none"},
            "appearance":{"label_background":"transparent","label_color":"yellow"},
            "icon_png":image.into_inner(), "plugin":{"provider":"missing.provider","action":"custom.future","schema":17,
                "settings":{"favorite":["Preserve",true,39]}}});
        value["dials"] = json!([
            {"label":"🎙️ Mic","rotation":"volume","step":1,"audio_target":"input:test-mic","press":{"type":"push_to_talk","target":"input:test-mic"},"display_style":"single_arc"},
            {"label":"Brightness","rotation":"brightness","step":3,"press":{"type":"none"}},
            {"label":"Media","rotation":"none","step":1,"press":{"type":"media_next"},"media_player":"org.mpris.MediaPlayer2.test"},
            {"label":"Volume","rotation":"volume","step":1,"press":{"type":"mute_toggle"}}
        ]);
        value["pages"][1]["dial_overrides"] = json!([null,null,null,{"label":"Custom","rotation":"brightness","step":2,"press":{"type":"none"}}]);
        let bytes = serde_json::to_vec_pretty(&value).unwrap();
        let original = Document::parse(&bytes).unwrap();
        assert_eq!(original.bytes(), bytes);
        let upgraded = original.upgraded().unwrap();
        value["version"] = 2.into();
        value["model"] = "plus".into();
        assert_eq!(
            serde_json::from_slice::<Value>(upgraded.bytes()).unwrap(),
            value
        );
        assert_eq!(original.bytes(), bytes);
        let unknown = upgraded
            .bindings()
            .unwrap()
            .into_iter()
            .find(|b| b.provider == "missing.provider")
            .unwrap();
        assert_eq!(unknown.schema, 17);
        assert!(!unknown.supported(false));
    }

    #[test]
    fn migration_preserves_opaque_large_integer_tokens_without_rounding() {
        let mut value = sample();
        value["pages"][0]["keys"][5] = json!({"label":"Keep", "action":{"type":"none"},
            "plugin":{"provider":"missing.provider","action":"custom.future","schema":17,"settings":{"integer":"precision-sentinel"}}});
        let number = "184467440737095516160001";
        let raw = serde_json::to_string(&value)
            .unwrap()
            .replace("\"precision-sentinel\"", number);
        let original = Document::parse(raw.as_bytes()).unwrap();
        let migrated = original.upgraded().unwrap();
        assert!(
            std::str::from_utf8(migrated.bytes())
                .unwrap()
                .contains(number)
        );
        assert_eq!(original.bytes(), raw.as_bytes());
    }

    #[test]
    fn model_metadata_and_counts_reject_implicit_conversion_or_future_formats() {
        let mut value = sample();
        value["model"] = Value::Null;
        assert!(parse(&value).is_err());
        value["model"] = "plus".into();
        assert!(parse(&value).is_err());
        value["version"] = 2.into();
        assert!(parse(&value).is_ok());
        value["model"] = "mk2".into();
        assert_eq!(parse(&value).err(), Some("invalid_key_count"));
        value["model"] = "xl".into();
        assert!(parse(&value).is_err());
        value["model"] = "plus".into();
        value["version"] = 99.into();
        assert!(parse(&value).is_err());
        value["version"] = 2.into();
        value.as_object_mut().unwrap().remove("model");
        assert!(parse(&value).is_err());
        value["model"] = "plus".into();
        assert!(
            Document::for_model(&serde_json::to_vec(&value).unwrap(), DeviceModel::Mk2).is_err()
        );
        value["serial"] = "unexpected-host-identity".into();
        assert!(parse(&value).is_err());
    }

    #[test]
    fn mk2_defaults_have_own_pages_all_fifteen_keys_and_no_dials() {
        let doc = Document::mk2_defaults().unwrap();
        assert_eq!(doc.model(), DeviceModel::Mk2);
        assert_eq!(doc.version(), 2);
        let mut value: Value = serde_json::from_slice(doc.bytes()).unwrap();
        let pages = value["pages"].as_array().unwrap();
        assert_eq!(
            pages
                .iter()
                .map(|p| p["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["Home", "Media", "System"]
        );
        assert!(
            pages
                .iter()
                .all(|p| p["keys"].as_array().unwrap().len() == 15)
        );
        assert_eq!(doc.descriptor().capabilities().touch, None);
        value["dials"] = json!([]);
        assert!(parse(&value).is_err());
        value.as_object_mut().unwrap().remove("dials");
        value["pages"][0]["dial_overrides"] = json!([null, null, null, null]);
        assert!(parse(&value).is_err());
        value["pages"][0]
            .as_object_mut()
            .unwrap()
            .remove("dial_overrides");
        value["audio_dial"] = true.into();
        assert!(parse(&value).is_err());
    }

    #[test]
    fn v2_cannot_enter_the_current_plus_control_or_renderer_path() {
        let plus = parse(&sample()).unwrap().upgraded().unwrap();
        for doc in [plus, Document::mk2_defaults().unwrap()] {
            assert_eq!(
                crate::pages::Pages::parse(doc.bytes()).err(),
                Some("layout_requires_device_api")
            );
        }
        assert!(crate::pages::Pages::parse(&serde_json::to_vec(&sample()).unwrap()).is_ok());
    }

    #[test]
    fn bounded_documents_reject_extra_keys_pages_and_oversized_input() {
        assert_eq!(
            Document::parse(&vec![b' '; LIMIT + 1]).err(),
            Some("config_too_large")
        );
        let mut value: Value =
            serde_json::from_slice(Document::mk2_defaults().unwrap().bytes()).unwrap();
        let key = value["pages"][0]["keys"][0].clone();
        value["pages"][0]["keys"].as_array_mut().unwrap().push(key);
        assert_eq!(parse(&value).err(), Some("invalid_key_count"));
        let page = sample()["pages"][0].clone();
        value = sample();
        value["pages"] = json!(vec![page; 17]);
        assert!(parse(&value).is_err());
    }

    #[test]
    fn action_identity_is_independent_of_labels_and_retains_selected_player() {
        let mut value = sample();
        value["pages"][0]["keys"][5] = json!({"label":"My song","action":{"type":"media_next"},"media_player":"org.mpris.MediaPlayer2.test"});
        let before = parse(&value).unwrap().bindings().unwrap();
        value["pages"][0]["keys"][5]["label"] = "🎵 Weiter".into();
        let after = parse(&value)
            .unwrap()
            .upgraded()
            .unwrap()
            .bindings()
            .unwrap();
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(&after).unwrap()
        );
        let media = after.iter().find(|b| b.action == "media_next").unwrap();
        assert_eq!(media.provider, crate::action_contract::BUILTIN_PROVIDER);
        assert_eq!(
            media.settings,
            json!({"media_player":"org.mpris.MediaPlayer2.test"})
        );
    }
}
