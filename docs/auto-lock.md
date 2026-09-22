# Auto-Lock

Enable **Auto-Lock** under **Your device** in the main Decksmith window. It is off
until explicitly enabled. The daemon handles locking without needing the editor
or GNOME menu to be open.

When GNOME or systemd-logind reports a locked graphical session, Decksmith blocks
keys, dial turns, dial presses, and touch gestures. Only the first key shows **Locked**; the other seven keys and the touch strip
are blank. Held actions and push-to-talk holds are cancelled;
queued actions from before the lock are invalidated. On confirmed unlock, the
current page is redrawn and input resumes after the input queue is quiet for
250 milliseconds. Unlock does not replay button releases or queued actions.

Detection runs on a separate thread with bounded calls, checked approximately
10 times per second when sources respond promptly. An operation already executing
when lock is detected cannot be recalled. This is a desktop convenience safeguard,
not a security boundary against another process running as your user.

If detection fails or becomes stale with Auto-Lock enabled, controls remain locked
until an unlocked state is confirmed or you explicitly disable Auto-Lock. The
panel distinguishes this from a confirmed session lock. An unavailable device
stays locked on reconnect. Brightness and page-changing requests are rejected
while locked; Auto-Lock itself can still be disabled from the panel.

GNOME ScreenSaver state is preferred, with systemd-logind LockedHint also able to
lock the device. Once GNOME reporting has been available, losing it does not fall
back to an unlocked logind hint. Other graphical sessions can use logind, but
Hyprland, KDE, Cinnamon, and other dedicated adapters have not yet been validated.
Custom locked artwork and appearance choices remain planned.

The preference is stored separately in `auto-lock.json` beside the existing
configuration. Previous releases ignore it, preserving rollback compatibility.

## Recovery and acceptance

If controls remain locked unexpectedly, unlock Fedora and check the Auto-Lock
status in Decksmith. You may turn Auto-Lock off to restore normal control. Closing
the editor does not stop the daemon. The versioned installer retains the previous
runtime and a configuration backup.

Automated coverage exercises unavailable/stale state, rapid lock/unlock cycles,
queued audio invalidation, held inputs, locked reconnect, the locked display, and
same-page restoration. The user confirmed that the physical desktop lock/unlock test worked on
September 15. This is user-reported acceptance, distinct from VM telemetry.

Fedora 45 Beta VM acceptance passed GNOME ScreenSaver active/inactive transitions,
locked-startup recovery, rejected page requests, same-page restoration, persisted
preference, and the native on/off switch. The main panel and locked strip were
rendered and visually inspected. This uses GNOME screen activation, not a physical
password-unlock test. The subsequent physical lock/unlock test was confirmed by the user. The requested
first-key-only appearance is a later refinement; it does not change input gating.
