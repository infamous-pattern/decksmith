# First native control panel

This records the initial panel checkpoint. The subsequent [visual editor](visual-editor.md) adds page/key editing and My layout.

Open Decksmith from the application menu, or run:

```sh
python3 apps/decksmith-studio/panel.py
```

The panel provides background-service Start/Stop, device connection/display status,
two saved layouts (Audio controls and Navigation), and a 0–100 brightness slider
with an explicit Apply button. Closing the window leaves the daemon running. Stop
Decksmith remains available as a separate application-menu entry. Login startup is
still off. First-start and service errors are reported in the panel; no HID device
is opened by the UI.

The native GTK4/libadwaita client uses installed PyGObject bindings, with all slow
service and bus calls off the GTK thread. It is an interim thin client, not the full
Rust Studio or visual page editor. ADR-0014 records this implementation choice and
the exact build-time dependency exception. Fedora development packages were not
installed. Runtime requirements are Python 3, PyGObject, GTK4 and libadwaita.

The Rust daemon owns cc.senecal.Decksmith on the user session bus, object
/cc/senecal/Decksmith and interface cc.senecal.Decksmith.Control1. GetStatus returns
api_version=1, connected, layout, brightness and display_ready. SetBrightness takes
a byte (0..100); SetLayout accepts only audio or navigation. Requests use a bounded
16-entry queue; worker acknowledgements have a two-second wait. A timeout does not
prove an idempotent request was never applied. The panel reports the error and
refreshes status; it does not automatically retry changes.

Brightness is a global device setting applied through the existing owned HID
adapter. Saved level is the last successfully set preference, not hardware readback.
It can be unknown until first applied. Brightness errors invalidate the hardware
session; successful settings are reapplied on reconnect. Layout acknowledgement
means desired frames were queued; display_ready becomes true after writes settle.

Preferences are saved atomically to $XDG_CONFIG_HOME/decksmith/control-panel.json
(or ~/.config/decksmith/control-panel.json). Persisted layout/brightness override
initial defaults for --run. A malformed preference file is reported as a startup
error, not silently ignored. The GUI does not yet edit arbitrary JSON profiles.
Existing --seconds diagnostics retain their explicit layout behavior and do not
expose the bus interface. The IPC is a trusted per-user session API, not a network
service or a security boundary between applications running as the same user.

All 47 Rust tests and the full quality gate pass. Added tests cover unknown preset
rejection and brightness dispatch against connected/disconnected virtual devices.
Python syntax was checked. A rendered snapshot of the native window was visually
inspected. Live D-Bus checks verified status, 50% brightness, rejection of 101%, both
layout selections and preference recovery after daemon restart. GUI feedback is
recorded separately below. No device brightness getter is claimed.

The user confirmed the panel looks good and brightness, layout selection and
Stop/Start all work. Decksmith's application-menu entry now opens the panel,
while Stop Decksmith continues to stop the service directly. The panel and
background controls are left available for normal use.

## Application-menu launcher correction

The original decksmith.desktop reused the earlier service-only shortcut identity.
Menu-launch logs still contained the service helper's “already running” output even
though the panel itself opened directly. The panel is now installed as
cc.senecal.Decksmith.Studio.desktop, matching its GTK application ID, with startup
notification enabled. scripts/install-panel-launcher.py validates and installs the
entry, removes only this checkout's legacy shortcut, migrates a matching GNOME
favorite and refreshes the desktop database. Stop Decksmith is unchanged.

The user verified that closing the panel and reopening it from the application
menu now works correctly.

When the app launches with background controls stopped, a one-time dialog offers
**Start controls** or **Not now**. Dismissing it leaves the service stopped and
does not prompt again during that launch. It never changes the login preference.
Unknown/unavailable status is not treated as a confirmed stop. Starting uses the
existing asynchronous service action and error feedback.
