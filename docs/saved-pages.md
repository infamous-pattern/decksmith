# Saved pages and navigation buttons

Display scheduling and completion semantics have changed; see display-scheduler.md.
Earlier synchronous-rendering descriptions below document prior checkpoints.


Audio action types are now available through config/audio.json; see audio-actions.md.
The navigation-only example below remains available.

Run with OpenDeck closed:

```sh
./target/debug/decksmithd --config config/navigation.json --exclusive --seconds 60
```

The hardware feature is required. The JSON file is read and validated before opening
the device. Changes take effect on the next launch; there is no live reload or editor.
The foreground run remains bounded to 1..120 seconds. --pages-demo now uses the same
bundled HOME/WORK configuration instead of the original fixed A/B screens.

Schema version 1 accepts 1..16 pages. Each page defines a name, RGB background and
exactly eight keys. Each key has a label and one action. Navigation actions are `none` or
`{"type":"go_to_page","page":1}`. Page indices are zero-based. Unknown fields,
unknown actions, out-of-range destinations, unsupported versions and files larger
than 64 KiB are rejected. Labels use 1..8 uppercase ASCII letters/spaces with a small
built-in diagnostic font. This is not the production typography or branding renderer.

The bundled layout has copper HOME and blue WORK pages, HOME/WORK navigation keys,
and six EMPTY keys with no action. These names organize pages; they do not launch
applications or provide a complete work profile. File paths and malformed contents
are not echoed in errors. No external programs, shell commands, network requests,
brightness changes or desktop shortcuts are supported by this schema. Fixed audio
action types are documented separately in audio-actions.md.

A key down arms its configured destination; its release dispatches once. A successful
page change cancels pending actions so held-key releases cannot dispatch an old-page
binding. Swipes move through page order without wrapping. Reconnect loads the first
page and clears pending actions. Long/double presses and hold-repeat are not yet
connected to this navigation action path; the existing core resolver remains separate.

Existing worker page_changed records retain the triggering input and session, and
are emitted only after successful rendering. Failed writes invalidate the connection;
logical state is not committed, although screens can be partially updated. The
existing worker queue, synchronous rendering and shutdown limits still apply.

All 35 tests and the full quality gate passed. Added tests cover invalid configuration,
unknown fields, destination bounds, byte limits, release-only dispatch, duplicate
release suppression and cancellation on page changes. Raw live logs remain in local/.

Live validation on 2026-09-10 recorded page changes on releases of the HOME and
WORK keys. The user confirmed readable labels and correct navigation, including
swipes. A key targeting the already-active page did not emit a redundant change.
