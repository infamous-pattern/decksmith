# Native workspace interface — September 17, 2026

The approved four-section concept is implemented in GTK/libadwaita. Home retains
background controls, login startup, Auto-Lock, saved-layout selection, brightness,
automatic-page status and live feedback. Pages manages page names, ordering,
defaults and application assignments. Keys & Dials edits the same draft and uses
the physical renderer for key and touch-strip images. About includes the original
Maker’s Mark, license, guide, Gitea source and support link.

Subtle blue, green, peach and lavender surfaces distinguish the four sections;
GNOME determines light/dark appearance, typography and accent color. The shared
header and navigation remain in place. Home and Keys reuse one device preview,
not separate rendering workers. Narrow desktop windows stack settings vertically.

## Interaction contract

- Choosing a preview key, dial or touch region selects it for editing. It does not
  execute its action. **Test action** explicitly runs a saved key action or dial
  press through the daemon. Save edits and select the corresponding device page
  first. Push to talk remains a physical hold/release test, never a latched click.
- Test requests validate the saved layout, page, connection, display readiness and
  lock epoch on the device worker. They do not overwrite physical held-input state.
- Drafts, selections, page assignments and history survive section navigation.
  Existing physical/automatic page-follow behavior is retained.
- Ctrl+S saves; Ctrl+Z and Ctrl+Shift+Z operate layout history. Tab and native focus
  rings support keyboard navigation. Controls expose contextual tooltips.
- Closing a dirty layout offers Keep editing, Discard and close, or Save and close.
  Failed saves retain the window and draft. Closing the window leaves the daemon
  running. Save is unavailable while locked or while background controls are stopped.
- Disconnection does not discard settings. The editor can open the saved local
  layout if the service is stopped; restart controls to Save and Apply.
- Transparent label background exposes the underlying artwork. Transparent key
  background means the page color; the physical LCD remains opaque.
- Choose Device identifies the current Stream Deck + and connection status. Full
  independent multi-device sessions and per-device drafts remain a later milestone;
  this release does not simulate additional devices.

## Performance and validation

Python remains a thin native client; Rust owns device operations. Status reads use
D-Bus directly without spawning systemctl on the healthy path. Preview requests
are coalesced, hidden previews do not enqueue work, unchanged key pixels reuse
textures, and failed preview requests back off. CSS is rebuilt only when needed.
Older undo snapshots use lossless compression with a combined 8 MiB budget and
50-step caps. Artwork and navigation references round-trip through history.

Checks include Python model/preview/history tests, Rust workspace tests, strict
Clippy, a private-bus VirtualDeck TestControl check, and native four-section GTK
checks at 1280×860 and 1024×768 in light/dark appearance. The checks cover
draft retention, selection, history and disconnected-state editing. Native
checks (`check_workspace_gtk.py`). The native test uses fake writes; `--render`
optionally asks the running daemon only for read-only preview images.

On this workstation, a short native Cairo-renderer check measured 168,812 KiB RSS
(~165 MiB), 0.13% of one CPU core while idle on About, and zero hidden-preview
requests. Forty history edits against a copy of the current 123,082-byte layout
had a 3.75 ms median / 3.86 ms p95 checkpoint time and used 1,921,424 compressed
history bytes. These are development samples, not V1 resource guarantees or a
long-running leak certification. Visible audio meters intentionally refresh.

Before V1: sustained live-meter profiling, large-layout stress, accessibility and
screen-reader review, minimum-window/high-DPI acceptance, additional hardware,
and broader GNOME/desktop integration certification remain release work.

### Workspace refinements

Selecting an Open application target fills the label with its application name (fitted to the current 24-character device label limit). Edit Label afterward to customize it. Layout appearance controls live under Pages settings; themes remain shared across pages. Test action appears above the selected control editor.

The GNOME menu offers Quit Decksmith: it prompts for unsaved edits, then closes the window and stops background controls. Keep editing cancels quitting. Start at login is unchanged. Closing only the window still leaves controls running. GNOME extension updates may require a new sign-in.

Themes and defaults expands inline within Pages → Layout appearance. Its collapsed summary shows the current theme and shared scope. Presets, font/color defaults, reset, and import/export share the same draft and Save and Apply workflow; undo/redo also refreshes the inline controls.

### Editor polish checkpoint

Dial controls are grouped into Assignment and Behavior, with Appearance collapsed by default. All pages / This page scope labels retain the existing inheritance behavior. The heading identifies the page and selected control; Unsaved preview distinguishes the draft from saved state. Common missing labels, URLs, application selections and invalid steps show guidance near the relevant control. About reads the installed release identifier and copies only an allowlisted diagnostic summary.

Audio application targets use a portable 32×32 PNG from locally installed artwork when selected. Refresh application icon updates an existing assignment without changing its label. Missing artwork uses the generic application symbol. PNGs are size/dimension checked, decoded at layout load and shared by physical and editor rendering; export/import and page overrides retain them.

### Stable key/dial geometry

Key buttons use a fixed-size GTK layout manager so changing child content cannot change their requested dimensions. The key toolbar retains its space while inactive for dials, and properties use a stable scrollable viewport. Native regression checks compare window size, preview/grid dimensions and positions, and property/workspace geometry after settled key → dial → key transitions in wide and compact layouts.

September 17 user acceptance: switching between keys and dials no longer shifts the content, and Refresh application icon works correctly. The installed interface checkpoint is `0.1.0-f0ae15327cb2`. Native wide/compact geometry checks and application-artwork round-trip/rendering checks passed. Sustained performance and broader accessibility/recovery certification remain future validation.

## Consistent columns — September 22

All five sections now share one persistent device preview on the right, with
settings or information on the left. Page selection stays above the preview;
clicking a key or dial opens its editor. The preview is not duplicated or moved
between tabs. Smaller windows reduce key-preview dimensions at the existing
window-width breakpoint; tab changes do not change the column arrangement.

### Font and control visibility polish

The editor now offers ten additional bundled families (Roboto, Open Sans, Lato,
Montserrat, Oswald, Raleway, Poppins, Nunito, Merriweather and Source Sans 3), plus
Viking Runes (decorative). The rune choice converts ordinary letters only when
rendering; the saved label remains readable and editable. Font menus are searchable.
See `assets/fonts/README.md` for source and license details.

The Pages section uses one outer scrolling form. Its page list has its own bounded
scroll area, while move, duplicate and delete buttons stay outside that list.
Plugin setup buttons can wrap, and the password hint is a tooltip to avoid forcing
an oversized left column. Native geometry checks include page-button bounds and
all five tabs at normal and compact window sizes.

### Validation and keyboard access

A fixed status line below the header explains why Save and Apply is unavailable:
invalid fields, an in-progress request, no changes, a locked session, or stopped/
unavailable background controls. Invalid drafts retain their explanation across
status refreshes. Common action errors identify the key and page. The button's
help text uses the same current explanation and resets when the condition clears.

Alt+1 through Alt+5 select Home, Pages, Keys & Dials, Plugins and About. Native Tab
navigation remains available, sidebar keyboard focus has a visible outline, and
keys/dials/touch regions expose their position and label to accessibility tools.
This is focused keyboard/focus acceptance, not a completed screen-reader or high-DPI
certification.
