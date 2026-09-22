# ADR 0016: Opt-in native plugin review bridge

Status: experimental first integration slice, September 21, 2026.

## Context and verified scope

The isolated OpenHomeB host has passed mock failure/recovery checks and scoped
live on/off, brightness and feedback trials with the user-selected Main_LED’s.
The user confirmed physical behavior and the live browser panel. This validates
the selected patched binary and action subset, not arbitrary OpenAction plugins.

Studio's `workspace_shell.py` owns navigation. Its Plugins tab previously displayed
only a roadmap placeholder. Existing actions are represented in the daemon's
`pages.rs`; the daemon also owns device input, lock handling and rendering. The
separate lab has no connection to that dispatch path or ownership of the USB device.

## Decision

Add an opt-in native GTK panel in the existing Plugins tab. A source review launch
sets `DECKSMITH_PLUGIN_LAB_CONNECTION` to a private, per-run connection descriptor
created by the already-tested lab host. Normal launches retain the placeholder.
Do not ship or auto-launch the lab binary as a production runtime in this slice.

The native page provides fixed on/off, five-point brightness changes, independent
Homebridge readings, plugin feedback and explicit reconnect. The approved target
is Main_LED’s only. No generic target editing or plugin installation is exposed.
Commands apply immediately and do not participate in layout Save and Apply.

Studio never executes the plugin, reads Homebridge credentials or accesses USB.
The descriptor must be a private regular file owned by the current user, with no
symlink following. It contains a version, loopback-only URL and transient command
token. Redirects and environment proxies are disabled; responses and commands are
bounded. These checks are not an OS sandbox or protection against the same user.
The lab continues to authenticate to the selected Homebridge server through its
existing no-auth mode; no authentication settings are altered.

One worker handles bounded HTTP requests; no network operation runs on the GTK
thread. Status polls at most once a second while mapped, with only one request in
flight. Leaving the tab removes the timer; closing the editor closes the worker.
Neither navigation nor reconnect replays commands. Backend serialization and
explicit recovery remain authoritative. Closing Studio leaves the accessory and
separately launched test host at their current states.

## Verification

- Four client boundary tests: private descriptor, rejected non-loopback/credential
  URLs, rejected shared/symlink descriptors, rejected commands before network.
- Native GTK check: rendering, command dispatch, polling stops off-tab, resumes
  on-tab, timer and worker cleanup.
- Existing workspace check: all five sections, draft/history retention, stable
  geometry, status gating and no hidden device-preview polling passed.
- Source review window opened at Plugins with real read-only Off/100% state and
  matching plugin feedback. No live action sent during setup.

Logs are in `local/plugin-lab-client-tests.log`, `local/plugin-lab-gtk.log`, and
`local/plugin-workspace-regression.log`; screenshot `local/plugin-native-review.png`.
The user accepted the native controls. The initial review left the installed
release and layout unchanged; the next checkpoint below adds a scoped hardware trial.

## Next integration gates

1. Review/accept the native interaction before choosing installation packaging.
2. Define versioned provider/action identities and opaque bounded plugin settings
   that survive missing-plugin save/export/import without loss of custom artwork.
3. Add a daemon-owned adapter through the same Auto-Lock, held-input cancellation,
   reconnect and stale-event gates as built-ins. Never make GTK the hardware host.
4. Validate rendering and assignment lifecycle before exposing physical key/dial
   assignment. Keep the compatibility subset explicit and version-pinned.
5. Implement runtime installation/lifecycle, per-plugin permissions/destinations,
   secrets migration and longer performance checks before general plugin support.

These remain V1.5 work. The review bridge is intentionally removable once the
daemon-managed production contract replaces it.

## September 21: scoped physical assignment trial

The user approved a separate HB TEST page for Main_LED’s. It adds explicit On and
Off keys, a one-point brightness dial override, and a Home navigation key.
Original pages and shared dials remain unchanged. Brightness adjustment does not
turn on an off light. The native editor preserves bounded, versioned provider,
action and settings data when labels or artwork change. Choosing a built-in
replacement explicitly removes the corresponding plugin binding. Unknown
providers and schema versions roundtrip but cannot execute.

The daemon sends supported commands to a private same-user Unix socket in the
runtime directory. A dedicated bounded worker checks lock/session, foreground
and page generations; cancellation closes the connection. The experimental host
cancels pending work, stops the child and requires recovery on disconnect or
uncertain completion. No automatic replay is allowed. Cancellation cannot undo
an accessory change already applied. This is not an operating-system sandbox.

The test host must be launched separately with `--live-main-led --hardware`.
It publishes a private, short-lived state snapshot for touch-strip feedback.
Missing or stale snapshots display Unavailable; absence of the host does not
fall back to system audio. Studio discovers the temporary native control panel
through the runtime connection descriptor. Plugin press bindings are reserved
and preserved, but do not execute in this trial. General action selection,
installation, supervised startup, destination permissions and secrets handling
remain future V1.5 work. The original OpenHomeB source repository is untouched.

Before replacing the previous installed release, the installer backed it up.
The original layout is also retained in `local/plugin-assignment-original.json`.
When rolling back to a release that predates plugin fields, restore the original
layout as well. Automated test records remain local; physical key/dial acceptance
must be confirmed separately by the user.

### Dial pacing

The daemon retains one in-flight operation plus one bounded relative brightness
intent. Rapid turns for the same binding and session are combined (maximum 100
percentage points pending), then sent in chunks of at most 20 points. Opposite
turns offset unsent motion. No new worker or growing per-detent queue is created.
Pending motion expires two seconds after the latest turn, and is discarded after
any failed operation. Lock, foreground, page and device session generations are
checked again before each dispatch. Key actions are never accumulated or replayed.
A busy brightness adjustment is therefore ordinary flow control rather than a
reason to reject every subsequent detent. This does not make the remote accessory
respond instantly; physical sustained-turn acceptance remains a separate check.


## Managed companion checkpoint

The scoped runtime is now installed independently of the development checkout.
See [managed runtime](../openhomeb-runtime.md) for startup, shutdown, recovery and
remaining limitations. Live lifecycle checks passed for Decksmith stop/start,
plugin child termination and host termination, without changing the light.
The native editor reloads the endpoint/token after a supervised restart.
