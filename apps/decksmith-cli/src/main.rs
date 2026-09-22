#[cfg(feature = "hardware")]
mod display_test;
mod replay;
use decksmith_core::InputEvent;
use decksmith_device::{DeckDevice, VirtualDeck};
use std::io::{self, BufRead, Write};

#[derive(Debug)]
enum CliError {
    Usage,
    Input,
    InvalidEvent,
    Read,
    Write,
    Discovery,
    Invariant,
    #[cfg(feature = "hardware")]
    Hardware(decksmith_device::DeviceError),
}
impl CliError {
    fn code(&self) -> &'static str {
        match self {
            Self::Usage => "invalid_command",
            Self::Input => "invalid_json",
            Self::InvalidEvent => "invalid_event",
            Self::Read => "input_read_failed",
            Self::Write => "output_write_failed",
            Self::Discovery => "discovery_failed",
            Self::Invariant => "internal_state",
            #[cfg(feature = "hardware")]
            Self::Hardware(error) => match error {
                decksmith_device::DeviceError::DeviceBusy => "device_busy",
                decksmith_device::DeviceError::InvalidReport => "hardware_invalid_report",
                decksmith_device::DeviceError::DeviceSelection => "hardware_device_selection",
                _ => "hardware_failed",
            },
        }
    }
    fn message(&self) -> &'static str {
        match self {
            Self::Usage => "unknown command; use --help",
            Self::Input => "expected a valid JSON input event (at most 16 KiB per line)",
            Self::InvalidEvent => {
                "event violates device bounds, queue capacity or monotonic timing"
            }
            Self::Read => "could not read input",
            Self::Write => "could not write output",
            Self::Discovery => "could not enumerate Linux HID devices",
            Self::Invariant => "virtual input was not available after injection",
            #[cfg(feature = "hardware")]
            Self::Hardware(_) => {
                "hardware operation failed; inspect error_code and check device access"
            }
        }
    }
}
fn output(value: &impl serde::Serialize) -> Result<(), CliError> {
    let mut out = io::stdout().lock();
    serde_json::to_writer(&mut out, value).map_err(|_| CliError::Write)?;
    writeln!(out).map_err(|_| CliError::Write)
}
fn run() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice(){
        #[cfg(feature = "hardware")]
        ["hardware", "info", "--exclusive"] => {
            let deck = decksmith_device::hardware::PhysicalDeck::open_only_plus().map_err(CliError::Hardware)?;
            output(&deck.info().map_err(CliError::Hardware)?)?;
        },
        #[cfg(feature = "hardware")]
        ["hardware", "monitor", "--exclusive", "--seconds", seconds] => {
            let seconds: u64 = seconds.parse().map_err(|_|CliError::Usage)?;
            if !(1..=120).contains(&seconds) { return Err(CliError::Usage); }
            let mut deck = decksmith_device::hardware::PhysicalDeck::open_only_plus().map_err(CliError::Hardware)?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
            while std::time::Instant::now() < deadline {
                if let Some(event) = deck.poll_event().map_err(CliError::Hardware)? { output(&event)?; }
            }
        },
        #[cfg(feature = "hardware")]
        ["hardware", "display-test", "--exclusive"] => {
            let mut deck = decksmith_device::hardware::PhysicalDeck::open_only_plus().map_err(CliError::Hardware)?;
            display_test::write(&mut deck).map_err(CliError::Hardware)?;
            output(&serde_json::json!({"display_test":"written","visual_confirmation":"pending"}))?;
        },
        #[cfg(feature = "hardware")]
        ["hardware", "logo-preview", "--exclusive"] => {
            // Render the approved full-color app-icon region without modifying the source.
            let sheet = image::load_from_memory(include_bytes!("../../../brand/decksmith-makers-mark-brand-sheet.png"))
                .map_err(|_| CliError::Hardware(decksmith_device::DeviceError::ImageEncoding))?;
            let key = sheet.crop_imm(840, 48, 212, 212)
                .resize_exact(120, 120, image::imageops::FilterType::Lanczos3).to_rgb8();
            let mut deck = decksmith_device::hardware::PhysicalDeck::open_only_plus().map_err(CliError::Hardware)?;
            deck.set_key_image(0, key.as_raw()).map_err(CliError::Hardware)?;
            output(&serde_json::json!({"logo_preview":"written","key":0}))?;
        },
        ["devices"]=>output(&decksmith_device::devices().map_err(|_|CliError::Discovery)?)?,
        ["virtual","info"]=>output(&VirtualDeck::default().geometry())?,
        ["virtual", mode @ ("emulate" | "resolve")] => {
            let mut deck = VirtualDeck::default();
            let mut replay = replay::Replay::new();
            let mut input = io::stdin().lock();
            let mut line = Vec::new();
            loop {
                line.clear();
                let n = std::io::Read::take(&mut input, 16385)
                    .read_until(b'\n', &mut line).map_err(|_| CliError::Read)?;
                if n == 0 { break; }
                if n > 16384 { return Err(CliError::Input); }
                if *mode == "resolve" {
                    let command = serde_json::from_slice(&line).map_err(|_| CliError::Input)?;
                    for record in replay.apply(command)? { output(&record)?; }
                } else {
                    let event: InputEvent = serde_json::from_slice(&line).map_err(|_| CliError::Input)?;
                    deck.inject(event).map_err(|_| CliError::InvalidEvent)?;
                    output(&deck.receive_event().ok_or(CliError::Invariant)?)?;
                }
            }
        },
        ["--help"]|[]=>writeln!(io::stdout().lock(),"decksmithctl devices | virtual info | virtual emulate | virtual resolve\nEmulate reads timestamped JSON events, one per line, from stdin.\nHardware feature: hardware info --exclusive | hardware monitor --exclusive --seconds 60 (1..120 seconds). hardware display-test --exclusive writes numbered diagnostic images. hardware logo-preview --exclusive shows the approved app icon on key 1. Close OpenDeck first.").map_err(|_|CliError::Write)?,
        _=>return Err(CliError::Usage),
    }
    Ok(())
}
fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_writer(io::stderr)
        .with_ansi(false)
        .init();
    if let Err(e) = run() {
        tracing::error!(
            operation = "cli",
            error_code = e.code(),
            message = e.message()
        );
        std::process::exit(1);
    }
}
