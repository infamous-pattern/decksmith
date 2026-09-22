# ADR-0013: Pin the upstream Plus adapter and review its license

Status: Accepted
Date: 2026-09-10

## Context

ADR-0004 selects elgato-streamdeck behind the project-owned device boundary.
The reviewed crates.io version 0.13.1 is MPL-2.0, rather than Apache-2.0.
Mozilla's FAQ explicitly permits combining MPL and Apache code in a larger work.

## Decision

Pin elgato-streamdeck =0.13.1 as an optional hardware dependency. Keep its source
unmodified, retain its license, and do not copy its implementation into original
Decksmith files. Original project code stays Apache-2.0. deny.toml allows MPL-2.0
only for this package/version; this is not a blanket license exception.

Use its synchronous interface, with bounded read timeout, behind PhysicalDeck.
It belongs on a dedicated future daemon worker; do not enable upstream async/Tokio.
All public device input uses Decksmith types. Hardware feature is off by default.

The selected upstream enables hidapi's default Linux static-hidraw backend, which
needs libudev headers. Fedora systemd-devel provides them. A signed matching Fedora
RPM was extracted into task-local build storage for compilation without installing
system packages. No upstream code or dependency manifest was patched to bypass this.

## Consequences

Before binary redistribution, package MPL license notices and make the exact covered
source available to recipients; retain notices for all other dependencies, including
the system libudev linkage. Release packaging is not implemented in this checkpoint.
Future upstream modifications remain separately licensed and require source provision.

Sources:
- https://crates.io/crates/elgato-streamdeck/0.13.1
- https://www.mozilla.org/en-US/MPL/2.0/FAQ/ (Q8, Q11 and Q13)
- https://docs.rs/elgato-streamdeck/0.13.1/elgato_streamdeck/
