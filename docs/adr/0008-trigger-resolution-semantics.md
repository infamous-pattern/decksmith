# ADR-0008: Separate raw input events from deterministic semantic trigger resolution

**Status:** Accepted  
**Date:** September 2026

## Context

Stream Deck + keys and dial pushes expose physical down/up edges, while users expect higher-level interactions such as short press, double press, long press, and hold/repeat. If each capability interprets timing independently, behavior becomes inconsistent, difficult to test, and prone to conflicts.

## Decision

Introduce a daemon-owned Trigger Resolver between normalized physical device events and binding/capability execution. Raw key/dial edges remain observable and may be bound explicitly for advanced edge-triggered workflows, but ordinary actions bind to semantic triggers.

The resolver uses monotonic time and deterministic conflict rules: a configured double-press binding defers the first short press; a recognized long press suppresses short/double; hold-repeat begins only after its configured threshold/delay and stops on release. Timing has global defaults with optional advanced per-binding overrides.

Keys support short, double, long, hold/repeat, raw press, and raw release. Dial pushes support corresponding short/double/long/hold semantics plus independent rotation. Touch supports only gestures the hardware protocol exposes.

## Consequences

- Capability implementations remain free of ad hoc timing logic.
- Input behavior can be replayed deterministically in VirtualDeck and CI tests.
- Double-press mappings introduce a small intentional delay before a single-press action when both are configured.
- The configurator must warn about combinations of raw-edge and semantic gesture bindings that intentionally execute more than one action in a sequence.
