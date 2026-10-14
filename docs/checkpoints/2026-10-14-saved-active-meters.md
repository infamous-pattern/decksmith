# Saved assignment meters — October 14, 2026

The opt-in connected runtime now owns its meter subscriptions instead of the
physical trial watching one fixed output. This is a development checkpoint;
the installed single-device V1 service, layouts and release archives are unchanged.

## Assignment and display boundary

The immutable provider plan caches meter targets from effective dial assignments,
including shared defaults and individual page overrides. Only each connected
touch-capable device's active page subscribes. MK.2 keys, disconnected devices and
inactive pages request no peak streams. Lock/suspend gates remove the union.
Existing implicit dial assignments keep their System sounds semantics. Explicit
default output and System sounds targets remain separate, as do legacy master
volume key actions. No saved labels, artwork or assignments are rewritten.

One shared worker deduplicates at most 32 logical targets across the bounded
device roster. The existing peak helper samples/publishes at 20 Hz; the coordinator
reads its latest state at most every 50 ms. Source discovery and process work stay
off the input coordinator. Repeated targets share the same physical peak stream;
the total remains capped at the previous four-dial path's worst-case **64 streams**.
If a whole application's stream set cannot fit, that target is unavailable;
partial application peaks and fallback to another output are not used.

Each changed union receives a new observation epoch. The helper is asynchronously
restarted for that union, so even an A → B → A change cannot reuse the earlier
publication. There can be a brief unavailable gap for retained targets during this
restart; incremental helper reconfiguration is not implemented in this slice.
Snapshots expire after 400 ms, and absent measurements stay distinct from measured
silence. UTF-8 JSON output and a bounded 32 KiB reader buffer cover the shared
target set. Empty subscriptions sleep on a condition variable until a change or
shutdown, rather than periodically polling for work.

The runtime filters peaks back to each device's current page before rendering.
Changed levels dirty only its touch strip; they do not repaint MK.2 keys or
invalidate another Deck's action context. Profile saves clear the edited device's
old levels and request an immediate subscription refresh. The worker is joined
on runtime shutdown before the caller starts native-state restoration. Peak data
is transient; no PCM recordings, media files or network exports are created.

## Qualification

The immutable snapshot
`8d2e4961a91fdaa1cc8378400f5d2ed83cb65041b632007298159fd833cb87ef`
is based on canonical commit `be83208`. Qualification forces a workspace rebuild
from the archived source when reusing the AI server's Cargo dependency cache.
It passed **301 Rust tests, zero failures and four intentional ignores**, strict
all-feature/all-target Clippy, formatting and build; **205 editor** and **111
script** tests, with one conditional skip per Python suite; native saved-profile
and workspace checks; connected execution; all six private-bus media cases;
the native live-editor fixture; and the legacy native editor check. Application
and device source bytes match that archive; documentation was completed afterward.

The Rust fixtures cover two VirtualDeck Plus units and one MK.2, including separate
targets, current-page overrides, partner context preservation, strip-only dirty
state, disconnected-target removal, lock clearing and joined shutdown. Additional
checks cover missing versus silent measurements, duplicate targets, stale
session/target snapshots, idle-worker wake/join, explicit master/System sounds
pixel isolation, and whole-application admission under the 64-stream budget.
The existing legacy control parity test also passes.

A separate **four-second read-only native helper check** requested 32 logical
targets and received **80 valid frames**. All 29 deliberately missing targets
remained unavailable, output measurements were present, and output identity,
raw channel volume values and mute were unchanged. The helper was terminated
and reaped. Brave produced no measurable stream during this sample, so this is
not active browser-meter acceptance or a resource soak.

Private evidence retains the source archive, complete qualification results,
native-helper outcome and earlier test-harness failures. This does not qualify
CPU/memory budgets or a new installed build.

## Paired physical output/app acceptance

A guarded physical Plus/MK.2 window against the qualified source displayed saved
output, Brave and deliberately missing-app dial assignments. The user confirmed
moving output/Brave signal arcs, separate app/output volume and mute behavior, and
an unavailable missing target with no borrowed values. The recording captured
**48 successful Plus audio effects and 18 MK.2 effects**, with zero failures or
issued-effect uncertainty.

The Plus's saved blue-page override replaced Brave with the output assignment.
Physical page states and the owned helper's arguments confirmed the union changed
from three targets to two, with the duplicated output sharing one subscription;
MK.2 stayed on copper. Unplugging only the Plus released the peak helper entirely,
while MK.2 continued native output actions. Reconnecting the Plus restored its blue
override; an isolated physical swipe back to copper restored the Brave subscription
and moving meter. The user confirmed clear displays and partner isolation.

Both devices completed joined full blanking. The guard verified exact original
output and Brave stream channel values/mute, pinned playback state, and unchanged
normal V1 configuration, build and saved page with a ready display. App recovery
was identity/membership pinned: fixture checks prove a replaced stream is refused
and an unchanged app requires no write. No input device was assigned or adjusted.
The installed runtime remains the original single-device V1 service.

## Physical System sounds isolation

A subsequent guarded Plus/MK.2 window used the unchanged microphone-qualified
source and saved System sounds, output and missing-app assignments. A generated,
quiet sixty-second ordinary audio tone (`media.role=music`) moved the output arc
without System sounds activity. A separate sixty-second event-role tone moved both
output and System sounds arcs. The user confirmed both phases on the physical
Plus, with the missing app unavailable and both Deck displays clear.

An independent read-only native meter recorded 1,208 ordinary-tone frames: 757 had
positive output activity and none had positive System sounds activity. The event
phase recorded 1,207 frames, including 709 with positive System sounds activity.
The fixture verified the actual stream roles; these companion readings support
native role isolation but are not recordings of the runtime's own meter stream.
After the tones ended, no event stream remained and all 100 follow-up frames
reported System sounds unavailable rather than borrowing output activity.

Generated audio and companion monitor processes were bounded, terminated and
reaped. Both device owners blanked successfully. The event-role record's raw
volume/channel map/routing/mute was unchanged; the guard and separate read-only
verification confirmed exact event/output state, pinned playback, configuration,
installed build and original ready Plus page. No System sounds volume/mute action
was requested, so this qualifies metering, not new event-volume action behavior.
The installed V1 service remains unchanged. Tone waveforms were generated test
signals; no microphone or playback audio was recorded.

## Next acceptance gates

The [paired physical microphone trial](2026-10-14-native-shared-microphones.md#paired-physical-microphone-acceptance)
accepted both input meters, native mute/PTT and exact state restoration. Output,
app, page override and System sounds metering have now passed their bounded
physical checks. Complete paired lock/suspend/startup/recovery, migration and
resource qualification before production activation. The shared worker's ordinary
subscription behavior and simulated checks do not replace physical lifecycle
acceptance.
