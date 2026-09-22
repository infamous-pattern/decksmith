# Everyday control feedback

Implemented September 14, 2026. Feedback describes the saved layout on the
connected device; it does not represent unsaved editor assignments as live.

## What you see

The Decksmith panel shows **Live control feedback** below **Edit layout** when a
control needs attention. Each entry identifies the key/dial, its label, the target,
the problem and a practical next step. Background controls shows an attention
count. Editing stays available while the checks run.

- **No audio / Start app:** no playback stream currently matches the assigned app.
  Start playback in that app. Paused apps can retain their streams and remain
  available; a silent stream is not the same as a missing stream.
- **Missing / Connect mic or output:** the named audio device is not available.
  Reconnect it or choose another target. Availability returns automatically.
- **No player / Open player:** no running MPRIS player matches the assignment.
  Open it and load media; some apps need their media-control integration enabled.
- **Unsupported / Check player:** the selected player reports that it cannot
  perform that command. Loading a playlist may help, or select another player.
  For example, Brave can expose Play/Pause without supporting Next Track.
- **App missing / Edit app:** the configured application launcher is unavailable.
  Install it or select another application.
- **Check target / Retrying:** a desktop service did not provide a usable status.
  This is unknown, not an assertion that the target is ready.

An affected key gets an amber border and exclamation mark. A dial panel keeps its
label/icon and shows the problem and short next step instead of a misleading volume
or activity display. Other keys, dials and meters keep their normal appearance.

A failed key action temporarily shows a red card with the target, failure and next
step; a failed dial action shows a red panel. After five seconds the ordinary
appearance or current availability warning returns. The panel retains the **Last
attempt** failure until a successful retry on that same control/target or navigation,
layout replacement or service restart clears it. Availability warnings clear when
a fresh check finds the target again; a historical failed attempt does not claim
that a later action has succeeded.

Timeouts explicitly say completion is unknown: part of an action may have happened.
Check the target before retrying a toggle or other action that could be repeated.
Queue-full messages distinguish an action that was not queued. Push-to-talk guard
failures ask you to release, check the mic, and press again; reconnecting a mic does
not automatically restart a failed held control.

## Target selection and recovery

Checks use the same stable audio targets and default-device lookup as the controls.
Application streams are rediscovered rather than retaining their numeric stream
IDs. Multiple matching streams count as the same app target. Explicit media-player
assignments stay within that player family. Automatic media checks share the same
previous-player preference and capability selection as media actions; Decksmith
does not jump to another player merely because the selected one cannot skip tracks.

Checks run on a separate bounded worker, approximately once per second, with
bounded desktop calls and a four-second overall inspection deadline. They do not
launch apps, change volume/mute, move streams, or restart audio services. Old replies
are rejected when the device session, page or bindings change. Failed action replies
from another page or replaced binding do not overwrite current feedback.

The physical renderer produces both key/dial warnings and saved-layout previews.
Unsaved layout previews exclude these saved-binding warnings, and the panel labels
its messages as live saved-layout feedback. Incomplete edits retain the prior valid
preview; no warning rewrites a user's label, icon, theme, scaling or action.

## Safe checks to try

1. Save any wanted edits, close and reopen Decksmith, and look below **Edit layout**
   for feedback. The background service must be running for live status.
2. Use normal Play/Pause and navigation. When a player genuinely lacks Next/Previous,
   inspect that key's warning; a failed attempt should explain the limitation rather
   than start controlling another player.
3. When convenient, stop and resume playback in an app assigned to a dial. If the app
   releases its audio stream, its warning should appear and clear when audio returns.
   No routing change or audio-service restart is needed.

## Validation and remaining coverage

Automated checks cover disappearing/reappearing app streams with new IDs, multiple
streams, named-device loss/recovery, default changes, unsupported commands with two
players, player restart, stale replies, failed-action expiry/retry, pixel parity and
unchanged saved configuration. Native GTK checks cover panel warning/recovery and
responsive editing. Live read-only inspection found Brave audio and its unsupported
Next capability, and verified the corresponding saved key warning. Deliberately
nonexistent app/player requests verified missing-target error handling without
changing playback. Saved-layout and audio snapshots matched across deployment.

Zoom, Discord, Firefox, Chrome and MPZ were discovered as installed on this desktop;
that is not live playback/control certification. Their real restart behavior,
physical USB hot-plug, audio-service restart, unusual endpoints and simultaneous
real players still need broader usage coverage. This increment does not certify
all desktops/devices, add integrations to apps without MPRIS/audio support, or
implement installation/startup recovery packaging.
