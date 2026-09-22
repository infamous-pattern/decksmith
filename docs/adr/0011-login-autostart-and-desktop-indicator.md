# ADR-0011: Start the daemon at graphical login and expose lightweight desktop quick access

**Status:** Accepted
**Date:** 2026-09-09

## Context

Decksmith controls persistent physical hardware. Requiring Decksmith Studio to remain open would waste resources and create an unreliable user experience. Users also need fast access to Studio while the daemon operates invisibly in the background. GNOME does not provide a traditional tray surface to ordinary applications by default.

## Decision

- `decksmithd` starts automatically with the graphical user session by default.
- Decksmith Studio does **not** auto-open and is launched only on demand.
- A **Start Decksmith at Login** preference controls daemon login activation and defaults to enabled.
- The existing Decksmith GNOME Shell companion provides the Decksmith panel icon/menu on the Fedora/GNOME reference desktop.
- On desktops with StatusNotifierItem support, `decksmithd` exposes a standard status item/menu.
- The quick menu opens/focuses Studio and surfaces device/runtime state, active profile/workspace, pause/resume, Preferences, and Diagnostics.
- Users may hide the indicator without stopping the daemon.

## Consequences

The heavy GTK UI does not consume resources continuously. Quick access remains available, GNOME needs no third-party tray extension, and background device behavior has a clear session-lifetime owner.
