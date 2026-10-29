# Display redraw diagnostics

The multi-device runtime has an opt-in diagnostic for investigating unexpected
redraws. Set `DECKSMITH_DISPLAY_DIAGNOSTICS=1` in an isolated diagnostic process
before it starts; omit it for normal operation. The installed desktop service is
not changed by this setting in a separate process.

The `decksmith::display_diagnostics` events contain a process-local anonymous
`device_slot`, an `event` and a fixed `reason`. Slots can change between runs and
must not be used as persistent device identities. The diagnostic does not log
application names, wallpaper paths, device serials, layouts, image pixels or
control assignments. Other logging in the process may have different privacy
properties and should be reviewed separately before sharing.

`invalidate` events identify the trigger that requested a full redraw, such as
`foreground_change`, `desktop_gate`, `device_attach`, `system_state`,
`plugin_state`, `audio_state` or `profile_apply`. Multiple invalidations can be
coalesced, and initial device setup can request a frame without an `invalidate`
event. `render_requested` records whether a full or touch-only render was
accepted by the renderer; `frame_submitted` means the rendered frame was queued
for the device actor. Neither event proves the physical USB write or display
completed. Timestamps and slots allow events to be correlated; a shared time is
not by itself proof of causation.

Use temporary data and simulated devices for initial checks. A desktop session
requires separate approval for any build installation, service restart or
configuration change. Revert any approved diagnostic configuration afterward.

## Isolated verification — October 29, 2026

A private D-Bus session and simulated Plus and MK.2 devices exercised the
hardware-enabled daemon build. The focused test verified diagnostics are quiet
when disabled, reported a foreground invalidation for both anonymous slots,
and recorded the requested and submitted full frames without exposing fixture
device names or serials. Fourteen existing simulated-device execution and
lifecycle tests and the active coordinator test passed with hardware support;
one of the coordinator diagnostics was explicitly ignored. Formatting and diff
checks passed. The earlier non-hardware execution tests also passed.

These checks establish the diagnostic behavior in isolation. They do not identify
the cause of the wallpaper-associated redraw on the physical desktop, prove a
physical USB write, or replace physical acceptance.
