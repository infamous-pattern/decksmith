# Physical discovery/transport foundation — October 13, 2026

This follows the local single-workspace/history slice and adds the dormant
[physical opener and enrollment seams](../hardware-discovery.md). It does not
install a desktop build, migrate V1 profiles or activate simultaneous control.

## Implemented

- Bounded, read-only Plus/MK.2 inventory, private locators and saved associations;
  missing/duplicate identity remains unassigned, with no public serial/path output.
- Exact snapshot/revision checks before physical opening, refreshed locator and
  handle-serial verification, one shared cooperative lease and bounded claims.
- Model-aware physical image bounds, all fifteen MK.2 key edges, rejected nonexistent
  surfaces, independent receipt/normalization state and model-specific blanking.
- Explicit enrollment of one saved device without recreating the supervisor or
  changing another owner's live context, page, held input, brightness or frame.

## Qualification

Ten new Rust fixtures cover missing/duplicate/oversized inventory, locator/model
changes, reordered enumeration, redacted Debug output, duplicate/limited claims,
linked/private lock files and shared lease lifetime, all MK.2 key edges and invalid
surfaces, image bounds/blanking/nonblocking reads, receipt/session reset, decoded
image mirroring, live enrollment preservation and atomic invalid enrollment.

The complete immutable snapshot
`badbec8e0c196d1b0745f1fed1472512a79c1871731c779d74b08fcbfddea255`
passed AI-server qualification: **266 Rust tests, zero failures and three ignored
environment/hardware cases**; all-feature workspace build, formatting and
all-target Clippy with warnings denied; Studio 191 tests and scripts 111 tests
(one conditional skip in each AI-server Python suite). Both the single-workspace
profile lab and existing V1 workspace native checks passed at 1920 × 1200 with
private directories, disposable desktop bus and no real audio server. The isolated
accessibility bus was unavailable; this does not certify Orca.

Local hardware-enabled workspace tests passed 265 cases before the final negative
enrollment fixture was added; the final device suite, including that fixture,
passed afterward. Final build/Clippy/format checks passed. The final Rust/Cargo/Python
source files were compared with the qualified immutable snapshot. Private evidence
retains outcomes, logs and native captures.

Desktop sysfs and the candidate's read-only inventory both detected the connected
Plus and standard MK.2, each with a valid unique identity. That diagnostic sent no
device reports, acquired no ownership lease and printed no serial/path. No actual
physical session was opened for qualification. No VM, scheduled checkout, production
service, saved layout or installed desktop build was changed.

## Remaining gates

Provide a bounded physical test host with independent cleanup and restoration of
production Plus ownership; verify MK.2 image orientation, keys, firmware/serial
verification, brightness and disconnect/reconnect behavior. Then connect production
roster/enrollment and device selection, async persistence/model display scheduling,
shared audio/media/microphone/plugin dispatch and full paired lifecycle/recovery.
Qualify accessibility, sustained resources and user-visible response on the actual
Plus/MK.2 pair before claiming V1.5 multi-device support. Changes remain local until
explicitly authorized for commit/publication.
