# Stream Deck Plus adapter — prepared, not hardware-certified

PhysicalDeck implements DeckDevice under the optional hardware feature. It supports
firmware queries, normalized input, RGB888 key and strip writes, and brightness.
Only the Plus is selected. Opening requires exactly one matching device; zero or
multiple matches fail rather than selecting arbitrarily. Serial numbers remain
internal and are not printed by the firmware-info command.

The adapter does not reset or write displays/brightness on open or drop. Key writes
encode through the selected library and flush its cache. Strip writes encode an
800x100 JPEG region. Invalid indices, dimensions, brightness and report shapes are
rejected. Full key/dial state reports become changed edges; rotation keeps signed
ticks. Touch maps only tap, long press and horizontal flicks; vertical-only swipes
are not synthesized into unsupported domain gestures.

poll_event is a fallible device contract. It drains already-normalized events before
reading another report, with a 20ms upstream read timeout. Timestamps use elapsed
monotonic time from this session's open, not wall-clock time. Session epochs reset
on reconnect; a future device manager must attach session identity and cancel old
trigger state. VirtualDeck retains its convenience receive_event method for replay.

## Live checks — close OpenDeck first

These commands are for a deliberate test window, not automated tests:

```sh
cargo run --locked -p decksmithctl --features hardware -- hardware info --exclusive
cargo run --locked -p decksmithctl --features hardware -- hardware monitor --exclusive --seconds 60
```

Monitor runs for the requested 1..120 seconds and prints raw normalized events, then exits. Neither
command changes brightness or images. --exclusive is an explicit caller assertion;
it does not detect or stop OpenDeck and is not an operating-system HID lock.

Firmware was read successfully on 2026-09-10: Stream Deck +, version 2.0.3.7.
An initial capture recorded a left swipe before failing; source inspection identified
the pinned upstream library returning two padding entries after the eight key states.
The adapter now accepts that exact ten-entry shape only when both padding entries
are false, with a regression test. Other malformed shapes still fail.

Before running, fully exit
OpenDeck and confirm it is not restarting automatically. After input/firmware checks,
use an agreed display-test window; image/brightness APIs are not yet exposed in CLI.

## Build prerequisites

Hardware builds require C compiler, pkg-config and libudev development headers
(Fedora systemd-devel), plus the Rust toolchain. Normal default-feature builds still
support VirtualDeck without these native headers. The full gate uses all features.

This workstation has the libudev runtime but lacks development headers. For this
checkpoint the signature-verified systemd-devel-259.8-1.fc44.x86_64 RPM was extracted
under the task's work/hid-build/sysroot. Only libudev.h/pkg-config metadata are used;
linking uses the installed system library. Nothing was installed system-wide.

The ignored local/hardware-env.sh sets this task's include/pkg-config paths. Source
it before ./scripts/check.sh in this checkout, or install normal development packages
later. Those absolute paths are intentionally not portable or committed.

## Validation and limits

Automated tests use synthetic upstream events and a fake transport. They verify edge
normalization, signed dial ticks, invalid reports, touch bounds, transport errors,
invalid outputs, and decoded JPEG dimensions. They do not certify raw HID bytes,
USB writes, orientation on the real display, firmware behavior, or reconnect.

A failed hardware transport call requires discarding the session and reconnecting;
do not retry queued image writes blindly, as the upstream cache can retain them.
Automatic reconnect, dedicated daemon workers, semantic live monitoring, per-device
identity and real display/brightness testing remain Phase 1 work.

Reference: https://docs.elgato.com/streamdeck/hid/stream-deck-plus/

## Live input checkpoint — 2026-09-10

With OpenDeck closed, firmware read succeeded (2.0.3.7). The corrected 60-second
capture exited successfully and recorded 112 normalized events: press/release on
all eight keys; positive and negative rotation on all four dials; push/release on
dials 2 and 3 (zero-based, the two rightmost); touch long press and both horizontal
swipe directions. Pushes on dials 0/1 and short touch taps were not observed and
remain unverified, not classified as failures. Raw captures remain ignored in local/.

The first longer capture stopped after one swipe. Inspection of the pinned upstream
source revealed ten key-state entries (eight keys plus two padding entries). A narrow
compatibility fix accepts only the known false padding, with a regression test.
CLI errors now distinguish invalid reports from selection/transport failures, and
monitor duration is explicit and bounded to 1..120 seconds.

All 26 tests, formatting, warnings-denied Clippy and all-feature dependency checks
passed. No display/brightness writes, reset, service install or reconnect test was
performed. The next live check should cover the two left dial pushes and short tap,
then an agreed display-test window. Remote CI still requires a runner.

## Display diagnostic and follow-up inputs — 2026-09-10

A further 60-second input capture completed with 51 events, including three short
touch taps. A new `hardware display-test --exclusive` command writes static,
numbered diagnostic images: keys 1–8 in device index order and four strip panels
numbered 1–4. Each panel has red/green top corners and blue/yellow bottom corners
so rotation and mirroring are visible. This is a test pattern, not branding.

The live command completed all eight key writes and the strip write successfully.
The user confirmed that all numbers and colored corners look correct on both
keys and strip, verifying placement and orientation visually. Brightness
is unchanged. The pattern remains on the device after exit; restarting OpenDeck
can repaint its configured layout. Previous images cannot be read back/restored
by this diagnostic. A failed write stops the command and may leave partial output.

All 27 tests plus formatting, Clippy and dependency checks passed before the live
write. No automatic reconnect or brightness test has been performed.

The final 60-second capture exited successfully with 36 events, including two
press/release cycles on each of the four dials. Combined captures now cover all
eight keys, all four dial pushes, both rotation signs on every dial, short touch
taps, touch long press and both horizontal swipe directions. This is basic live
coverage, not a latency, stress, hotplug or firmware-wide certification.

## Logo preview

`hardware logo-preview --exclusive` renders the full-color app icon from the
approved brand sheet (source region x=840,y=48,width=212,height=212) to 120x120
with Lanczos3 scaling and writes only key index 0 (top-left). The source artwork
is unchanged; this raster preview is not a production icon export. The live write
succeeded on 2026-09-10; appearance awaits user feedback. The full quality gate
passes. PNG decoding adds image's PNG feature; Cargo.lock retains flate2 1.1.5
to avoid duplicate miniz_oxide versions while satisfying the dependency audit.

A bounded foreground device-worker diagnostic is now available; see device-worker.md.
Production daemon installation and semantic dispatch remain pending.
