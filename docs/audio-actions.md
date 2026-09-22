# Audio action checkpoint

Audio execution and polling now use a separate worker; see audio-worker.md.
The synchronous-execution descriptions below record the earlier checkpoint.


The audio layout now includes the leftmost dial and live strip feedback; see
dial-audio.md. This document records the original key-action checkpoint.

With OpenDeck closed, run the hardware-enabled daemon:

```sh
./target/debug/decksmithd --config config/audio.json --exclusive --seconds 60
```

This optional saved layout adds VOL DOWN, VOL UP and MUTE to the HOME/WORK pages.
The original navigation.json and bundled --pages-demo remain navigation-only.
Audio keys act once on release. A successful page change clears armed actions;
reconnect starts fresh. Configurations accept volume_down, volume_up and mute_toggle
in addition to none/go_to_page. These are fixed action types, not command strings.

The adapter executes /usr/bin/wpctl directly with fixed arguments, never a shell.
It changes the current default audio sink by five percentage points, with a 1.0
(100%) volume limit, or toggles that sink's mute state. It does not affect microphone
mute. The default output is resolved by WirePlumber on each invocation; output
switching therefore changes which sink subsequent key presses control.

Each release produces action_result with session, action, success, error_code and
original input. Success means wpctl exited successfully, not independent state
readback. Child standard streams are discarded. A 500ms deadline kills and reaps
an unresponsive child; failures are reported and are not automatically retried
(toggle/relative actions might already have taken effect). The device remains
connected after an audio failure.

The current implementation runs these short commands synchronously on the device
worker, so an action can delay input polling. A dedicated action executor, live
volume/mute display feedback, per-application controls, dial bindings and general
binding-engine integration remain pending. The diagnostic still exits after at
most 120 seconds and installs no background service. No state restoration is
performed on exit; changes made by the user remain in effect.

All 37 tests and the complete local quality gate pass. Tests verify fixed command
arguments, default-sink targeting, the volume cap, release-only dispatch and
cancellation. They do not execute audio mutations. Local wpctl help was inspected
to verify supported set-volume/set-mute syntax on this Fedora workstation.

## Live validation — 2026-09-10

Physical key releases produced successful volume-down, volume-up and mute-toggle
results. The user confirmed all three controls work. Default-output readback after
the exercised sequence was 0.70 with no muted flag, matching the initial state.
Raw events are kept only in the ignored local/audio-live.jsonl capture.
