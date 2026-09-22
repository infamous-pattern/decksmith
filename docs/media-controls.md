# Media controls

Key actions and dial press actions now offer Media · Play / Pause, Media ·
Previous track, and Media · Next track. These control an already running MPRIS
player. They do not launch an application or start an arbitrary website.

With **Media player → Automatic**, initial selection is automatic: playing players first, then paused, then stopped,
with bus-name ordering to break ties. After a successful command, selection stays
with that player session, including while paused. The selected bus owner is
remembered in the user runtime directory and verified on each action. Closing
the player releases this selection; another player is then chosen automatically. Up to eight players are inspected with bounded
calls. The selected player must advertise support for the requested action;
otherwise the request fails without sending it to another player. Next/Previous
availability depends on the player and its current playlist.

The fixed helper runs on the existing action worker, with a two-second process
deadline, outside the device loop. It sends literal MPRIS methods via GIO without
a shell. Key/dial release dispatch retains existing one-action semantics.

Reference: [MPRIS Player interface](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html).
Checks cover automatic selection, capability rejection, exact single-player
method dispatch, key releases, editor choices and live layout validation.
The latest selector update detected Brave live. Explicit-player dispatch was verified with simulated players; physical multi-player acceptance remains a user check.

## Automatic labels and icons

Choosing a media action automatically assigns Play Pause, Previous Track, or
Next Track and a matching blue symbol. Icons are drawn at four times key size
and downsampled for crisp edges, with space below for the white caption.
The default caption is at the bottom with a transparent label background.
Existing media keys without an image receive these defaults as an unsaved draft
when the editor opens. Save and Apply persists them. Existing custom images are
preserved; selecting a different media action replaces the previous defaults.
Later manual label/image edits remain possible and are not reset by ordinary
form changes. The PNG is embedded in the key so copy/paste and export carry it.

## Choosing a specific player

After choosing a media action on a key or dial press, use **Media player** to
select Automatic or a running player such as MPZ or Brave. Start the player and
open/play media if it does not appear; the list updates while visible. Save
with Done (for dials), then Save and Apply. Existing layouts remain Automatic.

An explicit selection only controls that application. If it is closed or does
not support the requested command, the action fails instead of affecting another
player. A missing saved selection remains marked Unavailable. Common numbered
browser-instance suffixes are removed from the saved identity so restarting
Brave does not normally require reassignment. Other players that change their
published identity may need to be selected again. Multiple matching instances
of the same app are ranked by Playing, Paused, Stopped, then name; this selector
does not identify individual browser tabs or videos.

Explicit assignments do not replace Automatic's remembered player. Commands are
sent to the resolved unique bus owner, and player identifiers are validated.
The optional media_player field is preserved by layout save/load, copy, and
export/import for both keys and dials. The selector controls playback actions;
it is independent of the Audio Target used for application volume.

Validation: Rust dispatch/round-trip and validation tests, 48 Python tests,
GTK key/dial selectors and package round trips, rendered dialog inspection,
and read-only live Brave discovery passed. Missing and unsupported explicit
players were tested to send no command to another app. Dependency and workspace
quality checks passed.
