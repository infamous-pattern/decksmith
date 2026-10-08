# Verified Debian 13 preview installation

This is an **experimental Debian 13 x86_64 GNOME/Wayland preview** of Decksmith
1.0.0. It corrects the Fedora binary's glibc incompatibility. Fedora Workstation
44 remains the supported V1 platform; broad GNOME/Wayland support is a V1.5 goal.

The Debian-native package is built and signed separately. Do not use the Fedora
archive or historical preview URL installer on Debian. The original Fedora V1
release remains available and unchanged.

## Runtime dependencies and initial trust

Install these packages from Debian's trusted repositories:

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
git clone https://github.com/infamous-pattern/decksmith.git decksmith-debian-verifier
cd decksmith-debian-verifier
git checkout --detach dee2f2c285041dc19388ed579b90886e4b58341a
```

## Download and verify

```sh
mkdir downloads
for file in decksmith-debian13-x86_64.tar.gz decksmith-install.py package_io.py INSTALL.md SHA256SUMS candidate.json attestation.json; do
    curl --fail --silent --show-error --location \
        --proto '=https' --proto-redir '=https' --tlsv1.2 \
        --connect-timeout 10 --max-time 180 --retry 3 \
        "https://github.com/infamous-pattern/decksmith/releases/download/v1.0.0-debian-preview.1/$file" \
        --output "downloads/$file" || exit 1
done
python3 -I scripts/verify-install.py downloads --target debian13 \
    --source-sha dee2f2c285041dc19388ed579b90886e4b58341a \
    --bundle downloads/attestation.json
```

Expected bundle: `1.0.0-ca725bc8a704`, 409 manifest files, archive SHA-256
`28c887f3b83ac87acfdbff95c6c7178a69cf1f926a8739af83d43f38a2987717`.
Verification pins the Debian signing workflow, repository, public source commit,
main ref and GitHub-hosted runner identity. It verifies all six signatures and
then the complete checksums and archive admission. Network access may be needed
for trust roots; the CLI may request authentication. Nothing installs by default.

The signed `candidate.json` retains its staging/pending text. Preview acceptance
is a separate decision; signed metadata is never edited after signing.

## Install and start

Save open Decksmith edits and close its editor before updating. After verification:

```sh
python3 -I scripts/verify-install.py downloads --target debian13 \
    --source-sha dee2f2c285041dc19388ed579b90886e4b58341a \
    --bundle downloads/attestation.json --install
```

The helper re-verifies a private snapshot before running the per-user installer.
Existing layouts and login preferences are preserved, and the previous build and
configuration backup are retained. Installation does not start background controls
or enable login startup. Open Decksmith and use Start under Background controls
when ready, closing other Stream Deck controllers first. Updates to a running
service take effect after deliberately stopping and starting its controls.

For an isolated install, also add `--stage-root ./debian-stage`.
[Recovery and removal](installation.md#backup-and-recovery) preserve saved data.

## Tested scope and remaining limits

The package passes native compilation, Rust formatting/lint checks, 140 Rust
tests (three hardware/long diagnostics ignored), 59 script tests and 167 editor
tests (one environment-specific skip in each Python suite), and dependency
advisory/license/source checks. All six CI-built files have verified provenance.

The Debian 13 VM passes installation, runtime/library checks, backup/restore and
retained-data uninstall/reinstall, emoji labels and custom artwork preservation,
private-bus VirtualDeck previews/page changes/saving, and GTK rendering on GNOME
48 Wayland. The local native build was also accepted interactively for editor
sections, key/dial sizing, keyboard focus and artwork file-picker behavior in the
normal GNOME session. The signed package separately passed installation/recovery, runtime checks,
private-bus VirtualDeck and native rendering; it is installed in the test VM.
See the [exact qualification record](checkpoints/2026-10-08-debian-signed-preview.md).

Debian screen-reader acceptance, physical USB, real audio/device controls,
lock/suspend/hotplug recovery and long-term resource use are **not yet qualified**.
The VM uses VirtualDeck and no physical Stream Deck passthrough. Please report
Debian findings with distribution/session details and logs after removing private
data. This preview does not certify other distributions or Stream Deck models.
