# ADR-0010: Standardize the product identity on Decksmith

**Status:** Accepted
**Date:** 2026-09-09

## Context

The architecture baseline used DeckNative as a temporary working title. The project now needs stable terminology before Phase 0 repository scaffolding and package naming.

## Decision

Standardize on:

- **Decksmith** — product/project
- **Decksmith Studio** — GTK4/libadwaita configurator
- **decksmithd** — daemon
- **decksmithctl** — CLI
- **The Forge** — creation/editing area inside Studio
- **Blueprints** — reusable profile/template packages
- **Decksmith Foundry** — future community catalog
- **Tagline:** **Open Control, Forged for Linux.**

“Stream Deck” is used only descriptively for supported Elgato hardware and is not part of the product brand.

## Consequences

Source layout, user-facing names, package names, documentation, and executable naming use Decksmith terminology. The final reverse-DNS application ID and public namespace remain pending a namespace/trademark check before public launch.
