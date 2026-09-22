# Shared physical and editor touch-strip rendering

September 13, 2026. The editor uses the daemon's read-only Control1.PreviewTouch
method with (layout JSON, page byte), returning (RGB888 bytes, source mode).
Every frame is exactly 800×100×3 bytes. No Python reimplementation of captions,
bars, values, indicators or artwork is used.

For the active unchanged layout/page, the daemon returns Display's last successfully
sent touch frame. Drafts use Pages::touch_image, the same renderer called for page,
audio and brightness hardware updates, against a cloned device-state snapshot.
The method does not enqueue device commands, save configuration or change routing.
Unknown draft targets show -- until applied; disconnected previews clear audio
state. Source modes are live, draft, draft-missing-target and offline.

The editor refreshes at most ten times per second on a separate single-worker pool,
coalesces pending requests, rejects replies from older draft revisions, and updates
only the image/status note. Incomplete action fields retain the last valid layout preview, with a visible
last-valid notice, while Save and Apply stays disabled. Its live values continue
refreshing until the draft is valid again. Request failures retain the last image
with a paused/retrying notice, then recover automatically. No prior image means
an explicit unavailable/loading placeholder; retained imagery is never labeled
as current while a request is failing.
Gdk.MemoryTexture preserves the RGB buffer; the UI scales it to the approved strip
geometry, with transparent clickable sections and an editing selection outline.
Hardware JPEG encoding/display color and refresh timing can affect photographed
appearance; parity is of the renderer's source pixels, not camera measurements.

Validation: full Rust format/Clippy/workspace tests and dependency audit passed;
53 Python tests passed. Rust tests compare all 240,000 RGB bytes with VirtualDeck
writes for available/unavailable/muted targets, volume/brightness, navigation,
legacy layouts and an unsaved label; API tests verify sent-frame return without
queuing commands. Python tests round-trip texture pixels, reject stale results,
coalesce refreshes and verify clearly marked last-valid/error recovery. The exact
Clear key → Open website → empty URL → valid URL sequence is regression-tested;
URL validation remains enforced without blanking the unrelated touch-strip image. GTK tests verify the image hit
regions and stable requested enclosure width alongside existing editor interactions.

Live check: connected Stream Deck returned a 240,000-byte live frame; the updated
service preserved saved layout, brightness and active page. The live GTK check
rendered that frame and an unsaved label, verified clickable regions and confirmed
GetLayout was unchanged. No volume/mute changes were made for testing. A final
visual comparison on the physical device remains user confirmation.

Local preview: local/live-touch-editor.png. Reopen Decksmith to load the new editor.
The updated daemon is already running.

## Audio identity and mute styling — September 13 follow-up

Audio panels now reserve space beside the title for original vector category icons.
Explicit active-port, form-factor, or icon metadata identifies speakers/headphones;
input and application targets have microphone/application symbols. Unknown device
types retain a neutral audio waveform rather than guessing from the display name.
Unavailable targets use subdued symbols and --. Application symbols are generic,
not downloaded logos. Device brightness retains its existing display.

Muted audio panels add a red border and a crossed-out target symbol. Refreshes track
external mute and metadata changes using the existing state worker. Active-device
indicators remain visible inside the border. The subsequent
[live meter increment](audio-meters.md) uses the rounded blue bar for signal activity, with the numerical percentage showing configured volume.

Validation: Rust formatting, full-workspace Clippy/tests and dependency audit passed;
55 Python tests and the native GTK interaction check passed. Renderer tests cover
all five symbol categories, reversible mute styling, and physical/preview pixel
parity. The live editor displayed the new symbols and accepted an unsaved label
without changing GetLayout. Muted appearance was visually inspected in an exported
renderer fixture; live audio settings were not changed for that test. Physical
readability and live mute appearance still warrant user confirmation.

## Live meter follow-up

Meter levels travel separately from volume state and saved layouts. The shared
renderer caches static artwork and updates only the meter track on level changes. Physical
writes remain latest-only and one image per scheduler turn. The editor now requests
at most ten frames per second, with the existing single-request coalescing and
stale-draft rejection. Device disconnection clears meter state in draft previews.

The API caches one validated draft and reuses static strip artwork; unchanged live
requests return the sent frame directly. The Python client extracts the packed
byte array without per-byte object conversion. Meter validation passed 40 daemon
tests (one ignored), 60 Python tests, full-workspace checks and the dependency audit.
