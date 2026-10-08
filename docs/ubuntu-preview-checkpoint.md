# Ubuntu 26.04 preview preparation

Ubuntu is a compatibility preview target, not an expansion of Fedora V1 support.
Broad supported GNOME/Wayland distribution compatibility remains a V1.5 goal.
The existing Fedora and Debian release assets remain unchanged.

## Separate native package and trust boundary

The manually dispatched `Experimental Ubuntu candidate` workflow builds on
Ubuntu 26.04 x86_64, using the official image pinned to digest
`sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7`.
It retains Rust 1.97.1, locked dependencies, strict lint/tests and cargo-deny policy.
The native bundle builder records Ubuntu's actual distribution/library environment
and runs both binaries before bundling. Environment metadata alone does not
certify a supported platform.

The candidate archive is `decksmith-ubuntu2604-x86_64.tar.gz`. Preparation and
verification require explicit `--target ubuntu2604`; the verifier pins the separate
`ubuntu-candidate.yml` workflow, expected public source SHA, main ref and
GitHub-hosted runners. All six download subjects are attested and verified before
any downloaded installer/library code executes. The default target remains Fedora
44, including compatibility with older signed metadata. Debian and Ubuntu archives
cannot be silently substituted for one another.

## Dependencies and qualification

Ubuntu runtime prerequisites use the Debian-family package names:

```sh
sudo apt-get install python3 python3-gi python3-gi-cairo python3-pil python3-cairo \
  gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-rsvg-2.0 libglib2.0-bin \
  systemd libudev1 libpulse0 pulseaudio-utils wireplumber
```

Use the normal GNOME Wayland session bus, PipeWire, PipeWire-Pulse and WirePlumber.
Optional Homebridge credential storage needs Secret Service and `libsecret-tools`.
A Rust compiler is not needed for a prebuilt package. The installer does not install
system USB rules, activate controls or enable login startup. The historical Fedora
URL installer remains Fedora-only.

Ubuntu previously passed isolated installation, recovery and VirtualDeck checks
using the signed Fedora V1 candidate. The Ubuntu-specific signed candidate must
pass fresh signature/checksum/archive verification, staged data-preservation and
native rendering tests, normal per-user installation/service behavior and
interactive editor/file-picker checks before preview publication. Retain full
source/package identity and scope in the final checkpoint. Physical USB/audio,
lifecycle recovery and screen-reader acceptance are distinct gates; record any
untested checks explicitly rather than advertising official support.
