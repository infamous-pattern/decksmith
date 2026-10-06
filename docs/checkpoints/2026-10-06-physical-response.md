# Physical response follow-up — 2026-10-06

Installed clean artifact `0.1.0-2af1dcb9b80f`, source
`7f6ce75647ff7c029996857dc20498b3e3375d2c`, was observed through its existing
structured journal. No restart, diagnostic environment change, layout write or
automated volume change was performed. The user confirmed the physical controls
and display responded correctly during a requested two-minute exercise.

The 127.6-second first capture contained 15 successful action results, all matched
to queued requests, plus eight page-render requests. Six successful experimental
plugin-key actions had null dispatch stamps, as expected for that uninstrumented
path; they are excluded from timing statistics. No dial-turn timing was captured
in this first window.

| Observed route | Count | Dispatch median | Nearest-rank p95 | Maximum |
| --- | ---: | ---: | ---: | ---: |
| Physical application key | 3 | 0.024 ms | 0.024 ms | 0.024 ms |
| Physical Play/Pause key | 2 | 0.022 ms | 0.025 ms | 0.025 ms |
| Touch-strip mute | 4 | 0.026 ms | 0.027 ms | 0.027 ms |
| Swipe to page-render invocation | 8 | 0.014 ms | 0.015 ms | 0.015 ms |

A separate 50.8-second recording check requested a clockwise leftmost-dial click
and a click back on Applications. The user confirmed its displayed volume changed.
That window contained 14 successful physical audio-dial adjustments and one
successful touch mute, all matched to queued records, with no null receipt stamps.
Dial dispatch median was 0.0205 ms, p95 and maximum 0.035 ms. The touch mute measured
0.019 ms. The actual 14 adjustments must not be described as exactly two reports
or as established ordinary/rapid burst coverage.

The daemon PID and zero restart count were unchanged, layout hashes matched, and
there were no action errors or warning/error trace records in either capture.
The first window had no playback streams at either endpoint; the follow-up began
with none and ended with one uncorked stream. These endpoint observations do not
establish a sustained loaded workload or four active meter targets.

All observed timed routes were below the 25 ms dispatch target **for these short
samples only**. The boundary is pinned device-library report return to backend
entry or page-render invocation, excluding USB/firmware, backend completion,
rendering and transport completion. Journal arrival/display-settled records are
not camera measurements. Representative route/burst and loaded coverage,
brightness-dial timing, and state-change-to-visible-pixel response remain open.
The cause of previously absent dial records is not established by this later
successful capture. No complete V1 latency pass is claimed.

Raw evidence and per-route analysis remain under the ignored local
`v1-rune-input-timing-20261006` evidence directory. See
[timing boundaries](../input-latency.md) and the
[V1 acceptance plan](../v1-release-scope.md).

## Supplied physical video

The user supplied a 33.4-second, 3840×2160 recording containing 1,002 video frames
and confirmed it was captured in normal Video mode. Its nominal 30 fps gives
approximately 33.3 ms per frame, rather than the requested slow-motion sampling.
Inspection confirmed live meter activity and a microphone-panel transition from
red Muted/crossed-out artwork to an uncrossed microphone and green `70% Live`.
The microphone icon changes between frames 388 and 389 (12.933 and 12.967 seconds
in the clip). This identifies the visible change, not its latency from input.

The approximate journal window contains one successful microphone mute-toggle
with a 46-microsecond report-return-to-dispatch stamp. Camera metadata and journal
clocks are not precisely aligned, and the finger obscures the contact region.
No clearly identifiable dial-turn or page-change reference was available in this
recording. It therefore adds visual microphone-feedback evidence without closing
the visible-response gate. Further footage needs visible dial steps and page
swipes, preferably at 120/240 fps. Original footage and derived frame/journal
evidence remain local; no video or camera metadata was published.

## Original high-frame-rate follow-up

The second supplied recording contains 21,334 frames at 1920×1080 over
102.133 seconds. The user confirmed original 240 fps Slow-mo capture lasting
approximately 1 minute 42 seconds in real time, with no playback-speed edit.
The file is variable frame rate: most intervals are about 4.17 ms, with many
8.33 ms intervals. Measurements therefore use presentation timestamps rather
than dividing frame counts by 240.

Inspection shows output and microphone dial adjustments, red Muted/green Live
feedback, active meters, four completed page changes, media keys and application
keys. In the first page change, the first key begins changing at approximately
38.579 seconds and the last key reaches its destination appearance around
38.741–38.750 seconds. This is approximately **0.17 seconds of visible sequential
key repaint**, measured from the first changing key to the last destination key.
Adjacent-frame bracketing is about 4–8 ms; LCD fading, compression and manual
classification add uncertainty. The other three changes also visibly update
keys in sequence, but were inspected at coarser sampling and are not additional
precise latency samples.

This repaint interval excludes the delay before the first key changes. It must
not be called swipe-to-display latency or a pass against the 150 ms
state-change-to-image target. The encoder's electrical detent and the gesture
classification instant are not visible, and the camera and daemon clocks have
no independent synchronization. Consequently the clip establishes functional
visible response and exposes sequential page painting, while exact dial
input-to-pixel latency remains unmeasured. No additional recording is required
to substantiate these observations; a precise end-to-end claim needs an
independent input reference. Investigating page-render/transport scheduling is
a useful next step before closing the display-response gate.

The corresponding 110-second journal window contains these action results:

| Route | Results | Successful | Dispatch p95 | Maximum |
| --- | ---: | ---: | ---: | ---: |
| Audio dial adjustment | 80 | 80 | 0.041 ms | 12.862 ms |
| Touch mute | 9 | 9 | 0.031 ms | 0.031 ms |
| Play/Pause key | 2 | 2 | 0.020 ms | 0.020 ms |
| Application key | 3 | 3 | 0.066 ms | 0.066 ms |
| Experimental Homebridge key | 6 | 4 | Uninstrumented | Uninstrumented |

All 94 timed built-in action results succeeded and were below 25 ms at the
existing report-return-to-dispatch boundary. Four page requests were also
recorded. One experimental plugin result reported `plugin_busy` and another
`plugin_failed`; their causes are not established by the video. They require
separate follow-up and must not be included in an all-actions-success claim.
The records retain the same daemon PID throughout the window, rather than
showing a daemon restart. No sustained four-target or brightness-dial workload
is established here, and these results do not close the complete V1 timing gate.

Original footage, contact sheets, timestamp evidence and the matching journal
are retained only in the ignored local `video-1802` evidence directory. The
analysis made no service, layout or volume changes.
