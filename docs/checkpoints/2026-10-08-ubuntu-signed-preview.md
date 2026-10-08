# Ubuntu 26.04 signed preview qualification — October 8, 2026

## Candidate identity and scope

This is experimental Ubuntu delivery, not supported-platform certification.
Fedora Workstation 44 remains the supported V1 platform; broader GNOME/Wayland
compatibility is a V1.5 goal. Existing Fedora and Debian release assets are unchanged.

- Public source: `e627ea4f7f660a6ff6ad4772a013989b36b717c4` (clean source).
- Workflow: [Experimental Ubuntu candidate run 37820348331](https://github.com/infamous-pattern/decksmith/actions/runs/37820348331).
- Build: pinned official Ubuntu 26.04 image, Rust 1.97.1 and locked dependencies.
- Bundle: `1.0.0-22dc2982d0b3`; 412 manifest files.
- Archive: `decksmith-ubuntu2604-x86_64.tar.gz`.
- SHA-256: `0ccf7489a576a7711f6f6ba8de74690755b6f28a506fadfc6d4af22db72fb6c2`.
- Native runtime requirement observed: daemon GLIBC_2.43, client GLIBC_2.38.
- Test VM: Ubuntu 26.04.1, glibc 2.43, GNOME Shell 50.1, GTK 4.22.4,
  libadwaita 1.9.1, Wayland; normal portals and PipeWire/WirePlumber active.

## Verification and tests

All six CI download subjects passed hosted and independent retained-bundle
provenance verification against the exact repository, separate Ubuntu workflow,
source SHA, main ref and GitHub-hosted runner identity. Complete checksums,
archive/source/target/clean-tree admission and bootstrap byte identity passed.
The original signed Fedora V1 and Debian preview also passed the updated verifier,
including legacy Fedora metadata handling. Ubuntu requires explicit
`--target ubuntu2604`; cross-target substitution is rejected.

Formatting, strict all-feature lint and 140 Rust tests passed; three existing
hardware/long diagnostics were ignored. CI ran 80 script tests and 167 editor
tests, with one environment-specific skip in each suite. Both Python suites also
passed on the host. Dependency advisories, bans, licenses and sources passed
cargo-deny policy. This does not promise defect-free source or reproducible bytes;
system dependencies are obtained from Ubuntu repositories at build time.

The exact signed archive was copied over pinned SSH and checked against its
expected digest before any guest package code executed. It passed isolated
installation, runtime/integrity/library checks, staged backup/restore and
retained-data uninstall/reinstall. Emoji labels, settings and custom SVG remained
byte-identical; the normal saved configuration was unchanged by isolated testing.

Actual relocated GTK rendering and Rust key/touch previews passed on a private
headless GNOME 50 Wayland compositor, along with VirtualDeck startup, page
navigation, saving and layout validation. The source-matched workspace fixture
passed all sections, key/dial geometry, validation, appearance and logical
shortcuts. Its headless compositor keyboard traversal was explicitly skipped.

The signed package was then installed normally, retaining the previous build and
configuration backup. Runtime doctor reported ready. Background controls started,
acquired their session D-Bus name and correctly waited for an absent USB device.
After its lock monitor initialized, GNOME lock-state detection was available;
this is availability evidence, not a lock/unlock recovery test. Login startup was
kept disabled. The user accepted all editor sections, stable key/dial sizing,
visible controls, keyboard focus/navigation, and artwork picker open/cancel in the
normal Wayland session. Orca spoke tested control names and values clearly on
this exact signed package; no exhaustive screen-reader audit is claimed.

## Remaining limits

No physical Stream Deck or host audio was passed into the VM. Actual USB access,
device feedback, live audio/media and system actions, lock/unlock, suspend/resume,
hotplug, login/reboot lifecycle and long-term resource use remain unqualified on
Ubuntu. GNOME indicator/automatic-page extension compatibility and experimental
Homebridge also need their own Ubuntu acceptance. Retained private logs and native
screenshots are evidence, not public telemetry.

Preview publication and fresh public-download verification are the next steps.
[Verified Ubuntu preview installation](../ubuntu-preview-installation.md) is
supplemental guidance; signed archive files are never edited after signing.
