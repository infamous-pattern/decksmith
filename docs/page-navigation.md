# Live page navigation demonstration

Display scheduling and completion semantics have changed; see display-scheduler.md.
Earlier synchronous-rendering descriptions below document prior checkpoints.


This describes the original A/B checkpoint. The current saved HOME/WORK layout
and button actions supersede it; see saved-pages.md.

`decksmithd --pages-demo --exclusive --seconds 60` (hardware feature) writes two
local demonstration pages. Copper A is page 0; blue B is page 1. All eight keys
and the strip show a large page letter. Left swipe advances to B; right swipe
returns to A. Swipes at the corresponding boundary do nothing; pages do not wrap.
Taps, dial events and key edges remain logged inputs and have no desktop effect.

This explicit demo mode writes displays. Ordinary `--hardware` remains read-only.
OpenDeck must be closed; exclusivity is still a caller assertion. No brightness,
reset, application launching or system shortcuts are involved. The final page
remains after exit; no previous display contents are restored automatically.
The images are functional diagnostics, not replacement Maker's Mark branding.

The device-owning worker maps horizontal gestures into a local page action and
renders that page synchronously. A `page_changed` record contains the original
input, session and zero-based destination, and is emitted only after all writes
succeed. A failed update drops the connection and emits `disconnected`; a failed
initial render emits `render_failed` (repeated failures are suppressed). Opening
retries at the worker's existing interval. Logical page state changes only after
successful output. USB writes are not atomic across screens: partial output is
possible on failure. A new connection repaints page A before `connected` is emitted.

There is no configurable binding store, general action dispatcher, live key-press
resolver integration, render queue, frame budget or swipe animation yet. Slow
synchronous image writes can delay input reads. This checkpoint verifies the first
input-to-visible-action path, not production performance or desktop integration.

All 33 tests and the full local quality gate pass. Tests cover navigation boundaries,
ignored taps, write failure without logical commit, worker swipe routing, page reset
on reconnect and failed initial rendering without announcing a connection.

## Live validation — 2026-09-10

The physical Plus produced repeated page changes in both directions, with successful
display writes. Boundary swipes remained ordinary input records without a page
change. The user confirmed both directions visibly work correctly on keys and strip.
Live reconnect with the page renderer remains untested (worker reconnect was verified
previously in read-only mode). Raw capture is ignored under local/pages-live.jsonl.
