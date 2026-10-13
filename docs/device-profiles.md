# Saved device profiles and cached controls

This is the next dormant host slice of ADR-0017. Explicit profile storage now
connects private device identity, a validated V2 document, saved page and metadata
to the supplied-adapter supervisor. Normal startup still uses V1/Control1 and
the Plus renderer. No physical device is automatically enrolled or opened.

## Private persistence

The host supplies an explicit private store directory and authoritative discovery
snapshot. Enrollment binds its chosen snapshot slot to a random nonzero 128-bit
ID; enumeration order, USB path, label and serial never generate that ID. The
registry retains the serial privately for reconnect matching. Missing/duplicate
identities and attempts to replace an existing association are rejected.

Plus can explicitly import a V1 document. Its original bytes remain separately
alongside the upgraded V2 layout; embedded artwork, Unicode, labels and unavailable
provider settings are preserved. MK.2 always starts with its own Home, Media and
System pages. It cannot import the Plus document through this enrollment path.
The first explicit enrollment becomes the saved selection; subsequent attachment
or reversed discovery does not change that selection.

Each profile has its own edit revision. A save merges under the cooperative writer
lock against the latest catalog: a change to another device does not invalidate
this device's draft, while two stale edits to the same device cannot overwrite
each other. Profile name/brightness changes preserve physical identity. Global
selection/preferences changes use the catalog generation. Global preferences are
bounded opaque JSON retained byte-for-byte, including unknown/large-integer tokens;
this slice does not apply login/service settings or reinterpret their paths.

Immutable generations contain the private registry, preferences, model documents,
original imported Plus documents and a checksummed manifest. Files use 0600 and
the store root uses 0700. Owner, regular-file, no-follow, hardlink, shape/model,
revision and checksum checks precede loading/publication. The complete generation
is synced and validated before swapping the pointer. An interrupted generation
is never adopted; an active pointer survives failures before publication.
A sync failure after pointer publication reports an uncertain commit and must not
be replayed automatically. Checksums detect change; they are not authentication
against another process running as the same user.

A missing pointer with existing generations fails closed. Explicit restore writes
a verified selected generation into a new destination, preserving IDs and
preferences without changing the source. Full installed-build, systemd/login,
external Secret Service and non-layout asset-directory backups still belong to
the existing deployment/migration workflow; this store is not a whole-user backup.

Bounds are eight profiles, a 1 MiB layout and optional 1 MiB original per profile,
64 KiB each for registry/preferences/manifest, and sixteen retained generations,
including failed ones. No backup is automatically deleted. **Retention/compaction
must be completed before enabling a live editor**; reaching the bound stops saves.
Runtime page/dial events do not write generations. Saved-page persistence is an
explicit save, separate from the current session page; coalesced lifecycle writes
remain integration work.

## Device-addressed controls

Loading a catalog compiles action settings into an immutable table per device.
Key actions, effective shared/page-override dial assignments, preferred players,
plugin availability and touch navigation reuse validated settings. Dial inheritance
uses the existing resolver. Decoded artwork is not retained by these action tables;
layout topology is cached with the immutable document. Input lookup does no file
I/O, JSON parsing or image decoding.

The manager validates/drains genuine input edges. Resolution checks the exact
current device context and typed control before using that device's current page.
Navigation and brightness target that device only. All fifteen MK.2 key addresses
are accepted; its nonexistent dials/touch are rejected. Applying a saved profile
fences only its owner; lock/suspend gates still fence both. Reconnect retains page,
brightness and association, and rejects old connection contexts. Public summaries
omit serials, preference values, layouts and settings.

This store and control cache now also feed the explicit
[device-addressed editor/rendering lab](profile-editor-lab.md). Its separate test
endpoint and supplied VirtualDeck pair never activate on normal startup. Production
bus integration and authorization still require review. Dispatching these results to shared audio/microphone/media/
plugin services, runtime nonce/press generation, canonical targets, dynamic live
enrollment, production editor/device selection, asynchronous rendering and MK.2 HID
ownership still need integration. Replacing a manager/service must quiesce old writers and fence
all old contexts; do not rebuild it merely to add a newly enrolled device.

See [device sessions](device-sessions.md), [layout formats](layout-models.md),
[shared backend lifecycle](shared-action-lifecycle.md) and the
[qualification checkpoint](checkpoints/2026-10-13-saved-device-profiles.md).
