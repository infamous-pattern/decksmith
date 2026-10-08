# Debian 13 signed preview qualification — October 8, 2026

## Corrected compatibility blocker

The Fedora V1 archive requires GLIBC_2.43. Debian 13 provides glibc 2.41.
The separate native Debian package runs against Debian system libraries; its
daemon requires at most GLIBC_2.39 and client GLIBC_2.34. No system C-library
replacement or Fedora release modification is required.

## Candidate identity

- Public source: `dee2f2c285041dc19388ed579b90886e4b58341a` (clean source).
- Workflow: [Experimental Debian candidate run 37817883387](https://github.com/infamous-pattern/decksmith/actions/runs/37817883387).
- Environment: pinned official Debian 13 slim image, Rust 1.97.1 and locked Cargo dependencies.
- Bundle: `1.0.0-ca725bc8a704`; 409 manifest files.
- Archive: `decksmith-debian13-x86_64.tar.gz`.
- SHA-256: `28c887f3b83ac87acfdbff95c6c7178a69cf1f926a8739af83d43f38a2987717`.

The separate workflow's six download subjects passed hosted provenance verification
and independent retained-bundle verification against exact repository, workflow,
main source/ref and GitHub-hosted runner identity. Complete checksum coverage,
source/target/dirty-tree admission and bootstrap-byte identity passed. The verifier
also re-accepted the original signed Fedora V1 package, preserving legacy metadata
compatibility. Neither verification operation activated host controls.

## Checks completed

- Formatting and strict all-feature lint checks; 140 Rust tests passed and three
  hardware/long diagnostics ignored. Dependency advisories, bans, licenses and
  sources all passed cargo-deny policy.
- 59 script and 167 editor tests ran successfully in Debian CI; one environment
  skip in each Python suite. Both complete Python suites also passed on the host.
- Exact signed archive copied over pinned SSH and checked against expected hash
  before executing any guest code.
- Isolated guest installation, integrity/runtime doctor, glibc/library checks,
  backup/restore, retained-data uninstall/reinstall; emoji layout text, settings
  and custom SVG retained byte-for-byte.
- Actual signed relocated GTK editor, Rust-rendered key/touch previews, page
  switching, SaveLayout and layout validation using private-bus VirtualDeck on
  GNOME 48 headless Wayland. All tabs, key/dial geometry, validation, theme editing
  and logical shortcuts passed the source-matched workspace fixture. Compositor
  input traversal remains explicitly skipped in this headless fixture.
- User accepted the preceding native test build's editor sections, stable key/dial
  sizing, visible controls, keyboard focus/navigation and native artwork picker in
  the normal logged-in GNOME session. Application/editor code is unchanged in the
  signed candidate; these are user observations on the preceding build, not a
  second human test of the signed package.
- Signed package installed normally in the retained Debian test VM; runtime doctor
  ready; background service started, acquired its D-Bus name, reported no USB
  device, reported GNOME lock-state availability, and stopped successfully. Previous
  build/configuration backup retained; login startup remained disabled.
- Debian VM shut down cleanly; all four retained test VMs off. Autostart disabled.

## Explicit limits

This is experimental delivery, not supported-platform certification. Debian Orca
acceptance, actual USB access/device rendering, live audio/application controls,
lock/unlock, suspend/resume and USB reconnect recovery, startup/reboot lifecycle,
and long-term resource use are unqualified. No physical Deck or host audio was
passed into the VM. Private local logs and screenshots are retained; they are
not included as public telemetry.

Fedora Workstation 44 remains the supported V1 platform. Broad GNOME Wayland
compatibility is a V1.5 goal. The original Fedora V1 release remains unchanged.
The user accepted publication with these limits. The experimental
[v1.0.0-debian-preview.1 prerelease](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0-debian-preview.1)
is published. Fresh public copies of all seven assets matched the retained
candidate/signing bundle byte-for-byte; all six signed subjects passed signature,
checksum and archive admission again. Fedora `v1.0.0` remains the latest stable
release. [Verified Debian preview installation](../debian-preview-installation.md)
is supplemental guidance and does not modify the signed archive's `INSTALL.md`.
