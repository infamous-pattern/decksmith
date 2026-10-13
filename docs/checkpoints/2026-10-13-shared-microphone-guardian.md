# Shared microphone guardian — October 13, 2026

Base canonical source: `cd6969e7ad64ff9eba593a40975cdd97ae8daf50`.
This extends the local [worker bridge](2026-10-13-shared-worker-bridge.md)
and [shared-action/hold core](2026-10-13-shared-action-foundation.md).

## Implemented

- One shared microphone service and at most eight per-target guardians;
  overlapping Plus/MK.2 holds use the same guardian for the same microphone.
- Release, lock/disconnect/binding invalidation, lease expiry and explicit mute
  bypass the ordinary action queue. One device cannot refresh another's lease.
- Native helper watchdog, EOF/finally mute, bounded retries and acknowledged
  writes, while preserving the existing single-device helper entry point.
- Current-revision Live acknowledgement, confirmed-mute/child-exit handoff,
  guarded release/repress, failure isolation, consumed recovery presses and
  explicit retry after bounded mute-recovery exhaustion.
- Fixed target/failure budgets, nonblocking bounded pipes, independent owned
  process groups, confirmed-termination-before-replacement and bounded shutdown.
- Monotonic sampled-time serialization for concurrent calls and ticks.

[Lifecycle details](../shared-action-lifecycle.md) distinguish desired state from
completed audio writes and describe ownership, limits and remaining integration.
Factories/guardian adapters and authority callbacks are trusted non-panicking
built-ins; this does not add a generic plugin protocol or permissions boundary.

## Qualification

Sixteen Rust fixtures cover overlapping holds, delayed closure/repress, stale
acknowledgements, per-owner lease expiry, explicit mute, watchdog exit, bounded
retry, hung helper termination, out-of-order sampled ticks, capacity, shutdown,
blocked-press consumption, target failure isolation and failure-consumer pressure.
Harmless real child processes exercise nonblocking pipes, EOF acknowledgements,
malformed replies and reaping; they do not access a microphone.

Eleven Python guardian tests cover the legacy interface, discovery/release/error
mute, write acknowledgements, bounded retries, a broken reporting pipe and both
shared CLI modes with actual newline output. Audio commands are mocked.

The full local Rust workspace passes **230 tests, zero failures and three
explicitly ignored hardware/environment checks**. Formatting, all-target/
all-feature Clippy with warnings denied and the hardware-enabled build pass.
The complete Studio and installer-script Python suites pass: **182 and 111 tests**
respectively. The same source passes all six checks on the AI server's Fedora
Rust 1.98.1, also **230 passed, zero failed and three ignored Rust checks**.
Each server Python suite skipped one conditional check (gettext compilation and
the optional locally staged plugin-runtime fixture); those are not counted as
passed acceptance.
The desktop uses Rust 1.97.1. The server's Python suites ran with an isolated
Xvfb/private bus session; this is unit coverage, not native desktop acceptance.
Existing dependency advisories/bans/licenses/sources policy checks pass locally.
No dependencies or lockfile changed. Private evidence stays outside the scheduled
checkout, and jobs/timers were unchanged. The tested source archive has SHA-256
`a64f0c0b00f4643248d6345b56bd8e179d7cad7efc0e23d0abae74c371f37674`.

## State and next step

No normal startup or editor/API path selects these services. The installed app,
saved layouts, physical displays, audio levels and mute state are unchanged.
No native guardian was attached to real audio, and no VM or physical HID was
opened during these checks. Desired/fixture state does not close physical PTT,
actual audio, latency or resource gates.

Next compile bindings into the shared services, resolve canonical targets and
persist device/profile association; connect the device-addressed API/editor and
MK.2 hardware/rendering before a backed-up physical Plus/MK.2 acceptance build.
A native microphone toggle must resolve its final state before canceling holds;
blindly toggling after cancellation could unmute it. Shared observations/meters,
guarded media and provider permissions remain separate. This checkpoint accompanies source publication; release packages and the
production daemon are not updated by these checks.
