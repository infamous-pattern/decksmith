# Incremental display scheduling

The existing audio-layout command now queues screen images through nine bounded
slots: eight keys and one strip. Each slot retains its latest desired image and
last successfully sent image. Identical pixels are not resent; a newer pending
frame replaces the older one. The scheduler scans round-robin and writes at most
one image per device-worker iteration. Encoding and the individual USB write remain
synchronous, but a page no longer sends all nine images in a single operation.

Page rendering now creates desired state. The protocol uses page_requested rather
than page_changed: it does not claim USB completion. display_settled reports when
all desired images are sent, including initial connection and strip-only changes.
connected means a handle is open, not that the screens have finished painting.
audio_state means a fresh reading was accepted for display; display_settled marks
completion. Intermediate requested pages can be superseded before fully displaying.

During a pending redraw, key/dial-push edges are read and logged but not dispatched.
This prevents a key from acting on a partially visible page. Swipes and dial rotation
remain usable; newest desired frames replace pending frames. Page changes already
clear armed key/dial actions. Initial/reconnected sessions invalidate all cached
images and repaint from page zero. Failed writes are not marked successful; the
session is discarded and a disconnected record is emitted. Screens may contain a
partial page until reconnect succeeds. Queued frames are not saved across sessions.

Frame generation, JPEG encoding and individual HID writes remain on the device
thread. Audio replies or display completion can consume an iteration without an
input poll. This is cooperative scheduling, not a hard latency guarantee, a render
thread or a frame-rate target. The largest gain expected is removing repeated and
superseded screen writes; no numerical performance claim has been measured here.

All 43 tests and the full local gate pass. Tests cover latest-frame replacement,
identical-frame suppression, one write per flush, failed-write accounting, forced
repaint after reset, and consuming input during redraw without dispatching navigation
keys. The live test uses unchanged HOME/WORK and audio controls.

## Live capture — 2026-09-10

The 60-second capture exited successfully with 6 page requests,
69 display-settled records and 62 completed audio actions. No
action failures or disconnects were recorded. The final display-settled event
provides USB completion evidence; visual appearance is a separate user check.
Raw logs remain ignored under local/display-scheduler-live.jsonl.
