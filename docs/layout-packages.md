# Layout export, import and recovery

The editor provides Export layout, Import layout and Restore previous. Reopen the
panel after updating to see these controls. Export writes the current valid draft
to a .decksmith ZIP package with versioned manifest, behavior.json, appearance.json
and SHA-256-named PNG assets. Required artwork is embedded. This is the first layout
package checkpoint of the baseline's profile/theme/workflow packaging architecture;
standalone theme/workflow application is not implemented here.

Exports strip URL user information, query strings and fragments from website
actions without changing the live layout. The completion message reports how many
URLs changed. Queries needed for navigation must be re-entered after import. This
is structured-field sanitization, not a general secret scanner for arbitrary text,
URL paths or artwork. No browser cookies, desktop credentials or icon source files
are exported.

Imports are limited to 2 MiB of archive and total declared uncompressed data and
160 entries. Files are read in memory, never extracted. Duplicate entries, checksum
mismatches, unsupported versions and asset-name mismatches are rejected. The
reconstructed layout is validated with the daemon's existing complete schema and
image checks through read-only ValidateLayout before a confirmation appears.
Confirming loads a draft; Save and Apply changes the device. Discard restores the
previously loaded saved draft. Missing installed applications are listed on import.
Imported pages do not follow device indices until applied.

Before replacing layout.json, the daemon stages a copy to layout.previous.json.
A failed backup blocks replacement. This retains one previous saved version, not a
history: each subsequent save replaces it. Restore previous reads/validates that
backup and uses the same draft confirmation. Saving a restored layout backs up the
current layout first. The backup may contain original URL details and remains local.
The existing file/worker/settings operations are not a single transaction; previous
save-error/timeout semantics still apply.

## Verification — 2026-09-11

The full Rust quality gate passed (53 tests). Eighteen Python tests passed, including
package roundtrip with assets, URL sanitization without source mutation, checksum
rejection and malformed archives. Actual GTK checks verified import confirmation
and discard. A live export was decoded and validated through the daemon without
changing the current layout. Saving the same current layout created a recoverable
previous version, verified through GetPreviousLayout. Brightness and the active
page were retained/restored. A user-facing export was saved alongside the repository
as Decksmith-layout-2026-09-11.decksmith; no URLs required sanitization in that copy.
