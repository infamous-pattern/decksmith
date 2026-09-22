# Experimental OpenHomeB companion

The approved Main_LED’s integration can now run as a per-user companion to
Decksmith Background controls. It remains a scoped compatibility trial, not a
general plugin manager or accessory picker.

Starting Decksmith's background controls starts `decksmith-openhomeb.service`.
Stopping, quitting, or restarting those controls also stops or restarts the
companion and its plugin child. It follows the existing Start Decksmith at login
preference; the companion has no separate login enablement and opens no window.
Ordinary plugin exits and read outages reconnect with bounded backoff. Failed or
uncertain commands are not replayed; an uncertain action in a running host still
requires explicit Reconnect. All startup/recovery requests are reads, not writes.

The runtime lives in `~/.local/share/decksmith/plugin-runtime/releases/`, with a
`current` link. It includes the reviewed plugin binary, checksum receipt, Python
host and the WebSockets dependency. It needs the system Python interpreter and
network access to the approved Homebridge service, but no development checkout,
virtual environment, terminal session or browser tab. The current trial uses the
server's existing no-auth mode; no password is copied into the package. Inline account setup is available; enforced destination permissions remain future work.

A private runtime lock prevents duplicate managed hosts. Startup removes only
owned, private stale runtime artifacts and refuses to replace a live host or a
symlink. A crash restart re-reads the current light state and never reconstructs
an input queue. The editor reloads the connection endpoint/token after restart.
This is process supervision and lifecycle isolation, not an OS sandbox.

## Local installation and recovery

Build/stage using `scripts/install-plugin-runtime.py --lab PATH_TO_APPROVED_LAB`.
Use `--install-home HOME --activate` for the approved desktop installation. The
installer verifies the pinned binary and package files, stores the previous
integration in `plugin-runtime/backups/`, and restores it if activation fails.
It adds only a companion unit and `decksmith.service.d/openhomeb.conf`; it does
not change the layout, login preference, or the main daemon unit.

The HB TEST page retains 1% per click. Brightness changes are ignored while the
light is off; the On key must be pressed first. Runtime restart never turns on a
light or changes its brightness. The last physical setting belongs to the light,
not to a command queue that should run after reconnect.

## Govee brightness scaling

Some Govee models report 100% as 39% when AWS brightness scaling is enabled.
This can make a confirmed adjustment disagree with Homebridge's later reading,
causing the trial runtime to pause controls. It is not necessarily a process crash.
For an affected light, Govee recommends adding its device ID under Light Devices,
enabling **AWS Brightness No Scale**, and restarting the Govee integration.
See the [Govee maintainer's brightness guidance](https://github.com/homebridge-plugins/homebridge-govee/wiki/Common-Errors#brightness-issues).
This is a per-device Homebridge setting, not a global Decksmith conversion;
do not apply it to unaffected lights or hide inconsistent readings with a wider
confirmation tolerance. Back up the configuration and verify delayed readback
after adjusting brightness in both directions.

## Accessory assignments

In **Keys & Dials**, expand **Homebridge accessory**, refresh discovery, choose
an accessory and action, and click **Use assignment**. This changes only the
editor draft; **Save and Apply** activates the assignment. Labels and category
icons populate automatically, and labels remain editable. Existing HB TEST
assignments and page layout are preserved.

Discovery uses service type plus readable/writable characteristics. Lights,
sockets and switches default to Toggle on/off; explicit On and Off remain
available. Fans default to power toggle and offer a speed dial when the service
supports it. Lights offer brightness dials. Dials use the device's advertised
step: a light typically changes by 1%, while a fan may have discrete speed steps.
An off device stays off until an explicit power action.

Cameras, motion/contact/temperature and other recognized sensors, speakers,
microphones, batteries and security systems offer status only in this version.
Camera transport services are hidden. Status reflects Homebridge's reported
characteristic, not an independent physical observation; camera Active does not
prove that a video stream is reachable. Video streaming is not implemented.
A device can expose multiple useful services (for example, a fan and its light,
or a camera and its motion detector). They remain separate, identified by
Homebridge's stable service ID. Unknown services are not guessed to be switches.

The bounded discovery snapshot refreshes every five seconds through one catalogue
request, outside the GTK/rendering threads. Assignments store only a stable ID;
capabilities are checked again before each action. Disconnected or changed devices
retain their assignments and report unavailable. Writes go through the reviewed
plugin, followed by independent readback; uncertain results are not replayed.
The managed integration uses the server configured in the Plugins tab. Existing
installations retain their current server until a tested replacement is saved.

## Connection manager

The **Plugins** tab shows the configured Homebridge server, connection health,
and discovered function count. **Disable** stops the plugin child and Homebridge
polling, rejects inputs, and preserves device state and saved assignments.
**Enable** reconnects using reads only. The preference is saved privately in
`~/.config/decksmith/openhomeb.json` (or under `XDG_CONFIG_HOME`) and survives
companion restarts and login. It does not change Start Decksmith at login.
The lightweight manager remains available while disabled or while the server is
offline; if Background controls itself is stopped, start it on Home first.

Reconnect never replays previous input. The tab distinguishes connection loss,
unavailable accessories, authentication requirements, and unconfirmed commands.
Check the physical device before reconnecting after an unconfirmed command.
Enable/Disable is unavailable during an operation, so it cannot interrupt a
write midway. Server/account editing is available inline under Connection setup.

## Server and account setup

Expand **Connection setup** on Plugins. Enter the complete server address
(including `http://` or `https://` and the port), then use **Test Connection**.
Testing reads device information and does not change the current configuration
or operate devices. If required, enter the Homebridge username/password and its
current two-factor code, then test again. **Save Connection** becomes available
only after a successful test; changing a field invalidates that test. Tests expire
after two minutes. Prefer HTTPS when available; redirects are not followed.

Passwords are saved through Secret Service using `secret-tool` (Fedora's
`libsecret` package) and GNOME Keyring, with no plaintext fallback. If the keyring
is locked or unavailable, unlock it and retry. Leaving the password blank reuses
the saved password only for the same server and username. Servers that disable
authentication need no account or keyring entry. JSON settings contain only the
server, username, enabled/removed flags and an opaque keyring reference. Previous keyring
entries are retained so configuration backups remain usable.

Two-factor codes are used for a single sign-in and are never saved. The host
and plugin share the resulting session in memory, preventing a second submission
of the same code. A restart or expired/revoked session may require a new code in
Connection setup. Ordinary password-only connections can sign in using Keyring.

Changing servers preserves all assignments by their existing service IDs; devices
not present on the selected server show unavailable. Saving a failed connection
retains the previous configuration. No device command is replayed on reconnect.

## Access review and connection removal

Plugins → **Access and assignments** explains current access and lists assignments
from the saved layout. Refresh status or reopen the tab after Save and Apply.
Shared dial assignments appear on each inheriting page; page overrides appear
in their place. Missing accessories remain listed as unavailable.

The integration reads Homebridge discovery/status and performs supported assigned
power/level actions. Cameras, sensors, speakers and microphones are status-only;
no video or microphone audio is streamed. Homebridge account access applies to
the connection; this list is not a per-device permission enforcement system.
The plugin runs as the desktop user, without a full OS sandbox.

**Remove Connection** opens an inline confirmation with Cancel. Confirming stops
the plugin and clears the saved server/account, without changing any physical
device, layout assignment, login preference, or Homebridge setting. A durable
Not configured state prevents fallback to the trial server after restart. Set up,
test and save a new connection to resume controls. Removal does not uninstall the
integration package.

The optional password checkbox deletes only the Keyring entry referenced by the
current connection. It is unchecked by default. Older credentials retained for
configuration backups are not purged. With the box unchecked, the saved password
remains in Keyring. If deletion fails, the connection remains removed and the UI
reports that cleanup was not confirmed; unlock Keyring and retry removal with
the checkbox selected. Deletion cannot be undone; a restored config backup may
need a newly entered password. No real credential or active connection is removed
by the automated regression tests.

## Security boundary

The optional companion is updated separately from the main app. Include the
preview.2 panel authorization fix when rebuilding it. See the
[security review](security-review-2026-09-22.md). Homebridge HTTP connections are
unencrypted; prefer HTTPS where available or restrict use to a trusted LAN.
