# ADR-0007: Store visual assets on disk by content hash and make themes independently shareable

- **Status:** Accepted
- **Date:** September 2026

## Context

Profiles, themes, workflows, and visual assets need to be portable while keeping SQLite small and preventing duplicated image data. User customization should be shareable independently of behavior.

## Decision

Store asset bytes under the XDG data directory using content hashes/deduplication. SQLite stores asset metadata and references only. Define separate versioned export/import artifacts for profiles/behavior, themes/appearance, workflows, and explicit bundles. SVG is the preferred scalable visual source format, with common raster formats supported. Exports never include secrets by default.

## Consequences

Positive:

- duplicate artwork is stored once
- theme sharing does not modify behavior mappings
- profile packages can declare dependencies/assets predictably
- database backups remain compact

Negative:

- backup/recovery must keep database and asset store consistent
- import validation must verify hashes and missing dependencies
