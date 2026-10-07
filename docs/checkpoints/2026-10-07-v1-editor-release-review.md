# Signed V1 editor and delivery review — October 7, 2026

The Fedora Workstation 44 VM exercised packaged editor modules from signed
candidate `1.0.0-c5e4fea53afe`. All 389 archive members passed manifest checks;
30 supplemental test modules and the native fixture matched the accepted source
before staging. Source fixtures are separate from signed archive members.

The six download subjects were independently reverified on the host against the
retained signing bundle and source `68c42421ceaa65bb033993b56f0babdde2a14a6e`.
Checksums, archive admission and bootstrap agreement passed. The hosted candidate
and latest published documentation Quality runs were confirmed successful.
No desktop installation or public publication occurred.

## Results

Thirty-one focused tests passed for draft/source separation, invalid-save rejection,
undo/redo, page/name changes and emoji validation, foreground page following,
status refresh coalescing, save explanations, validation announcements, Quit/save
gating and preview scheduling. They exercise packaged modules with controlled
dependencies; they do not certify real backend actions.

| Native fixture | Result | Window geometry |
| --- | --- | --- |
| Standard text, light | Pass | 1280 × 768 throughout |
| Standard text, dark | Pass | 1280 × 768 throughout |
| Standard text, compact | Pass | 1024 × 768 throughout |
| Enlarged text, default width | Pass | 1280 × 768 throughout |
| Enlarged text with compact width | **Fail / known limitation** | 1024 px available, 1070 px requested |

Passing modes check mounted controls, consistent preview position, stable key/dial
geometry, reachable page tools, status gating, invalid-field feedback, refresh
preserving focus/error text, draft retention, keyboard traversal, history, theme
undo/redo, automatic/custom labels, cancelled Quit retaining edits and explicit
save. Writes are mocked. Final enlarged-text Home, Keys, Dials, Pages and About
screenshots were inspected; lower settings use vertical scrolling. Blank fake
previews are fixture content, not physical renderer-parity evidence.

The compact/enlarged combination reproducibly clips content and is **not passed**.
Use the default 1280-pixel window width with enlarged text. The 1070-pixel request
is fixture-specific, not a universal minimum for all fonts/scales. More flexible
narrow-window behavior is follow-up work and a draft-release known limit. No
production-code fix, candidate modification or rebuild was made.

Configurator RSS snapshots ranged from approximately 123.26 to 125.79 MiB.
Four-second CPU observations ranged from 1.89% to 2.11% of one logical core.
These finite synthetic observations use software rendering, fake previews and
the plugin placeholder. Observed memory is below the 250 MB configurator target;
this is not a sustained idle or physical resource benchmark.

## Harness repairs and limits

The first native attempt failed a focus assertion on an inactive window. A
logical-focus retry then failed traversal and was not counted as a pass. Inspection
showed GNOME's Activities overview. The final fixture waited for an active window;
the overview was closed using only the VM virtual keyboard, and the **original
focus/traversal assertions** passed. Only the fixture activation wait changed,
not application focus logic. Cairo software rendering avoided the VM Mesa/Zink
device warning. These are environment repairs, not claimed product fixes.

Private D-Bus, an in-memory settings backend and temporary XDG paths isolated the
normal guest. Accessibility activation was disabled in this fixture; human
screen-reader acceptance is separate. DPI enlargement is not compositor scaling.
Portal/private-bus warnings are not GPU or lock certification. The normal guest
service stayed inactive. Configuration/artwork, integration, release links and
startup fingerprints matched before/after. Temporary files were removed, normal
shutdown completed, and all four VMs were verified off.

## Prepared delivery

The original six subjects and signing bundle were copied byte-for-byte to a
private promotion directory with hashes. Supplemental [draft notes](../v1-release-notes-draft.md)
and [verified installation guidance](../v1-verified-installation.md) are prepared
separately from signed `INSTALL.md`. Proposed public V1 URLs do not exist yet and
have not been download tested.

The user approved deferring numerical visible-response qualification beyond V1;
the 150/100 ms targets are not claimed passed. Resource reconciliation, remaining
representative action/dispatch coverage and final release/download verification
remain in the [release ledger](../v1-release-review.md). Private logs and images
remain under `local/v1-final-20261007/release-review/`.
