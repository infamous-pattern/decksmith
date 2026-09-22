# Application and website actions

Reopen Decksmith, turn on Edit keys, choose a spare key, and select Open application
or Open website. Application choices come from visible installed desktop entries;
selecting one tries to capture its themed icon. Labels remain editable. Website
URLs must use http:// or https://. Confirm the URL with Enter/the entry's apply
control, or leave the field, to try fetching its icon. Wait for the icon to appear,
then Save and Apply. A late icon result remains an unsaved draft change if saving
has already completed. Text remains available when no icon can be loaded.

Artwork now includes App or website icon alongside Text only and Maker's Mark.
Changing a website URL clears the previous icon. A completed fetch does not override
an artwork choice made while it was running. Retrieval reads the supplied page for
icon links, then tries /favicon.ico, without browser cookies or credentials. It is
best effort: each network operation has a two-second timeout; page data is capped
at 256 KiB and icon data at 64 KiB. Failed or unavailable icons do not prevent using
the launch action. Icons are converted to 120x120 PNG on a dark background and
embedded in the layout so rendering does not need a live website connection.

Physical keys launch on release through the existing bounded action worker. The
worker invokes a fixed Python/GIO desktop helper with literal arguments and a
two-second completion timeout; configurations do not provide shell commands.
Application targets are desktop IDs, resolved against installed visible desktop
entries. Website targets are passed to the default browser through GIO. A helper
success means the desktop accepted the launch; it does not certify the app/page
finished loading. A timeout can occur after launch, so requests are not retried.
The action worker is shared with audio; launches can briefly delay audio actions
but do not run inside the HID loop. Editor key clicks continue to navigate/select;
non-navigation actions are executed using the physical device at this checkpoint.

The schema adds open_application {desktop_id}, open_website {url}, application_icon
artwork, and optional icon_png bytes. Layout input is now capped at 1 MiB (including
CLI/custom-layout reads); each icon is capped at 64 KiB and must decode as a 120x120
PNG. Decoded RGB is cached at layout load. Existing saved assignments are retained.

## Verification

All 53 Rust tests and the full quality gate pass, including launch target validation
and icon persistence/decoding. Eleven Python tests pass, and actual GTK checks cover
app picking, website validation and existing editing/navigation workflows. Files
and the user's Gitea website were opened successfully through the helper. Both
icons were retrieved, converted and visually inspected. The daemon was updated and
the existing layout retained. Physical button launch confirmation remains pending.

## Physical launch confirmation — 2026-09-11

After restarting Decksmith, the device connected with the saved custom layout,
45% brightness and PAGE 1 displayed. The user confirmed that physical Key 6 opened
YouTube and physical Key 7 opened Snapshot (org.gnome.Snapshot.desktop). This
completes the pending physical application/website launch check for this checkpoint.

## Crisp artwork and custom images — 2026-09-11

Website discovery now ranks declared icon sizes, inspects decoded dimensions,
selects the largest ICO frame and prefers sources at least 120px wide/high. Invalid
candidates no longer prevent fallback. It reads up to 1 MiB of HTML and 256 KiB per
icon, with a 10-second overall budget checked between requests and a two-second
per-request timeout. The budget can overrun by one in-flight operation. Website
artwork is rendered once from its source using Lanczos resampling and preserved
proportions. A small-source warning recommends choosing a larger original.

Every key now has Choose image, opening a source preview with Fit/Crop, artwork
size (48–120px) and dark/black/white backgrounds. PNG, JPEG, WebP and ICO are
supported; imports are limited to 8 MiB and 16 megapixels. The preview warns for
small sources. Use artwork changes the draft; Save and Apply updates the device.
Only the final 120x120 PNG is embedded. Reopen the original file to make further
sizing changes without resampling an already-reduced saved key image. Refresh
website icon retries discovery. A manual selection is protected from stale fetch
results. Pillow is now a runtime dependency (12.3.0 already installed on this host).

YouTube's supplied 144x144 PNG replaced the formerly enlarged tiny favicon. The
new rendered key was visually inspected and saved to the existing YouTube key,
preserving its URL/action and all other keys. The prior layout was backed up in
local/before-youtube-artwork-upgrade.json. Fourteen Python tests pass, including
high-resolution candidate ranking, invalid-image fallback, largest ICO frame and
fit/crop geometry. Real GTK checks exercise the custom preview controls and Apply;
syntax/diff checks pass. Rust code is unchanged from the prior 53-test gate.

The user confirmed the replacement YouTube artwork looks crisp on the physical
Stream Deck key.
