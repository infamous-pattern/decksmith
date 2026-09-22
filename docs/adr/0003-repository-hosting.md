# ADR-0003: Gitea for private incubation; GitHub canonical when public

- **Status:** Accepted
- **Date:** September 2026

## Context

The project will be built locally on Fedora, is intended to become open source, and a local Gitea service is available.

## Decision

Use Gitea as the private writable origin during incubation. At the first public developer preview, GitHub becomes the sole canonical public collaboration repository. Configure Gitea as a mirror/backup afterward.

## Consequences

Positive:

- private/local control during unstable early work
- public discoverability and contributor workflow when ready
- local backup remains available

Negative:

- issue metadata created in Gitea may require manual migration, so early issue usage should remain light
