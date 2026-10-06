# Current physical response video — 2026-10-06

The user supplied an original 240 fps slow-motion recording of approximately
1 minute 47 seconds on installed bundle `0.1.0-257113d71d41`, source
`1a0101da0f38d9c5d90c9131d5238d9ee74ef0e9`. The file contains 22,594 frames over
106.718 seconds at 1920×1080. Its presentation duration agrees with the reported
real-time duration. Frame spacing varies: median approximately 4.167 ms, maximum
12.5 ms. Frame comparisons use presentation timestamps, not a fixed 240 fps divisor.

Inspection confirms visible changes from all four audio dials, red Muted and
green Live feedback, application/media keys and completed page changes. The first
page's key repaint begins between approximately 44.439 and 44.448 seconds; its
destination key content is clear by approximately 44.494–44.498 seconds. This is
roughly **50–60 ms of visible key repaint**, allowing for frame bracketing and
manual classification. The earlier candidate's inspected page repaint took about
170 ms. The pages, content and recording conditions differ, so this is descriptive
evidence of shorter repaint, not a controlled percentage-speedup claim.

Frame-level inspection of the following transitions also shows destination key
content settling over tens of milliseconds. Media-health outlines arrive in
subsequent updates, and the hand partly obstructs later transitions. These must
not be treated as ten isolated, fully unobstructed latency samples or a complete
page/touch-strip settlement percentile.

The corresponding approximate journal window contains 152 successful action
results and six physical swipe page requests. Its 132 audio-dial results have
report-return-to-backend-entry median 0.022 ms, nearest-rank p95 0.035 ms and maximum
0.062 ms. Ten touch mute actions, four application launches, two play/pause actions
and four volume keys also succeeded. No restart occurred; the same desktop daemon
remained active. This short recording does not establish four active independent
meter targets or extend the earlier sustained resource measurements.

The video has no independent electrical detent/gesture marker, and the camera
and journal clocks are not precisely synchronized. Fingers obscure touch contact
and parts of some dial turns; many turns are bursts rather than isolated events.
Consequently repaint duration and functional feedback are established, while exact
input-to-first-pixel and input-to-complete-display latency remain unqualified.
Neither sparse examples nor approximate camera/journal alignment close the V1
visible-response targets. See the [recording protocol](../input-latency.md#visible-response-recording-protocol).

The original video, camera metadata, frame evidence and raw logs remain local and
ignored. No original footage or personal camera metadata was published. The
analysis did not restart controls, modify layout or issue device/audio commands.
