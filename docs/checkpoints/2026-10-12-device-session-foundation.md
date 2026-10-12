# Device identity and session foundation — October 12, 2026

First part of implementation slice 3 in [ADR-0017](../adr/0017-device-scoped-actions-and-settings.md).
Base canonical revision: `32129351f1a88f7dec4e4fdc03bcdc6b54a03d96`.
The preceding model/layout foundations were published to both source hosts with
matching tree `fe719a5e007630948f0201271921ff316eda6e5b` and separate histories.

## Implemented

- Pure core device IDs and device/control context fences.
- Bounded private registry codec, Unicode names, explicit revision-checked
  enrollment and missing/ambiguous identity rejection; serial-free status.
- A supplied-adapter supervisor with independent page/brightness, connection and
  binding counters, input-edge state and local faults. Reversed reconnect order
  preserves associations; equal counters do not merge device context.
- Fair nonblocking input rounds and bounded independent drains. Lock, suspend,
  page and reconnect fences reject stale feedback and prevent held-input replay.
- Fenced key feedback and best-effort complete blanking across supplied devices.

See [implementation and remaining boundaries](../device-sessions.md).

## Qualification

Mixed Plus/MK.2 and identical-MK.2 fixtures use shared test probes around actual
VirtualDecks so tests inspect routed images, brightness and input effects after
the manager takes ownership. Cases cover registry persistence codec/privacy,
limits and invalid documents, stale enrollment, ambiguous identity, reconnect
ordering, local failure, fairness under a noisy unit, bounded reconnect drains,
page isolation, held inputs across lock/binding changes, suspend rotation replay,
stale writes and callbacks after supervisor replacement, malformed reports,
unavailable surfaces, model mismatch and pair stop/blanking.

The full all-feature Rust workspace suite passed **186 tests, zero failures and
three explicitly ignored environment/hardware checks**, including fourteen new
registry/session cases. Formatting, all-target/all-feature Clippy with warnings
denied and the hardware-enabled build passed. Cargo-deny passed advisories,
bans, licenses and sources. The device crate reuses the existing serde_json
dependency; no new package version, helper process or service was introduced.

An initial full run exposed two migration lock-contention failures. Writer scope
now explicitly unlocks on every exit, preventing a duplicated/inherited descriptor
from retaining its file-description lock. A duplicate-descriptor regression was
added; all ten migration-store tests and the final parallel workspace suite pass.
No lock, integrity check or competing-writer rejection was disabled.

## Limits and next step

This supervisor is not wired to the production daemon, renderer, editor or action
backends. Private atomic registry persistence, ID generation, document/preferences
association and live enrollment remain to be integrated. No HID opener or
ownership-lock behavior changed. Shared audio/meter/plugin/PTT coordination,
device-addressed IPC and physical pair acceptance are not proved by these tests.
The library's topology bounds do not substitute for native layout validation.

No production service, installed build, saved layout, physical device or VM was
changed. Next: shared action/hold lifecycle and device-addressed dispatch, followed
by native document/session and private persistence integration before enabling
physical or editor access.
