# Connected device execution and independent output — V1.5 development

The supplied-adapter runtime connects saved model-specific action tables, the
shared action/microphone services and independent render/I/O owners. It is an
opt-in development path. Normal V1 startup and the saved-only workspace remain
unchanged; this checkpoint does not activate production multi-device controls.

## Connected flow

Each verified attachment is wrapped in one bounded I/O owner. The coordinator
polls mailboxes, validates input with the existing model-aware manager, resolves
the current page's cached settings and submits a context-tagged effect to one
shared action service. Named audio and explicitly selected media settings compile
through the existing bridge. Injected backends must validate the fence at every
external write boundary. Unsupported providers/aliases report an error; they do
not become system-volume actions.

The runtime publishes a bounded current-context view for workers. It includes the
random startup identity, foreground revision, descriptor/control capability,
connection/binding/lock generations and the I/O owner's live epoch. Page/config/
lock/foreground/disconnect changes revoke affected contexts before mutation.
An I/O failure immediately invalidates its lease; queued effects and late feedback
are discarded. A started target reservation remains owned until its backend ends,
so cancellation cannot allow overlapping writers on the same target.

Device-local navigation and brightness retain their own destination. Ordinary
audio/media commands share the target-serialized service. Push-to-talk holds use
one microphone coordinator with typed device/control tokens; release bypasses the
ordinary action queue. A hold is renewed only while its own I/O worker has recently
completed a real input probe. A stalled worker cannot keep its microphone live
through supervisor heartbeats, and another Deck cannot renew that hold.

## Independent output

Each saved device has one sleeping render owner, and each attached/retiring
connection has one sole I/O owner. There is no backend pool or meter set copied
per device. Caps remain eight render owners and eight I/O owners, including
retiring writers, plus the shared action pool (at most four) and optional single
microphone coordinator. With two devices and two action workers, this path has
seven workers including the microphone coordinator. No CPU/memory threshold is
claimed from these structural bounds; physical resource qualification is pending.

Render requests retain one latest pending snapshot and one reply per unit. A
compiled renderer/document cache is reused until its immutable layout changes;
input dispatch neither parses JSON nor decodes artwork. Model-specific pixels
reuse the accepted renderer, including Plus touch-strip composition and MK.2
72-pixel key output. Meter-only observations render just the touch strip instead
of regenerating keys. An initial full frame is allowed to finish before bursts
of meter observations schedule their newest strip state.

The I/O owner retains bounded latest desired/sent slots for the actual surfaces:
nine on Plus, fifteen on MK.2. Each turn polls input and issues at most one output
operation. Identical pixels are suppressed; newer desired pixels replace pending
ones. A complete frame is validated and published to its mailbox atomically.
JPEG encoding and transport calls run here, away from the coordinator and other
Decks. The existing Plus changed-region transport optimization stays in its adapter.

Desired/queued/render-pending state is distinct from successfully written output.
Readiness requires the current desired surfaces to finish. New key/dial-push downs
cannot arm on a partially drawn page; an already held control may still release
during a same-binding redraw. Page changes independently clear armed controls.
An empty mailbox does not establish an input drain: only an actual empty adapter
probe acknowledges the new epoch. Received input timestamps are retained, never
manufactured for virtual fixtures. The input mailbox is capped at 128; overflow
fails that owner rather than silently dropping an edge.

A fence suppresses selected/stale output and invalidates sent caches. An already
issued USB call cannot be revoked; its completion is not credited to a later
generation, which requires a fresh redraw. Slow/retiring writers retain their
owner slot and physical claim. They cannot be replaced by another writer for the
same identity. Stop signals all owners first, then joins them and reports actual
blanking results. Every owned surface is attempted after the last in-flight write.
Rust cannot forcibly end an arbitrary hung adapter thread; bounded native I/O and
outer recovery/timeout qualification remain required before activation.

## Isolated acceptance

Build with all features, then run on a fresh private test destination:

```sh
DECKSMITH_ISOLATED_TEST=1 target/debug/decksmithd \
  --execution-lab /tmp/decksmith-connected-check-new --seconds 45
```

The supplied duration is a maximum scenario deadline (10–120 seconds), not a
minimum soak duration. This command refuses existing destinations and requires
explicit isolation. It uses only fake transports, audio and microphone adapters;
it never discovers/opens HID, starts a production bus or sends native effects.
It writes `execution-result.json` to the private test store.

The connected scenario exercises both models, shared target effects, overlapping
microphone holds, release during redraw, strip-only metering, a blocked MK.2 write
while Plus actions/display remain available, superseded pages, owned profile apply,
lock/foreground fences, reconnect and completed all-surface blanking. Separate
regressions cover cancellation after an action backend wait, explicit unsupported
provider errors, stalled-I/O microphone expiry, hidden new presses, actual drain
acknowledgement and the maximum retiring-owner budget.

## Activation gates

Connect the existing Linux provider observations and canonical app/System sounds/
default-player resolution, guarded native media/application/system/plugin effects,
input-mute policy and shared feedback. The existing named-audio adapter can be
supplied explicitly; the acceptance command does not select it. Add live roster
extension to the shared services rather than rebuilding healthy owners; the current
execution runtime deliberately requires an unchanged set of private associations
when applying a saved profile. Link the device-addressed API/editor and real
lock/suspend/reconnect supervisor, preserve exact saved layout/preferences during
migration, then repeat physical paired lifecycle/response/resource acceptance.
Do not switch an installed V1 service to this path yet.

The [October 13 qualification checkpoint](checkpoints/2026-10-13-connected-execution.md) records the final immutable source and observed limits.
