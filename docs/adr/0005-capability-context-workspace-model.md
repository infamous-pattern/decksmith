# ADR-0005: Use a Capability Engine with Context and Workspace hierarchy

- **Status:** Accepted
- **Date:** September 2026

## Context

The reference products show that a simple "button -> action" model becomes limiting once the Stream Deck + dials, touch strip, application-aware behavior, multi-state controls, and live providers are treated as first-class surfaces. The project also needs task-oriented navigation without duplicating capability implementations.

## Decision

Model functional behavior as reusable **Capabilities** rather than device-specific actions. Support Commands, Adjustments, Stateful Controls, Navigation, and Dynamic Providers. Organize user configuration as Profile -> Workspace -> Page with binding scopes of Global/Profile/Workspace/Page. Add temporary Context Layers and generic Control Views that can repurpose compatible device surfaces without rewriting saved page configuration.

Every profile has a Home Workspace. The Context Engine resolves active application/profile/workspace/page plus temporary Context Layers deterministically.

## Consequences

Positive:

- one capability can serve keys, dials, workflows, Control Views, CLI, and future surfaces
- richer Stream Deck + interactions do not require one-off action types
- persistent/global controls and task-oriented workspaces are cleanly represented
- dynamic Linux/provider state becomes a core concept

Negative:

- domain model is more sophisticated than a flat page/action mapper
- precedence and context transitions require strong tests and clear UX
