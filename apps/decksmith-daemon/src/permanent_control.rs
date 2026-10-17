//! Stable Control1 compatibility facade for the selected named device.
//! The profile service remains the single disk/preview owner. This adapter never
//! opens HID, selects a different device, or starts an alternate daemon.
use crate::{foreground::Foreground, session_lock::Gate};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json, value::RawValue};
use std::time::Duration;
use zbus::{Connection, Proxy, fdo};
const BUS: &str = "cc.senecal.Decksmith.ConnectedWorkspace";
const PATH: &str = "/cc/senecal/Decksmith/ConnectedWorkspace";
const IFACE: &str = "cc.senecal.Decksmith.DeviceWorkspace2";
type Result<T> = fdo::Result<T>;
fn failed(reason: impl ToString) -> fdo::Error {
    fdo::Error::Failed(reason.to_string())
}
#[derive(Deserialize)]
struct Profile {
    id: String,
    token: Box<RawValue>,
    layout: Box<RawValue>,
    name: String,
    brightness: u8,
    saved_page: u8,
}
struct Api {
    connection: Connection,
    gate: Gate,
    foreground: Foreground,
}
impl Api {
    async fn proxy(&self) -> Result<Proxy<'_>> {
        Proxy::new(&self.connection, BUS, PATH, IFACE)
            .await
            .map_err(failed)
    }
    async fn request<
        B: Serialize + zbus::DynamicType,
        R: for<'d> zbus::wire::DynamicDeserialize<'d>,
    >(
        &self,
        method: &str,
        body: &B,
    ) -> Result<R> {
        self.proxy()
            .await?
            .call_with_flags(method, zbus::proxy::MethodFlags::NO_AUTO_START, body)
            .await
            .map_err(failed)?
            .ok_or_else(|| failed("Missing control reply"))
    }
    async fn text<B: Serialize + zbus::DynamicType>(
        &self,
        method: &str,
        body: &B,
    ) -> Result<String> {
        self.request(method, body).await
    }
    async fn catalog(&self) -> Result<Value> {
        serde_json::from_str(&self.text("ListDevices", &()).await?).map_err(failed)
    }
    async fn selected(&self) -> Result<(Value, Profile)> {
        let catalog = self.catalog().await?;
        let id = catalog["selected"]
            .as_str()
            .ok_or_else(|| failed("No saved device is selected"))?;
        let profile: Profile =
            serde_json::from_str(&self.text("GetProfile", &(id,)).await?).map_err(failed)?;
        if profile.id != id {
            return Err(failed("Selected device reply mismatch"));
        }
        Ok((catalog, profile))
    }
    fn unlocked(&self) -> Result<()> {
        if self.gate.snapshot().locked {
            Err(failed("Session is locked"))
        } else {
            Ok(())
        }
    }
    async fn save(&self, raw: &str) -> Result<()> {
        self.unlocked()?;
        if raw.len() > 1_048_576 {
            return Err(fdo::Error::InvalidArgs("Layout too large".into()));
        }
        let (catalog, p) = self.selected().await?;
        let page = catalog["sessions"]
            .as_array()
            .and_then(|ss| ss.iter().find(|s| s["id"] == p.id))
            .and_then(|s| s["page"].as_u64())
            .and_then(|n| u8::try_from(n).ok())
            .unwrap_or(p.saved_page);
        self.text(
            "SaveProfile",
            &(p.id, p.token.get(), raw, page, p.name, p.brightness),
        )
        .await
        .map(|_| ())
    }
}
// Handlers call the profile endpoint on the same bus. Yield while awaiting its
// reply so the shared D-Bus runtime can continue serving both endpoints.
#[zbus::interface(name = "cc.senecal.Decksmith.Control1")]
impl Api {
    async fn get_status(&self) -> Result<String> {
        let catalog = match self.catalog().await {
            Ok(catalog) => catalog,
            Err(_) => return Ok(json!({"api_version":1,"mode":"permanent-service","running":true,"connected":false,"display_ready":false,"layout":"custom","active_page":null,"device_count":0,"attention":[],"reason":"waiting_for_device","auto_lock":self.gate.snapshot(),"automatic_pages":self.foreground.snapshot()}).to_string()),
        };
        let id = catalog["selected"].as_str();
        let summary = catalog["devices"]
            .as_array()
            .and_then(|ds| ds.iter().find(|d| d["id"].as_str() == id));
        let mut status = catalog["sessions"]
            .as_array()
            .and_then(|ss| ss.iter().find(|s| s["id"].as_str() == id))
            .cloned()
            .unwrap_or_else(|| json!({"connected":false,"display_ready":false}));
        status["api_version"] = json!(1);
        status["mode"] = json!("permanent-service");
        status["running"] = json!(true);
        status["selected_device"] = json!(id);
        status["layout"] = json!(id.map(|s| format!("profile:{s}")).unwrap_or_default());
        status["active_page"] = status["page"].clone();
        status["device_count"] = json!(catalog["devices"].as_array().map_or(0, Vec::len));
        if let Some(d) = summary {
            if status.get("brightness").is_none_or(Value::is_null) {
                status["brightness"] = d["brightness"].clone();
            }
            status["device_model"] = d["model"].clone();
            status["device_name"] = d["name"].clone();
        }
        status["auto_lock"] = serde_json::to_value(self.gate.snapshot()).map_err(failed)?;
        status["automatic_pages"] =
            serde_json::to_value(self.foreground.snapshot()).map_err(failed)?;
        status["attention"] = json!([]);
        Ok(status.to_string())
    }
    async fn get_layout(&self) -> Result<String> {
        Ok(self.selected().await?.1.layout.get().into())
    }
    async fn validate_layout(&self, layout_json: &str) -> Result<String> {
        let (catalog, p) = self.selected().await?;
        let model = catalog["devices"]
            .as_array()
            .and_then(|ds| ds.iter().find(|d| d["id"] == p.id))
            .and_then(|d| d["model"].as_str())
            .ok_or_else(|| failed("Unknown device model"))?;
        let model = match model {
            "plus" => decksmith_core::device_model::DeviceModel::Plus,
            "mk2" => decksmith_core::device_model::DeviceModel::Mk2,
            _ => return Err(failed("Unsupported device model")),
        };
        crate::layout_document::Document::for_model(layout_json.as_bytes(), model)
            .map_err(failed)?;
        Ok(layout_json.into())
    }
    async fn preview_keys(&self, layout_json: &str, page: u8) -> Result<Vec<u8>> {
        let (_, p) = self.selected().await?;
        self.request("Preview", &(p.id, layout_json, page, false))
            .await
            .map_err(failed)
    }
    async fn preview_touch(&self, layout_json: &str, page: u8) -> Result<(Vec<u8>, String)> {
        let (_, p) = self.selected().await?;
        let frame = self
            .request("Preview", &(p.id, layout_json, page, true))
            .await
            .map_err(failed)?;
        Ok((frame, "saved-device".into()))
    }
    async fn save_layout(&self, layout_json: &str) -> Result<()> {
        self.save(layout_json).await
    }
    async fn show_page(&self, page: u8) -> Result<()> {
        self.unlocked()?;
        let (_, p) = self.selected().await?;
        self.text("ShowPage", &(p.id, p.token.get(), page))
            .await
            .map(|_| ())
    }
    async fn set_brightness(&self, percent: u8) -> Result<()> {
        self.unlocked()?;
        if percent > 100 {
            return Err(fdo::Error::InvalidArgs(
                "Brightness must be 0 through 100".into(),
            ));
        }
        let (_, p) = self.selected().await?;
        self.text("SaveMetadata", &(p.id, p.token.get(), p.name, percent))
            .await
            .map(|_| ())
    }
    async fn set_auto_lock(&self, enabled: bool) -> Result<()> {
        crate::control::set_auto_lock_preference(enabled, &self.gate)
    }
    async fn report_foreground(&self, desktop_id: &str) -> Result<()> {
        self.foreground.report(desktop_id).map_err(failed)
    }
    async fn get_previous_layout(&self) -> Result<String> {
        let (_, p) = self.selected().await?;
        self.text("GetPreviousProfile", &(p.id,)).await
    }
    async fn test_control(&self, layout_json: &str, page: u8, slot: u8) -> Result<()> {
        self.unlocked()?;
        let (_, p) = self.selected().await?;
        self.text(
            "TestControl",
            &(p.id, p.token.get(), layout_json, page, slot),
        )
        .await
        .map(|_| ())
    }
    async fn set_layout(&self, name: &str) -> Result<()> {
        let (_, p) = self.selected().await?;
        if name == "custom" {
            return Ok(());
        }
        let file = match name {
            "audio" => "audio.json",
            "navigation" => "navigation.json",
            _ => return Err(fdo::Error::InvalidArgs("Unknown layout".into())),
        };
        let raw = std::fs::read_to_string(crate::runtime_paths::config(file)).map_err(failed)?;
        // Validation is model-specific: a legacy eight-key preset must never
        // replace a fifteen-key profile through silent truncation/remapping.
        self.validate_layout(&raw).await?;
        self.unlocked()?;
        self.text(
            "SaveProfile",
            &(p.id, p.token.get(), raw, 0u8, p.name, p.brightness),
        )
        .await
        .map(|_| ())
    }
}
pub(crate) fn start(
    gate: Gate,
    foreground: Foreground,
) -> std::result::Result<Connection, &'static str> {
    let connection = futures_lite::future::block_on(
        zbus::connection::Builder::session()
            .method_timeout(Duration::from_secs(7))
            .build(),
    )
    .map_err(|_| "control_adapter_bus_unavailable")?;
    // Calls have a bounded reply deadline; no auto-start or name replacement.
    let mut builder = zbus::connection::Builder::session();
    builder = builder
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name("cc.senecal.Decksmith");
    futures_lite::future::block_on(
        builder
            .serve_at(
                "/cc/senecal/Decksmith",
                Api {
                    connection,
                    gate,
                    foreground,
                },
            )
            .build(),
    )
    .map_err(|_| "bus_registration_failed")
}
