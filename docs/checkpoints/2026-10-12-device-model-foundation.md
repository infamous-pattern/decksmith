# Plus/MK.2 model foundation — October 12, 2026

## Implemented scope

First implementation slice of [ADR-0017](../adr/0017-device-scoped-actions-and-settings.md),
based on source revision `55caa4bd061d7fae14f98cc8f8a03b5d3665cb91`.

- Pure core model descriptors distinguish Plus and 15-key MK.2, their key-grid
  and pixel dimensions, supported operations and optional touch capabilities.
  Only known models construct descriptors; raw imported geometry cannot create
  unbounded frame allocations through this API. Upstream HID types stay private.
- Typed key/dial/touch addressing is available without the old dial-index + 8
  convention. Production dispatch conversion belongs to a later slice.
- `DeckDevice` requires a descriptor and derives compatible geometry from it.
  The current physical adapter, display cache, service wrapper and test doubles
  supply their existing Plus descriptor. Hardware selection is still Plus-only.
- VirtualDeck accepts a model; its default remains Plus. It uses bounded key
  collections and lazy RGB buffers, reusing capacity on valid image updates.
  MK.2 has 15 72-pixel keys, brightness and no dial/touch surface. Legacy geometry
  reports absent touch dimensions as `(0, 0)`; descriptor capabilities use `None`.
- Blanking uses descriptor dimensions and skips absent surfaces, still attempting
  every available surface after an error and preserving brightness.

## Validation

Seven new model-contract tests cover all 15 MK.2 key frame writes and ordered
press/release edges; Plus boundary/gesture compatibility; rejection of missing
controls and wrong image dimensions without mutation; key-only blanking including
a failed key write; FIFO/queue-overflow atomicity; distinct control addressing;
and independent frames, brightness, queues, reconnect and resolver cancellation
for mixed-model and identical-MK.2 simulation pairs.

The complete all-feature Rust workspace suite passed: **153 tests passed,
zero failed, three explicitly ignored**. The ignored tests require a native JPEG
diagnostic VM, disposable live microphone or isolated session-bus environment;
they were not counted as acceptance. Formatting and all-target/all-feature Clippy
with warnings denied passed. Cargo-deny 0.20.2 passed advisories, dependency bans,
licenses and sources with the existing policy and locked dependencies. Its
advisory database was placed in a writable test-evidence directory after the
default cache lock was read-only; no check or policy was disabled. No dependency
or lockfile change was needed.

All tests used source-built binaries/fixtures; no physical device was opened or
production service restarted. No VM was started, package installed, desktop
build changed, layout migrated or release published for this slice.

## Limits and next slice

This establishes descriptor and simulation contracts, not physical MK.2 support
or a multi-device daemon. Synthetic corner pixels preserve the RGB buffer
contract; they do not certify physical JPEG orientation. Pair tests use independent
VirtualDecks and explicitly cancel their individual resolvers; they do not prove
the future manager, shared microphone coordinator or page/action routing.

Next: model-aware saved layouts and action identity/compatibility, with lossless
Plus import, independent MK.2 defaults, bounded V2 packages and interrupted
migration/matched binary-configuration rollback tests. Device registry/identity,
multi-device manager/API, integrated editor, shared services and physical Plus/MK.2
acceptance follow. Existing Plus production controls and published support scope
remain unchanged.
