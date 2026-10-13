# Shared action/hold foundation — October 13, 2026

Base canonical source: `cd6969e7ad64ff9eba593a40975cdd97ae8daf50`.
This is the next simulation slice of [ADR-0017](../adr/0017-device-scoped-actions-and-settings.md),
following the [session foundation](2026-10-12-device-session-foundation.md).

## Implemented

- Full device/runtime/foreground scope, instance-safe request/reply identity and
  bounded fair pending/active scheduling with cross-device target serialization.
- Compatible same-direction scalar combining, retaining reversals/discrete
  actions, earlier receipt and intervening same-target barriers.
- Explicit checks before handoff and feedback, stale-result suppression and
  retention of started target reservations until their external writer ends.
- One desired microphone coordinator with separate owner heartbeats, overlapping
  hold leases, fresh-down requirements, urgent latest mute updates, write-boundary
  validation, guardian failure isolation and instance/revision-safe replies.
- Explicit shared budgets; no new dependency, worker thread or helper startup.

See [lifecycle details and required native adapter](../shared-action-lifecycle.md).

## Qualification

The new manager/VirtualDeck fixtures cover mixed Plus/MK.2 and identical MK.2
owners; raw key edges; overlapping holds and last release; per-owner expiry;
disconnect/page/lock/foreground cancellation; old-connection release after reversed
reconnect; stale guardian replies; target limits; backwards time; queue saturation
and urgent release; start/target ordering; rapid adjustment combining and reversal
at a clamped limit; source-specific feedback/failure; stale running reservations;
stop/terminated workers; capacity, nonce and replacement-instance rejection.

The full all-feature Rust workspace suite passed **203 tests, zero failures and
three explicitly ignored environment/hardware tests** on both the Fedora desktop
(Rust 1.97.1) and the isolated AI-server snapshot (Fedora Rust 1.98.1). Seventeen
new shared-lifecycle fixtures pass. All-target/all-feature Clippy with warnings
denied, formatting and the hardware-enabled build pass on both hosts.
The existing Cargo-deny advisories/bans/licenses/sources policy passes locally;
there are no dependency or lockfile changes.

The AI server's newer Clippy identified inherited constant-sized pixel-loop
warnings. The image/feedback scanning paths and matching assertions now use
fixed-size chunk slices and direct array membership, preserving their existing
pixel/remainder semantics without allocations. Existing image/theme/feedback
tests pass on both toolchains. No warning was disabled. The initial failed
lint outcomes and successful corrected snapshot remain in private evidence.

The final tested archive has SHA-256
`e71f7ad176c95d7cd0b7121508d6ec62196ae477ad43d684f0c2cb66febb9b66`.
The source snapshot was copied to a separate private manual-test directory;
the server's scheduled checkout, jobs and timers were unchanged. No VM was started.

## State and next step

The Plus (`0fd9:0084`) and MK.2 (`0fd9:0080`) were both observed through a read-only
USB inventory. No physical device was opened, drawn on or attached to a VM.
The installed Plus service, layout, audio and media settings are unchanged.
The foundation has not been wired into production execution; desired mic states
and fixture scalar effects are not real audio acceptance. This adds no physical
MK.2 or multi-device support claim.

Next is the bounded shared native worker/guardian adapter and validated binding/
target compilation, preserving independent watchdog mute and authoritative
write gates. Then integrate document/preferences/registry persistence and the
device-addressed API/editor before physical Plus/MK.2 handoff. The runtime nonce,
press-sequence producer, alias canonicalization, service-replacement drains,
live microphone/system audio, meter sharing and resource checks remain gates.
