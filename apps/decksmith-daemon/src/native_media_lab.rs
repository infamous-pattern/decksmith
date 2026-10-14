//! Private-bus acceptance only; the fixed fixture name cannot select real apps.
use crate::{
    shared_native_media::{NativeBackend, Session},
    shared_worker::{Authority, Compiled, Service},
};
use decksmith_core::{
    device_context::{DeviceContext, DeviceId},
    device_model::ControlAddress,
    shared_dispatch::ActionScope,
};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, &'static str>;
pub fn run(args: &[String]) -> Result<()> {
    let [_, settings_path, case] = args else {
        return Err("native_media_lab_arguments");
    };
    if std::env::var("DECKSMITH_ISOLATED_TEST").as_deref() != Ok("1") {
        return Err("native_media_lab_isolated_only");
    }
    if !std::env::var("DBUS_SESSION_BUS_ADDRESS")
        .is_ok_and(|address| address.starts_with("unix:path=/tmp/dbus-"))
    {
        return Err("native_media_lab_private_bus_required");
    }
    let settings: Session =
        serde_json::from_slice(&crate::layout_store::read(Path::new(settings_path), 4096)?)
            .map_err(|_| "invalid_native_media_session")?;
    settings.validate()?;
    if !matches!(
        settings.name.as_str(),
        "org.mpris.MediaPlayer2.DecksmithFixture"
            | "org.mpris.MediaPlayer2.DecksmithFixture.instance7"
    ) {
        return Err("native_media_lab_fixture_only");
    }
    let expected = match case.as_str() {
        "success" | "family" | "automatic" => [Ok(()), Ok(())],
        "unsupported" => [Err("media_unsupported"), Err("media_unsupported")],
        "replacement" => [
            Err("native_media_session_changed"),
            Err("native_media_session_changed"),
        ],
        "uncertain" => [
            Err("native_media_effect_uncertain"),
            Err("native_media_effect_blocked"),
        ],
        _ => return Err("native_media_lab_arguments"),
    };
    let id = |n| DeviceId::from_bytes([n; 16]).unwrap();
    let runtime = id(99);
    let authority: Authority = Arc::new(|_| true);
    let backend = NativeBackend::connect()?;
    let settings = if matches!(case.as_str(), "family" | "automatic") {
        let selectors = vec![settings.selector.clone()];
        let mut selected = backend.resolve_media(&selectors, Some(&settings))?;
        if selected.len() != 1 || selected[0] != settings {
            return Err("native_media_lab_selection_mismatch");
        }
        selected.remove(0)
    } else {
        settings
    };
    let service = Service::start(runtime, &[id(1), id(2)], authority, vec![Box::new(backend)])?;
    for n in 1..=2 {
        service.submit(
            ActionScope {
                runtime,
                foreground: 1,
                device: DeviceContext {
                    supervisor: 1,
                    device: id(n),
                    connection: 1,
                    binding: 1,
                    lock_epoch: 0,
                    control: ControlAddress::Key(0),
                },
            },
            Compiled::media_bound(
                settings.clone(),
                if case == "unsupported" {
                    "next"
                } else {
                    "play_pause"
                },
            )?,
            None,
        )?;
    }
    let end = Instant::now() + Duration::from_secs(5);
    let mut results = Vec::new();
    while results.len() < 2 && Instant::now() < end {
        if let Some(reply) = service.poll()? {
            results.push(reply.completion.result);
        } else {
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    service.join();
    if results != expected {
        eprintln!("native fixture results: {results:?}");
        return Err("native_media_lab_outcome_mismatch");
    }
    println!(
        "{}",
        serde_json::json!({"status":"passed","case":case,"results":results,"private_mpris_fixture":true,"physical_hid":false,"native_audio":false})
    );
    Ok(())
}
