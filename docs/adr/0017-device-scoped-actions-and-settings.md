# ADR-0017: Device-scoped actions, settings and recovery for V1.5

**Status:** Proposed implementation decision; architecture review complete

**Date:** October 12, 2026

**Reviewed source:** `55caa4bd061d7fae14f98cc8f8a03b5d3665cb91`

**Implementation checkpoints:** model descriptors/VirtualDeck and the
model-document/action-identity/offline-migration foundations are implemented.
See the [model qualification](../checkpoints/2026-10-12-device-model-foundation.md)
and [layout/migration qualification](../checkpoints/2026-10-12-model-layout-migration.md).
Live registry/preference assignment, runtime/API, renderer/editor and physical
integration remain staged proposals. The findings table describes the reviewed
revision before these slices.

This review defines the next implementation slices. It does not change the
installed daemon, saved layouts or hardware support claims. The user confirmed
that a newly added MK.2 should start with **its own default pages**, rather than
copying the Plus layout. Independent simultaneous Plus/MK.2 control is required
for V1.5; each model still needs physical acceptance.

## Conclusion and scope

Keep the native client/daemon split, project-owned HID adapter, deterministic
core resolver and bounded worker approach. Introduce a device manager and
explicit device/action context around these boundaries. Generalize geometry and
saved layouts before enabling MK.2 or simultaneous devices. Reuse existing
rendering and Linux integrations through small compatibility adapters.

The v0.4 architecture and ADRs remain the design baseline, but distinguish design
from delivered code: current V1 stores layout/preferences in JSON, implements a
single Plus session, and has a limited experimental OpenHomeB bridge. SQLite,
the full profile/workspace hierarchy and a generic plugin host are not implemented
by those paths. Do not combine device support with a storage or UI rewrite.
ADR-0001's SQLite decision remains a separately tracked architecture discrepancy;
continuing JSON for these slices is an interim proposal, not a claim that the
original persistence requirement has been completed.

## Findings from current code

Locations below refer to the reviewed revision; they are navigation pointers,
not claims that later source will retain those line numbers.

| Area | Working foundation | Gap before MK.2 / two devices |
| --- | --- | --- |
| Device contract | `DeckDevice` separates HID from domain and UI | `crates/decksmith-core/src/lib.rs:5` describes only Plus geometry; `crates/decksmith-device/src/lib.rs:94` always blanks a touch surface; VirtualDeck has eight fixed key buffers and Plus input bounds |
| Hardware selection | Sole ownership and ambiguity rejection | `crates/decksmith-device/src/hardware.rs:71` opens exactly one Plus; `ownership.rs:26` acquires one process-wide lock for each physical open; normalizer at `hardware.rs:296` has fixed eight-key/four-dial state |
| Saved layout | Strict bounded parsing, custom labels/artwork and unknown plugin bindings | `apps/decksmith-daemon/src/pages.rs:44` accepts version 1 with eight keys/four dials; `control.rs:14` and `control.rs:65` address one preference/layout file; no durable device association |
| Device session | Recovery generations, lock gates, resume draining | `apps/decksmith-daemon/src/main.rs:221` starts one session; `worker.rs:65` owns one page, display and backend set; a second copy would duplicate backend workers |
| Display | Latest-wins frames, one write per turn, duplicate suppression | `apps/decksmith-daemon/src/display.rs:5` fixes nine slots, with slot 8 meaning touch; `pages.rs:1627` renders 120-pixel key images |
| Action lifecycle | Bounded audio/plugin dispatch, stale-session and lock checks | `audio_worker.rs:22` requests carry page/session but no device; `audio_worker.rs:239` has one active session counter; plugin queue compatibility at `plugin_runtime.rs:27` cannot distinguish equal counters from different devices |
| Push-to-talk | Release bypasses normal queues; overlapping holds work in one session | `ptt.rs:10` indexes holds by `u8`; `worker.rs:606` uses key index and dial index + 8; two independent guardians for the same microphone could mute each other's holds |
| Gestures | Core resolver has deterministic timing tests and CLI/VirtualDeck replay | `pages.rs:754` arms actions on down and executes on release; the general `PressResolver` is not wired into the production daemon |
| Client and packages | Integrated editor, native accessibility and bounded archive validation | `apps/decksmith-studio/editor.py:181` builds eight keys/four columns; `workspace_shell.py:41` exposes a Plus-only informational selector; layout package version 1 has no device-model metadata |
| Plugins | Bounded provider/action/schema/settings; unknown bindings retained | `plugin_binding.rs` supports a specific OpenHomeB subset, not generic protocol compatibility or enforceable permissions |

The pinned `elgato-streamdeck` 0.13.1 already includes `Kind::Mk2`, PID `0x0080`,
15 keys, five columns/three rows and 72-pixel JPEG key images. Inspection of
`src/info.rs` confirms this adapter seam; it does not establish Decksmith support.
Elgato's [Classic-family HID reference](https://docs.elgato.com/streamdeck/hid/stream-deck-classic/)
documents MK.2 identity, geometry and image upload orientation. Keep orientation,
encoding and model-specific report padding inside the adapter; do not apply the
Plus's two padding-byte rule to MK.2 or rotate images twice. Physical tests must
confirm key ordering, upright images, brightness and input edges.

## Decision: ownership and identity

One daemon owns a `DeviceManager`; Studio never opens HID. The manager retains
the existing cooperative daemon-wide ownership lease once for its lifetime,
rather than letting every device worker reacquire the same exclusive file lock.
It enforces one live owner per physical unit and supervises bounded discovery and
reconnect. Existing competing-application checks remain safeguards, not an OS
HID exclusivity or sandbox guarantee.

Separate these identities:

- **DeviceId:** generated durable ID used by configuration, IPC and the editor.
- **PhysicalIdentity:** private model/VID/PID/validated serial association used
  to find a unit again. USB path is only a connection hint, never durable identity.
- **ConnectionGeneration:** advances whenever that unit disconnects/reopens.
- **BindingRevision:** changes when its page, assignment or configuration changes.

Reversed reconnect order must not move layouts. Persist disconnected devices and
their names. If serial identity is missing, duplicated or otherwise ambiguous,
show an unassigned device and require an explicit association; never guess from
enumeration order. Do not overwrite an existing device association during a
model change. Public layout exports omit physical serials and host device IDs.

Each device owns layout/default page, active page, brightness, input-edge state,
gesture resolvers, display cache, retry/backoff and status. Lock, foreground app,
resume observation, audio/media/system state and plugin supervision are shared
services. Apply shared desktop observations independently to each device's page
rules. Manual navigation on one unit does not change another unit's page.
Start-at-login and Auto-Lock remain global preferences; adding a device does not
create another daemon service or change those saved preferences.

## Decision: descriptors and rendering

Use project-owned model descriptors with key grid, key image dimensions, dial
count, optional touch surface/gestures and supported output operations. Preserve
the distinction between geometry and supported operations. Do not leak upstream
`Kind`, image transforms or report types through the public contract.

Bound vectors by validated descriptors rather than eight-key/four-dial arrays.
Check nonzero dimensions, control counts and pixel-buffer multiplication against
explicit allocation limits before allocating or accepting a descriptor.
Represent controls as typed `Key(index)`, `DialPush(index)`, `DialRotation(index)`
and supported touch controls, rather than an offset that assumes eight keys.
VirtualDeck takes the same descriptors and validation rules as physical devices.
Reject events, images and assignments for unavailable controls; blank only the
actual displays. MK.2 must not create a fake touch strip or meter process.

Keep raw input local to the adapter. The manager attaches device/context metadata
without changing the existing raw event variants unnecessarily. Render at the
selected descriptor's dimensions, with shared appearance/layout semantics and
aspect-preserving artwork. Include model, dimensions, appearance revision and
asset identity in cache keys. Keep Plus region-write optimization model-specific;
generalize latest-wins frame slots and fair write scheduling for each device.

## Decision: action identity and dispatch

Introduce a project-owned action definition registry: stable provider/action ID,
settings schema version, supported control/trigger capabilities and permissions.
Display labels and translated names never identify an action. Keep user labels,
appearance and assets separate from action identity/settings.

For the first slice, normalize existing built-in enum variants through a total,
explicit mapping into the registry while retaining their existing execution
handlers. Do not rewrite every saved V1 action or switch on display labels.
Retain existing plugin provider/action IDs and schemas unchanged. A bounded,
structurally valid unavailable provider/schema remains saved and non-executable;
never fall back to a system-volume action. Known settings use version-specific
validation; semantic settings migration requires an explicit tested adapter.
Separate persisted definitions from compiled bindings, transient held state and
render caches incrementally within the current modules. Avoid extracting new
crates merely to reorganize files; the core types must stay free of I/O/HID.

Every dispatch/result carries `request_id`, DeviceId, connection generation,
binding revision, control/trigger identity, monotonic receipt time, lock epoch
and relevant foreground revision. Validate context before queueing, immediately
before execution and before applying feedback. A stale completion cannot repaint
or change the current page. Cancellation rejects future work; an external effect
already completed is not automatically undone or blindly retried.

Keep queue/result ownership per device while sharing backend access. Serialize
conflicting writes to the same audio/media/accessory target; preserve order for
accepted discrete presses. Coalesce only compatible relative adjustments for the
same target and context, never toggle, launch, confirmation or held-input events.
Provider failure affects that provider's assignments, not all built-in actions.

## Decision: cancellation and microphone holds

| Event | Scope and required behavior |
| --- | --- |
| Page/binding change, device disconnect or device replacement | Invalidate only that device's pending actions/gestures, holds and display feedback; other device continues |
| Shared lock or suspend | Fence all devices, clear pending work and held state; reopen/drain independently; restore pages without replay |
| Plugin disable/crash | Reject affected provider work and show unavailable; preserve settings; do not retry uncertain external effects automatically |
| Editor device switch | Change selected draft only; never execute an action, cancel another device's valid hold or implicitly apply unsaved edits |
| Stop/Quit/shutdown | Cancel all work/holds and attempt complete blanking of every connected surface; bounded shutdown reports devices it could not clear |

Use one microphone-hold coordinator keyed by target, with hold leases identified
by `(DeviceId, connection generation, control, press sequence)`. Release/cancel
removes the owner's lease; microphone stays live while another valid lease
exists. Expire each owner's heartbeat independently so one healthy device cannot
keep a failed device's hold alive. Preserve the existing fail-safe guardian:
last release/lease expiry/service stop attempts mute, not restoration to Live.
Explicit mute must cancel active holds for that target; unmute must never occur
as an automatic recovery of a canceled hold. Test target changes and failures.
Canceled physical holds require a new down edge before acquiring another lease.

Wire general press gestures to the existing core resolver in a later slice.
Preserve action-on-release for legacy single bindings; defer single only with a
double assignment, suppress single/double after a recognized Hold, and retain
PTT on raw press/release with explicit conflict validation. Resolver instances
are per device/control/binding; rotation and touch retain their own semantics.

## Decision: persistence and IPC compatibility

Add distinct versions for device-store, per-device layout and exported package.
Use a validated V2 layout with bounded key/dial collections and model descriptor
metadata, and a device registry referencing independent configurations. V1
Plus layouts remain loadable through a lossless adapter; unknown future format
versions are rejected without rewriting their files. No silent key truncation,
invented dials, label reset, artwork substitution or plugin settings loss.
Retain the existing 16-page, 1 MiB layout, 2 MiB package and 4 KiB plugin-settings
limits initially, with a separate explicit total budget for multiple devices.
Do not silently raise these limits to accommodate decoded caches or extra units.

Before migration, back up the complete legacy configuration/assets and record
the old executable and restore path. Stage validated files in a new generation,
sync them, and atomically activate the generation pointer; incomplete generations
cannot become current. Keep original V1 files intact for rollback. An older
binary must be restored with the matching V1 configuration snapshot; same-build
installer tests do not prove cross-schema rollback.

Attach the legacy layout to an unambiguous Plus only. If hardware is absent or
ambiguous, retain it as an unassigned legacy profile for explicit association.
Create the MK.2's own model-appropriate default pages when it is assigned. A
future explicit copy/import flow offers compatibility review rather than copying
automatically. Package V2 records required model/capabilities but no live host
identity. Preserve V1 import; export V1 only when content is fully representable.

Keep one session-bus name and add a versioned device-addressed API alongside
Control1. List devices, fetch descriptor/status and scope mutation/preview/save
to DeviceId plus expected revision. Legacy clients refer only to an explicitly
configured legacy Plus, and fail clearly if it is unavailable; never choose the
first connected unit. Update Studio, CLI, GNOME menu and service scripts together.
Rust remains authoritative for validation and preview; Python fast validation
uses shared fixtures. Keep separate drafts/undo per device and prompt to resolve
unsaved changes on device switching. Selection does not change physical pages.

## Decision: resource and plugin boundaries

One HID owner/scheduler per device, bounded fair dispatch and latest-only render
slots; avoid a thread/process per key or action. Move I/O and encoding off GTK's
main thread. Retain dirty-region/duplicate suppression, reuse decoded assets and
share desktop observations. Dedupe meter subscriptions by target and distribute
timestamped snapshots; the existing helper's four-target bound is real and must
not be raised at only one end of its protocol. Plus/MK.2 does not itself require
more than the Plus's four visible dial meters.

For RGB desired/sent frames alone, the proposed Plus/MK.2 pair needs
`2 * (8*120*120*3 + 800*100*3 + 15*72*72*3) = 1,637,760` bytes.
This is an allocation estimate, not total process memory or a measured resource
result: decoded assets, fonts, helper processes and additional caches also count.
Set explicit cache/message/queue budgets and measure idle/loaded CPU, RSS/PSS,
wakeups, helper counts and fairness before enabling the pair in a release.

The internal registry/context is a prerequisite for a native plugin contract,
not that contract's final wire format. Keep native plugins out of HID/GUI
ownership; mediate permitted operations and secrets in the host. Process crash
isolation alone does not enforce permissions. Before generic host implementation,
complete the required OpenAction mapping of manifest IDs, version negotiation,
key/dial/touch events, settings, property inspectors, rendering and unsupported
operations. Record verified versions and compatibility gaps in a separate host
ADR; the current OpenHomeB subset is not blanket OpenAction/marketplace support.

## Implementation sequence and acceptance

1. **Descriptors and VirtualDeck:** add identity/control types, Plus/MK.2
   descriptors, optional touch, bounded dynamic storage and model fixtures.
   Prove Plus behavior unchanged, 15-key bounds, invalid surfaces, blanking,
   upright frame contracts and deterministic input/gesture replay. No desktop
   install or physical ownership change is needed for this first slice.
2. **Layouts/actions and migration:** V1 compatibility adapter, model-aware V2,
   action registry mapping, atomic configuration generations and package fixtures.
   Test custom Unicode labels, theme/artwork, links, dial overrides, unknown
   plugins, failed writes, interrupted migration and matched binary/config rollback.
3. **Device manager and shared services:** context-addressed dispatch/API, per-unit
   recovery, shared observation/meter/PTT services and target serialization. Test
   equal generation numbers on different devices, overlapping microphone holds,
   stale results and one failed device/provider without interrupting another.
4. **Integrated client and MK.2 adapter:** descriptor-driven preview/settings,
   independent drafts/default pages, unavailable-device status and safe selector.
   Qualify all 15 physical keys, brightness, artwork orientation and recovery.
   Never advertise key-only support from upstream coverage alone.
5. **Pair qualification:** identical and mixed VirtualDeck fixtures; reversed
   reconnect order, ambiguity, rapid simultaneous input, lock/suspend, quit/stop
   blanking, persistence, migration and sustained resource checks. Then physical
   Plus + MK.2 acceptance. Physical identical-model testing stays separately
   recorded, not inferred from simulation.

Follow these foundations with gestures, sticky actions and dial features, using
the same context rules. A generic plugin host still requires its contract,
permission and resource gates. This ADR adds no runtime dependency and does not
claim numeric display-response or below-1% CPU qualification.

## Related decisions

- [Native UI/daemon split](0002-native-ui-daemon-split.md)
- [Project-owned device adapter](0004-reference-device-hid-adapter.md)
- [Capability/context model](0005-capability-context-workspace-model.md)
- [Deterministic triggers](0008-trigger-resolution-semantics.md)
- [VirtualDeck and physical acceptance](0009-virtualdeck-hardware-in-loop.md)
- [Experimental plugin bridge](0016-plugin-native-review-bridge.md)
- [Current V1.5 roadmap](../ROADMAP.md#v15-feature-plan--organized-october-8)
- [Hardware acceptance matrix](../hardware-support-matrix.md)
