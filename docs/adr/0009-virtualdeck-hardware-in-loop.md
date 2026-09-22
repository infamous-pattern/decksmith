# ADR-0009: First-class VirtualDeck plus mandatory hardware-in-loop release testing

**Status:** Accepted  
**Date:** September 2026

## Context

Development and CI need deterministic Stream Deck behavior without requiring every contributor or runner to own hardware. At the same time, real HID transport, firmware behavior, touch gestures, reconnect, and USB concurrency can diverge from simulations. Existing Linux Stream Deck projects have encountered bugs that were invisible in emulators.

## Decision

Implement VirtualDeck in Phase 0 behind the same project-owned device abstraction as physical hardware. It uses real device geometry/capability descriptors, the production renderer, the Trigger Resolver, and normal binding/capability paths. Tests can create multiple virtual devices, inject raw events, disconnect/reconnect devices, and inspect rendered output/state.

VirtualDeck does not replace physical testing. Hardware-in-loop tests remain explicit release gates for reference-device HID, key/dial/touch input, rendering, reconnect, suspend/resume, and later identical-device concurrency.

## Consequences

- Most development and CI can run without attached hardware.
- Multi-device logic can be tested at scale before acquiring multiple devices.
- Physical Stream Deck + testing remains necessary before Beta/1.0.
- Device and renderer contracts must avoid implementation details that only make sense for USB hardware.
