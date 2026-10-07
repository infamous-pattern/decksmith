# Signed V1 physical recovery follow-up — October 7, 2026

The physical Fedora Workstation 44 desktop still runs frozen signed candidate
`1.0.0-c5e4fea53afe`, public source `68c42421ceaa65bb033993b56f0babdde2a14a6e`.
No desktop installation, saved-data edit or temporary assignment was needed.
Quit/relaunch deliberately stopped and started background controls.

## Completed user-operated cases

| Case | User observation |
| --- | --- |
| Lock then unlock Fedora | Locked display appeared; the same page and working controls returned. |
| Unplug USB, wait, then reconnect | The same page, keys, dials and touch strip returned without restarting Decksmith. |
| Suspend Fedora, then resume | The user reported everything worked after suspend and restore. |
| Quit/blanking, then relaunch | In response to the full-display blanking and saved-page restoration test, the user reported Quit and restart worked perfectly. |
| Login startup | In response to the sign-out/sign-in test, the user confirmed controls started automatically with the editor closed. |

The subsequent read-only status check confirmed connected/display-ready,
Auto-Lock enabled and available through GNOME, and an unlocked session. The
original daemon process remained active with zero automatic restarts, including
a separate read-only check after suspend/resume.
After the deliberate Quit/relaunch, the daemon had a new process identity,
was active with zero automatic restarts, and again reported connected/display-ready.
Login startup remained enabled. After the user-confirmed login test, read-only
checks again found the device connected/display-ready, zero automatic restarts
and no owner of the editor application bus name. The observer did not itself
trigger or continuously monitor sign-out/sign-in.
An earlier status read overlapped the unplug test and correctly reported no
connected device; it is not used as post-lock recovery evidence.

These are finite user-confirmed tests of the exact candidate, not automated
observations of every pixel or input throughout recovery. The subsequent
[human accessibility review](2026-10-07-v1-accessibility.md) records completion
of the requested keyboard, screen-reader and visual checks. Precise visible-response timing,
resource regression reconciliation, optional companion qualification and final
publication guidance/decision also remain open.

The preceding qualification documentation was committed and pushed to Gitea
and GitHub with identical source trees and separate histories. GitHub Quality
run [37646023986](https://github.com/infamous-pattern/decksmith/actions/runs/37646023986)
passed for the corresponding documentation revision; it does not publish V1 or
replace the accepted release artifact. Raw lifecycle status remains private under
`local/v1-final-20261007/physical-lifecycle/`.
