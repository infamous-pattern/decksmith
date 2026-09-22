# Phase 0 checkpoint — 2026-09-10

## Verified locally

- Fedora Workstation 44; GNOME Shell 50.4; Wayland session.
- Rust 1.97.1 and Cargo 1.97.1 available.
- PipeWire and WirePlumber active outside task sandbox.
- Stream Deck + detected as USB 0fd9:0084.
- Compiled decksmithctl probe opened /dev/hidraw14 read-only successfully.
  This path is dynamic and is rediscovered on every run. No serial is stored.
- Workspace build, formatting, Clippy with warnings denied, and 3 core tests passed.
- CLI JSON event round-trip passed; malformed JSON and out-of-range keys rejected.
- One-shot virtual daemon harness ran successfully.

## Remaining Phase 0 work

1. Original v0.4 ZIP imported and verified; approved brand reference sheet imported separately; production icons pending.
2. Apache-2.0 accepted; private james/decksmith Gitea repository created and local history pushed.
3. Complete Phase 0 logging/error conventions, deterministic Trigger Resolver skeleton
   and timing-policy tests, portable CI and dependency-policy gate.
4. Phase 1: implement upstream physical-device adapter, event decoding, keys, dials,
   touch, images, brightness and reconnect tests.
5. Extend the existing device contract and VirtualDeck as production paths are added.

GTK4, libadwaita and libudev development metadata were unavailable to pkg-config
inside the task environment. Recheck host development packages when adding those
dependencies. Current foundation needs none of those native development libraries.

The read-only probe verifies discovery and handle access only. Phase 0 is started,
not complete. No persistent daemon, GNOME extension, udev rule or autostart setting
was installed. The user's device display and control configuration were not changed.

## Protocol reference

Geometry and USB identity checked against Elgato's official reference:
https://docs.elgato.com/streamdeck/hid/stream-deck-plus/

Physical transport should use independently implemented documented behavior or an
appropriately reviewed dependency. No third-party project source was copied.

## Baseline reconciliation update

v0.4 original documents are now imported. See `baseline-reconciliation.md`.
Device code now lives in decksmith-device; CLI and daemon live under apps/.

## Gitea setup

Public repository: https://github.com/infamous-pattern/decksmith

The owner approved using the existing desktop public key as a repository-scoped
read/write deploy key. Git uses SSH with strict host verification against the
previously retained Gitea host key. Machine-specific SSH settings are in local
Git configuration; the host-key file is ignored under local/. No private keys
or authentication tokens are stored in the repository.

## Unattended development checkpoint

Added deterministic press resolver and timing-policy tests, plus VirtualDeck
lifecycle and output inspection. All 13 unit/integration tests, formatting and
Clippy passed. See trigger-resolver.md for exact semantics and integration limits.
OpenDeck was left running; no physical-device tests or writes were performed.
Logging conventions, dependency gate and CI remain outstanding for Phase 0.

## Second unattended checkpoint

- Added JSON tracing to CLI/daemon and typed device errors.
- Added bounded CLI input and tests against logging raw malformed input.
- Added explicit-clock semantic replay through VirtualDeck, including lifecycle cancellation.
- Shared one-command gate includes current cargo-deny advisory/license/source/version checks.
- 19 unit/integration tests pass, with formatting and warnings-denied Clippy.
- Gitea reports no runners. A manual workflow is prepared, not remotely verified.
- CONTRIBUTING.md now defines coding conventions and the ADR process.

OpenDeck and physical hardware remain untouched. Next infrastructure decision is
where to run isolated repository-scoped CI. Physical Phase 1 testing needs a window
with OpenDeck stopped. Phase 0 is not marked complete while CI is unverified and
remaining architecture-contract details are still tracked in baseline-reconciliation.md.

## Hardware adapter preparation

Optional upstream-backed PhysicalDeck is implemented with synthetic transport tests.
See hardware-adapter.md and ADR-0013. No live device open/input/display operation
was performed. OpenDeck remains untouched; the next step is the agreed live test window.

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

## First verified display output — 2026-09-10

Short touch taps now register. All eight key images and the full touch strip were
written using the new numbered diagnostic command. The user confirmed correct
number placement and corner colors. All 27 tests and the full local quality gate
pass. Brightness and reconnect remain untested; remote CI still awaits a runner.
See hardware-adapter.md for the detailed live evidence and diagnostic limitations.

The final input capture also verified press/release on all four dials, closing the
remaining basic input-coverage cases.

## Device worker recovery — 2026-09-10

Added a bounded foreground hardware worker to decksmithd. All 30 tests and the full
quality gate pass. A live unplug/replug produced a new session and resumed key input
without restarting the process. See device-worker.md for protocol and remaining
service, cancellation, identity and semantic-dispatch limitations.

## First live navigation action — 2026-09-10

An explicit two-page demonstration now maps left/right swipes to visible page
changes. The user verified both directions on the physical device. All 33 tests
and the local gate pass. See page-navigation.md for behavior and limits; this does
not yet implement configurable bindings, desktop actions or live key semantics.

## Saved page configuration — 2026-09-10

Added validated JSON pages, readable key labels and release-triggered go-to-page
actions. The bundled HOME/WORK layout also supports swipes. All 35 tests and the
local quality gate pass. See saved-pages.md for schema and outstanding production
renderer, binding-engine and desktop-action work.

## First desktop action adapter — 2026-09-10

An optional audio layout adds fixed WirePlumber volume and output-mute actions,
triggered once on release. All 37 tests and the complete local gate pass. See
audio-actions.md for the live test and the current synchronous-execution limits.

## Dial audio feedback — 2026-09-10

The optional audio layout now maps the leftmost dial to volume and push-release
to mute, with independently polled state on the touch strip. All 39 tests and the
local quality gate pass. See dial-audio.md for bounds and current polling limits.

## Audio worker isolation — 2026-09-10

Audio commands and readback now execute outside the device-owning thread, using
bounded nonblocking queues and device-session cancellation. All 40 tests and the
full local quality gate pass. See audio-worker.md for overload/shutdown semantics.

## Incremental display scheduler — 2026-09-10

Display requests now coalesce per screen and skip duplicate frames, with at most
one image write per scheduler turn. Desired pages and completed displays have
distinct records. All 43 tests and the local quality gate pass. See
display-scheduler.md for transient input behavior and remaining blocking work.

## Manual background app — 2026-09-10

Added graceful signal shutdown, a cooperative physical-device lock, best-effort
OpenDeck detection, a manually started user service and application-menu start/stop
entries on this desktop. Restored approved copper HOME artwork. All 45 tests and
the local gate pass; stop/restart, duplicate-client rejection, simulated competitor
detection and a real reconnect were verified. Automatic login startup remains off.
See background-app.md for limitations and operation.

## Native control panel — 2026-09-10

Added the GTK4/libadwaita panel and versioned D-Bus interface for status, saved
layouts, brightness and service controls. Preferences survive daemon restart.
The user confirmed the appearance and all controls work. All 47 Rust tests and
the local gate pass; Python syntax and a rendered native-window snapshot were
checked. See control-panel.md and ADR-0014 for the thin Python client choice and
remaining full-Studio scope.

## Current checkpoint

This file retains the historical Phase 0 record. For the current implementation,
user-confirmed behavior, and remaining work, see the
[September 12, 2026 checkpoint](checkpoints/2026-09-12.md).
