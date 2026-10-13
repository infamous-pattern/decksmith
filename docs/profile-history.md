# Verified saved-profile history archives

The explicit profile store still bounds its active generations to sixteen. An
optional, **explicit** archive operation now keeps the newest two to eight
validated generations in that store and preserves older ones in a separate
private archive. The workspace lab's button keeps eight; it never deletes a
previous archive or runs cleanup from idle polling.

## Copy, verify, then prune

The caller supplies the expected active generation and a new destination outside
the source store. Stale revisions, nested/existing destinations, invalid bounds,
unpublished/incomplete/corrupt generations and unexpected files are rejected.
The cooperative writer lock covers the operation. Every generation is validated
before creating the archive, including profile model, identity, original legacy
bytes, preferences and checksums.

The archive contains copied immutable generations, their original names/IDs and
an independently valid current pointer. Files/directories retain private modes;
all copies are synced and revalidated before any source file is removed. The
active generation is never retired. Recovery can use the existing explicit
restore-to-new-destination operation on an archived generation.

Before pruning, a durable private journal is written in the source and a matching
provenance record in the archive. Remaining source files must exactly match their
verified archived counterparts before removal. No backup is removed from an
archive, and no unknown file is unlinked. The source's current pointer, profile
revisions, original imported layouts and live device contexts remain unchanged.
An already-selected device is a no-op and consumes no history generation.

## Interrupted operations

A failure while copying leaves the source untouched. A prepared archive may be
retained even if pruning never begins. Once the source journal exists, mutations
are blocked until explicit recovery; reads of the active generation remain usable.
Resume verifies the exact source/destination, generation, matching journal and
complete archive before continuing. It permits missing retired files only as part
of that specific journaled cleanup. A corrupt or wrong archive cannot release the
write block. A filesystem sync failure reports uncertainty rather than replaying
an operation automatically.

The lab exposes history counts and an explicit Archive older saves / Resume
verified archive button. Destinations use opaque random sibling names; callers
cannot supply arbitrary archive paths over this test API. Provenance includes
private filesystem paths and must remain private; device summaries, diagnostic
copying and portable layout exports do not include it. Checksums detect change,
not authentication against another process running as the same user.

## Release boundaries

Archives can grow over time. An automatic archival schedule, disk quota/pruning
policy, async disk worker and production recovery UI still require review before
live activation. The lab host processes this explicit cold operation on its
single supplied-adapter loop; that is **not** a production input-scheduling design.
The store is not a complete installed-build, systemd/login, Secret Service or
external-asset backup; matched deployment recovery remains separately required.
These fixtures and the [workspace lab](profile-editor-lab.md) never touch a
production store or physical HID.
