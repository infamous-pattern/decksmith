# Editor accessibility and geometry checkpoint — September 22, 2026

## Finding and fix

At 1024×768, focusing a key label could move the entire settings form left and
clip the beginnings of field labels. A collapsed Homebridge expander still
contributed a 371-pixel minimum width because its two action buttons were arranged
side by side. The settings viewport was 332 pixels wide; focus scrolled it
horizontally by 31 pixels. The homogeneous key/dial stack propagated this width
requirement even to unrelated controls.

Stacking Refresh accessories and Use assignment vertically removes that width
pressure. It changes only the inline expanded action layout. No assignments,
labels, themes, preview proportions or background polling behavior were changed.
The native regression now verifies that key and dial fields require no horizontal
scrolling and their left edge remains within the settings viewport.

## Verification

- 142 Python editor tests passed.
- Native compact 1024×768 checks passed in libadwaita high contrast.
- Native 1280×860 dark-mode checks passed at 150% text sizing. A process-local
  GTK Xft DPI of 144 changed the reference font from 14.667 pixels to 22 pixels;
  this checks real enlarged text, not compositor fractional scaling.
- Checks exercise Alt+1–5, keyboard focus traversal reaching keys and dials,
  Ctrl+S, undo/redo, inline validation, focus retention during status updates,
  draft retention across tabs, discard/quit cancellation, visible page tools,
  and stable window/preview geometry when switching key and dial editors.
- Screenshots were visually reviewed for readable field beginnings, visible
  focus, native high-contrast controls and enlarged text. Long headings can
  ellipsize and tall settings can require vertical scrolling.

An initial GTK_THEME=HighContrast experiment was discarded: overriding GTK's
legacy theme did not represent native libadwaita high contrast. The accepted run
used ADW_DEBUG_HIGH_CONTRAST=1. GDK_DPI_SCALE alone was also not treated as proof of
enlarged text; the final run explicitly used GTK Xft DPI.

The native checks use isolated drafts and fake writes/actions. These results do
not certify screen-reader announcements, every monitor/scaling combination or
all third-party themes. No host compositor or global desktop accessibility preference
was changed. The subsequent VM checks below cover actual fractional scaling.

Evidence and screenshots are retained under `local/accessibility-2026-09-22/`.

## Desktop installation

Installed development build `0.1.0-bdfacd1a1cdb`, retaining previous build
`0.1.0-739b30218d15` and an installer configuration backup. Activation was disabled:
both live background services retained their PIDs and zero restart counts.
All five saved configuration files matched the backup byte for byte. A fresh
isolated window loaded the installed code and passed the full compact native
regression with read-only renderer previews and fake saves/actions. Existing
user windows must be closed and reopened after saving edits to load the update.

## Fedora 44 VM verification

Tested the installed development build on Fedora Workstation 44 with GNOME 50.4.
The native isolated editor regression passed at each actual Mutter display scale:

| GNOME appearance | Display scale | Result |
| --- | --- | --- |
| Light | 125% | Passed |
| Dark | 150% | Passed |
| High contrast | 150% | Passed |

The virtual monitor used 1920×1200 with logical display coordinates. Scale values
were read back from Mutter after applying them. The editor remained 1024×768 in
logical coordinates, and assertions covered visible fields, page controls, stable
key/dial geometry, keyboard navigation, drafts and save/validation behavior.
Eighteen screenshots were reviewed across Home, Pages, Keys, Dials, About and
Plugins. The blank preview artwork in these runs is intentional fake renderer
output; it is not a device rendering test.

Setup attempts initially used physical coordinate mode, which Mutter rejected
before the editor ran. Switching the test driver to logical coordinate mode
resolved this. The rejected attempts are not application failures or passing runs.

The VM's original 1280×800 display at 100%, color scheme, high contrast,
accessibility preferences and experimental-feature list were restored and read
back. The host desktop and physical Stream Deck were not changed. Test saves and
actions were isolated from real assignments and services.

Raw logs, display readbacks and screenshots are retained locally under
`local/vm-accessibility-2026-09-22/guest-results/`.

### Screen-reader check

Orca 50.2 was run through its GNOME user service with temporary diagnostic
logging. With Orca active, the full compact native regression passed. An AT-SPI
snapshot exposed 80 named nodes, including navigation, save/discard controls,
individual keys and dials. Orca's speech-output log contains the Decksmith window
name, navigation descriptions and pressed/not-pressed selection states.

An initial focused attempt failed its focus assertion because GNOME had
idle-locked the VM. Unlocking the test session and rerunning passed. Earlier empty
Orca diagnostic files are not counted as announcement evidence; the successful
service-based run is in `local/vm-accessibility-2026-09-22/final-results/`.

This is a bounded screen-reader smoke test, not certification of every field,
announcement order or audible speech quality. A deliberate human-paced,
keyboard-only listen-through remains open. The fast automated traversal can
coalesce speech events and cannot substitute for that review.

The temporary Orca service override was removed; screen-reader and toolkit
accessibility preferences were restored to false, and Orca was confirmed inactive.

### Paced announcement follow-up

A separate four-second-per-step run on the unlocked VM desktop passed. Orca's
speech log announced the key name, Label field and current text, text deletion,
replacement text, Save and Apply and its ready-to-save description, dial name,
Touch-strip label and current value, and Pages/About navigation. The initial
attempt was in GNOME's overview and did not deliver useful focus announcements;
that attempt is not counted. The accepted run followed dismissal of the overview.

This improves control-level announcement coverage beyond the fast regression.
It still does not certify audible quality or a complete human keyboard workflow.
The invalid empty-label step blocked saving, but no spoken validation reason was
captured; automatic error-announcement behavior needs a focused follow-up before
claiming full screen-reader support. Evidence: the `orca-paced` logs under
`local/reviewer-readiness-2026-09-22/`.
