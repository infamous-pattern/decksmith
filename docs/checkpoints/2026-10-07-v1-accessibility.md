# Signed V1 human accessibility follow-up — October 7, 2026

The reference Fedora Workstation 44 desktop runs signed candidate
`1.0.0-c5e4fea53afe`, public source `68c42421ceaa65bb033993b56f0babdde2a14a6e`.
The installed release identity was checked when recording this result. No
application code, installed artifact or saved configuration changed.

## Completed human checks

The user opened the editor and used Tab and Shift+Tab through the sidebar,
device preview and settings. In response to a question covering visible focus,
reachability of Save and Apply, awkward jumps and clipped controls, the user
reported: “Keyboard navigation and focus work correctly.”

The user also enabled GNOME Screen Reader and, in response to the requested
control-name/value check, reported: “Yes, screenreader works.” This is human
acceptance of spoken controls on the reference desktop; no synthesized audio or
automated speech inspection was used.

For the final visual questions, the user reported: “Yes, they all look good.”
The requested checks covered Home, Pages and Keys & Dials under GNOME light,
dark, High Contrast and Large Text, plus a 125% or 150% display-scaling check
where offered, including readability, clipping and Save and Apply reachability.
This is recorded as user-confirmed visual acceptance. The exact chosen scale,
monitor geometry and theme settings were not independently captured; no numeric
contrast audit or additional screenshot set is inferred from the reply.

This records human acceptance for the requested traversal, not an exhaustive
AT-SPI audit of every control or a claim about another display setup.
The requested keyboard, spoken-control and visual reviews are complete for this
reference setup. Other monitors, exhaustive control/state coverage and draft/error
edge cases are separate. Prior-build screenshots and automated smoke tests remain
comparison evidence.

The [physical lifecycle checkpoint](2026-10-07-v1-physical-lifecycle.md) records
completed user-operated recovery and background-only startup cases. Precise
visible-response timing, resource reconciliation, optional-companion coverage,
other untested action/fault cases and the release decision remain separate.
