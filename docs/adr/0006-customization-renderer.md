# ADR-0006: Customization-first layered renderer with behavior/appearance separation

- **Status:** Accepted
- **Date:** September 2026

## Context

User customization is a core design criterion. Required use cases include one artwork image spanning the entire Stream Deck + key grid, a separate full-width touch-strip background, individual control backgrounds, state-specific visuals, themes, icons, typography, and live WYSIWYG editing. The eight Stream Deck + keys are physically separate displays, so panoramic artwork must be sampled through device geometry rather than manually sliced by the user.

## Decision

Keep functional mappings and appearance as separate persisted domains. Implement a layered renderer with explicit `DeviceGeometry` and appearance inheritance:

Theme -> Profile -> Workspace -> Page -> Control -> State.

The compositor supports a panoramic key-grid canvas, independent touch-strip canvas, control/state overrides, SVG-first assets, labels/values/status overlays, and production-render-model reuse by both the GTK preview and hardware path. Static backgrounds are a V1 feature; continuously animated decorative backgrounds are deferred.

## Consequences

Positive:

- themes can be changed/shared without changing what controls do
- panoramic backgrounds require no manual image slicing
- preview and hardware can remain visually consistent
- future devices can use the same appearance model through different geometry

Negative:

- rendering/cache/inheritance logic becomes a significant subsystem
- live preview requires update coalescing to protect USB/device responsiveness
