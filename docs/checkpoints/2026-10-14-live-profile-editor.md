# Live profile editor bridge — October 14, 2026

The experimental connected runtime now accepts saves from the integrated editor
through a separate, explicitly selected live endpoint. The normal V1 service,
launcher, configuration and saved-only workspace remain unchanged. This is a
development checkpoint, not production multi-device activation.

## Save and refresh boundary

One bounded worker owns file publication, preview rendering and provider
preparation. It projects the requested profile without publishing it, validates
model/page/name/brightness, prepares the action table and layout bytes, derives
the new audio/media requirements and resolves newly needed media selectors.
Player pins used by another device are retained, including unavailable pins;
saving one Deck is not permission to choose a different player for its partner.

The coordinator checks the request's current device context, revokes that Deck's
old input/action scopes and fences its input while the worker publishes the save.
The store checks the exact base generation under its existing writer lock. The
coordinator then applies already prepared controls, page, brightness and layout,
updates the provider book and releases the fence. It does not parse the document,
perform filesystem/discovery work, spawn a helper or join an old reader in this
phase. Other device contexts and I/O owners remain intact. Saved document bytes,
deliberate labels/artwork and other profiles are preserved.

The audio observer can change subscriptions without replacement. Each reply has
an observation epoch; a late reply for the previous target list cannot become a
new target's value. Empty subscriptions sleep until reconfigured. Live status is
published at most every 50 ms during ordinary polling, with immediate updates at
profile-operation boundaries. API and coordinator queues each remain bounded
to eight requests. Profile calls retain their five-second deadline and explicit
uncertain outcome; the editor does not retry a mutation automatically.

Cancellation before publication releases the edited Deck's fence without replaying
buffered input. A known published save finishes application even if the reply
deadline expires. An uncertain commit or rejected post-publication context keeps
that Deck blocked and requires review/reload; it never resumes old buffered
actions or silently applies the document to a replacement session. The other Deck
continues. Arbitrary stuck filesystem operations cannot be forcibly cancelled in
a Rust thread; bounded disk ownership and recovery/status handling still need
production filesystem-failure qualification.

## Explicit editor and test endpoints

Session-bus name `cc.senecal.Decksmith.ConnectedWorkspace`, object
`/cc/senecal/Decksmith/ConnectedWorkspace`, interface
`cc.senecal.Decksmith.DeviceWorkspace2` reuse the bounded profile API. Bus-name
replacement and auto-activation are disabled. The snapshot reports
`mode: connected-workspace`, live session readiness and `live_profiles: true`;
Test Action remains unavailable. Enrollment/live roster extension is not enabled.

Open `apps/decksmith-studio/connected_workspace.py` explicitly after starting a
test endpoint. It reuses the single editor, model geometry, independent layout/
Home drafts and device chooser, with **Save and Apply** for the selected device.
The transaction already applies its selected page; the editor does not send a
second page command that could race the post-save input drain. Explicit later
page navigation still sends a real request. Display readiness uses the actual
display acknowledgement, not merely input readiness.

`--connected-workspace-lab FRESH_ABSOLUTE_DIRECTORY --seconds 1..600` requires
`DECKSMITH_ISOLATED_TEST=1` and a private test bus. It uses two VirtualDeck actors
and simulated effects, with harmless synthetic peer inputs. The optional native
`--connected-edit-native-lab` uses the prior timed physical trial's settings and
requires both explicit physical/native test flags. It exposes the live endpoint
only after both displays and input owners are ready. Neither command changes
normal startup or migrates the production store. Physical trials still require
an independent backup/timeout/state-restoration guard covering the targets tested.

## Qualification

Immutable source snapshot
`f4637fb29e231beaa30983c4b5f3bd1561de815563ea46effd2386e55e9cd946`
passed **297 Rust tests, zero failures, four intentional ignores**; strict
all-feature/all-target Clippy, formatting and build; **203 editor** and **111
script** tests, with one conditional skip per Python suite; native saved-profile,
saved-workspace and legacy editor checks; connected execution; and all six
private-bus media cases. Current application/device source bytes match the
qualified archive. Documentation was completed afterward.

The separate native GTK live-editor fixture completed device-addressed layout
saves, a changed application selector, switching between eight-key and fifteen-key
models in one stable window, retained independent layout/Home drafts, metadata/
brightness application, model-sized preview textures and verified history
archiving. With a test-only 250 ms delay after each reservation, the partner
completed **12 simulated action replies while the other save was pending**.
Both joined blanking operations succeeded. Native screenshots were inspected;
this server's limited emoji fonts produced glyph fallbacks, while Unicode data
and saved custom labels remained intact.

Runtime fixtures verify cancellation without publication, changed-store-generation
refusal without losing another device's selection, edited hold cancellation with
the partner's microphone hold/action/context preserved, post-disconnect ticket
rejection, and refusal to change a partner's pinned media owner. Client tests check
actual display acknowledgement and preservation of explicit page requests. The
editor-level save transaction omits only its redundant post-save page command.

The qualification opened no physical handles, accessed no real audio, started no
VM and changed no scheduled checkout. Private evidence retains the archive, logs,
screenshots and complete outcomes. No physical latency or CPU/memory budget is
qualified by this fixture. The installed V1 service was not stopped or updated.

## Paired physical editor follow-up

The guarded Plus/MK.2 trial against canonical commit `076670a` verified native
editor label saves on both physical devices. The MK.2 retained its custom
"Testing" label and the Plus retained "Plus 🎵" across metadata saves. Home
settings applied independent brightness values of **30% on MK.2 and 60% on Plus**;
these are the actual saved and observed values, rather than the initial requested
40%/60% pair. MK.2 physical Next selected its blue page while Plus stayed copper.
Three successful MK.2 output volume/mute effects and one successful Plus dial
adjustment were recorded with zero failed replies and no uncertain native effects.
The user confirmed the physical levels, mute, touch strip and device isolation.
Action assignments were unchanged. The editor also populated its normal media
artwork/default appearance fields when saving the initially minimal trial layouts.

The first window did not record the reported Plus playback presses, so that result
was not counted as verified. A separate focused window saved a label-only change
through the same live SaveProfile endpoint, then recorded **four physical dial
press/release events and two successful playback effects**. The user confirmed
Brave paused/resumed using the third physical Plus dial after that save. This
focused save was prepared by the test driver; the earlier label/metadata saves
were performed in the native editor.

Both windows completed joined full blanking on both devices and exact restoration
of original output channel values/mute and the pinned Brave playback state.
The original V1 build, configuration hashes and page were unchanged, and its
connected ready display returned. The temporary editor and endpoint were removed.
Private evidence retains profiles, counts, native outcomes and restoration checks.
The existing single-device V1 service is still installed; this checkpoint does
not enable production multi-device operation or qualify latency/resource budgets.

## Remaining activation gates

The expanded live editor path still needs changes to audio/media assignments with
appropriate native-state restoration and physical acceptance. The subsequent
[saved meter bridge](2026-10-14-saved-active-meters.md) replaces the original
single-output trial meter; expanded physical assignment acceptance is still open.
Offline saves are available in the saved-only
workspace; this first connected save path requires a ready session. Add controlled
recovery for post-publication disconnect/lock and filesystem failure, live roster
extension, input mute/PTT policy, other native/plugin providers and full paired
lock/suspend/startup/migration/resource qualification before installing or releasing
this runtime. Initial editor roster/status reads also retain the earlier synchronous
lab behavior and require production UI pressure qualification.
