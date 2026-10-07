# Input timing and qualification

The reference target remains p95 below 25 ms from input receipt to capability
dispatch. A timing result must state its boundaries, artifact, workload, sample
counts and exclusions; successful interaction alone does not certify it.

**October 7 scope decision:** numerical qualification of the 150 ms key and
100 ms local dial visible-feedback targets is deferred beyond V1 by user approval.
They remain engineering goals, not achieved results. Functional physical feedback
and recovery, internal dispatch and resource review remain required. The protocol
below is retained for future numerical qualification; see the
[current release review](v1-release-review.md).

## Diagnostic boundaries

The physical Plus adapter records a process-local monotonic `Instant` immediately
after the pinned device library returns a report, before Decksmith normalizes it.
Every normalized event from that report retains the same receipt stamp, including
events waiting in the adapter queue. The stamp is absent after an empty or failed
poll and is not reused across a new device session. `InputEvent.timestamp_ms` and
its serialized format are unchanged. Virtual/editor inputs have no physical stamp.

The following optional microsecond fields are emitted in existing records:

| Record and field | Endpoint and interpretation |
| --- | --- |
| `action_queued.report_receipt_to_queue_us` | Successful enqueue on the action worker; includes adapter normalization and queued edges, but excludes waiting in the action queue |
| `action_result.report_receipt_to_dispatch_us` | Immediately before a built-in audio/media/system/application backend is invoked; includes waiting behind snapshots or earlier actions, excludes backend completion |
| `action_result.report_receipt_to_dispatch_us` for brightness | Immediately before the owned-device brightness call |
| `page_requested.report_receipt_to_dispatch_us` | Immediately before page rendering is requested; excludes rendering and visible transport completion |
| `poll_return_to_dispatch_us` | Legacy normalized-event-return to enqueue/page-request proxy; retained for comparison, not interchangeable with receipt or backend-dispatch timing |

Absent receipt stamps, rejected submissions and uninstrumented plugin paths emit
`null`, not a fabricated zero. Push-to-talk dispatch is not covered by these
fields. A successful backend dispatch still needs a matching success/error result
and separate completion/visible-feedback evidence. Cancelled old-session, locked
or foreground-stale work does not produce a new backend-dispatch sample.

Receipt is defined at the library report-return boundary. USB/firmware delay,
kernel buffering, and parsing performed inside the upstream library precede this
stamp and are not measured. Do not describe these fields as raw HID system-call
timing, physical press-to-feedback latency or proof of the visible key/dial targets.
The page-command timings measured in VirtualDeck also start at a different boundary.

## Checks and next measurement

Regression tests verify that multiple queued edges retain their original receipt,
empty/invalid/failed polls and new sessions have no stale timestamp, and virtual
devices do not invent one. A blocked-snapshot/backend test independently observes
backend entry: the reported delay includes waiting behind the snapshot and excludes
the later backend completion wait. Missing/future clock values remain absent.

This instrumentation adds bounded clock metadata to existing event/action queues;
it does not change queue limits, input polling, meter frequency, lock safety or
action selection. The full all-feature Rust suite, warnings-denied Clippy,
formatting and pinned dependency audit must pass before packaging it.

After installed-runtime and physical acceptance, gather representative key, page,
audio/media and brightness dial samples at ordinary and quicker input rates, both
with active meters and without playback. Report per-route counts and percentiles,
failures/cancellations and queue delay. Measure visible updates separately. Helper
fault workloads must be identified separately from ordinary operation. Final V1
latency acceptance remains open until representative evidence on its exact artifact
and declared receipt boundary is available.

## Optional recording diagnostics

For sustained VM qualification, `decksmithd --virtual-service PATH --seconds N`
accepts a bounded 1–3600 second run with VirtualDeck. Physical probe commands
retain their 1–120 second limit. The live VirtualDeck service waits 20 ms only
when its ordinary input poll is empty, matching the physical adapter's idle wait;
pending output and ready inputs do not wait. This prevents the live simulation
from spinning on an empty deterministic queue. The in-memory VirtualDeck used by
unit tests remains immediate and deterministic, with no fabricated receipt stamps.
A harness must stop its owned daemon and remove
temporary audio modules/processes on completion or failure. Virtual workloads
can exercise helpers and resource stability, but do not provide physical HID
receipt or visible-pixel latency evidence.

For a missing-record investigation, an explicitly started daemon may use
`DECKSMITH_INPUT_DIAGNOSTICS=1`. This enables aggregate report/normalized-event
counts on its existing physical connection and aggregate worker-output counts.
It adds no second HID reader and does not record raw report bytes, labels or
action targets. Summaries include current lock/drain/display-pending state and
are logged at most once every two seconds. The window expires after 120 seconds;
the counter allocation is then released and reconnects cannot extend the window.
Remove the environment setting after starting that diagnostic run so later
starts do not repeat it. Ordinary startup leaves diagnostics disabled.

## Visible-response recording protocol

Use the current installed artifact and record its archive hash/source separately.
A clip from an earlier artifact cannot qualify a later repaint optimization.
Record the original high-frame-rate file with the physical input and destination
display simultaneously visible. Start with ten isolated dial detents and ten page
changes, pausing between actions. Avoid destructive system keys and cover neither
the labels nor the display with the hand. Retain unedited originals locally rather
than publishing private desktop content.

Verify decoded frame count, frame timestamps, recording mode and real-time duration
before converting frame intervals to milliseconds. Nominal playback rate alone is
not a capture rate: exported slow motion can change the time base. If the real-time
mapping cannot be established, report visual behavior without a latency number.

For every usable event, retain frame evidence for the last unchanged input pose,
first clearly changed input pose, last old display state, first new state and first
fully settled destination. For a page, settlement includes all destination keys
and the touch strip, not just the first key. Dial volume text and mute indicators
are feedback; asynchronous activity-bar motion alone is not an action response.
Exclude obstructed, ambiguous, overlapping or externally changed events, recording
the exclusions and original event count rather than selecting only fast examples.

Camera-visible movement is a mechanical reference, not an electrical detent or
HID receipt timestamp. State this distinction. A confirmed movement onset preceding
the recognized isolated input can provide a conservative movement-to-settlement
upper bound, including frame uncertainty. Such a bound below the target supports
that event; a bound above the target does not by itself prove software failed,
because hand travel may dominate. Unclear input anchoring cannot close the gate.
Report first-change and fully settled intervals separately, sample counts, bounds
and per-route nearest-rank p95 only where the timing reference is established.
Sparse examples are not representative p95 evidence. Precise electrical-input
timing requires a synchronized external input marker or capture apparatus.

Keep this result separate from hardware-report-return dispatch and logged USB-write
completion. Do not align camera and journal wall clocks approximately and describe
the difference as exact input-to-pixel latency. The final release assessment must
state which physical routes, active-meter conditions, artifact and target boundaries
were actually measured and which remain open.

Adapter counts reset on connection replacement. Worker-output counts span the
worker's lifetime and include both queued and completed action records, and may
include editor-generated actions. They therefore are not one-to-one physical
event counts or latency samples. Report counts distinguish repeated key-state
reports from actual normalized edges, and queued edges from new reads. Use the
counts to locate a recording gap; qualify normal-operation timing/resource use
with the diagnostic window closed. A periodic snapshot does not guarantee final
counter delivery when a connection or daemon fails.
