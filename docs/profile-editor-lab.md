# Device-addressed editor and rendering lab

This developer test connects the saved-profile store to native GTK editing and
model-sized display writes through the supplied-adapter manager. It uses a Plus
and MK.2 **VirtualDeck pair only**. Normal app startup, Control1, production stores,
USB ownership, audio subscriptions and installed builds remain unchanged.

## Data flow

The separate session-bus endpoint `cc.senecal.Decksmith.ProfileLab`, at
`/cc/senecal/Decksmith/ProfileLab`, implements `ProfileLab2` methods:

- `ListDevices`: redacted identity summaries, saved selection and session states.
- `GetProfile(device_id)`: original validated V2 document, saved page and edit token.
- `SelectDevice(device_id, catalog_generation)`: save only the selection.
- `SaveProfile(device_id, token, layout, page, name, brightness)`: validate the
  destination model, reject stale edits, publish its generation and apply only
  that owner's controls/rendering. Other device drafts and contexts stay intact.
- `SaveMetadata(device_id, token, name, brightness)`: update only that device’s
  name/brightness while preserving its saved layout and current page.
- `ArchiveHistory(catalog_generation)`: explicitly archive older verified saves
  or resume its exact journal; leaves active device contexts unchanged.
- `ShowPage(device_id, token, page)`: navigate/render only that device; does not
  publish a generation for every page switch.
- `Preview(device_id, layout, page, touch)`: read-only native raster preview;
  cannot navigate, save or write either device.

Each mutation token contains a random startup identity, profile revision and
connection/binding generations. A previous process, old connection or stale draft
cannot become a new write merely because its device ID is still present. A queue
of eight requests bounds work. A timed-out request is explicitly uncertain:
reload before deciding whether to retry. Cancellation skips work still queued;
it does not claim to undo an already started filesystem/display operation.

Layout messages are bounded to 1 MiB. Private associations and global preferences
are omitted from device summaries. Unknown provider settings, including integers
larger than u64, remain raw in profile replies. There is no implicit default-device
or Control1 fallback when an ID, model, connection or service is unavailable.

## Rendering and editor behavior

The Plus reuses the accepted 120-pixel key and 800 × 100 touch renderer. MK.2 has
fifteen 72-pixel keys in a 5 × 3 grid and no touch surface/dial editor. Its key images
reuse the current style/text/artwork composition, downsampled with Lanczos3; crispness
still requires physical acceptance. Validation and native action configuration are
unchanged. New/duplicated MK.2 pages keep all fifteen positions.

Rendering validates the complete model-sized frame and exact owner context before
writing any surface. A transport failure attempts a full blank and retires only
that adapter. Other sessions remain available. Stop/expiry/signal handling attempts
to blank all supplied adapters. USB partial-write recovery still needs qualification.

The explicit lab launcher now uses the same five-section native workspace and one
reusable editor. Choose Device loads the selected profile and model geometry in
place. A bounded draft shelf retains independent layouts, undo/redo and view
state without keeping hidden windows or duplicate preview workers. Dirty Home
name/brightness settings also survive selection and unrelated layout saves.
A remote layout change keeps the old expected revision for a retained dirty draft;
unchanged saved layout values can use the fresh metadata/page context. Discarding
Home settings reloads their current saved values.

Home's name, brightness percentage/minus/plus, explicit Apply and verified-history
controls address the selected VirtualDeck. Login and Auto-Lock rows are visibly
unavailable in this test; desktop service controls, live Test Action and audio/
plugin dispatch are not enabled. The prototype does not use the production bus
or fallback to default-device/global settings. All tabs retain the preview on the
right and settings on the left. Model-sized previews fence late responses across
selection; an MK.2 never acquires dial or touch controls.

Renderers compile once per applied profile. A single bounded draft-renderer cache
avoids repeatedly decoding identical artwork. Key preview requests do not also
render the touch surface. Model-sized GTK textures use the actual dimensions.

## Running an isolated developer check

Use a disposable session bus and an explicit new private destination. The host
refuses an existing directory; it never adopts or migrates a user's store.
`DECKSMITH_ISOLATED_TEST=1` is mandatory. The service accepts a bounded lifetime of
1–3600 seconds and uses only built-in fixtures and random opaque IDs.

```sh
cargo build --workspace --all-features
DECKSMITH_ISOLATED_TEST=1 dbus-run-session -- sh -c '
  target/debug/decksmithd --profile-lab /tmp/decksmith-profile-review-new --seconds 300 &
  host=$!
  trap "kill $host 2>/dev/null; wait $host 2>/dev/null" EXIT
  python3 apps/decksmith-studio/profile_lab.py
'
```

The example launcher may need a moment for bus registration; automated acceptance
waits for the endpoint before opening GTK. Automated checks use an isolated virtual
X server, private test directories and mocked/virtual resources, not the desktop
user's production bus or physical HID. The retained generation bound remains sixteen;
stop and explicitly start a fresh test store when reached. An explicit
[verified history archive](profile-history.md) can now reclaim the active budget;
production quota, automatic retention, asynchronous disk scheduling and recovery
UI are still gates. The lab must not be enabled as the production editor.

## Remaining activation gates

Complete the archive/quota/async disk policy and production recovery UI, live
enrollment without rebuilding unrelated sessions, actual production service/login/
lock integration for the single-window selector/Home preferences, shared audio/
microphone/media/plugin routing, independent asynchronous display scheduling and
physical MK.2 HID ownership/transport. Complete matched backup/recovery, hot-plug,
lock/suspend/quit and sustained resource tests for the physical pair before deployment.
The lab proves these editor/storage/render seams; it does not qualify simultaneous
physical control or announce public MK.2 support.

## Saved-device service integration

The separate [saved-device workspace](device-workspace.md) now reuses this native
editor with real read-only discovery, explicit enrollment and persistent reopening.
It does not acquire HID handles or apply physical actions. The virtual lab remains
fresh-only and isolated on its own endpoint; normal startup still uses Control1.
