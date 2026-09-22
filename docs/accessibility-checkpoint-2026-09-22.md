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
all third-party themes. No compositor or global desktop accessibility preference
was changed. Full screen-reader and actual fractional-scaling checks remain open.

Evidence and screenshots are retained under `local/accessibility-2026-09-22/`.

## Desktop installation

Installed development build `0.1.0-bdfacd1a1cdb`, retaining previous build
`0.1.0-739b30218d15` and an installer configuration backup. Activation was disabled:
both live background services retained their PIDs and zero restart counts.
All five saved configuration files matched the backup byte for byte. A fresh
isolated window loaded the installed code and passed the full compact native
regression with read-only renderer previews and fake saves/actions. Existing
user windows must be closed and reopened after saving edits to load the update.
