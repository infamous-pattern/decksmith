# Device registry and session supervisor foundation

This is the first part of ADR-0017's third implementation slice. The new modules
are compiled library components tested with supplied VirtualDeck adapters. The
production daemon still runs its existing single-Plus worker. There is no new
device selector, Control2 bus endpoint, automatic saved-layout migration or
physical MK.2 support in this change.

## Identity and private inventory

`DeviceId` is an opaque, nonzero 128-bit value with a canonical lowercase hex
encoding. The future host enrollment flow must supply random bits; paths,
serials, names and discovery order must never generate an ID. IDs are validated
on deserialization. `DeviceContext` carries a process-local supervisor instance,
device ID, connection generation, binding revision, lock epoch and typed control.
Recreating the supervisor cannot reuse a previous supervisor's callback context.
The eventual wire API also needs a startup nonce to fence previous daemon processes.
This context is a stale-work fence, not
an authorization or plugin permission grant.

The version-1 private registry codec records device ID, known model, validated
serial, custom name and brightness. Names allow Unicode and emoji, up to 64
characters/256 bytes, excluding control characters. Brightness is 0–100. There
are at most eight enrolled devices, sixteen discovery candidates and 64 KiB of
encoded inventory. Serial validation permits 1–64 ASCII letters, digits, dots,
underscores and hyphens. The model selects a known VID/PID in the physical
adapter; arbitrary vendor/product identities are not accepted here.

Enrollment is explicit and checks an expected registry revision. Existing IDs
and serial/model associations cannot be silently replaced. Missing/invalid or
duplicate serials remain unassigned; unique new units also remain unassigned
until enrolled. Snapshot-local slots only locate candidates within that snapshot.
Reconnect order does not select a layout or become durable identity.

Private encoding intentionally contains serials. Do not include it in layout
exports, diagnostics or repositories. Public status summaries omit serials,
and debug formatting redacts them. The codec performs no filesystem I/O: private
atomic persistence, generated IDs and integration with the cold migration
generations still need a host adapter. This is not an activated device database.

## Independent sessions and fair polling

The manager retains disconnected inventory and separate page, saved brightness,
input-edge state, connection generation, binding revision and fault status for
each enrolled unit. Layout topology is supplied separately from a fully validated
document; bounds checking here does not replace semantic/image validation. Each
session remains non-dispatching until configured and its initial ready input is
drained. The cold legacy Plus profile and MK.2 defaults have not been associated
automatically with physical units.

Attachment requires the current discovery revision, a unique assigned candidate,
the expected model and no existing owner for that ID. The host still must open
and verify the correct physical unit under one daemon-wide ownership lease. A
matching descriptor alone cannot verify which serial an arbitrary supplied
adapter opened. No physical opener or ownership-lock changes are implemented
by this seam.

Polling rotates its starting unit and delivers at most one input per unit per
turn. A draining unit gets at most 32 reads per turn, so a noisy reconnect cannot
starve another unit. These bounds require the future physical adapter's poll to
be genuinely nonblocking. Incoming events are checked against model surfaces,
rotation/touch validity and connection-local monotonic time. Physical receipt
timestamps are retained when the adapter supplies them; VirtualDeck invents none.

Page or layout changes fence only the affected unit. Disconnects, ambiguous
identity and transport/report failures close only that unit's owner. A subsequent
attachment advances its connection generation; old feedback cannot write to it.
Device ID remains part of the fence even when two devices have equal counters.

Shared lock/suspend state fences every unit on both transitions. Each drains
independently before resuming. A control held across a fence must be released and
pressed again; queued rotation and release events do not become fresh actions.
Saved pages remain intact. Stop attempts every surface on every owned device,
continues after an error and returns IDs that could not be blanked. Adapter write
timeouts and end-to-end shutdown timing remain physical integration requirements.

## Remaining integration

The supervisor emits context-tagged input and accepts fenced key feedback; it
does not execute action bindings, render layouts or start backend services.
It must not be activated simply by copying the existing worker per device.
Before production integration, complete:

- Private atomic registry/profile/preferences persistence and explicit enrollment,
  with lossless legacy association and matched executable/configuration recovery.
- Device-addressed dispatch/API, startup nonce, request IDs, foreground revisions and queue/result
  ownership, including action-side checks immediately before external effects.
- One shared audio/media/plugin observation service, deduplicated meters, target
  write serialization and a microphone coordinator with per-device hold leases.
- Dynamic renderer caches, scheduler/write timeouts and independent retry/backoff.
- Descriptor-driven editor/drafts and a verified physical opener/MK.2 adapter.
- Resource, accessibility, migration and physical Plus/MK.2 pair acceptance.

See [ADR-0017](adr/0017-device-scoped-actions-and-settings.md),
[layout models](layout-models.md) and the
[session checkpoint](checkpoints/2026-10-12-device-session-foundation.md).
