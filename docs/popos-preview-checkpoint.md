# Pop!_OS 24.04 COSMIC preview preparation

This is initial experimental compatibility, separate from supported Fedora V1
and the V1.5 GNOME/Wayland certification goal. Existing Fedora, Debian and Ubuntu
release assets are unchanged. COSMIC uses its own desktop services; GNOME Shell
extensions and GNOME-specific actions are not COSMIC support.

## Package and verification boundary

The manually dispatched `Experimental Pop!_OS COSMIC candidate` workflow builds
on the official Ubuntu 24.04 base, pinned to image digest
`sha256:534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55`.
This matches the tested Pop!_OS 24.04 guest's Ubuntu-derived library baseline;
it is not a claim that the container runs COSMIC. Bundle metadata honestly records
**Ubuntu 24.04** as its build environment. Actual desktop qualification takes
place in the separate **Pop!_OS 24.04 COSMIC** VM.

Rust 1.97.1, locked dependencies, strict formatting/lint/tests, Python tests and
cargo-deny advisory/license/source checks precede packaging. Both binaries must
run before bundling. The distinct archive is
`decksmith-popos2404-x86_64.tar.gz`. Its preparation and verifier require explicit
`--target popos2404`, the separate `popos-candidate.yml` signer, the exact public
source SHA, main ref and GitHub-hosted runners. All six subjects must verify
before downloaded installer code executes. Cross-target substitutions fail.
The historical Fedora URL installer is not a COSMIC installer.

## Initial compatibility work

The updated Pop!_OS VM is registered in virt-manager's User session with autostart
disabled. It previously passed dependency/integrity checks, relocated native
GTK rendering across all tabs and key/dial selection, shared previews,
VirtualDeck navigation/save and the real COSMIC file picker using the unchanged
Debian preview. Those checks are initial evidence, not acceptance of the new
signed COSMIC candidate. Package hints now follow Pop!_OS's Ubuntu/Debian ancestry.

First-launch sizing also exposed an unmapped-parent monitor lookup: an oversized
1440×900 request on a 1280×800 display caused COSMIC to choose the 900×540 minimum.
The editor now bounds its first request to an available monitor even before the
parent maps. At 1920×1200, the source-matched native workspace passes tab/layout,
focus, settings and gauge-selector checks. COSMIC can still choose a narrower
window on a 1280×800 display; manually enlarge the window if controls clip.
The final signed package passed native and normal-session rendering at 1920×1200;
the user accepted layout, keyboard navigation and the artwork picker. Orca was
not tested. See the [exact-package qualification record](checkpoints/2026-10-09-popos-cosmic-signed-preview.md).

Do Not Disturb now requires a running GNOME Shell before reading or changing
GNOME's notification setting. COSMIC reports it unavailable; changing an ignored
GNOME preference must not masquerade as COSMIC notification control.

## Runtime dependencies and limitations

Use the normal COSMIC Wayland session, session D-Bus, PipeWire, PipeWire-Pulse
and WirePlumber. Runtime dependencies use Ubuntu-family names:

```sh
sudo apt-get install python3 python3-gi python3-gi-cairo python3-pil python3-cairo \
  gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-rsvg-2.0 libglib2.0-bin \
  systemd libudev1 libpulse0 pulseaudio-utils wireplumber
```

Optional Homebridge password storage requires a running Secret Service provider
and `libsecret-tools`. Installing the bundle does not add system USB rules,
start controls or enable login startup. No Rust compiler is required to run a
prebuilt package.

Qualification must record signed-byte identity, installation/recovery and saved
state preservation, native editor/file-picker behavior, and normal service
start/stop separately from VirtualDeck. Physical USB/audio, system actions,
lock/suspend/hotplug and login recovery, screen-reader support, long-term resource
use and COSMIC-specific panel/active-window integration remain unqualified until
separately accepted. Do not substitute GNOME extension behavior or enable Auto-Lock
as a protective guarantee before a real COSMIC lock/unlock test.

References: [System76 Pop!_OS](https://system76.com/pop) and
[COSMIC desktop documentation](https://system76.com/support/pop-basics/).

## Final signed candidate

The final candidate 1.0.0-5b198a52075d passed independent signature/archive admission,
isolated installation/recovery and native COSMIC tests, followed by normal
installation and service start/stop. The user accepted the editor and file picker.
Public prerelease approval/download verification are pending.
[Installation guide](popos-preview-installation.md) and
[qualification record](checkpoints/2026-10-09-popos-cosmic-signed-preview.md)
record exact identity and the remaining limits.
