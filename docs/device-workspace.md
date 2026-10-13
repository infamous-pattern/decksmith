# Saved-device workspace integration — V1.5 development

The opt-in saved-device workspace connects actual read-only hardware discovery,
private persisted profiles and the native device chooser. It can run alongside
installed V1 without acquiring USB ownership or altering physical controls.
It is a development integration checkpoint, not released multi-device control.

## Explicit startup and enrollment

Build the hardware-enabled daemon, then start a separate workspace service with
an absolute private store destination whose parent already exists:

```sh
cargo build --workspace --all-features
target/debug/decksmithd --device-workspace "${XDG_STATE_HOME:-$HOME/.local/state}/decksmith-device-workspace" --run
```

If `XDG_STATE_HOME` is unset, use an explicit absolute directory beneath
`~/.local/state` instead. The service exclusively creates a missing store or
validates and opens an existing one. Invalid/corrupt/incomplete stores are never
silently replaced. No production layout, preference or service is migrated by
normal startup. Stop the separately launched process to close this endpoint;
installed V1 continues running.

Optionally prepare a caller-selected V1 Plus layout for explicit import:

```sh
target/debug/decksmithd --device-workspace "$HOME/.local/state/decksmith-device-workspace" \
  --import-plus "$HOME/.config/decksmith/layout.json" --run
```

The bounded read refuses symlinks, nonregular/replaced files, oversized input and
non-V1 Plus documents. Preparing an import does not enroll a device. Choose the
separate import button to retain the original bytes and upgrade a validated copy.
The MK.2 always starts with its own fifteen-key defaults. Global login/lock/audio
connection settings remain with the installed service; this command imports only
the selected layout. Saved profiles contain private identity associations and may
contain private provider settings; do not publish their stores or archives.

Open the native workspace explicitly:

```sh
python3 apps/decksmith-studio/device_workspace.py
```

Its application identifier differs from the service identifier. It never starts
or auto-activates a service, changes the installed launchers or falls back to
Control1. A new store shows explicit model/slot enrollment choices. Choose Device
then provides the same single editor with independent drafts, model geometry,
Home name/brightness settings and retained history. An absent saved device remains
editable. Add detected device and Refresh re-read the roster; missing or duplicate
identities cannot be enrolled. Discovery status is separate from control readiness.

The UI says **Save profile**, and explicitly states that physical changes are not
applied. Test Action, login and Auto-Lock controls are unavailable here. Device
name, saved brightness, pages and artwork are persisted only in this workspace's
private store. Neither selecting a device nor changing a page controls the live
Deck. Closing this window does not stop the normal service.

## Device-addressed API and bounds

Session-bus name `cc.senecal.Decksmith.DeviceWorkspace`, object
`/cc/senecal/Decksmith/DeviceWorkspace`, interface
`cc.senecal.Decksmith.DeviceWorkspace2` exposes the existing bounded profile
methods plus explicit `EnrollDevice(runtime, discovery_revision,
catalog_generation, slot, name, brightness, import_plus)`.

`ListDevices` reports `mode: saved-workspace`, saved identities, selection,
disconnected manager sessions and a redacted discovery roster. No raw hardware
serials or paths are returned. `actions_available` is false. A detected unit is
not advertised as connected/ready without an owned adapter. Discovery runs once
per five seconds, independently of UI activity, and refreshes before enrollment.
Its revision changes when availability, identity or snapshot order changes;
requests from an older process or snapshot are rejected. Bus-name replacement
is explicitly disabled; duplicate services fail before touching a store. Slots never become
persistent identities. Explicit enrollment merges one new profile without
rebuilding other devices' state. The device limit remains eight; discovered
candidates remain bounded to sixteen.

Profile mutations use startup/profile/connection/binding tokens. Offline saves
validate the destination model and preserve unrelated profiles. Offline page
browsing refuses a live adapter; the workspace also refuses saves if an adapter
is accidentally attached. Read-only previews reuse the accepted model renderer
and bounded cache. Queue size, request limits, timeout uncertainty and verified
history behavior match the [profile lab](profile-editor-lab.md).

Bus handlers await bounded asynchronous replies, avoiding executor starvation
under concurrent requests. Disk publication and preview rendering run on this
endpoint's serialized service thread; GUI requests use the editor's request worker. There is no physical input
loop to stall in this slice. Move persistent writes, previews and physical display
work off the input path before combining it with live sessions. Initial roster and
status reads still use the inherited synchronous lab reads; move those to bounded
background requests before production UI activation as well. Automatic archive
quota/retention and recovery UI remain gates.

## Qualification and remaining work

See the [October 13 checkpoint](checkpoints/2026-10-13-saved-device-workspace.md).
The fresh isolated native fixture enrolls both models, imports a prepared Plus
layout, edits Key 15, saves while disconnected, switches in one stable window and
retains another device's layout/Home drafts. It requires a completed outcome and
screenshots, not just process exit success. Rust fixtures cover restart identity/
selection/page persistence, stale requests, discovery failure/ambiguity, original
import retention and refusal of accidentally live adapters.

The actual workstation coexistence test used a separate private store, detected
both attached models, enrolled and reopened their profiles, and verified unchanged
production configuration/build/page/brightness and a ready Plus display. No HID
handles were opened by the workspace and no production service was restarted.

Before activation, complete shared action target resolution/execution, microphone
holds, live audio feedback, lock/suspend integration, asynchronous display/disk
scheduling, migration/recovery policy and paired lifecycle/resource acceptance.
The existing production `--config ... --exclusive --run` path is unchanged.

Further [restart and concurrent-request qualification](checkpoints/2026-10-13-device-workspace-hardening.md) covers endpoint ownership, bounded busy responses and cancellation.
