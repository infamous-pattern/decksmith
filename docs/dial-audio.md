# Dial audio and touch-strip feedback

Audio execution and polling now use a separate worker; see audio-worker.md.
The synchronous-execution descriptions below record the earlier checkpoint.


The optional `audio_dial: true` field in config/audio.json enables the leftmost dial
and live audio feedback. Other layouts omit it and retain their prior behavior.
Use the existing `--config config/audio.json --exclusive --seconds 60` command.

Rotation changes the current default output by one percentage point per reported
tick. Multiple ticks in one report are combined into one command. A report is capped
to ±20 points; excess ticks in unusually large reports are intentionally discarded.
Volume remains capped at 100%. A dial push arms mute; release toggles it once. Other
dials remain unassigned. Page changes and reconnect clear any pending dial push.
The schema also supports volume_adjust with a nonzero percent from -20 through 20.

The strip shows the current page, output percentage and VOL/MUTED. Readback uses
`wpctl get-volume` with LC_ALL=C, a 500ms process deadline and bounded output parsing.
Non-finite, negative, malformed and implausible values are rejected. Values above
100% set by another application are displayed honestly (up to 1000%), even though
Decksmith's adjustments cap output at 100%.

While idle, state is polled no faster than every 250ms, and unchanged state does not cause a USB
write. An action forces a fresh read on the next worker iteration. External desktop
changes therefore also appear while the worker runs. An unavailable read displays
AUDIO UNAVAILABLE rather than a stale number; recovery repaints the current state.
The audio_state record carries independently read percentage/mute data or null when
unavailable. Failed strip writes discard the device session as before.

Polling, action processes and rendering still share the device worker. The 250ms
interval is not a latency guarantee; slow subprocesses/USB writes delay polling.
There is no native PipeWire subscription or asynchronous action queue yet. The
static strip stops updating when this bounded diagnostic exits. No volume or mute
state restoration occurs automatically. These labels remain diagnostic typography.

All 39 tests and the full local quality gate pass. Added coverage verifies signed
and capped dial reports, ignored unassigned dials, push/release and page cancellation,
and strict readback parsing. Automated tests do not mutate system audio.

## Live validation — 2026-09-10

Physical dial rotation produced successful volume commands and changing independent
audio_state readbacks. The user confirmed dial adjustment, push-to-mute and displayed
feedback all work correctly. The capture began at 70%, unmuted. Raw logs remain in
ignored local/dial-live.jsonl. External-change and audio-server-loss recovery are
implemented but have not been separately exercised in this hardware checkpoint.
