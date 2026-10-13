# Shared worker bridge — October 13, 2026

Base canonical source: `cd6969e7ad64ff9eba593a40975cdd97ae8daf50`.
This extends the uncommitted [shared-action/hold foundation](2026-10-13-shared-action-foundation.md)
with a daemon-side bridge; it does not activate the multi-device runtime.

## Implemented

- One shared pool of one to four bounded executors, with fair owner scheduling,
  same-target serialization, immutable compiled payload retention, compatible
  scalar combining, discrete-action barriers and retained first receipt time.
- Source/current-scope checks before execution, after backend waits via a supplied
  write fence, on completion and again at consumer receipt.
- Fixed queue/result limits and nonblocking cancellation. Idle workers sleep
  without periodic polling; termination joins belong outside the HID input loop.
- Provider panic isolation without automatic retries/restarts. A writer's target
  remains reserved until its backend is destroyed. Failed destruction stops the
  service and retains the reservation; losing the last worker stops submission.
- A dormant fixed native named-device audio adapter: fresh level/mute reads,
  one clamped write, literal arguments, a final fence check, bounded command output
  and 500 ms timeouts with owned process-group cleanup.

Input mute remains rejected by the native adapter pending shared guardian
integration. Default/alias, application audio, System sounds, selection and plugin
actions remain rejected by compilation pending canonical resolution. Explicit
media player commands can compile for controlled backends, but their guarded
native adapter is unfinished. There is no automatic fallback to system volume.
See [scope and lifecycle details](../shared-action-lifecycle.md).

## Qualification and limits

Eleven new controlled fixtures pass. They verify independent concurrent targets,
same-target ordering, cancellation after a backend wait, stale consumer feedback,
combined adjustment payload/timing, toggle barriers, queue saturation,
active-writer shutdown, provider isolation, final-worker loss, failed destructor
reservation retention, result-consumer backpressure and native command plans.
They do not issue real audio commands or open HID.

The Fedora desktop passes the full all-feature Rust workspace suite: **214 tests,
zero failures, three explicitly ignored hardware/environment checks**. All-target/
all-feature Clippy with warnings denied, formatting and hardware-enabled build
are checked with the same source. No dependencies or lockfile were changed.
The same source snapshot passes all four checks on the AI server's Fedora Rust
1.98.1, also **214 passed, zero failed and three ignored**. The desktop uses Rust
1.97.1. Existing dependency advisories/bans/licenses/sources checks pass locally.
Private AI-server evidence is retained without changing the scheduled checkout,
jobs or timers. No VM was started. The tested source archive has SHA-256
`a8b61663bafb00ac3153ea1373ddebecc6c41313fdbcb4fc99c705dd5f9b2169`.

## State and next step

No production startup, editor or device API selects this service. No saved layout,
audio level, installed build or physical display changed. Compile-time presence
and controlled-backend tests do not prove actual native audio, resource footprint
or simultaneous Plus/MK.2 behavior.

Next connect one fail-safe guardian per microphone to the shared leases, with
write acknowledgements, last-release/explicit-mute cancellation and bounded
mute retry. Complete canonical app/default/System-sounds resolution and guarded
media adapters before switching production execution. Persistent device/profile
association, native API/editor, MK.2 HID/renderer and physical pair qualification
remain separate milestones. This checkpoint accompanies source publication.
