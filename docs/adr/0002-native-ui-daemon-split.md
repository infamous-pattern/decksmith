# ADR-0002: Native GTK configurator with separate Rust daemon

- **Status:** Accepted
- **Date:** September 2026

## Context

The product must feel native on Fedora GNOME and the hardware must continue working when configuration UI is closed.

## Decision

Use GTK4/libadwaita for the configurator and a separate Rust `systemd --user` daemon for hardware, actions, profiles, persistence, and plugin supervision. Communicate over a versioned D-Bus API.

## Consequences

Positive:

- native GNOME UX
- durable background behavior
- clear process boundaries
- CLI and future clients can reuse daemon API

Negative:

- more IPC/API design than a monolithic desktop application
- daemon/GUI version compatibility must be managed explicitly
