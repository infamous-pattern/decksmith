# Phase 0 press resolver

Implemented in decksmith-core::trigger, using caller-supplied monotonic milliseconds.
Instantiate one PressResolver per device, control and binding context. Keys and dial
pushes share this resolver. Rotation and hardware touch gestures remain independent
raw events; this module does not invent touch motion or execute capabilities.

Defaults from v0.4: long 500ms, double window 300ms, repeat delay 600ms,
repeat interval 100ms. Bindings select double recognition and repeat behavior.
Without double binding, short fires on release. With double binding, the window
starts at first release; a second release at the deadline qualifies. A long press
fires at its threshold, suppressing short/double for that held sequence. Repeats
carry their scheduled timestamps and stop on release, including a release exactly
at a repeat deadline. Duplicate edges do not restart a held sequence.

Call advance even while input is idle. Process input edges at a timestamp before
advancing the scheduler to the same timestamp. Cancellation drops held and pending
state on disconnect, lock or binding replacement. The owner is responsible for
routing and cancelling resolvers; persistent daemon integration is still pending.
Sequence IDs are local to a resolver, not globally unique identifiers.

Invalid time or timing configuration produces typed errors. Failed calls are atomic.
A maximum of 4096 due triggers per call bounds catch-up allocations; callers must
advance in smaller increments for longer gaps. No sleeping, HID access, wall-clock
reads, or action execution occurs in this component.

VirtualDeck now provides disconnect/reconnect and read-only RGB snapshots. Disconnect
clears queued input and rejects input/writes. Reconnect retains diagnostic frames
and the monotonic clock. Device-manager lifecycle events and injected transport faults
are not implemented yet.

Validation: 8 resolver tests, 3 device unit tests and 2 cross-crate integration tests.
Integration covers shared key/dial semantics, device isolation, disconnect cancellation,
stale input disposal, frame inspection and disconnected write rejection.

## CLI replay

Run `cargo run --locked -p decksmithctl -- virtual resolve < examples/triggers.jsonl`.
The script supports input (nested InputEvent), advance, disconnect and reconnect
records. All records use monotonic timestamp_ms. Input edges update their own
control; explicit advance records service all control deadlines. Send all input
edges at a timestamp before the advance for that timestamp. EOF does not advance
time or release controls. Add explicit advance records to observe deferred actions.

This developer preset enables double recognition and repeat for every key/dial.
Control numbers 0..7 are keys and 8..11 are dial pushes. Rotation and hardware touch
are emitted as passthrough records; no new touch gestures are synthesized. Records
include the virtual device ID and resolver-local sequence. Profile/binding IDs are
not populated because the production binding manager does not exist yet.

Disconnect cancels all resolver states without firing pending actions. Input while
disconnected fails. Reconnect starts accepting input again without resetting time.
Replay exits on the first error; earlier successful lines remain on stdout. Logs
are JSON on stderr, with error_code and operation fields and no raw input payload.
