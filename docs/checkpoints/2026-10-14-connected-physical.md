# Connected physical pair checkpoint — October 14, 2026

The user authorized continuation after the Impeccable review and saved all open
edits before the timed hardware handoff. This qualifies the supplied-adapter
runtime with the physical Stream Deck + and 15-key MK.2, using shared **simulated**
level/mute effects. It does not enable production multi-device controls or qualify
native audio, media, microphone, desktop or plugin providers.

## Source and isolated checks

Snapshot `87df69d5d78d58520cbc40105bc28f8243829110c29c0ec3b162f1e8711440ab`
passes **284 Rust tests, zero failures, four intentional ignores**; strict
all-feature/all-target Clippy, formatting and build; **192 Studio** and **111
script** tests, with one conditional skip in each suite; native profile,
device-workspace and legacy interface regressions; and the existing isolated
connected-execution scenario. Source bytes used for the local hardware build
match the qualified snapshot. No AI-server VM or scheduled checkout was changed.

The new `--connected-physical-lab FRESH_ABSOLUTE_DIRECTORY --seconds 20..600`
command additionally requires `DECKSMITH_PHYSICAL_TEST=1`. It requires one unique
Plus and one unique MK.2, creates separate private test profiles, and uses the
real connected runtime, shared action queue, model renderers and sole I/O owners.
Two injected backends share one memory-only level/mute target; no native effect
adapter or microphone guardian is supplied. New regressions check the action
allowlist and run the same trial on a virtual pair through navigation, rendering,
stop and joined blanking. Fixture compilation/schema mistakes were corrected
before qualification or opening physical devices.

## Physical observations

Two bounded windows ran for approximately five minutes and four minutes
35 seconds. The user confirmed clear displays and working shared level/mute
feedback from MK.2 keys and Plus dial rotation. The first window recorded
**100 shared diagnostic writes**, successful device-tagged replies from both
Decks and no action-feedback failures. This is a workload observation, not a
resource or latency threshold qualification.

An isolated MK.2 Next press was confirmed by the user and telemetry: MK.2 page 1
(blue), Plus page 0 (copper), with only the MK.2's key surfaces redrawn. MK.2
brightness reached 55 independently. Unplugging the MK.2 left Plus actions and
display available; reconnect restored MK.2 page 1 and its retained brightness.

The reverse window recorded continuing MK.2 navigation and shared effects while
Plus was absent, followed by a second Plus connection and complete key/strip
redraw. Initial user reports did not yet have matching post-reconnect input
records; one focused check supplied that evidence: one Plus second-dial click
changed the diagnostic level to 51%, and one swipe moved Plus to page 1 while
MK.2 remained on page 0. A final leftmost-dial click changed only Plus brightness
to 51%; two second-dial press/release cycles produced mute/unmute feedback.
The second window recorded seven shared writes and no action-feedback failures.

Each window finished with two successful joined blanking results. The independent
handoff guard restored normal V1, connected/display-ready status and the original
saved page. Configuration hashes and the installed build target were unchanged.
Private evidence retains the binary hash, backups, live/final status, interaction
confirmations, source hash and restoration results. The initial transport error
on each deliberately unplugged unit was expected; its partner stayed available.

## Limits and next gate

Shared effects were simulated. Real native audio/media target resolution,
microphone policy, shared observations/feedback, application/system/plugin
effects, device-addressed API/editor, live roster extension and native lock/
suspend/recovery startup remain activation work. Repeat acceptance on the finished
native path, including saved-state migration/rollback, lifecycle and resource
measurements. This diagnostic CLI is opt-in; the installed V1 service was restored,
not upgraded. No numeric visible-response or sustained CPU/memory claim follows.

Impeccable is retained as optional review guidance, with its GTK/GNOME limitations
documented in the [toolbox](../research-toolbox.md); no tools or hooks were installed.
