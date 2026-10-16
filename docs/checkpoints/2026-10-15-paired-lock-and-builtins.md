# Paired lifecycle and ordinary actions — October 15, 2026

This is development qualification, not a daily-runtime activation or public
release. The user's desktop remains on the accepted simplified editor with the
working V1 daemon. The new connected runtime is still a timed opt-in lab.

## Scope and implementation

The user narrowed the upcoming V1.5 delivery to tested simultaneous Plus/MK.2
support and accepted interface/gauge work. General plugins, catalog/compatibility
and the plugin-provider monitor track move to V2. Existing experimental Homebridge
bindings must be preserved; their execution is not yet bridged to the new runtime.
The reviewed Deck-Switch/Deckhand ideas and independent implementation boundaries
are recorded in the research toolbox and roadmap.

The lock slice adds an explicitly opted-in real session observer, combining host
and manual gates without allowing host unlock to override another safety gate.
Host epochs invalidate queued effects and held controls even if a complete lock/
unlock occurs between coordinator turns. Model-sized Locked frames go through
the sole device I/O owners; reconnect while locked stays locked, and unlock
restores each device's page after input drain. Frames are cached per model.
Suspend/reopen and persistent startup remain unwired and are reported as such.

Ordinary application/website/system actions now have a shared-worker route with
an authorization handshake after discovery and before effects. Named audio-device
selection rechecks exact server/node identity. Uncertain results are not replayed.
Legacy adapter defaults and saved assignments remain unchanged. Independent
shutdown/reboot confirmation still requires an explicit user click; it has no
automatic countdown. Native acceptance of these routes is still required.

## Verification and findings

An immutable pre-change AI-server baseline passed 305 Rust, 216 Studio Python
and 111 script tests, native virtual-profile checks, format/Clippy/build, a private
lock-proxy check and simulated reconnect stress. It is baseline evidence, not
qualification of later edits.

Focused local lock qualification passed 311 Rust tests, strict Clippy and format
checks, including paired held-input cancellation, locked reconnect, manual gates,
stale lock epochs and delayed effects/USB writes. Five specialized diagnostics
remain intentionally ignored in the ordinary suite. Built-in mocked tests cover
discovery-to-effect fencing, helper failures and missing/ambiguous targets.

A real private GIO subprocess fixture found a callback-userdata mismatch missed
by mocked tests: the callback was skipped when passed None. Tuple userdata fixes
the binding, and child setup now fails closed if session isolation fails. The
successful fixture uses the production launch function and proves the child's
session/process group is independent and explicitly reaps its harmless child.
Both the original failure and fixed result are retained privately. No real app,
website, power action or live audio was exercised by this fixture.

The final immutable post-fix snapshot passed the full AI-server suite: 311 Rust,
227 Studio Python and 111 script tests, format/strict Clippy/build, VirtualDeck
smoke checks, native two-profile acceptance and an explicit private lock-proxy
check. A 60-second eight-simulated-device stress run completed 25,618 reconnect
cycles. All 179 runtime-source hashes matched the tested snapshot afterward.
Archive SHA-256: `b61d34362a0f44a19d439ddb93c96384dcd8a90b8b69c49440dfc155ad89e83a`.
Three specialized JPEG/native-microphone diagnostics remain skipped; earlier
physical microphone acceptance is a separate record. No live audio, HID, VMs or
normal/scheduled services changed. Source archives, all runs and the original
GIO failure remain in private evidence. Simulation is not physical acceptance.

The private physical handoff now rejects a changed candidate, saved state,
installed binary or service before interruption. Recovery must verify the same
installed build, saved data, ready connection and page; a changed layout does not
have its new page silently overwritten. Mocked failure checks cover these gates.
A dedicated diagnostic lock mode uses no real audio provider. Automated checks
alone do not establish physical acceptance or a desktop runtime promotion.

## Next acceptance and desktop policy

The user requested the desktop stay on the latest qualified build for daily
feedback. Preserve the previous package and configuration backup, protect unsaved
editor work, verify recovery after an update and state any limits. The latest
compiled lab is not automatically a suitable daily build.

Before replacing the existing runtime, complete physical paired lock acceptance,
suspend/reopen, a persistent entry point and editor/control API, saved-layout
migration and rollback, existing action parity (including experimental Homebridge
handling), start-at-login/quit blanking, accessibility and idle/loaded resources.
Retain independent profiles and the working desktop throughout qualification.


## Physical paired lock acceptance

The user saved and closed the editor and approved a ten-minute bounded trial.
The accepted checks completed in 532.567 seconds; this is a focused interactive
trial, not a ten-minute idle/loaded resource soak. The exact tested binary was
used with real GNOME lock observation and simulated audio only.

- Both devices showed Locked on the first key, with other keys and the Plus
  touch strip blank, then restored their independent blue MK.2/copper Plus pages.
- Page and brightness inputs made while locked were observed but discarded,
  without replay after unlock or changes to the retained values.
- Each device was disconnected and reconnected while locked, independently.
  Each returned to Locked, and both original pages recovered on unlock. The
  connected partner remained safely locked and available afterward.
- Fresh MK.2 input changed the simulated shared level from 50% to 51%; a fresh
  Plus dial click changed its own brightness from 50% to 51%. Independent pages
  remained intact, with no failed action feedback.

The initial automatic restoration stopped conservatively on a service-definition
hash mismatch. The original failure is retained. Review proved the raw difference
was exactly runtime CPU/IO weights changing between 100 and 300; executable
settings and installed binaries were unchanged. A foreground recheck restored
normal controls successfully, verified unchanged saved configuration/build and the
original ready Plus page. The user confirmed normal Plus controls and a blank
MK.2, as expected for the retained single-device daily runtime.

The private guard now normalizes only the two known runtime priority overrides,
validating their exact allowed property and numeric range. Other directives,
unknown overrides, changed permanent service settings/binaries or configuration
remain detected. Legacy raw fingerprint snapshots remain explicitly supported.
The guard's 31 focused checks pass, including rejection of an executable directive
hidden in a priority override. No additional Deck restart was needed for this fix.
This guard correction is separate from the physical application acceptance; it
must not be reported as automatic recovery having passed on the first attempt.

Physical evidence and source/binary hashes are retained privately. Suspend and
persistent startup have not been tested or activated by this trial. The next
slice must acknowledge actual microphone/actor quiescence before sleep, reopen
identities independently after wake and preserve the healthy partner if another
unit is missing. A desired-state hold count alone is not mute acknowledgement.
