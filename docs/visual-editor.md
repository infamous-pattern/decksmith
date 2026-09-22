# Visual page and key editor

The four-section [native workspace](workspace-ui.md) now hosts this editor.

Open Decksmith from the application menu and choose **Keys & Dials**. The editor
loads the daemon's active layout into a separate draft. Select a page and one of
its eight keys to edit the label, choose text or the approved Maker's Mark artwork,
and assign an action. Supported actions are no action, page navigation, volume
up/down, mute toggle, and a nonzero volume step from -20 through 20 percent.
Page names and key labels use 1–24 ASCII letters (uppercase or lowercase), numbers,
or spaces. Entered case is preserved. Existing saved names are unchanged.

**Add page** creates an eight-key page, up to the schema limit of 16. Page names
can be changed without breaking numeric navigation destinations. **Discard changes**
restores the loaded or last saved draft. Closing an unsaved draft asks whether to
keep editing, discard and close, or save and close. The window stays open while a request is pending.

**Save and Apply** validates and saves one personal layout called **My layout**,
then requests a redraw from the device owner. Built-in Audio controls and Navigation
remain available. Saved artwork refers to the existing approved source; the editor
does not import external images yet. Existing audio-dial configuration is retained.
Page deletion/reordering, background-color editing, multiple named profiles and
arbitrary action plugins are outside this checkpoint.

## Persistence and interface

Control1.GetLayout returns the active version-1 JSON configuration. SaveLayout
validates the entire supplied JSON with the existing bounded Rust parser before
writing $XDG_CONFIG_HOME/decksmith/layout.json (or ~/.config/decksmith/layout.json)
using a temporary file and rename. It then queues the custom layout and saves the
active-layout preference. SetLayout additionally accepts custom. Restart loads the
saved custom layout when selected in control-panel.json.

These steps are not a transaction across disk, the worker and hardware. An error
can occur after the custom file was saved or after the active layout changed;
a timeout does not prove that nothing happened. The editor retains the draft on
error and does not retry automatically. Successful acknowledgement means the
worker accepted the desired frames, not that all USB writes have completed. The
panel's display status reports when rendering has settled.

The Python GTK client never opens HID. It uses the existing trusted per-user D-Bus
API, with blocking calls off the GTK thread. This extends the interim client in
ADR-0014; it is not the complete planned Rust Studio.

## Verification

The full Rust quality gate passes: formatting, Clippy with warnings denied, all
48 tests, and dependency advisory/license/source checks. Three Python model tests
cover isolated drafts/discard, page creation and navigation references, validation
and saved-state tracking. Python syntax checks pass. A snapshot of the actual GTK
editor was rendered and visually inspected. Live D-Bus rejected an invalid layout
and reported a connected device with its existing brightness preference retained.
Physical editing/save feedback is recorded below when confirmed by the user.

A custom layout was saved through the live editor. An automated live check captured
GetLayout, restarted the daemon, and confirmed the recovered configuration was
identical, custom remained selected, brightness remained 45%, and the connected
device reached display_ready. Physical action confirmation remains pending.

## Page selector crash correction

Switching pages previously replaced the page selector model inside its own
selection notification. GTK could repeatedly notify/rebuild without returning
to the event loop. The editor now keeps both page-name models stable, updates
names only when changed, and ignores unchanged or invalid selections.

The real GTK regression check (`timeout 20s python3
apps/decksmith-studio/check_editor_gtk.py`) exercises 60 HOME/WORK transitions,
draft retention, renaming, adding a page and discarding. It passes with the fix;
the original version fails to return and is terminated by a five-second watchdog.
This check uses a fixture and never saves a layout or opens HID.

## Device follows editor page selection

Choosing an existing page now calls Control1.ShowPage(byte) through the device
worker queue. The saved page appears on the device; unsaved key edits remain draft
until Save and Apply. New pages prompt for saving before device selection. Saving
also selects the page being edited instead of leaving the device on HOME.
Selection is transient and does not change the persisted layout or brightness.
The worker rejects disconnected-device and out-of-range page requests.

The full quality gate and GTK regression check pass. Worker tests cover WORK,
invalid-index rejection without changing the current page, and return to HOME.
The live ShowPage(1) request completed and the device logged display_settled page 1.

## Label entry usability correction

The user reported that typing `Test` on WORK key 7 disabled Save and Apply.
The restriction was uppercase-only validation, not a two-character limit.
The editor now converts typed ASCII letters to uppercase in key labels and page
names, preserving the existing renderer/schema limits. Invalid length, punctuation
or blank labels still disable saving with a visible reason and button tooltip.
Page-selection completion no longer replaces a draft validation error message.
The real GTK check covers WORK key 7, lengths 1–8, `Test` becoming `TEST`, invalid
input and recovery; the three draft-model tests also pass.

## Editor follows the displayed device page

GetStatus now includes active_page, a zero-based index only when the connected
device's display queue has settled; otherwise it is null. The editor reads that
value on opening and follows changes through the panel's existing status polling.
Device-originated selection updates never send ShowPage back to the device.
Draft edits survive selection changes. Synchronization is skipped while a request
is pending or the active layout no longer matches the draft's source layout.
The existing editor selector still requests a device page explicitly.

The GTK regression verifies opening on WORK and following a reported HOME change.
All 48 Rust tests, three Python draft tests and the full quality gate pass. Live
ShowPage requests to HOME and WORK were followed by matching settled active_page
status; the device was left on WORK. Physical swipe feedback remains a user check.

## Page-name labels follow renames

Renaming a page updates every key label that exactly matches its previous name,
across all pages, including keys with artwork or non-navigation actions. Custom
labels such as BACK remain unchanged; action destinations retain their numeric
page references. The selected key's label field, grid, page selector and destination
names update together. Changes remain draft until Save and Apply and can be
reverted with Discard changes. Four model tests and the GTK regression check pass,
including cross-page label propagation, selected-field refresh and discard.

## Persistent navigation-label links and stale-label repair

Keys can now persist follow_page_name=true for go_to_page actions. Rust resolves
such labels from the destination page on load/save and rejects the flag on other
actions. The draft recognizes labels matching their destination name as linked;
renames preserve these links. Typing a different custom label removes the link.
This complements exact-name propagation and supports recovery of stale linked text.

The user's existing saved TACOS/TUESDAYS layout still contained HOME/WORK labels.
A backup was retained in local/before-page-name-repair.json, and the four matching
navigation keys were repaired through SaveLayout with their destination links set.
Other labels/actions were preserved. The active page was restored. An older open
editor must be closed/reopened to avoid resaving its pre-repair draft.

The full quality gate passes with 49 Rust tests; five draft-model tests and the
GTK regression check pass. Tests cover stale linked text, further renaming,
persisted Rust resolution and rejection of incompatible actions.

## Letters, numbers and spaces in names

The user-entered `Page 1` was rejected because of the digit, not the space.
Both Python validation and the Rust parser now accept ASCII digits as well as
uppercase letters and spaces; the existing renderer already supports digit glyphs.
The editor continues to uppercase typed letters. The eight-character total limit
includes spaces. Linked navigation labels accept the same character set.

All 50 Rust tests, six draft tests and the GTK regression check pass. The GTK check
covers `Page 1` becoming `PAGE 1`, names with spaces at the limit, and rejection
beyond it. The updated daemon was activated with the saved layout retained.

## Next and previous page actions

The action menu now includes Next page and Previous page. Serialized actions are
next_page and previous_page. On key release they resolve relative to the current
page in stored page order. Last-to-first and first-to-last wrap; a one-page layout
produces no navigation. Swipe left/right use the same resolution and wrap behavior.
Direct Go to page remains available. No existing key assignments are changed.

The full quality gate passes with 51 Rust tests. Tests cover release-only behavior,
relative direction on a three-page layout, wraparound and one-page no-ops for
buttons/swipes. Six Python model tests and the real GTK action-selection regression
pass. The updated service was activated with the saved layout preserved. Physical
button/swipe feedback for the new behavior remains a user check.

## Page management

The Pages sidebar lists names, assigned apps and the Default marker. Select a
page to rename it, assign an app or choose it as default. Add creates an unsaved
page; Save and Apply publishes draft changes to the device.

Move up/down changes page order. Go-to-page indices are remapped so actions keep
pointing to the same pages. Duplicate inserts an independent copy after the current
page with a unique COPY N name. Its self-navigation follows the copy; other targets
retain their destination. The existing 16-page limit applies to add and duplicate.

Delete is unavailable for the final page. A confirmation reports the number of
incoming navigation links on surviving pages; confirming clears those actions to
No action and removes their page-name link flag. Surviving destinations are
remapped. Labels are retained so cleared keys remain recognizable for reassignment.
All operations affect the draft until Save and Apply; Discard restores it.

The draft retains each page's original saved index, allowing editor/device
synchronization while the draft is reordered. New and duplicated pages have no
saved device counterpart until applied. Device changes to a deleted draft page
are ignored until apply/discard resolves the difference.

Nine Python model tests pass, including reorder/remapping, duplication independence
and self-links, deletion references, single-page protection and discard. Real GTK
checks cover move/duplicate controls and delete cancel/confirm paths. Python syntax
and diff checks pass. The Rust runtime is unchanged from the previous 51-test gate.
The user's existing saved layout and key assignments were not modified.

## Interactive navigation keys

Clicking a key with Go to page, Next page or Previous page now navigates the editor
and uses the existing device page-selection request. Draft page origins preserve
the correct device destination after reorder; unsaved new pages remain draft-only.
An Edit keys switch disables click navigation so navigation buttons can still be
selected and customized. Other key actions select the key for editing; clicking
them does not execute audio or other system actions at this checkpoint.

Nine model tests and the GTK regression pass. Real clicked signals verify that
HOME/WORK navigate while Edit keys selects without navigating, including completion
of asynchronous preview requests. Python syntax and diff checks pass.

Application/website launch actions and automatic icons are now described in
[launch-actions.md](launch-actions.md). This supersedes the earlier action-menu
scope and adds portable embedded icons while preserving the existing editor modes.

Layout export/import and previous-version recovery are now available. See
[layout-packages.md](layout-packages.md) for file format, draft behavior and checks.

## Mixed-case key labels and artwork captions — 2026-09-11

Key labels now preserve uppercase and lowercase ASCII letters, digits and spaces
(up to eight characters). Page names retain their existing uppercase behavior.
The device's pixel font includes distinct lowercase glyphs; labels are no longer
silently uppercased by the editor. A custom-cased navigation label detaches from
page-name following unless it exactly matches the destination name.

Label position offers Hidden, Top, Middle or Bottom, including keys with Maker's
Mark artwork and custom/automatic images. Image captions have a dark backing band
and light text. Text-only keys also support the position controls. Existing keys
without label_position retain previous behavior: image-only artwork and centered
text-only labels. The editor preview shows the placement; GTK uses its UI font,
while the device uses the existing pixel-font style with lowercase additions.
Appearance packages preserve label_position and mixed-case text.

The full quality gate passes with 54 Rust tests, including mixed-case parsing,
distinct glyphs, top/middle/bottom pixel regions and unchanged Hidden output.
Eighteen Python tests and the real GTK regression pass, including case retention,
position selection and package roundtrip. Live read-only validation accepted
YouTube with bottom position. The daemon was updated with the saved layout
unchanged and the prior page restored. Physical placement confirmation is pending.

## Transparent captions and text colors

Label background now offers Dark (the existing default) or Transparent. Text color
provides Default cream, White, Black, Red, Orange, Yellow, Green, Blue and Purple.
These apply at every visible label position. Transparent leaves the image pixels
unchanged except where the text itself is drawn. Existing keys without these
optional fields retain their original style. The editor preview and appearance
packages carry both settings. Text-only keys retain their page background.

The full quality gate passes with 55 Rust tests, including pixel checks for
transparent backgrounds and selected colors. Eighteen Python tests and the GTK
regression pass, including controls and appearance-package roundtrip. Live read-only
validation accepted a transparent yellow YouTube caption. The updated daemon was
activated with the saved layout unchanged and its active page restored. Physical
style selection remains available for the user to test after reopening the panel.

## Caption preview priority correction

The user confirmed the yellow/transparent caption was correct on the physical key
but wrong in the editor. The later-added general caption CSS provider overrode the
separate color/transparency provider at the same priority. The caption-specific
provider now uses application priority + 1. An actual saved-layout window reported
rgb(255,225,60) for Key 6 and its snapshot visibly showed yellow YouTube text with
no dark label backing. The GTK regression now asserts the resolved widget color,
not only draft values, and passes. No device/configuration change was necessary.

## Smooth physical key typography

Key captions now use anti-aliased Liberation Sans Bold, automatically fitted to
the key width. This replaces the diagnostic pixel font for keys, including mixed
case and artwork overlays. Positions, colors, transparency and hidden labels are
preserved. ADR-0015 documents font provenance, licensing and renderer dependencies.
The physical touch strip still uses its previous renderer.

All 55 Rust tests and the complete quality gate pass, including anti-aliased edge
pixels, transparent-background blending and position checks. A key-resolution PNG
was rendered using the production code and visually inspected. The updated daemon
redrew the device without changing its saved layout. The user confirmed the yellow
YouTube caption on physical Key 6 looks much better and is clean/readable.

## Drag keys to swap positions

With **Edit keys** enabled, drag a key onto another key on the current page.
The two complete key definitions swap, preserving actions, page links, artwork,
and label formatting. The moved key becomes selected. Use **Save and Apply**
to update the device, or **Discard changes** to undo the draft changes.

Drags from outside the editor, repeated drops, and drops after the draft is
replaced or the page changes are rejected. Navigation mode keeps its existing
click behavior. The GTK regression exercises the connected prepare/drop signals,
checks complete key preservation, and verifies stale-drop and navigation-mode
rejection. Physical pointer interaction remains the user acceptance check.

## Copy, paste and clear keys

Enable **Edit keys**, select a key, and use **Copy key**, **Paste key**, or
**Clear key** beneath the grid. Copy stores the complete key in this editor;
select a destination on this page or another page and paste to replace it.
Each paste is independent and retains artwork, label styling, and actions.
Copied page links track destination page reordering and linked-name changes;
pasting a link to a deleted page is rejected.

Clear replaces the selected key with a blank, inactive key. Paste and Clear
only change the draft: **Save and Apply** updates the hardware, while
**Discard changes** restores the saved layout and empties the key clipboard.
The clipboard is local to this editor and is not shared with other applications.
Replacing/importing the layout also starts a fresh clipboard.

Validation includes GTK button activation for cross-page paste, copy independence,
clear and discard; model tests cover copied links after page rename/reorder/delete.

## Longer key labels

Key labels accept up to 24 ASCII letters, digits and spaces, preserving case.
For example, `Volume Up` is valid. The physical key font automatically shrinks
as needed. Page names retain their separate eight-character limit. Copy, Paste
and Clear preserve any validation explanation instead of hiding it with a
success message; unchanged layouts explain why Save is disabled in its tooltip.

Multiword key labels that exceed the available width at the default font size
wrap at a space into two balanced lines before shrinking further. Both lines
move together for top, middle and bottom placement and retain the chosen text
color/background. Single words shrink without splitting. Touch-strip labels
retain their single-line layout. GTK captions also wrap at word boundaries.

## Simplified editor organization

The editor opens in Edit keys mode. Label, Action, and any required action fields
remain visible. The collapsed **Appearance** section holds artwork, label
position, text color, and label background. **Dials…** remains in the header;
the header menu holds Export, Import and Restore previous. The menu beside the
page selector holds page rename, move, duplicate and delete. Copy/Paste/Clear
appear beneath the grid only in Edit keys mode. The content scrolls on smaller
screens, with Save and Apply always in the header. Scoped preview colors take
precedence over desktop button themes so keys retain their actual layout colors.

## Individual key background colors

Under **Appearance → Key background**, choose Page color (the default), white,
black, red, orange, yellow, green, blue or purple. Text-only keys render this
solid color beneath their label. Full-key artwork covers the color; choose
Text only to reveal it. Label background remains a separate control.

The optional `background_color` appearance field travels with copied or moved
keys and through layout packages. Clearing a key resets it to the page color.
Existing layouts retain their page backgrounds. Tests verify rendered pixels,
unchanged neighboring keys, JSON/package preservation and the GTK control.

## Undo and Redo

Use the header arrow buttons or Ctrl+Z / Ctrl+Shift+Z to undo and redo draft
changes. The editor retains up to 50 steps, including key settings, swaps,
paste/clear, dial settings and page operations. Nearby typing in the same key
text field is grouped. A new edit after Undo clears the redo branch.

Undo and Redo do not write to the device: Save and Apply is still required.
Saving, discarding, importing or reopening starts a fresh history. Undo/Redo
clears the internal key clipboard so copied page references cannot become stale.
Page destinations and the device-to-draft page mapping are restored together.
History controls are disabled while a save or other device request is pending.

## Stable preview geometry

The current integrated preview uses fixed 108×108 key content and zero theme
padding. Its hardware column is clamped to the approved 554-pixel casing width
(508-pixel inner area), so changing action settings cannot stretch the device.
Remaining window width goes to the settings panel. Keys, strip, dials and selection
outlines retain the same proportions when selecting website/application actions
or resizing the window. A native allocation check verified identical geometry
across 27 selections, all existing key actions, dial settings, a long application
name, and 1006×820 / 1250×900 windows. Both website and application states were
visually inspected against the approved website-view baseline.

## Non-interrupting panel status refresh

The main panel polls status on a separate bounded worker without disabling
controls or setting the user-action busy flag. Polls do not overlap. Starting a
user action invalidates any older poll response so it cannot overwrite the
result of the action. Controls are initially disabled only until status arrives;
actual service/settings changes retain their deliberate busy state. A live GTK
check verified Edit layout stayed enabled across repeated device status polls.

## Shared and per-page dials

Choose a dial or touch-strip section in the integrated editor. Shared settings apply
to every inheriting page. Customize for this page copies that dial's current settings
into a page-specific override; edits then affect only this page. Use shared settings
removes the override and follows the latest shared value. Other dial slots keep their
own inheritance. Save and Apply, undo/redo, duplicate/reorder pages and layout packages
include these choices. A shared edit does not overwrite customized pages.

The optional page field `dial_overrides` contains exactly four slots: `null` means
inherit and a full dial definition means override. Existing top-level `dials` remain
shared defaults. Layouts without the new field retain their existing behavior;
legacy `audio_dial` defaults are synthesized only when needed. This increment does
not implement sticky keys, automatic application switching, or multiple devices.

2026-09-16: User accepted the Pages sidebar appearance. Page-name entry now
preserves mixed case and digits, including names such as Joplin Notes 2.

The user confirmed mixed-case page names work and approved their appearance.
The editor now requests a 1440×900 window bounded by its monitor, with the
compositor constraining it to the usable desktop. Wide windows place key/dial
settings beside the physical preview. At 1340 px or narrower, settings appear
above the preview so basic editing is immediately discoverable; scrolling may
be needed to see the entire device. The Pages sidebar remains alongside.

The user approved the wider three-column opening layout. Assigning an action
to an empty key now fills its label with a readable action default. Changing
the action updates the previous default label; other custom text is retained,
including for media actions. Page-navigation defaults use the destination name.
Legacy layouts have no label provenance, so a label matching its prior action
default (ignoring case) is treated as automatic. Clear-key placeholders become
visible when an action is assigned. Changes remain drafts until Save and Apply.
