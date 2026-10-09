# Verified Pop!_OS 24.04 COSMIC preview installation

This is an **experimental Pop!_OS 24.04 COSMIC x86_64 Wayland preview** of Decksmith
1.0.0. It provides a separately built and signed COSMIC package. Fedora Workstation
44 remains the supported V1 platform; broad GNOME/Wayland support is a V1.5 goal.

The Ubuntu 24.04-baseline COSMIC package is built and signed separately. Do not use the Fedora
archive or historical preview URL installer on COSMIC. The original Fedora V1
release remains available and unchanged.

## Runtime dependencies and initial trust

Install these packages from Pop!_OS's trusted repositories:

```sh
sudo apt-get install git gh curl python3 python3-gi python3-gi-cairo python3-pil \
  python3-cairo gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-rsvg-2.0 libglib2.0-bin \
  systemd libudev1 libpulse0 pulseaudio-utils wireplumber
```

Use a normal COSMIC Wayland session with PipeWire, PipeWire-Pulse and WirePlumber.
A Rust compiler is not required to run this package. Optional Homebridge password
storage also needs a running Secret Service (a compatible local keyring) and
`libsecret-tools`. The installer does not install system USB rules or grant USB
access. See [USB rules and recovery](installation.md).

Review the verifier source before use. The initial trust is that reviewed source
and the GitHub CLI from a trusted package source; a checksum alone does not prove
authenticity. A recent CLI with `gh attestation verify` is required. If your
installed CLI lacks that command, update it from the official GitHub CLI package
source before continuing. Stop if verification is unavailable; do not bypass it.

Run the remaining commands as your regular desktop user:

```sh
git clone https://github.com/infamous-pattern/decksmith.git decksmith-cosmic-verifier
cd decksmith-cosmic-verifier
git checkout --detach be238905bb3f6aa793dcda2df063fa5a2f206cbc
```

## Download and verify

```sh
mkdir downloads
for file in decksmith-popos2404-x86_64.tar.gz decksmith-install.py package_io.py INSTALL.md SHA256SUMS candidate.json attestation.json; do
    curl --fail --silent --show-error --location \
        --proto '=https' --proto-redir '=https' --tlsv1.2 \
        --connect-timeout 10 --max-time 180 --retry 3 \
        "https://github.com/infamous-pattern/decksmith/releases/download/v1.0.0-popos-preview.1/$file" \
        --output "downloads/$file" || exit 1
done
python3 -I scripts/verify-install.py downloads --target popos2404 \
    --source-sha be238905bb3f6aa793dcda2df063fa5a2f206cbc \
    --bundle downloads/attestation.json
```

Expected bundle: `1.0.0-5b198a52075d`, 415 manifest files, archive SHA-256
`d4c8824980cf417082897805bf64f01e26172fa9fe6df14367816972326961d9`.
Verification pins the COSMIC signing workflow, repository, public source commit,
main ref and GitHub-hosted runner identity. It verifies all six signatures and
then the complete checksums and archive admission. Network access may be needed
for trust roots; the CLI may request authentication. Nothing installs by default.

The signed `candidate.json` retains its staging/pending text. Preview acceptance
is a separate decision; signed metadata is never edited after signing.

## Install and start

Save open Decksmith edits and close its editor before updating. After verification:

```sh
python3 -I scripts/verify-install.py downloads --target popos2404 \
    --source-sha be238905bb3f6aa793dcda2df063fa5a2f206cbc \
    --bundle downloads/attestation.json --install
```

The helper re-verifies a private snapshot before running the per-user installer.
Existing layouts and login preferences are preserved, and the previous build and
configuration backup are retained. Installation does not start background controls
or enable login startup. Open Decksmith and use Start under Background controls
when ready, closing other Stream Deck controllers first. Updates to a running
service take effect after deliberately stopping and starting its controls.

For an isolated install, also add `--stage-root ./cosmic-stage`.
[Recovery and removal](installation.md#backup-and-recovery) preserve saved data.

## Tested scope and remaining limits

The candidate uses a pinned Ubuntu 24.04 build image and accurately records that
build environment. Desktop acceptance is on Pop!_OS 24.04 with COSMIC, not GNOME.
The Ubuntu 26.04 archive requires newer libraries and is not a COSMIC download.

The exact signed package passed installation, runtime/integrity and library checks,
backup/restore and retained-data uninstall/reinstall, native editor rendering and
VirtualDeck navigation/save. The user accepted all editor sections, stable key/dial
sizing, keyboard focus/navigation and artwork picker open/cancel in the normal
COSMIC session at 1920×1200. Background controls started and stopped normally
without a USB device; login startup remained disabled. Orca is installed but was
not tested. See the [qualification record](checkpoints/2026-10-09-popos-cosmic-signed-preview.md).

This preview does not provide the GNOME panel icon/extension, automatic window-based
page switching, or GNOME-specific Do Not Disturb, Night Light, Bluetooth and lock-key
integration on COSMIC. Unsupported providers report unavailable; DND does not
change a GNOME preference while pretending to control COSMIC notifications.
Shared system services such as power and audio are separate from desktop-specific
integration and require their own physical acceptance.

Physical Stream Deck USB access and feedback, real audio/media/system actions,
lock/suspend/hotplug and login recovery, accessibility and long-term resource use
are not qualified on COSMIC. The VM uses VirtualDeck without physical passthrough.
Auto-Lock availability is not a guarantee of COSMIC lock/unlock safety; leave it
off until independently tested. Initial sizing on a 1280×800 display can still
clip controls; enlarge the window if needed. The reviewed VM display is 1920×1200.

Please report findings with distribution/session details and logs after removing
private data. This initial preview does not certify other COSMIC distributions,
GPU configurations or Stream Deck models.
