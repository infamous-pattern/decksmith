# ADR-0004: Stream Deck + is the reference device; hide upstream HID crate behind an adapter

- **Status:** Accepted
- **Date:** September 2026

## Context

The initial hardware is a Stream Deck +. Public Elgato HID documentation and the Rust `elgato-streamdeck` ecosystem substantially reduce the need for hardware reverse engineering.

## Decision

Certify the Stream Deck + first. Use a project-owned `DeckDevice` abstraction and implement it using the selected upstream Stream Deck Rust library. Do not expose upstream HID/library types outside the device crate.

## Consequences

Positive:

- fast hardware bring-up
- upstream changes are isolated
- mock devices are easy to implement
- future device models can reuse the core action/profile architecture

Negative:

- adapter maintenance is required
- unsupported upstream device quirks may require targeted patches or contributions upstream
