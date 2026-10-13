# Saved device profiles and routing — October 13, 2026

Base canonical source: `71a657ac4e011c7f9e78b1a397cf56243ae1bb3c`.
The preceding shared-worker/guardian foundation was published to Gitea as
`71a657a` and to the separate public GitHub history as `9e1ca8a`; both verified
source tree `f23307cbb6561cd5323db9997396b80476751906`.

## Implemented

- Explicit private registry/profile generations with stable random IDs,
  model-specific enrollment, private physical association and saved selection.
- Original Plus layout bytes retained; upgraded settings preserved; independent
  MK.2 defaults; opaque global preferences and per-device names/brightness/page.
- Per-profile edit revisions and serialized merge against the latest catalog,
  with stale same-device rejection, bounded checksummed publication and recovery.
- Cached model-aware key/dial/touch actions using the existing dial inheritance
  resolver, without file/JSON/artwork processing during input lookup.
- Exact source-device/context validation, local navigation/brightness, independent
  profile application and reversed-reconnect recovery using supplied VirtualDecks.

## Qualification

Twelve new fixtures cover independent metadata/revisions, legacy bytes/preferences,
private serial redaction, model/page/identity rejection, corruption/explicit restore,
interrupted publication, permissions/symlinks/hardlinks/writer locks, generation
capacity, all fifteen MK.2 key addresses, independent page/brightness, reversed
reconnect, per-owner profile fencing, lock gates, cached routing after saved-file
corruption and parity with legacy Plus keys/dials/page overrides.

The local and isolated AI-server hardware-enabled Rust workspace checks each
passed **242 tests**, with zero failures and three environment/hardware checks
ignored. Both hosts passed the complete build, formatting and all-target/all-feature
Clippy checks with warnings denied. Local dependency policy checks also passed.
The AI server ran 182 Studio and 111 script tests successfully, with one conditional
skip in each suite.

The tested source snapshot has SHA-256
`151caa594b82c8c6ee3517f7916c5c680746d4c77c8005558646231d0b6b557e`.
Private evidence is kept outside the scheduled checkout. Source modules were
compared byte-for-byte with that snapshot after qualification. The generation-capacity
fixture caught and corrected a boundary error before qualification. No dependency
or lockfile changed.

## State and remaining gates

The new store is explicit and uses test directories only; no live layout,
preferences, service, brightness or physical display changed. No physical HID or
real audio was accessed, and no VM or scheduled job was started/altered.

The native device-addressed API/editor and renderer are not wired. Before live
activation, complete retention/compaction, hot enrollment without rebuilding
unrelated sessions, shared-backend routing/canonical targets, press/runtime identity,
matched backup/recovery and MK.2 hardware/physical pair qualification. The existing
V1/Control1 path continues to reject V2 layouts. Cached action resolution is not
proof of physical MK.2 support, audio effects, UI accessibility or resource targets.
This subsequent work remains local until explicitly committed/pushed.
