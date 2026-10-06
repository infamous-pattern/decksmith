# Rune Outline integration — 2026-10-06

The user selected Rune Outline after an offline comparison and confirmed the
representative physical key and touch-strip preview was clear. That temporary
preview restored the saved layout and normal controls.

The integrated library contains 14 Rune Outline icons and the existing 134 Tabler
icons. New media/system assignments prefer suitable Rune artwork; loading a saved
layout does not migrate its images. Custom captions, colors and app/device images
remain intact. Touch symbols retain red Muted/crossed-out audio icons and green
Live text. Five cached alpha masks retain 5 KiB, with no per-frame SVG decoding.
Apache-2.0 provenance/modification notices and mixed-library export notices are
included. See [appearance and icons](../appearance-and-icons.md).

Validation: 167 Python tests passed; 136 Rust tests passed with three existing
hardware-dependent tests ignored. Formatting, Clippy including all targets/features,
and dependency advisories/licenses/bans/source checks passed. The library-inset
regression also verifies Rune, Tabler and imported sources against VirtualDeck.
Fedora Workstation 44 native tests covered the visible gallery, Cancel/Escape,
caption/action preservation, media defaults and Tabler fallback in light and dark
mode. The same packaged build passed installation and configuration-preservation
checks in the guest before desktop installation. All test VMs were stopped afterward.

Installed candidate: `0.1.0-2af1dcb9b80f`, built from clean source
`7f6ce75647ff7c029996857dc20498b3e3375d2c`. Desktop checks confirmed all installed
file checksums, hardware reconnection/display readiness, available lock detection,
and unchanged saved configuration, layout, active page and brightness. The previous
build and configuration backup remain available through the managed installer.
No public release or remote push was performed for this integration.

This icon milestone does not close the remaining V1 qualification gates or the
post-V1 below-1% idle CPU goal.
