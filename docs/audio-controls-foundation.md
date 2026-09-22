# Audio controls foundation

## Using the controls

In the dial editor, choose **Audio volume**, then select an **Audio target**:
system sounds, default microphone, a named output/input, or an installed/discovered application. An app needs a matching active audio
stream before volume or mute can take effect. **Toggle mute** on that dial affects the same target.
Each dial's percentage and glossy level bar follow its own target. Muted fills
are gray; an unavailable target shows `--` rather than system volume.

Key actions now include **Audio · Adjust volume**, **Audio · Toggle mute**, and
**Audio · Select device**. The last action selects the default output or input;
applications explicitly pinned to another device may retain their route.
These settings remain draft changes until Save and Apply.

Audio targets refresh automatically; the refresh icon also allows a manual refresh.
Applications may expose streams only after playing audio. Discovery is
asynchronous, output-monitor sources are excluded from the microphone list,
and a missing saved assignment is retained for reconnection.

## App volume versus volume inside an app

Selecting an application such as **Brave** controls its playback volume through
Fedora. This is separate from the application's own controls. Turning a Brave
volume dial in Decksmith therefore does **not** move the volume slider in YouTube.
That is expected behavior.

Example: **YouTube volume → Brave app volume → Schiit Magni output volume**.
All three stages affect what you hear. If YouTube is muted, increasing Brave or
Magni volume will not unmute the video. Likewise, increasing YouTube volume will
not unmute Brave or the output device.

The Brave target affects all matching Brave playback streams, including other
tabs. It does not isolate YouTube from another tab, and browser-based Zoom or
Teams sessions may share that browser target. Decksmith's displayed percentage
is the selected target's audio level, not the website's volume-slider position.

**System sounds** replaces System output in the target picker. The stable
`system` target now reads/writes only the saved event-sound role via libpulse's
stream-restore extension. Named outputs still control hardware listening volume;
ordinary Volume Up/Down/Mute keys still use the current default output. Existing
configured dials without an explicit target resolve to system sounds. The old
`audio_dial` diagnostic mode retains its original output behavior.

System sounds works with no active event stream. A missing saved record defaults
to 100% unmuted (matching GNOME); reads never create it. Adjustments and mute write
one record with PA_UPDATE_REPLACE and immediate application, preserving its saved
routing and channel balance. Because PipeWire may not apply stream-restore writes
to running streams, the helper also updates currently active `media.role=event`
streams explicitly (at most 16); vanished short alerts are tolerated.
Unsupported/disconnected servers report unavailable;
they never fall back to output volume. The persistent reader reuses its libpulse
connection, reads once per snapshot, and bounds individual operations to 400 ms.
No new package is required beyond the existing pulseaudio-libs dependency.

The meter only matches streams marked `media.role=event`; it never meters all
speaker audio for this target. Brief alerts can finish between the two-second
stream-discovery passes, so their meter may remain idle. Percentage/mute remain
available independently. DND, disabled event sounds and app-specific notification
streams are separate settings; this control does not override them.

API reference: [PulseAudio volume-control UI guidance](https://wiki.freedesktop.org/www/Software/PulseAudio/Documentation/Developer/Clients/WritingVolumeControlUIs/).

Play/Pause and track controls use a separate media-control interface. Their
availability does not guarantee that an app currently exposes a playback stream
for volume control.

## Target matching and behavior

Application identities prefer application.id, then process binary, then
application name. All playback streams with that property are controlled
as one app; individual browser tabs are not promised as separate targets.
Device names, rather than transient numeric IDs, are saved. Current stream IDs
are resolved immediately before executing an action. Missing targets fail
without falling back to system volume. Restart matching depends on the app
continuing to expose the same identifying property.

Adjustments are limited to ±20 percentage points per event and 0–100% final
volume. The displayed app level is the highest matching stream/channel level;
all matching streams must be muted to display muted. Toggle mute makes a mixed
set consistently muted. Multiple stream operations are sequential, not atomic.

## Implementation and validation

The fixed Python helper uses the installed pactl interface to PipeWire, with
literal arguments and bounded command deadlines. It runs on the existing
audio worker, never the HID thread. Target reads are batched, duplicate queued
reads are coalesced, and stale results from a different layout are ignored.
The outer helper has a three-second deadline. Default-system controls retain
the existing WirePlumber path.

Checks cover app isolation, mixed mute state, bounded adjustments, unavailable
target behavior, device selection arguments, target validation, dial press and
rotation dispatch, independent touch-strip feedback, editor choices and package
round trips. Live inventory and default input/output level reads were verified
on Fedora. The dialog was rendered and inspected. Live mute/route changes were
not performed during the active voice session; physical interaction needs a
user check after assigning targets.

### Automatic touch-strip labels

Selecting a different audio target in Dial controls fills the touch-strip label with its display name, without the App/Input/Output prefix. You can edit the label afterward. Refreshing the inventory, reopening the dialog, and switching between dials preserve your custom label. Selecting another target supplies a new default name. Generated names are normalized to supported letters, numbers and spaces and shortened to the 24-character limit. Use Done, then Save and Apply to send the change to the device.

Verified with the GTK fixture: target selection, manual override, inventory model refresh, dial switching, long device names, and saved layout serialization. All 31 Python tests passed.

Selecting Device brightness also supplies the editable default label “Brightness”. Returning to a configured dial preserves its custom label.

### Touch mute and selected-device markers

Tap any audio-volume touch-strip section to toggle mute for its assigned target. A mute-only dial also supports tapping. Brightness-only and inactive dials ignore taps; long presses do nothing, and horizontal swipes retain page navigation. Taps during a display transition are ignored, like key presses. Missing app/device targets never fall back to the default output.

Named input/output dial sections and Select device keys now show a bottom status marker: green means the current default, gray means available but not the default, and amber means unavailable or status unknown. This indicates the selected default device, not whether audio is currently playing through it. Feedback uses the existing bounded background polling and updates after selection changes made in Fedora too. Markers are on the physical device; the layout editor remains an appearance preview.

Validation: Rust tests cover section boundaries, brightness exclusion, long press and swipe separation, status colors, and discovery for device-selection keys with no audio dials. Python tests cover default input/output changes and missing devices. Physical tap acceptance remains a user check after deployment.

Deployment verification: daemon restarted successfully with saved layout, brightness, and active page preserved. All 32 daemon tests, workspace checks, 32 Python tests, and dependency checks passed. Renderer-generated key and strip fixtures were visually inspected.

### Automatic audio target discovery

Visible Audio Target pickers now share an event subscription to Fedora audio. Opening/closing app streams and connecting/disconnecting devices update the list automatically; the refresh button remains available. Volume changes do not rebuild the list. Saved missing targets stay selected, and refreshes never replace custom dial labels. Discovery stays outside the UI thread, coalesces event bursts, retries after an audio-service disconnect, and stops the subscription when the last picker is hidden.

Validation: 36 Python tests and the GTK fixture passed, including unavailable/reappearing targets and unchanged-model preservation. A real silent playback stream was detected on appearance and removal, and the subscriber stopped when closed. No volume, mute, default-device, or saved-layout changes were made during the live check. Save open edits and reopen Decksmith to load this editor update.

### Recovery after missed events or failed discovery

Visible target pickers now refresh when opened and reconcile every five seconds, in addition to immediate audio-event updates. This allows a failed inventory read or missed event to recover without restarting background controls. Unchanged lists still avoid model rebuilds, and selections and labels remain intact. The timer stops with the last visible picker.

Verified while Brave was playing: a temporary picker recovered the live Brave target after an intentionally failed inventory read with event notifications disabled. No audio or layout settings were changed. The exact earlier user restart sequence was not reproduced, so this confirms recovery coverage rather than proving the original cause. All 37 Python tests, the GTK fixture, and live stream arrival/removal checks passed.

### Installed audio applications

Decksmith scans the desktop application catalog asynchronously at launch and refreshes it when audio choices are opened and reconciled. It recognizes likely audio apps by desktop categories, with runtime-binary rules for common native and Flatpak browsers, conferencing, chat, and media players. These are discovery rules, not a persisted installation list. Unrecognized wrapper commands are omitted rather than targeting a generic Flatpak, shell, or Electron process.

Installed apps can be assigned before playback. Choices display “Installed” until a matching playback stream is found, then “Audio detected”. This denotes a stream, not proof of audible sound. Live apps outside the installed catalog are included too. Matching installed/live entries are merged by runtime identity; labels use the clean application name. Existing saved target identities continue working. A runtime match is required before volume or mute can act; an inactive or mismatched target never falls back to system volume. Conferencing web apps share their browser's audio target.

Uninstalled apps disappear from discovery when no live stream remains; saved layout assignments stay unavailable. Native/Flatpak packages sharing a runtime binary share a volume target. Runtime mappings are candidates until observed; Brave was verified live on this desktop, while other installed apps still need live acceptance as used.

Validation: 42 Python tests and the GTK fixture passed, including installed/live merging, uninstall, duplicate entries, wrapper exclusion, audio-service unavailability, and clean automatic labels. Live discovery showed Brave with audio detected and installed Zoom, Discord, MPZ, Chrome, Firefox, and other media apps. Save open edits and reopen Decksmith to load this editor update; background controls can remain running.

### Push-to-talk and microphone feedback

For a key, select **Microphone · Push to talk** and choose a named microphone. For a dial, choose **Push to talk** under Dial press and select that same microphone as its Audio target. Choose Audio volume for rotation if you also want microphone gain control; No action disables rotation while retaining microphone status and push-to-talk.

Hold the key or dial down to unmute; release to mute. If both controls hold the same microphone, it stays open until the last hold ends. The matching touch-strip section shows the actual Muted/Live state. A push-to-talk key's bottom marker is green for live, gray for muted, and amber for unavailable. This key marker reports microphone mute state, while the device marker on the strip still identifies the default device.

Touch taps retain toggle-mute behavior; the touch surface has no reliable release event for hold-to-talk. A tap can therefore leave the microphone live until another tap or a push-to-talk release. Saving an assignment leaves the current microphone mute setting unchanged; check the displayed state before a call. These controls affect Fedora's microphone source, not the separate mute button inside Zoom, Discord, or Teams. An app's own mute can still prevent transmission.

Page changes, layout changes, disconnects, and normal background-control shutdown cancel active holds. Named microphones prevent a default-input change from redirecting a hold. Releases bypass the normal action queue and are processed even during redraws. A separate guardian attempts to mute if its parent closes the pipe, stops sending heartbeats, or terminates. A failed/expired hold does not reopen automatically; release and press again. Muting still depends on a responsive audio service; hardware loss or an operating-system failure cannot be guaranteed away. Volume/status feedback reflects actual audio state rather than assuming a successful command.

Validation: virtual-device tests covered hold/release, overlapping key and dial holds, cancellation, stalled owner, shutdown, EOF, heartbeat timeout, and termination without touching the physical microphone. Rust tests covered releases during redraw, page changes/disconnect cancellation, named-target validation, and mirrored rendering. GTK checks covered key/dial assignment and package round trips. Physical interaction acceptance remains a user check after configuring a key and dial.
