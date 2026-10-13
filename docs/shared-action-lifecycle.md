# Shared action and microphone lifecycle foundation

These pure core components extend ADR-0017's third slice. They are compiled and
tested with manager/VirtualDeck fixtures; the production daemon still uses its
existing Plus worker and microphone guardian. No service, audio setting, saved
layout or physical display is changed by this implementation.

## Scope and authority

Action scopes carry the device context, foreground revision and a host-supplied
random runtime nonce. Opaque request IDs also contain the dispatcher instance
and sequence, so recreating a dispatcher cannot confuse a previous reply with a
new request. Microphone updates carry their runtime and coordinator instance
plus a target revision. No process ID, serial or display label generates identity.

The host supplies the authoritative current-scope predicate, including model
capabilities, readiness, connection, binding and lock state. The fixtures use
`DeviceManager::current`, runtime and foreground checks. Unavailable lock
reporting must be mapped to blocked state. Selecting and starting an action do
not execute it; the backend adapter must recheck authority at its external-write
boundary if it waits or makes multiple writes. These are lifecycle fences, not
plugin permissions, authentication or a public wire contract.

`TargetId` is an immutable, bounded, debug-redacted resource key. Structural
validation accepts up to 512 bytes without control characters; it does not prove
a real device/app exists. The host must canonicalize aliases such as default
microphone before scheduling. Two names for the same actual resource must not
create separate reservations or microphone guardians.
Changing a resolved target or selected action (including a dial stack entry)
must fence its compiled binding; do not silently reuse a current scope for a
different meaning.

## Bounded scheduling and feedback

There are at most eight configured owners, 32 pending actions overall, 16 pending
per owner and four selected/started reservations. These are scheduler limits,
not a claim that a backend worker pool is implemented. No timer/thread/helper is
created by the scheduler. Device ownership uses the same eight-device bound.

Owner selection is fair. Same-target requests preserve submission order across
devices, including a request behind another target in an owner's queue. Only
one selected-but-not-started request per owner prevents backend start-order
inversion. Once started, independent targets can progress separately. A stalled
started write keeps its target reservation even after its device scope becomes
stale; release requires completion or confirmed worker termination. Never release
it merely because a page changed, allowing another writer to overlap it.

Only adjacent, compatible, same-direction bounded scalar adjustments combine.
They require identical scope/control and canonical target, with no intervening
same-target request from another device. They preserve the earliest receipt time
and return the existing request ID with a submission count. Reversals remain
separate because clamping at 0/100 changes their result. Toggle, set and activation
requests do not combine or retry. Generic plugin settings/effects must not be
classified as scalar adjustments without a verified adapter.

Scopes are checked on submission, selection, immediately before handing off and
on completion. Stale completions free only their own reservation and suppress
feedback. Already issued external effects are not undone. Failure feedback stays
with its source scope; a failure does not globally disable unrelated targets.
Stop rejects future work and cancels pending/selected requests, retaining started
reservations until their writers have actually ended.

## Microphone holds

One coordinator maintains leases for all owners. A hold identifies its full scope,
typed key/dial-push control and genuine down/press sequence. There is at most one
active lease per owner/control. Lease heartbeats are per device; another healthy
device cannot extend an expired/disconnected owner's lease. The current fail-safe
500 ms timeout is retained. Expiry, invalid scope or explicit release removes only
that lease; the target remains desired Live while another valid owner holds it.
The last removal changes its desired state to Muted.

Consumed down sequences remain remembered after release, expiry, explicit mute
or guardian failure. Duplicate acquisition does not refresh the lease. A heartbeat
cannot recreate an expired hold. A fresh down is required to try again. The
device input layer must suppress held controls across fences; general gestures
and producer press-sequence generation still need native integration.

There are at most eight target rows, including unacknowledged mute writes, and
152 remembered owner/control slots (eight units times the conservative 19-control
bound). A rejected target-budget acquisition consumes its down, preventing a
still-held key from opening a mic later when space becomes free. Target rows are
retired after a successful latest mute acknowledgement. These bounds do not
increase the meter helper's separate four-visible-target limit.

Latest desired microphone updates bypass the ordinary action queue and remain
pending until acknowledged. `prepare_update` checks current owner leases before
a guardian write, and acknowledgement checks them again. Late Live/Muted replies
cannot replace newer desired state or a replacement coordinator. Failed writes
cancel that target's holds and leave a latest Muted request pending; the eventual
guardian adapter needs bounded mute retry/backoff. There is no automatic Live
retry. Backwards monotonic time and revision exhaustion fail closed. Stop leaves
all known targets desired Muted.

Desired Live is not evidence of actual microphone state. The existing guardian's
independent watchdog and mute-on-exit behavior must be retained by the native
adapter. An explicit mute during holds cancels those holds. A microphone toggle
must be compiled to an explicit final mute state before cancellation changes the
observed state; blindly toggling after cancelling/muting could reopen it.

The coordinator should live with the shared service owner, not be recreated on
ordinary provider failure. Replacing the shared owner requires a new runtime
nonce plus device input fences/drains before accepting fresh events. Callback
instance tags alone cannot prove that an arbitrary supplied token came from a
new physical down. The host adapter must enforce this sequence.
Quiesce old owned writers/guardians before starting replacements; instance tags
suppress stale feedback but cannot stop an external write already in progress.

## Next integration

The abstract effect classes and target keys do not compile saved actions or
execute existing helpers. Before enabling these in the daemon, connect validated
built-in definitions/settings and canonical targets; generated runtime nonce and
press IDs; authoritative shared gates; bounded off-HID audio/media/plugin workers;
trigger identity and validated action payloads retained with the request;
one fail-safe guardian per mic target; deduplicated observations/meters; verified
worker termination; and independent status/rendering feedback. Then connect
native layout/preferences persistence and the device-addressed API/editor.

Private live registry activation, MK.2 HID/JPEG orientation, simultaneous physical
acceptance, accessibility and sustained resource qualification remain separate.
The attached Plus and MK.2 were listed read-only by USB IDs during this work;
visibility does not qualify hardware support.

See [ADR-0017](adr/0017-device-scoped-actions-and-settings.md),
[device session boundaries](device-sessions.md) and the
[qualification checkpoint](checkpoints/2026-10-13-shared-action-foundation.md).

## Bounded host worker bridge — October 13

The daemon now includes a dormant shared execution service, compiled by the
hardware build and exercised with controlled backends. It is not selected by any
production startup or editor/API path. One service owns one to four executor
threads; it does not copy the legacy Plus worker per device. Pending and in-flight
limits remain the core's 32/16/four limits, with a 64-result consumer budget.
A stalled consumer stops the service rather than silently dropping results or
allocating indefinitely. Idle workers sleep on a condition variable, with no
periodic polling or helper startup.

The compiled payload remains with its source request through scalar combining.
Only named input/output volume, named mute and an explicitly selected MPRIS player
can compile in this slice. Volume turns combine to at most 100 percentage points;
mute and media operations keep their discrete order. Default devices, application
stream aliases, System sounds, plugins and device selection remain rejected until
canonical resource resolution covers their overlap. The built-in identity
projection remains separate from execution and permissions.

Workers check authoritative scope before dispatch and give the backend a fence
for every external write. Consumer receipt checks the scope again, preventing
an already-completed reply from updating a different page or binding. The supplied
authority callback must be bounded, non-panicking and independent of the service
mutex. Publish new session authority, release its own lock, then invalidate the
service; never call service methods while holding an authority lock used by workers.

The initial native adapter invokes only the fixed `pactl` executable with literal
arguments. Named input/output volume performs a fresh read, computes a clamped
level and rechecks the fence just before the single write. It uses no default
fallback and handles a combined adjustment without splitting it into many writes.
Output mute similarly reads and writes an explicit final state. Commands have
500 ms timeouts, bounded read output and owned process-group cleanup. Input mute
is deliberately unsupported by this native adapter until cancellation is connected
to the shared microphone guardian. Guarded native media, app/System-sounds targets,
provider permissions and shared observations are also unfinished. Compilation of
an action is not a claim that every backend supports it.

Backend implementations must finish or terminate owned I/O before returning;
errors are not retried. A provider panic unwinds and destroys that backend before
releasing its target reservation. Other workers can continue; losing the last
worker stops the service. If backend destruction itself panics, termination cannot
be proved: the service stops and retains that target reservation. This contract
is for trusted built-in adapters, not an in-process third-party plugin boundary.
Shutdown cancellation is nonblocking; joining the bounded workers belongs after
HID ownership stops, outside the input loop.

Eleven controlled fixtures cover parallel independent targets and same-target
serialization; cancellation after backend waits; stale replies at consumer receipt;
combined scalar payload/timing and discrete barriers; full queues; active-writer
shutdown; provider failure isolation; final-worker failure; failed destructor
reservation retention; result backpressure; and native read/write plans with no
audio access. Native subprocess launch, actual audio and physical dual-device
acceptance remain separate gates. See the
[worker checkpoint](checkpoints/2026-10-13-shared-worker-bridge.md).

## Shared microphone guardian bridge — October 13

The dormant daemon-side microphone service now connects the core leases to one
process per named microphone, with at most eight targets and one coordinator
thread for all device owners. It has no normal-action queue: acquire/release,
per-device heartbeat, scope invalidation and explicit mute change the latest
bounded state and wake the coordinator. Native process operations run outside
the state mutex and outside HID ownership. Idle or exhausted-retry owners sleep
on a condition variable; active guardians are serviced at a 20 ms cadence.

Each fixed Python guardian retains the independent 750 ms missing-heartbeat
watchdog and attempts named-source mute on EOF, exit, or discovery failure.
New shared modes report `live`/`muted` only after the corresponding audio command
succeeds. The existing one-target legacy entry point remains supported. Pipe
output and heartbeats are bounded/nonblocking in the coordinator; one stalled
helper cannot block releasing another microphone. No default microphone fallback
is allowed. `--mute` never attempts unmute and has three bounded command attempts.

Live acknowledgements are accepted only for the current desired revision and
valid leases. A verified mute plus successful child exit can satisfy the latest
Muted request for that same exclusively owned target; it never acknowledges a
newer Live request. A release/repress cannot start a new guardian until the old
one exits. Unexpected watchdog exit or protocol/heartbeat failure consumes the
old holds, requests mute and blocks reopening until mute recovery is confirmed.
A down received during recovery is consumed; it cannot silently reopen the mic
when recovery succeeds.

Closing helpers have a 2.5 s deadline. Only after terminating their owned process
group may the coordinator start a standalone mute recovery process. There are at
most three recovery-process attempts, spaced by 250 ms; each process has its own
three bounded mute command attempts. Only an explicit mute/retry request resets
an exhausted process budget. Failure records are bounded to eight targets; a
stalled failure consumer stops new Live ownership rather than growing memory.
Unknown writer termination prevents replacement of that target. Shutdown cancels
holds, attempts mute and joins outside HID ownership, with a ten-second cleanup
budget. Failure or missing acknowledgement is reported, never counted as proof
the microphone muted. The callback and guardian/factory adapters are trusted,
bounded, non-panicking built-ins; they are not an in-process plugin interface.

Concurrent APIs and coordinator ticks serialize their sampled monotonic times at
one high-water mark. This avoids treating an older sampled tick arriving after a
newer heartbeat as a backwards clock. The pure core's backwards-time rejection
remains unchanged. Authority callbacks use the same nonblocking/no-lock-inversion
contract as the shared action service.

Controlled fixtures, harmless child processes and mocked helper commands test
these boundaries without accessing real audio. Production startup still uses the
legacy Plus controller. Input mute/toggle binding compilation, canonical default/
app targets, persistent device association, native device API/editor and physical
Plus/MK.2 acceptance remain gates. See the
[guardian checkpoint](checkpoints/2026-10-13-shared-microphone-guardian.md).
