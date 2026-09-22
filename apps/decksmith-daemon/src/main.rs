#[cfg(any(feature = "hardware", test))]
mod audio;
#[cfg(any(feature = "hardware", test))]
mod audio_icon;
#[cfg(any(feature = "hardware", test))]
mod audio_target;
#[cfg(any(feature = "hardware", test))]
mod audio_worker;
#[cfg(any(feature = "hardware", test))]
mod control;
#[cfg(any(feature = "hardware", test))]
mod display;
mod feedback;
#[cfg(any(feature = "hardware", test))]
mod foreground;
mod key_text;
mod launch;
mod level_bar;
#[cfg(any(feature = "hardware", test))]
mod pages;
mod plugin_binding;
mod plugin_runtime;
#[cfg(any(feature = "hardware", test))]
mod ptt;
#[cfg(any(feature = "hardware", test))]
mod resume;
mod runtime_paths;
#[cfg(any(feature = "hardware", test))]
mod session_lock;
mod system_actions;
mod theme;
#[cfg(any(feature = "hardware", test))]
mod worker;
use decksmith_device::{DeckDevice, VirtualDeck};
use std::io::{self, Write};
fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_writer(io::stderr)
        .with_ansi(false)
        .init();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--self-check"] {
        match runtime_paths::check() {
            Ok(report) => {
                println!("{report}");
                return;
            }
            Err(code) => {
                eprintln!("{code}");
                std::process::exit(1);
            }
        }
    }
    #[cfg(feature = "hardware")]
    if let [mode, path] = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
        && *mode == "--validate-layout"
    {
        match load_pages(path) {
            Ok(_) => {
                println!("Layout valid");
                return;
            }
            Err(code) => {
                eprintln!("{code}");
                std::process::exit(1);
            }
        }
    }
    #[cfg(feature = "hardware")]
    if let [mode, path, duration, seconds] = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
        && *mode == "--virtual-service"
        && *duration == "--seconds"
        && let Ok(seconds @ 1..=120) = seconds.parse::<u64>()
    {
        let result = control::startup(path).and_then(|(pages, settings)| {
            run_session(
                || Ok(VirtualDeck::default()),
                Some(seconds),
                Some(pages),
                Some(settings),
                false,
            )
        });
        if let Err(code) = result {
            eprintln!("{code}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "hardware")]
    if let [mode, path, exclusive, run] = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
        && *mode == "--config"
        && *exclusive == "--exclusive"
        && *run == "--run"
    {
        let result = control::startup(path)
            .and_then(|(pages, settings)| hardware(None, Some(pages), Some(settings)));
        if let Err(code) = result {
            tracing::error!(error_code = code, "Background session failed");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "hardware")]
    if let [mode, path, exclusive, duration, seconds] = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
        && *mode == "--config"
        && *exclusive == "--exclusive"
        && *duration == "--seconds"
        && let Ok(seconds @ 1..=120) = seconds.parse::<u64>()
    {
        let result = load_pages(path).and_then(|pages| hardware(Some(seconds), Some(pages), None));
        if let Err(code) = result {
            tracing::error!(error_code = code, "Configuration or device worker failed");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "hardware")]
    if let [mode, exclusive, duration, seconds] = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
        && matches!(*mode, "--hardware" | "--pages-demo")
        && *exclusive == "--exclusive"
        && *duration == "--seconds"
        && let Ok(seconds @ 1..=120) = seconds.parse::<u64>()
    {
        if let Err(code) = hardware(
            Some(seconds),
            (*mode == "--pages-demo").then(pages::Pages::default),
            None,
        ) {
            tracing::error!(
                error_code = code,
                "Device worker stopped; invalidate its sessions"
            );
            std::process::exit(1);
        }
        return;
    }
    if args != ["--virtual-once"] {
        tracing::error!(
            operation = "startup",
            error_code = "invalid_command",
            "Use --virtual-once, --hardware/--pages-demo --exclusive --seconds N, or --config PATH --exclusive --seconds N (1..120; hardware feature required)"
        );
        std::process::exit(2);
    }
    tracing::info!(
        operation = "startup",
        mode = "virtual",
        version = env!("CARGO_PKG_VERSION"),
        "Starting one-shot harness"
    );
    let deck = VirtualDeck::default();
    let result = writeln!(
        io::stdout().lock(),
        "{}",
        serde_json::json!({"mode":"virtual","geometry":deck.geometry(),"persistent":false})
    );
    if result.is_err() {
        tracing::error!(
            operation = "output",
            error_code = "output_write_failed",
            "Could not write harness result"
        );
        std::process::exit(1);
    }
    tracing::info!(operation = "shutdown", "Harness complete");
}

#[cfg(feature = "hardware")]
fn hardware(
    seconds: Option<u64>,
    pages: Option<pages::Pages>,
    settings: Option<control::Settings>,
) -> Result<(), &'static str> {
    if decksmith_device::ownership::competing_application_running() {
        return Err("competing_application");
    }
    // Verify ownership before spawning any workers; the physical session acquires it again.
    let check = decksmith_device::ownership::acquire().map_err(|_| "device_busy")?;
    drop(check);
    run_session(
        decksmith_device::hardware::PhysicalDeck::open_only_plus,
        seconds,
        pages,
        settings,
        true,
    )
}
#[cfg(feature = "hardware")]
fn run_session<D: DeckDevice + Send + 'static>(
    open: impl FnMut() -> Result<D, decksmith_device::DeviceError> + Send + 'static,
    seconds: Option<u64>,
    pages: Option<pages::Pages>,
    settings: Option<control::Settings>,
    physical: bool,
) -> Result<(), &'static str> {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    use std::time::{Duration, Instant};
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, stop.clone())
        .map_err(|_| "signal_setup_failed")?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, stop.clone())
        .map_err(|_| "signal_setup_failed")?;
    let (bus_connection, controls) = if let Some(settings) = settings {
        let (connection, context) = control::start(
            settings,
            pages.as_ref().map(pages::Pages::json).unwrap_or_default(),
        )?;
        (Some(connection), Some(context))
    } else {
        (None, None)
    };
    let worker_stop = stop.clone();
    let (tx, rx) = mpsc::sync_channel(256);
    let thread = std::thread::spawn(move || worker::run(open, tx, worker_stop, pages, controls));
    let deadline = seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));
    let mut ownership_check = Instant::now();
    let mut result = Ok(());
    while !stop.load(Ordering::Relaxed) && deadline.is_none_or(|deadline| Instant::now() < deadline)
    {
        if physical && ownership_check.elapsed() >= Duration::from_secs(1) {
            ownership_check = Instant::now();
            if decksmith_device::ownership::competing_application_running() {
                result = Err("competing_application");
                break;
            }
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(record) => {
                let mut out = io::stdout().lock();
                if serde_json::to_writer(&mut out, &record).is_err() || writeln!(out).is_err() {
                    result = Err("output_write_failed");
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    stop.store(true, Ordering::Relaxed);
    let worker_result = thread.join().map_err(|_| "worker_panicked")?;
    tracing::info!(operation = "shutdown", "Device and audio workers stopped");
    drop(bus_connection);
    result.and(worker_result)
}

#[cfg(any(feature = "hardware", test))]
fn load_pages(path: &str) -> Result<pages::Pages, &'static str> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|_| "config_read_failed")?;
    let mut bytes = Vec::new();
    file.take(1048577)
        .read_to_end(&mut bytes)
        .map_err(|_| "config_read_failed")?;
    pages::Pages::parse(&bytes)
}

#[cfg(any(feature = "hardware", test))]
mod meter;
