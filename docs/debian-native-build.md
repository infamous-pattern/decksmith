# Debian 13 native build and compatibility checkpoint

## Current result — October 8, 2026

Decksmith now builds and runs in the Debian 13 test VM using Debian's own system
libraries. This corrects the binary compatibility blocker; it does not turn the
published Fedora V1 archive into a Debian download or certify full Debian support.
The existing signed `v1.0.0` release remains unchanged. Broad supported
GNOME/Wayland distribution compatibility remains a V1.5 goal.

The Fedora archive required `GLIBC_2.43`, which Debian 13's glibc 2.41 cannot
provide. A native build with the locked dependencies and Rust 1.97.1 produced a
daemon whose highest required glibc symbol version is 2.39 and a command-line
client requiring at most 2.34. No replacement of Debian's C library is necessary.

The bundle builder now records the actual distribution, architecture and C-library
environment instead of identifying every build as Fedora 44. It also runs the
daemon's self-check and VirtualDeck check and the client's help command before packaging, rejecting copied
binaries that cannot execute on the named environment. Runtime dependency hints
use Debian/Ubuntu package names there and check the Python GI Cairo bridge needed
by the editor. These metadata fields identify a build target; they do not by
themselves establish supported-platform or release acceptance.

## Native build

Build on Debian 13 x86_64 as a regular user from a reviewed source checkout. Install
runtime dependencies from Debian's normal repositories:

```sh
sudo apt-get install python3 python3-gi python3-gi-cairo python3-pil python3-cairo \
  gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-rsvg-2.0 libglib2.0-bin \
  systemd libudev1 libpulse0 pulseaudio-utils wireplumber
```

Use a GNOME Wayland session with the normal session D-Bus, PipeWire,
PipeWire-Pulse and WirePlumber services. Optional Homebridge credential storage
also needs Secret Service and `libsecret-tools`. A detected USB device needs its
own access rule; building and installing do not grant device access.

Build prerequisites are separate from runtime dependencies:

```sh
sudo apt-get install build-essential binutils git pkg-config libudev-dev
rustup toolchain install 1.97.1 --profile minimal
rustup run 1.97.1 python3 scripts/build-bundle.py --output dist/debian13
```

Use Rustup from its official source if it is not already installed. Debian's
default compiler is not the tested toolchain for the current locked dependencies.
The builder enables both binaries' hardware features, remaps private build paths,
and writes the checksummed archive and matching installer helpers together.

For an isolated installation test, choose the exact archive emitted by the
builder and run its installer as your normal user:

```sh
python3 scripts/decksmith-install.py --stage-root ./debian-stage \
  install ./dist/debian13/decksmith-REPLACE-WITH-EXACT-BUNDLE-ID.tar.gz
python3 ./debian-stage/data/decksmith/app/current/scripts/runtime-doctor.py
```

Staging keeps the normal user installation, services and saved layout untouched.
For a deliberate normal test-VM installation, omit `--stage-root`. Installation
preserves saved data and does not enable login startup or start controls
automatically. Do not use the Fedora URL installer or the published Fedora
archive for this Debian test. A locally built bundle is not a signed public
release; public Debian delivery needs its own reviewed build/signing workflow
and qualification before publication.

## Verification and remaining gates

The retained Debian test guest provided glibc 2.41, GTK 4.18, libadwaita 1.7 and
GNOME Shell 48.7. The native development bundle passed:

- Hardware-enabled native compilation and 140 Rust tests; three hardware/long
  diagnostics were ignored by the suite.
- 39 script tests and 167 editor tests; each Python suite skipped one
  host-specific test on Debian.
- Runtime doctor, archive integrity, binary loading, and required glibc symbols.
- Staged installation, backup/restore and uninstall/reinstall, preserving emoji
  labels, saved settings and custom artwork.
- Actual GTK editor rendering on a private headless GNOME Wayland compositor.
- Private-bus VirtualDeck startup, key/touch previews, page switching, saving,
  and layout validation.
- Native tab layout, key/dial geometry, page controls, theme editing and logical
  sidebar shortcuts. The existing normal guest saved state was preserved.
- Normal per-user installation and background-service start/stop. With no USB
  passthrough, the service correctly waited for a device. Login startup remained
  disabled; the native test build is installed for later interactive VM checks.

The relocated bundle GUI check was corrected to inspect Panel's presented,
integrated editor instead of creating a hidden second editor. The workspace
check's explicit `--headless` mode reports unavailable compositor input-focus
traversal as skipped; its ordinary mode retains the original strict assertion.

This isolated display does not certify human keyboard/screen-reader acceptance,
file chooser/portal behavior in a full logged-in session, physical USB/audio,
lock/suspend/hotplug recovery, or long-term resource use. Those checks and signed
Debian-specific downloads remain required before advertising supported Debian
installations. Private local logs, package inventories, bundle checksums and
screenshots are retained with this compatibility checkpoint.

## Separate signed Debian candidate

The manually dispatched `Experimental Debian candidate` workflow builds from clean
public `main` in the pinned official Debian 13 slim image (October 5, 2026). It
uses the same pinned Rust toolchain, locked dependencies, lint/test checks and
cargo-deny policy as the Fedora workflow. It runs both binaries before bundling
and stages `decksmith-debian13-x86_64.tar.gz`, never the Fedora archive name.
The signing job attests all six download subjects on GitHub-hosted runners.
Candidate artifacts are preparation evidence, not an accepted public release.

The preparation and verification helpers default to Fedora 44 for compatibility
with existing releases. Debian must be selected explicitly with `--target debian13`. Its verifier pins the separate
`debian-candidate.yml` signing workflow, the full expected public source SHA,
`refs/heads/main`, and GitHub-hosted runners. An archive, target, source, checksum
or signing mismatch stops installation. A signed Debian candidate cannot be
silently accepted as a Fedora package.

For an accepted Debian candidate, use a reviewed local copy of the verifier and
the exact public source SHA recorded with that candidate:

```sh
python3 -I scripts/verify-install.py /path/to/debian-downloads \
  --target debian13 --source-sha REPLACE_WITH_FULL_ACCEPTED_PUBLIC_SHA \
  --bundle /path/to/debian-attestation.json
```

Add `--install --stage-root ./debian-stage` for an isolated verified installation,
or `--install` for a deliberate normal per-user install. Neither starts controls
or enables startup. The existing Fedora URL installer remains Fedora-only.

On October 8, the user also checked the installed native build in the Debian
VM's full GNOME session: all editor sections, key/dial sizing, visible controls,
and keyboard navigation/focus worked correctly. The native artwork file picker opened and returned cleanly to the editor.
Debian screen-reader acceptance remains untested; signed-candidate checks
are recorded separately when completed. Fedora 44 remains
the supported V1 platform while Debian-specific preview qualification proceeds.
