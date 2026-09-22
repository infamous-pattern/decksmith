# Manually controlled background app

The Decksmith application-menu entry now opens the control panel, which includes
Start/Stop. See control-panel.md. Stop Decksmith still stops the service directly.


Decksmith can now run until stopped:

```sh
./target/debug/decksmithd --config config/audio.json --exclusive --run
```

SIGTERM and SIGINT ask the main loop to stop, join device/audio workers, and release
the device. The existing bounded --seconds commands remain available. The helper
uses a transient systemd user service, not a login-enabled unit:

```sh
./scripts/decksmith-service.sh start
./scripts/decksmith-service.sh stop
./scripts/decksmith-service.sh status
./scripts/decksmith-service.sh logs
```

The service runs the current checkout's debug build with config/audio.json. Start
is idempotent when already active; stop sends SIGTERM. Restart is manual, and systemd
has a five-second stop timeout as a fallback for a stuck operation. Normal shutdown
was verified without that fallback. Existing HID enumeration/open and blocked output
limitations remain; this is not a hard real-time shutdown guarantee.

On this desktop, validated Decksmith and Stop Decksmith launchers were registered
under ~/.local/share/applications. They call this checkout's helper; moving the repo
requires updating those launchers. Generated launcher files are ignored under local/.
There is no tray indicator, graphical status panel or login autostart yet. If starting
from the menu appears to do nothing, run the helper's status/logs commands; startup
errors currently appear in the journal, not a graphical dialog.

## Ownership behavior

PhysicalDeck takes an advisory file lock at $XDG_RUNTIME_DIR/decksmith-device.lock
for the lifetime of an open device session. All Decksmith physical CLI and daemon
paths use it. A second client is rejected with device_busy. Runtime-directory
ownership and private permissions are checked before acquiring the lock. The file
can remain after shutdown; its existence does not mean a lock is held. Do not delete
it to bypass a running instance. The lock is per user and released while disconnected.
The service name prevents duplicate starts through the helper.

OpenDeck detection checks exact process names in /proc at startup/open and once per
second while running. A detected competitor causes Decksmith to stop its workers and
exit with competing_application. It does not stop OpenDeck. This is best-effort:
renamed/inaccessible processes, another user and startup races are not excluded by
a kernel USB ownership lock. Stop Decksmith before starting OpenDeck, and close
OpenDeck before starting Decksmith.

## Branding and validation

The HOME key now renders the approved copper Maker's Mark app icon and navigates
home. artwork: makers_mark is an optional key field. The source sheet is unchanged;
its established app-icon crop is decoded once when loading a layout and reused.
Both bundled navigation and audio layouts include it. This remains a raster preview,
not a new production artwork export.

All 45 tests and the local gate pass. Live checks verified rejection of a second
physical client, clean service stop/restart and automatic Decksmith exit when a
harmless temporary process was named OpenDeck. The real OpenDeck app was not started
for that competing-process test. A subsequent background session survived unplug/
replug and resumed controls; the user confirmed the logo and controls work.

The longer smoke test began at 19:27:15 UTC and was checked at 19:31:22 UTC
on 2026-09-10. The same service process remained active with zero automatic restarts.
The captured session recorded 2 connections, 1 disconnect, 22 audio
action results and 4 page requests, with no failed audio actions. This is a
short stability smoke test, not an overnight soak or release certification. The
service is intentionally left running for use, with login autostart still off.
