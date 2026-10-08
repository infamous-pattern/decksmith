# Verified Ubuntu 26.04 preview installation

This is an **experimental Ubuntu 26.04 x86_64 GNOME/Wayland preview** of Decksmith
1.0.0. It provides a separately built and signed Ubuntu package. Fedora Workstation
44 remains the supported V1 platform; broad GNOME/Wayland support is a V1.5 goal.

The Ubuntu-native package is built and signed separately. Do not use the Fedora
archive or historical preview URL installer on Ubuntu. The original Fedora V1
release remains available and unchanged.

## Runtime dependencies and initial trust

Install these packages from Ubuntu's trusted repositories:

```sh
sudo apt-get install git gh curl python3 python3-gi python3-gi-cairo python3-pil \
  python3-cairo gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-rsvg-2.0 libglib2.0-bin \
  systemd libudev1 libpulse0 pulseaudio-utils wireplumber
```

Use a normal GNOME Wayland session with PipeWire, PipeWire-Pulse and WirePlumber.
A Rust compiler is not required to run this package. Optional Homebridge password
storage also needs a running Secret Service (normally GNOME Keyring) and
`libsecret-tools`. The installer does not install system USB rules or grant USB
access. See [USB rules and recovery](installation.md).

Review the verifier source before use. The initial trust is that reviewed source
and the GitHub CLI from a trusted package source; a checksum alone does not prove
authenticity. A recent CLI with `gh attestation verify` is required. If your
installed CLI lacks that command, update it from the official GitHub CLI package
source before continuing. Stop if verification is unavailable; do not bypass it.

Run the remaining commands as your regular desktop user:

```sh
git clone https://github.com/infamous-pattern/decksmith.git decksmith-ubuntu-verifier
cd decksmith-ubuntu-verifier
git checkout --detach e627ea4f7f660a6ff6ad4772a013989b36b717c4
```

## Download and verify

```sh
mkdir downloads
for file in decksmith-ubuntu2604-x86_64.tar.gz decksmith-install.py package_io.py INSTALL.md SHA256SUMS candidate.json attestation.json; do
    curl --fail --silent --show-error --location \
        --proto '=https' --proto-redir '=https' --tlsv1.2 \
        --connect-timeout 10 --max-time 180 --retry 3 \
        "https://github.com/infamous-pattern/decksmith/releases/download/v1.0.0-ubuntu-preview.1/$file" \
        --output "downloads/$file" || exit 1
done
python3 -I scripts/verify-install.py downloads --target ubuntu2604 \
    --source-sha e627ea4f7f660a6ff6ad4772a013989b36b717c4 \
    --bundle downloads/attestation.json
```

Expected bundle: `1.0.0-22dc2982d0b3`, 412 manifest files, archive SHA-256
`0ccf7489a576a7711f6f6ba8de74690755b6f28a506fadfc6d4af22db72fb6c2`.
Verification pins the Ubuntu signing workflow, repository, public source commit,
main ref and GitHub-hosted runner identity. It verifies all six signatures and
then the complete checksums and archive admission. Network access may be needed
for trust roots; the CLI may request authentication. Nothing installs by default.

The signed `candidate.json` retains its staging/pending text. Preview acceptance
is a separate decision; signed metadata is never edited after signing.

## Install and start

Save open Decksmith edits and close its editor before updating. After verification:

```sh
python3 -I scripts/verify-install.py downloads --target ubuntu2604 \
    --source-sha e627ea4f7f660a6ff6ad4772a013989b36b717c4 \
    --bundle downloads/attestation.json --install
```

The helper re-verifies a private snapshot before running the per-user installer.
Existing layouts and login preferences are preserved, and the previous build and
configuration backup are retained. Installation does not start background controls
or enable login startup. Open Decksmith and use Start under Background controls
when ready, closing other Stream Deck controllers first. Updates to a running
service take effect after deliberately stopping and starting its controls.

For an isolated install, also add `--stage-root ./ubuntu-stage`.
[Recovery and removal](installation.md#backup-and-recovery) preserve saved data.

## Tested scope and remaining limits

The package passes native compilation, Rust formatting/lint checks, 140 Rust
tests (three hardware/long diagnostics ignored), 80 script tests and 167 editor
tests (one environment-specific skip in each Python suite), and dependency
advisory/license/source checks. All six CI-built files have verified provenance.

The Ubuntu 26.04 VM passes installation, runtime/library checks, backup/restore and
retained-data uninstall/reinstall, emoji labels and custom artwork preservation,
private-bus VirtualDeck previews/page changes/saving, and GTK rendering on GNOME
50 Wayland. The user accepted editor sections, stable key/dial sizing, visible controls,
keyboard focus/navigation and the artwork file picker on this exact signed
package in the normal GNOME Wayland session. Orca spoke tested control names
and values clearly; this is a limited human check, not exhaustive accessibility
certification. Normal installation and background-service start/stop are tested
separately from the private-bus fixtures.
See the [exact qualification record](checkpoints/2026-10-08-ubuntu-signed-preview.md).

Physical USB, real audio/device controls,
lock/suspend/hotplug recovery and long-term resource use are **not yet qualified**.
The VM uses VirtualDeck and no physical Stream Deck passthrough. Please report
Ubuntu findings with distribution/session details and logs after removing private
data. This preview does not certify other distributions or Stream Deck models.
