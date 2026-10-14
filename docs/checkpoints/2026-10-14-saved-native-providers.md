# Saved native provider bridge — October 14, 2026

The opt-in connected runtime now derives audio/media provider requirements from
each device's immutable saved document. It uses the same effective key/dial
routing as the editor, including inherited dials and page overrides. It does not
rewrite labels, artwork, assignments, layout bytes or the installed V1 service.
The prior real output/Brave physical acceptance remains a separate checkpoint;
the expanded app/System sounds path needs its own physical acceptance.

## Scope and ownership

One bounded plan covers up to eight saved devices, 32 distinct audio selectors
and eight media selectors. One read-only observer deduplicates all audio targets,
using the existing helper in batches of at most 13. State expires after 500 ms;
reads taking that long publish unavailable rather than old data. The coordinator
copies a snapshot only on a new observation/expiry or changed device context.
Volume/mute changes redraw only a Plus strip unless a key's existing live state
also changes. The current trial still uses one meter for its selected output;
meter subscription wiring for all active saved targets remains work.

Exactly one native executor serializes logical audio selectors, including two
application-property aliases that resolve to the same streams. App selectors
retain the existing property-match semantics, including simultaneous streams.
System sounds uses the existing persistent event-role record and updates only
matching event streams. Older untargeted master-volume actions retain their
default-output meaning through an internal compatibility selector; this does not
add a new default-output option to the editor or make System sounds control it.

The new app/event helper completes discovery and server/stream-identity reads,
then asks the host to authorize each individual write. The host rechecks the
device/context fence after those reads. An abort stops before the next write;
already completed writes are not undone. Stream index/serial replacement,
missing targets and changed server identity fail without selecting another app.
The prepared event-role write does not reconnect after authorization. A lost,
malformed or timed-out write acknowledgement is uncertain, is never replayed,
retires the helper and blocks the logical target for the trial.

Media family selectors resolve to an exact running instance and unique bus owner
during setup, outside input/render ticks. Automatic selection can retain the
existing validated player-preference record when its owner is still present.
Pins stay stable through pause; missing explicit apps remain unavailable rather
than switching players. Commands still check ownership, capability and the device
fence immediately before their bounded write. All aliases of an owner share its
media reservation. No titles, URLs or metadata are requested by this adapter.

## Saved changes and activation limits

The connected runtime accepts label/appearance/page changes whose provider set
is unchanged, preserving other device contexts. A change to the provider set
returns `native_plan_refresh_required` before revoking/applying that document;
the host must prepare a new plan off the coordinator before activating it.
Offline editor saves remain available through the existing saved-workspace path.
The live API/editor refresh bridge has not been activated by this checkpoint.

Input mute/push-to-talk policy, live roster extension, other native/plugin actions,
active meter subscriptions, provider refresh and device-addressed API/editor
wiring remain. Qualify the expanded native path on both physical Decks, then
repeat paired lifecycle, migration/rollback and resource checks before replacing
daily V1. A successful fixture or build is not physical or production acceptance.

## Verification record

Immutable source snapshot
`e7bc3669857c29e94fce6bae6e3ed32f260dcfd1fe4ec16a3b269efb73112808`
passed **293 Rust tests, zero failures, four intentional ignores**; strict
all-feature/all-target Clippy, formatting and build; **201 editor** and **111
script** tests, with one conditional skip per Python suite; saved-profile,
device-workspace and legacy native editor regressions; and the isolated connected
execution scenario. Current application source bytes match the qualified archive.

Six private-bus media cases passed through the actual adapter/service: explicit
success, unsupported capability, replaced owner, uncertain write with blocked
retry, saved family selection and automatic selection. Pure fixtures verify
retaining a paused preferred owner, missing explicit apps without fallback,
effective shared/page-override requirements and immutable custom labels/layouts.
Guarded audio fixtures cover multi-stream mute, clamped/coalesced adjustment,
server/stream replacement, post-discovery abort, event-role-only writes,
legacy output behavior and non-replayed uncertain acknowledgements. The event
writer's prepared mode is checked to refuse reconnection after authorization.

The AI-server run opened no physical handles, accessed no real audio, started no
VM and changed no scheduled checkout. A malformed-reply test exposed that a
unit enum variant accepted extra fields; explicit empty struct variants fixed
that protocol validation before the final qualification. Private evidence retains
the immutable archive, logs and result records. No physical latency, CPU/memory
budget or production acceptance follows from these tests.
