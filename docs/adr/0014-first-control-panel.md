# ADR-0014: First native control panel and versioned D-Bus boundary

Status: Accepted implementation checkpoint, 2026-09-10.

ADR-0002 requires GTK4/libadwaita, a separate Rust daemon, and a versioned D-Bus API.
This first thin panel uses the workstation's installed PyGObject bindings for GTK4
and libadwaita because GTK development packages are absent. Hardware ownership,
layout validation, brightness and preference persistence remain in Rust. This is an
interim client, not the full Rust Studio workspace planned in the architecture.
A future Rust client can use the same versioned bus boundary. No baseline document
is changed or treated as completed by this checkpoint.

The session-bus name is cc.senecal.Decksmith, object /cc/senecal/Decksmith, interface
cc.senecal.Decksmith.Control1. Methods are GetStatus (versioned JSON string),
SetBrightness (byte) and SetLayout (known preset name). Mutations cross a bounded
queue to the device owner; clients never open HID. It is a trusted per-user session
API, not a network service or a multi-user authorization boundary.

The zbus dependency requires syn 2 while existing macros use syn 3. The dependency
gate has one exact-version duplicate exception for syn 2.0.119, a build-time parsing
library under the existing allowed licenses. Other duplicate versions remain denied;
advisory, license and source checks remain enforced.
