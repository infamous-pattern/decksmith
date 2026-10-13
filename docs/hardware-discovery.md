# Physical discovery and model transport foundation

The October 13 V1.5 slice provides an explicit Plus/MK.2 opener and saved-profile
enrollment seam. It remains dormant in normal startup. Installed V1 still owns
one Plus through its existing worker; this is not released multi-device support.

## Discovery and identity

`Inventory::discover` enumerates the pinned HID dependency without acquiring a
Decksmith lease, opening a Deck session or sending device reports. It admits only
the known Plus and standard 15-key MK.2 product IDs. Other revisions, including
scissor/module variants and XL, retain their separate certification gates.

The inventory is bounded to 16 supported HID entries, sorted by private locator.
Its slots are temporary, never stable IDs. Each snapshot carries the existing
project-owned private identity candidates; `Catalog::associations` resolves these
against the privately saved registry. Missing/invalid/duplicate identities remain
unassigned. Duplicate entries are deliberately retained rather than hidden by
the dependency's serial-deduplicating convenience list. Public summaries and Debug
output omit serials and USB paths.

The source-build command `decksmithctl hardware inventory` is a read-only
diagnostic. It returns temporary slots/models and reasons for non-association
against an empty registry; `new_device` there does not imply a missing production
profile. It does not enroll devices or read production configuration.

## Ownership and opening

One `HardwareOwner` acquires the existing cooperative Decksmith lease. Every
returned session retains the same lease, even if the owner is dropped first.
At most eight identities may be claimed; a second handle for the same identity
is rejected. A failed open releases its claim. The private lock refuses links,
nonregular files, shared permissions and foreign ownership. This is cooperative
application protection, not OS-exclusive HID ownership against unrelated software.

`Controls::discover_hardware` supplies an inventory snapshot to the supervisor.
`attach_physical` first proves an assigned saved identity, exact snapshot/revision
and an unopened session. The owner rejects missing/duplicate identities or USB
paths, refreshes inventory, checks the enumeration used for opening, reads the
opened handle's serial and revalidates afterward. The pinned dependency opens by
model/serial, not an exposed path constructor; these checks reject locator changes
and ambiguous selection without claiming an atomic USB hotplug transaction.
Transport failure after opening follows the existing affected-device fence.

Constructing a session sends no reset, brightness or image write. Serial verification
does read a feature report. Attach is a separate operation which applies only that
device's saved brightness. Cold enumeration, open and verification belong off the
GTK and input-processing threads. Production asynchronous scheduling remains open.

## Model-specific transport and enrollment

`PhysicalDeck` uses the project descriptor for image dimensions and key bounds.
Plus keeps eight 120 × 120 keys, four dials and its existing changed-region touch
renderer. MK.2 uses fifteen 72 × 72 keys and rejects all dial/touch reports and
touch image writes. The pinned dependency owns JPEG conversion and MK.2 image
mirroring; a decoded asymmetric fixture verifies that transformation once.
Receipt timestamps remain shared by queued edges from the same report. New
sessions start with independent normalization and empty input/display caches.
Blanking attempts all model-supported surfaces.

After an explicit store enrollment, `Controls::enroll_saved_profile` compiles
only the new profile and extends the existing supervisor. It refuses identity
reassignment, missing old records, repeat enrollment, invalid topology/page and
blocked lock/suspend state. An extension invalidates pending discovery opens;
it preserves other sessions' connections, bindings, held inputs, pages, brightness
and frames. There is no implicit V1 migration, automatic enrollment or action
execution in this slice.

See the [qualification checkpoint](checkpoints/2026-10-13-hardware-discovery.md)
and [saved profile contract](device-profiles.md).

The subsequent [physical MK.2/paired trial](checkpoints/2026-10-13-physical-mk2-pair.md)
opened the actual devices, confirmed all MK.2 keys/orientation and independent
paired pages/brightness, and verified MK.2 disconnect isolation/recovery. The resumed trial also checked Plus removal/recovery while MK.2 continued and
Plus touch/dial-push input. Shared backends, production integration, reversed
physical order and full paired lifecycle/resource qualification remain gates. The installed desktop stays on V1 ownership.
