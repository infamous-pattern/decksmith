# Verified V1 installation — preparation guide

**V1 is not published yet.** The commands below describe the proposed `v1.0.0`
delivery path and must be checked against the actual release after promotion.
For the existing public preview, use [preview installation](installation.md).

This guide is supplemental release guidance. It is not the signed archive's
`INSTALL.md`, and it does not alter that file or any candidate download. The
historical one-command preview shortcut remains pinned to preview.3 and checks
integrity only. Do not use it as the authenticated V1 installation path.

## Dependencies and trusted verifier

The supported destination is Fedora Workstation 44 x86_64 in a GNOME/Wayland
desktop session. As an administrator, install packages from Fedora's trusted
repositories:

```sh
sudo dnf install git gh curl python3 python3-gobject python3-pillow python3-cairo gtk4 libadwaita librsvg2 glib2 systemd systemd-libs pulseaudio-libs pulseaudio-utils wireplumber
```

Run subsequent steps as the regular desktop user. Review the project source and
verification helper before using it. The initial trust comes from that reviewed
source and Fedora's GitHub CLI package; a downloaded helper cannot authenticate
itself. Obtain the accepted full public commit from the release evidence:

```sh
git clone https://github.com/infamous-pattern/decksmith.git decksmith-v1-verifier
cd decksmith-v1-verifier
git checkout --detach 68c42421ceaa65bb033993b56f0babdde2a14a6e
```

## Download and verify

After the approved release is published, download its original six subjects and
retained signing bundle into a new directory:

```sh
mkdir downloads
for file in decksmith-linux-x86_64.tar.gz decksmith-install.py package_io.py INSTALL.md SHA256SUMS candidate.json attestation.json; do
    curl --fail --silent --show-error --location \
        --proto '=https' --proto-redir '=https' --tlsv1.2 \
        --connect-timeout 10 --max-time 180 --retry 3 \
        "https://github.com/infamous-pattern/decksmith/releases/download/v1.0.0/$file" \
        --output "downloads/$file" || exit 1
done
python3 -I scripts/verify-install.py downloads \
    --source-sha 68c42421ceaa65bb033993b56f0babdde2a14a6e \
    --bundle downloads/attestation.json
```

The helper verifies all six signatures against the accepted source, repository,
workflow, ref and hosted-runner identity, then complete checksums and archive
admission. Expected result: bundle `1.0.0-c5e4fea53afe`, 389 manifest files and
archive SHA-256 `4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2`.
Its default is verification only. Network access and a working recent GitHub CLI
are needed; authentication may be requested by that CLI. Verification against a
retained bundle can still fetch trust roots. Stop if verification fails or is
unavailable. Do not skip signatures or infer authenticity from checksums alone.

The signed `candidate.json` retains its original staging/pending text. Release
acceptance is a separate recorded decision, not an edit to signed metadata.

## Install and start deliberately

After verification succeeds, install through the same helper:

```sh
python3 -I scripts/verify-install.py downloads \
    --source-sha 68c42421ceaa65bb033993b56f0babdde2a14a6e \
    --bundle downloads/attestation.json --install
```

It re-verifies and snapshots the files before running the per-user installer.
Installation preserves saved data and the existing login preference. On a fresh
installation login startup is off; the helper never activates controls. Save any
open edits and reopen Decksmith to load the installed editor. When ready, close
other Stream Deck controllers and use Start under Background controls. An update
of a currently running service takes effect after deliberately stopping and
starting those controls; merely installing files does not restart them.

Use [installation and recovery](installation.md) for backup, compatible rollback,
uninstall/reinstall, USB access rules and the optional GNOME indicator. The per-user
installer does not install system USB rules. Broad distribution and device support
remain outside the supported V1 combination.
