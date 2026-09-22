# ADR-0001: Use SQLite for local persistence

- **Status:** Accepted
- **Date:** September 2026

## Context

The application stores profiles, pages, control mappings, action instances, plugin metadata, device preferences, and other local desktop configuration. It does not require a network database for its core use case.

## Decision

Use SQLite as the sole V1 configuration database, with WAL, foreign keys, ordered migrations, and pre-migration backups. Store secrets separately using Secret Service/libsecret.

## Consequences

Positive:

- no database server dependency
- transactional configuration
- easy backup/restore
- straightforward RPM deployment
- excellent Rust support

Negative:

- multi-host synchronization is not provided by the core database
- future distributed/fleet features would need a different service layer rather than simply exposing the local database
