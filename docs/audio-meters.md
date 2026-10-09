# Live audio meters

Implemented September 13, 2026 for the Stream Deck + prototype. The October 9
development build adds the dual-arc presentation below, also included in the
[experimental COSMIC preview](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0-popos-preview.1).
The original Fedora V1, Debian and Ubuntu downloads retain the classic display.

## Touch-strip styles

Choose a dial's **Appearance → Touch-strip style** in Keys & Dials. The updated
build defaults existing audio and device-brightness dials to **Dual-arc
gauge** without rewriting saved assignments. **Classic bar** remains available
per dial, including shared defaults and page overrides.

The current development build also offers **Single-arc meter** for audio targets.
It shows one wider centered arc of measured audio activity, without a volume
percentage or pointer. Existing turn/press actions, target selections and labels
stay unchanged. Microphone Live and Muted status remains centered inside the arc;
missing/silent measurements and No Audio retain distinct states without a yellow
border. Its geometry is cached once and the sampling budget is unchanged.
Brightness remains a setting gauge because it has no audio signal to meter.
The published COSMIC preview has Dual-arc and Classic bar; Single-arc is newer
development work and does not change any existing release assets.

The dual-arc layout keeps the device icon and label across the top, with the
configured percentage in the right-hand column. The inner blue arc and pointer
show configured volume or brightness. For audio controls, the independent outer
arc shows actual measured signal: green below 80, yellow from 80, orange from 90,
and red from 95 on the normalized meter scale. A silent measured signal leaves
the outer track empty; an unavailable measurement shows a gray dash.

Mute retains the red crossed-out device icon and **Muted** text, dims the volume
gauge and suppresses signal fill; it does not add a red border. Live microphone
controls retain green **Live** text. **No Audio** uses two lines in the value
column without a yellow border. Brightness has only the setting arc, with no
audio measurement. The existing default-device marker remains below the gauge.

The preview and physical Deck share the same renderer. Arc geometry and pointer
masks are cached once, and meter ticks reuse the static frame; the 20 Hz sampling
budget is unchanged. Regression checks cover every integer level, silence,
missing measurements, mute, enlarged text, style persistence and preview/device
pixel parity. On October 9 the user accepted the installed local build's physical
audio-dial/mute response, blue setting arc, independent signal arc, red Muted and
crossed-out icon, and green microphone Live text. A controlled resource comparison
remains before a public Fedora gauge update. COSMIC resource use and physical
hardware/audio remain unqualified in its experimental preview.

## Classic bar presentation

Each audio dial panel has three separate displays:

- The numerical percentage shows configured volume and remains visible as activity changes.
- The rounded bar fills with actual measured signal activity. Silence or mute
  leaves the track empty; unavailable measurement shows a gray dash. Muted targets
  show red Muted text and a crossed-out icon without a red border. There is no volume-position line
  over the bar and no separate segmented activity row.
- The short centered marker below the bar identifies the current Linux default
  input/output: green is default, gray is not default, amber is unknown.

Brightness retains its light-bulb icon, percentage and bar, with no audio meter.
The physical display and editor use the same renderer. Reopen the editor after an
update to load its faster refresh. A newly assigned draft target may remain
unavailable until Save and Apply starts monitoring that target.

Activity colors follow the same normalized measured level that sets the fill
width: **blue below 80**, **yellow from 80**, **orange from 90**, and **red from 95**.
These thresholds are on the signal meter's normalized scale, not the configured
volume percentage. The entire active fill changes color;
brightness retains its blue set-value bar.

## Sources and limitations

Local PipeWire's PulseAudio-compatible interface supplies peak-only measurements.
System output follows the default output's monitor; named outputs use their own
monitor. Default/named microphone targets use their input source. Application
meters monitor only the matching playback streams, even if other apps share the
same output. For multiple matching streams, the strongest measured stream is shown,
not a reconstructed mix. App tabs are not individually selectable.

Missing targets, unsupported monitor connections, permissions failures and stale
measurements show unavailable; they never substitute another device or app. Sources
are rediscovered about once every two seconds, so reconnects/default changes can take a
short time to settle. More than 16 matching streams is treated as unavailable.
Legacy text-only touch layouts do not gain a meter; configure audio dials to use it.

This is a visual peak-activity display, not a calibrated loudness/SPL meter. The
scale spans approximately -60 to 0 dBFS with a quick rise and short decay. Monitor
peaks can precede hardware volume, so changing output volume need not change the
meter proportionally. Signal fill uses a logarithmic peak scale, independently of the configured
volume percentage. Mute suppresses signal fill regardless
of the monitor tap.

Only transient numeric peak values are retained. Decksmith does not record, save,
or transmit audio content. Monitoring a selected microphone can activate the
Linux microphone-use indicator and can keep the selected source awake. Meter monitoring ends when background controls
stop, the device disconnects, or the target is removed.

## Engineering and verification

A separate helper uses libpulse PA_STREAM_PEAK_DETECT at 20 Hz and publishes numeric
levels at at most 20 Hz. A libpulse read callback drains arriving peaks immediately
and coalesces them to the strongest value for the next display update; values are
consumed once, and silence/suspension clears pending activity. It connects only to the local user audio socket. Inventory
refresh runs separately from peak processing. Long-lived helpers reuse read-only
inventory until a libpulse device, stream, volume or default-device notification
invalidates it. A five-second watchdog bounds missed notifications; caching is
disabled while the listener is unavailable, and actions always resolve targets
freshly. The Rust worker uses a latest-only
snapshot, bounded input, target/session validation and a 400 ms stale timeout;
no subprocess work runs in the HID ownership loop. Static strip artwork is cached.
The native main loop drains pending work within a fixed bound and stops as soon
as it reports no dispatched events. The 10 ms tick and 20 Hz meter updates are
unchanged; an iteration or context error still disconnects the helper. A Fedora
44 null-sink check measured two independent synthetic app peaks of 0.05 and 0.40
after this change without changing default routing or user volumes.

Failed helpers retry with bounded backoff. Peak streams use the minimal peak-detection
flag with a 50 ms fragment request, without changing hardware latency. The actual
source name is checked before accepting peaks; a moved or failed stream immediately
loses its cached measurement. Streams stuck connecting are retried after three
seconds, at most once per inventory refresh.

On this PipeWire host, a READY Brave monitor occasionally delivered no peak packet
for over 500 ms. The helper therefore decays the last measured peak at 30 dB/s after
150 ms, with a hard one-second expiry. Missing, failed, moved and removed streams
do not receive this grace period. This is a bounded display envelope, not a new
measurement or an indefinite held level. The Rust 400 ms timeout still detects a
helper that stops publishing. The editor coalesces read-only requests.

Validation includes exact physical/editor source-pixel parity for measured, silent,
muted and unavailable states, marker/brightness preservation, stale session/target
rejection, and native editor draft/hit-region checks. An isolated temporary null
sink test measured two independent synthetic app peaks of 0.05 and 0.40 correctly,
without changing user output routing or volumes. A cached-render check produced
100 frames in about 5 ms on this desktop. Live output, mic and Brave measurements
were observed, followed by a live editor check without saved-layout mutation. Steady-state
read-only preview requests measured 35 ms median / 38 ms maximum over 40 requests
on this desktop, with display-ready status retained throughout. These timings are
observations, not guarantees for other hosts.
Physical smoothness/readability still needs user confirmation; unusual devices,
surround layouts and server implementations have not all been hardware-tested.

The implementation follows PulseAudio's official
[volume-control UI monitoring guidance](https://wiki.freedesktop.org/www/Software/PulseAudio/Documentation/Developer/Clients/WritingVolumeControlUIs/)
and [stream flags](https://raw.githubusercontent.com/pulseaudio/pulseaudio/master/src/pulse/def.h).

The final bar design retains numeric configured volume without a white line over
the meter. Signal colors, mute/silence/unavailable rendering and shared pixel parity
were checked. The short default-device marker remains below the track.

Audio dial assignments can inherit shared defaults or override individual slots per
page. Meter subscriptions and volume/mute feedback follow the effective assignments
of the active page. A newly selected target can briefly show unavailable while
monitoring starts; no other source is substituted.

September 13 recovery verification: the callback collector's final deployed run
kept output, Wave and Brave measurements available for all 436 sampled device
frames over 90 seconds while a native editor test made repeated unsaved edits.
The editor completed 91 responsive checks; saved layout and audio settings were
unchanged. The glossy renderer is regression-tested at every integer level from
0 through 100 to prevent the previously observed low-fill floating-point panic.

### Refresh budget

Volume and mute snapshots refresh every 500 ms, with an immediate refresh after
control actions. A persistent read-only helper shares one fresh snapshot across
system volume and assigned targets; failed connections are discarded and restarted
on the next poll. Meter samples remain independent at 20 Hz. Audio inventory
reconciles every two seconds, and visible pickers also react to audio topology
events. System indicators keep their two-second cache. Hidden editor previews
make no requests; visible touch previews coalesce requests at up to 20 Hz.

Audio volume, mute and device-selection actions use a separate lazily started
helper over bounded local pipes. Successive actions reuse the Python process,
but each action still resolves its target and volume freshly; action reads do
not use the subscription cache. The request order, per-action clamping and
mute-toggle behavior are preserved. This removes repeated interpreter startup
without coalescing or dropping dial steps. Replies and requests are size-limited,
and a three-second transport timeout discards an unresponsive connection.
Malformed/lost replies never trigger an automatic mutation retry, since the
action may already have happened. Only a subsequent independent input creates
a new connection. Explicit action errors remain distinct from transport errors.
The action helper is released after 30 seconds idle, including while the device
is disconnected; stopping the worker releases both read and action helpers.
Each connection owns a separate process group, and cleanup terminates its command
descendants along with the helper.
Qualification must measure actual dispatch delay, resources and recovery on the
installed artifact, rather than infer them from process reuse.

## Touch-strip region update candidate

The physical Plus adapter now retains a single 240,000-byte RGB baseline and uses
changed rectangles aligned to the original JPEG block grid. This reduces image
encoding and USB payload work for live meter changes while keeping the complete
800x100 logical frame and meter update rate. Initial connection, uncertain writes
and explicit blanking use a full strip; a failed transfer never advances the cache.
The underlying [Plus region-write API](https://docs.rs/elgato-streamdeck/0.13.1/elgato_streamdeck/struct.StreamDeck.html#method.write_lcd)
is already available in the pinned hardware library. Source and native-VM codec
checks pass. The clean candidate passed Fedora 44 runtime and lock/page restoration
checks, and the user confirmed the physical labels, meters, icons, page switching
and mute update correctly without stale areas or flicker. A valid five-minute quiet
sample measured 3.75% service CPU and 2.11% daemon CPU,
versus 3.83% and 2.09% previously. This does not establish a repeatable idle saving;
the original below-1% daemon CPU target is still unmet. By user decision on
October 5, it is now a post-V1 performance milestone, rather than a V1 release
blocker. V1 still requires final-artifact idle/loaded resource and regression
checks, responsiveness and complete recovery qualification; see the
[current resource acceptance decision](v1-release-scope.md#v1-resource-acceptance-and-post-v1-cpu-milestone).
