# Physical MK.2 and Plus/MK.2 trial — October 13, 2026

The user authorized physical MK.2 testing followed by simultaneous testing, and
confirmed saved edits before the handoff. A bounded source-built test host uses
separate private test profiles and the new model opener/manager. It never opens
production configuration itself, exposes a production bus or executes desktop,
audio, microphone, media or plugin actions. Its only effects are device images,
device brightness and device-local page navigation.

## Qualification and recovery preparation

The immutable host snapshot
`983f9c01396b4fdd581b3caa8de117811e8615ac7762b2f53f7020f88c1205f6`
passed AI-server checks: 268 Rust cases, no failures and three previously ignored
environment/hardware cases; build, format and strict Clippy; Studio 191 and scripts
111 cases (one conditional skip each); and both native workspace regressions at
1920 × 1200. Two new fixtures check bounded model test documents, safe action
whitelisting, owned pages/brightness and report privacy. Local focused checks,
build and strict Clippy also passed.

Private handoff tooling captures the current page/build, hashes and backs up the
configuration with private permissions. It starts a bounded user-service test
with an independent timeout and stop-post restoration. A real disposable timeout
service verified stop-post execution. Controlled failure checks verified cleanup
and refusal to hand off changed saved state. Successful termination blanks the
test adapters, releases their shared lease, restarts the unchanged production
Plus service and restores its previous page. No startup settings are changed.

## Physical results

**MK.2-only:** all fifteen keys produced matching down/up events (21 each in
total). The user confirmed upright numbers in three rows of five, asymmetric
corner colors, green press feedback and brightness decrease/increase. The opened
handle passed serial verification and model-specific image writes. The user also
confirmed unplug/replug recovery, but that first recording did not corroborate a
new connection; the paired trial below provides separate recovery evidence.

**Simultaneous Plus/MK.2:** both devices opened under one cooperative lease and
showed independently addressed copper/blue pages. The user confirmed page changes
affect only the selected device, independent brightness and correct Plus strip
updates. The recording contains 24 matching MK.2 down/up events, nine matching
Plus down/up events and 79 Plus dial reports. A precise physical dial check was
user-confirmed as a 1% change.

With the MK.2 deliberately left disconnected, telemetry showed its transport
failure/disconnected state while the Plus stayed on connection generation 1 and
continued receiving page keys and dial input. The user confirmed that behavior.
On replug the same MK.2 profile identity recovered, reached connection generation
3, cleared its transport error and retained brightness; the user confirmed page/
display/key recovery while the Plus continued working. Subsequent user page
changes mean the later telemetry snapshot alone is not a visible page-restoration
measurement. Reconnect page acceptance includes the user's observation.

Both runs ended successfully. Independent recovery checks confirmed a ready
production Plus display, the original current page/build and unchanged saved
configuration. Test blank writes succeeded; physical blank-screen observation
after cleanup remains a separate human check. Private reports retain snapshots,
input counts, user confirmations, backups and recovery records. No VM or scheduled
checkout was changed.

## Further automated and physical gates

A bounded maximum-roster stress fixture exercises eight virtual Decks, held input
across failure, fresh releases, reversed enumeration, stale feedback rejection,
owned pages and unchanged other-device contexts. Its explicit 120-second AI-server run passed **51,341 reconnect cycles**. A
120-second read-only desktop follow-up recorded 24 ready/connected samples and
unchanged configuration; the normal production Auto-Lock became active while
the user was away. A default-feature test also passed; the final import cleanup
only gates hardware-specific time imports. Final focused host fixtures and strict
Clippy passed. Private evidence retains these outcomes. These checks do not prove
physical paired lifecycle, visible response timing or resource thresholds.

## Resumed physical checks after the user returned

A fresh guarded paired trial verified the reverse failure direction. The Plus was
left unplugged; telemetry showed its disconnected/transport-error state while the
MK.2 retained connection generation 1 and produced 23 matching key presses and
releases. On replug the Plus reached connection generation 2 and cleared its error;
the user confirmed that its keys/entire strip returned while MK.2 remained working.

The user confirmed Plus-only swipe navigation. The recording contains three
swipe/touch inputs, eight dial push/release events after checks of all four dials,
and 12 dial-turn reports. A separate timed touch-only follow-up recorded two touch
inputs after isolated tap/hold instructions. These are aggregate counters; exact
tap versus long-press classification relies on the user-directed actions rather
than a separately typed gesture log. Do not infer gesture threshold measurements.
The first timer restored production before the original tap/hold sequence was
captured; that sequence was not counted as trial evidence and was repeated in the
explicit follow-up.

Both additional trials ended cleanly, with verified original build/page, ready
production Plus display, unchanged configuration and normal Auto-Lock preference.
The MK.2 is blank between trials because installed V1 still controls only Plus.
Private evidence preserves each exclusive test store, timed outcomes and restoration
records. No test host remains running; no VM or scheduled test state was changed.

## Remaining release gates

Initial paired hardware/input/isolation acceptance is complete. Physical reversed
connection order, paired lock/suspend and sustained resource/response acceptance
remain open. Shared audio/microphone/plugin dispatch and production workspace/
enrollment integration also remain gates. These hardware-only trials do not
install or release multi-device control. The workstation returns to its existing
production Plus service afterward.
