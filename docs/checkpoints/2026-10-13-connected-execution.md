# Connected shared execution/output checkpoint — October 13, 2026

The user authorized shared action execution and independent display updates.
This slice connects the saved-profile manager to the shared action/microphone
workers and bounded per-device render/I/O ownership. It uses supplied adapters;
normal production V1 and the saved-only workspace remain unchanged.

See [the runtime design and activation limits](../device-execution.md).

## Qualification

The immutable final source snapshot
`449ad451ab336c780ca127099ba369d021776e17053fc0285c5e8c43751a5aa2`
passed AI-server qualification: **282 Rust cases, zero failures, four intentionally
ignored environment/hardware/stress cases**; strict all-target/all-feature Clippy,
format and build; **192 Studio** and **111 script** cases (one conditional skip
each); native profile, saved-device and legacy workspace checks at 1920 × 1200;
and the isolated connected-runtime executable scenario. No VM, physical HID
handle, real audio target or scheduled checkout was changed.

Nine new regressions cover five device-owner cases and four connected-runtime
cases. The owner fixtures verify independent progress during a blocked write,
latest-frame replacement, actual drain acknowledgement, owner-local transport
failure, retiring writers consuming the fixed budget, and release of an existing
hold during redraw while preventing new hidden presses from arming.

The connected pair runs actual normalized input through cached model bindings,
the shared scheduler, injected effect/guardian backends, native model renderers
and independent output owners. It verifies shared named-target volume effects,
overlapping microphone holds, a strip-only meter update without key writes,
continued Plus input/display while MK.2 output waits, page supersession, owned
profile apply, lock/foreground fences, reconnect and completed all-surface blanking.

Separate cases prove an old started action cannot write after its own page changes,
unsupported providers report an explicit error without changing audio, and an I/O
stall cannot renew its owner's microphone hold. The healthy device continues
executing actions; the expired hold stays muted after I/O resumes.

The executable scenario exited successfully and recorded four injected audio
effects with device-tagged feedback. It does not execute native audio/media,
application/system/plugin commands or unmute a real microphone. No physical
receipt/visible-response timing or native idle/loaded resource claim is inferred.
Private evidence retains source hashes, logs and the final scenario outcome.

## Corrections during this slice

The first fixture used a dial push-to-talk target that disagreed with its audio
assignment, and then empty labels; normal layout validation rejected both. The
fixture now uses valid independent named assignments and labels. A scheduler test
incorrectly assumed the final key-write index instead of allowing round-robin
ordering; it now checks every latest surface/value. An initial meter-only fixture
had no available target state, so its observation could not change the rendered
strip; the corrected fixture supplies valid state and verifies unchanged key writes.
These rejected/failed fixtures are not acceptance evidence.

Readiness now also includes pending render intent, not only already queued I/O.
Foreground changes fence existing held inputs while keeping pages. Attachment
validates the exact discovery revision/assignment before spawning an I/O owner.
Profile apply refuses changes to the runtime's private association roster.
Transport panic/failure invalidates the live lease before cleanup, and blanking
results are reported only after the sole owners finish.

## Remaining activation work

Connect canonical app/System sounds/default-media target resolution, shared Linux
observations/feedback and guarded native effect adapters; integrate live roster
extension, device-addressed API/editor and native lock/suspend/recovery startup.
Repeat physical paired input/action/display/lifecycle and resource qualification
on the Plus/MK.2. Native I/O timeout and matched migration/rollback gates remain.
The new source is not installed or released by this checkpoint.
