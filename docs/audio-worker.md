# Separate audio worker

A manually controlled long-running session is now available; see background-app.md.
Earlier bounded-only descriptions below record previous checkpoints.


The audio layout and hardware command are unchanged. All wpctl command execution
and audio-state reads now run on a separate owned thread. The HID worker uses only
nonblocking queue submission and result checks; USB writes remain on the HID thread.

Requests use a bounded FIFO of 32 entries. Full action queues report audio_queue_full
immediately; the rejected action has not been submitted. Status reads may be skipped
when busy and are requested again on a later polling interval. There is no automatic
retry of rejected mutations, coalescing of rotations or reordering of mute toggles.
Queued actions emit action_queued, followed by action_result when completed. Results
retain the originating session and input timestamp. Queued does not mean succeeded.

Replies use a separate 64-entry queue. A blocked consumer stops the audio worker
rather than blocking its thread. The HID worker reports audio_worker_failed and
rejects subsequent audio actions; restart the foreground command to recreate the
worker. The last displayed audio value can remain frozen on this terminal worker
failure, so consumers must treat audio_worker_failed as invalidating audio feedback.
Ordinary read failures still display AUDIO UNAVAILABLE through an audio_state null.

The active device session is shared atomically. Requests from a disconnected session
are discarded before execution, and old-session replies are ignored by the device
worker. Already-started commands can still complete after disconnect; there is no
rollback. Accepted queued actions are not individually completed after their session
is invalidated: consumers must cancel the whole session. Changing pages does not
cancel already-queued audio commands, only actions still armed by a held key/dial.

Dropping the audio worker signals shutdown, invalidates its session, discards queued
work and joins its thread. An in-flight wpctl command retains its existing 500ms
deadline. Status reads are queued at roughly 250ms intervals while idle; action
completion requests fresh feedback. There is no hard end-to-end latency guarantee.
Display rendering, HID open/enumeration, stdout and the existing shutdown caveats
remain outside this isolation improvement.

All 40 tests and the complete local quality gate pass. A synchronized fake backend
blocks an audio action while the test submits requests and polls nonblocking results,
fills the request queue to verify explicit rejection, changes sessions, then confirms
queued stale actions never execute. No system audio is changed by this test.

## Live check — 2026-09-10

The physical audio-layout capture recorded matched action_queued/action_result pairs
with successful execution, independently read audio states and repeated page changes
in the same session. No action failures were observed in the inspected capture.
This is functional regression evidence, not a measured input-latency benchmark.
Raw logs remain in ignored local/async-audio-live.jsonl.
