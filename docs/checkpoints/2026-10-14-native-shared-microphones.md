# Native shared microphone qualification — October 14, 2026

## Scope and accepted policy

This is the opt-in connected runtime, not a production activation or an installed
V1 change. Both device models share one microphone owner. The accepted policy is:

- Mute cancels every active push-to-talk hold for that microphone across devices.
- The last release always returns to Muted, including when it was Live before a hold.
- An ordinary explicit unmute remains Live across page changes; it is not a hold.
- A cancelled/failed hold cannot reopen without a fresh physical press.

Named input mute now goes to the microphone owner instead of the ordinary action
queue. Releases, scope cancellation and device lease expiry remain independent of
slow output/app/media work. Pending manual transactions and holds share the same
eight-target bound. An idle owner sleeps without unconditional empty-state wakes.
The runtime joins the microphone owner before reporting completion to restoration.
Input mute keys also participate in saved key audio-state observation.

## Native boundary

A new opt-in helper uses a private retained pin containing the selected source
name, index, object serial and Pulse socket identity. Initial discovery reserves a
pin even on failure; retries cannot silently discover a new microphone. Each write
checks the pin on the same libpulse connection that issues the indexed command,
then confirms actual mute state. Named/default replacement fallback is absent.
The binding follows the stable source-info prefix in the
[official PulseAudio header](https://github.com/pulseaudio/pulseaudio/blob/master/src/pulse/introspect.h).

PTT needs fresh parent heartbeats; EOF, lease loss or cancellation attempts Muted.
Ordinary unmute additionally needs a host commit after rechecking its exact
request/scope. Losing that commit attempts Muted. A late/uncertain acknowledgement
is not replayed as another toggle. The owner closes/joins an old writer before
starting a new writer for the same target; failed mute retries remain bounded.
Legacy V1 `ptt_guard.py` and its existing modes are unchanged.

## Verification

Immutable source SHA-256:
`0db06374913431840c95156fcb60f109aafc48b32a25cd8abc0dd50f4a95cc2b`.
All 175 archived app/crate files match the final local source bytes.

AI-server checks passed:

- 305 Rust tests, zero failures, five intentional fixture-dependent ignores.
  The private native microphone acceptance test was then explicitly executed
  and passed, bringing this qualification to 306 executed Rust tests.
- Strict all-feature/all-target Clippy, formatting and build.
- 216 Studio and 111 script Python tests, with one conditional skip in each suite.
- Native GTK saved-profile/workspace/live-editor checks, connected VirtualDeck
  execution, six private MPRIS scenarios and legacy native GTK regression checks.
- A separate private PipeWire server with one synthetic microphone: real libpulse
  hold/release, heartbeat loss, committed manual mute/unmute, lost-commit mute,
  incorrect serial refusal and retained-invalid-pin refusal. Every owned process
  was reaped. No physical audio device or VM was used.
- The Rust native owner against that private server: paired overlap, last release,
  Mute cancelling both holds, consumed old presses, persistent ordinary unmute,
  lease expiry, final joined mute and private pin-directory cleanup.
- Private recovery fixtures preserve exact input raw channel values/mute, avoid
  writes for an unchanged microphone, and refuse a changed source identity.

The first server fixture attempts failed before microphone writes because of
socket path length/configuration and fixture inventory assumptions. They remain
private evidence; the complete final immutable-source qualification passed.

## Additional unattended acceptance

The user deferred the physical microphone test and authorized additional AI-server
testing. The unchanged qualified production-source bytes passed an expanded
private fixture on October 14:

- 60 paired VirtualDeck cycles, 120 independent saved-profile updates, 60
  three-way microphone overlaps, 60 Mute overrides and 60 locks while held.
- Twelve alternating disconnect/reconnects, with the partner continuing and the
  disconnected device recovering its own page. Meter changes never repainted keys.
- Fourteen verified history archives, retaining the current profiles/contexts and
  respecting the sixteen-generation active-store bound. The initial fixture hit
  that intentional bound; the corrected fixture used the existing explicit
  archive operation instead of bypassing it or deleting test saves unverified.
- Nine native private-microphone fault cases: EOF before authorization, early
  commit rejection, SIGTERM while held, malformed heartbeat, missing commit,
  repeated use, wrong socket pin, broken report pipe and server restart under
  the same source name. Forty hold/manual-unmute/manual-mute cycles preserved
  raw microphone volume and identity. A replacement server received no old-pin write.

The Rust process RSS samples moved from 24,296 to 24,628 KiB and stabilized by
cycle 41; active thread samples stayed at nine, dropping to two after shutdown.
These are short synthetic-fixture observations, not production CPU/whole-process-
tree resource qualification. Test writers were reaped, all server VMs remained off,
the qualified candidate binary was unchanged, and no desktop/HID/microphone access
occurred. A private test-only Rust module was appended to an extracted source copy;
all original source prefixes and the retained binary were verified unchanged.
No production application code changed for this additional acceptance.

## Paired physical microphone acceptance

The same qualified source passed a guarded physical Plus/MK.2 trial. The user
confirmed:

- Each Deck's HOLD MIC key shows green Live while held and red Muted on release.
- A MK.2 key and Plus dial can overlap; only the last release returns to Muted.
- MK.2 Mute and Plus dial mute each cancel every active hold. Old held controls
  cannot reopen the microphone; a fresh press works.
- Ordinary Plus unmute persists until MK.2 Mute, and survives ordinary page changes.
- Changing the Plus page while its PTT dial is held cancels that hold.
- Both saved microphone panels show moving signal arcs, clear percentages and
  correct Live/Muted text; input volume adjustment works.

A read-only observer took 5,596 samples against the exact retained source identity
and recorded seventeen mute-state observations/transitions. Physical receipts
recorded sixteen MK.2 key edges, two Plus key edges, fourteen Plus dial-push edges,
one dial-turn report and three touch events. These corroborate the combined test;
not every guided gesture has an individually matched receipt. PTT/mute bypasses the
ordinary completion queue, so its completion count is not an action total. The lab's
legacy `native_microphone: false` capability flag is stale; direct source-state
observations and physical input records establish the actual microphone exercise.

The initial temporary layout was rejected for empty unused-key labels before
microphone controls ran. Exact restoration passed; the failure remains retained
as harness evidence. Corrected labels and both connected Preview validations passed
before the successful retry. No production source changed during this test.

Both device owners completed full blanking and the trial exited successfully. The
guard and a separate read-only verification confirmed the original microphone raw
channels/mute, output raw channels/mute, pinned Brave playback, configuration,
installed build and normal ready Plus page. The trial endpoint and process were
absent. The user confirmed normal Plus controls restored and MK.2 blank, as expected
with the unchanged single-device installed V1.

## Physical held-device recovery follow-up

A second guarded native microphone window passed both directions of USB recovery:

- Unplugging MK.2 during its only hold returned the microphone to Muted; Plus
  continued. MK.2 reconnected without reopening the mic, and a fresh hold worked.
- Unplugging Plus during its only dial hold returned to Muted; MK.2 remained
  connected and ready. Plus reconnected with a clear strip, no stale unmute and a
  working fresh dial hold.
- With a MK.2 hold surviving an overlapping Plus hold/release and subsequent Plus
  disconnect, the user confirmed the mic stayed Live until MK.2 released, then Muted.
- Final Plus reconnect restored clear displays with the mic Muted.

Pinned read-only source observations corroborated Live/Muted sequences. Disconnect
snapshots showed the expected missing owner and ready partner; final connection
counts were three for Plus and two for MK.2, with thirteen MK.2 key edges and five
Plus dial-push edges. No provider failure or issued-effect uncertainty was reported.
Expected disconnected transport errors cleared on reconnect.

A requested MK.2 blue-page check was user-confirmed, but its snapshot showed MK.2
on page zero and Plus on page one, with no new MK.2 key edges. It is not counted as
new MK.2 page-navigation evidence. Prior paired navigation acceptance remains
separate. Live-through-disconnect isolation is user acceptance supported by combined
source-state and receipt evidence; this is not synchronized visible-response timing.

Both owners blanked and the native trial exited normally. Guarded and independent
read-only verification confirmed exact microphone/output/playback/configuration/
installed-build/page restoration. The trial endpoint was absent, and all three
previously sampled owned host processes had exited. This is a sampled cleanup
check, not a complete machine-wide process audit. The user confirmed the original
Plus page and controls returned, with MK.2 blank under unchanged installed V1.

## Remaining gates and limits

System sounds metering was not exercised in the microphone run; a later
[guarded role-isolation trial](2026-10-14-saved-active-meters.md#physical-system-sounds-isolation)
accepted it separately. Physical held-device disconnect/reconnect is accepted above;
paired lock/suspend/startup, migration and long resource qualification remain
separate gates. Input meter and paired microphone policy acceptance above are
complete for this opt-in trial, not production activation.

This native microphone path deliberately requires named inputs. Default microphone
aliases and automatic rebinding after a source/server replacement are not enabled.
Eight distinct microphone pins are retained per runtime; selecting a replacement
or exhausting that lifetime pin budget needs an explicitly prepared fresh runtime.
Numeric visible-response latency, long resource soak and production lifecycle
qualification are not claimed here. The daily installed V1 remains unchanged.
