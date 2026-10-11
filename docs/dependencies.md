# Dependencies and source builds

For the recommended signed-download setup, start with [Install Decksmith](install-decksmith.md).
It offers to install missing runtime packages from your OS repositories. Python
3.11+ is needed by the setup helper. Signature verification uses GitHub CLI
2.102.0+ from the system or an optional pinned temporary upstream copy. The package catalog in `scripts/runtime_dependencies.py` supplies
the helper and development runtime doctor's package hints. Installation itself
runs as your desktop user; administrator access is used only for OS packages.

Debian 13, Ubuntu 26.04 and Pop!_OS 24.04 have separate experimental packages and
distribution-specific dependency lists in their linked preview guides. Do not
use the Fedora package list on those systems. This page's command examples are
for Fedora Workstation 44.


The signed V1 bundle targets **Fedora Workstation 44, x86_64, GNOME on
Wayland**, with **Stream Deck +** as the validated physical device. It uses a
per-user archive installer; RPM packaging is not yet available. Other
distributions, desktops, architectures and Stream Deck models need their own
validation. Independent control of multiple devices is still planned.

### Running a prebuilt bundle

A Rust compiler is **not required** to run a prebuilt bundle. The bundle includes
Decksmith's binaries, Python source, bundled icons, fonts and documentation; the
following system libraries and tools must already be installed:

| Fedora packages | Used for |
|---|---|
| `python3`, `python3-gobject` | Native editor and integration helpers; Python access to GTK and D-Bus through GObject introspection. |
| `gtk4`, `libadwaita` | GNOME interface widgets. The runtime check verifies the required `Gtk.FileDialog` and `Adw.Dialog` APIs. |
| `python3-pillow`, `python3-cairo`, `librsvg2` | Artwork decoding, resizing and SVG icon rendering. |
| `glib2` | GIO/D-Bus integration and the `gdbus` utility. |
| `systemd`, `systemd-libs` | Per-user background service, login-session integration and `libudev` device discovery. |
| `pulseaudio-libs`, `pulseaudio-utils` | `libpulse` live audio meters and `pactl` application/device audio controls. |
| `wireplumber` | PipeWire session management and `wpctl` volume controls. |

Install these on Fedora with:

```sh
sudo dnf install python3 python3-gobject python3-pillow python3-cairo gtk4 libadwaita librsvg2 glib2 systemd systemd-libs pulseaudio-libs pulseaudio-utils wireplumber
```

Decksmith also needs a normal graphical user session with its session D-Bus and
working audio services. On Fedora Workstation, these normally include PipeWire,
`pipewire-pulseaudio` and WirePlumber. The PulseAudio client packages above work
with PipeWire's compatibility service; they do not require replacing PipeWire
with the PulseAudio server. Do not run Decksmith as root.

Physical device access requires a suitable **udev access rule**. Existing rules
from another controller may already provide access. The bundle includes
`packaging/udev/70-decksmith-plus.rules`; the per-user installer does not install
system-wide rules automatically. Close other Stream Deck controllers before
starting Decksmith's background controls. A connected device is not required to
install or run the VirtualDeck checks.

### Optional features and desktop services

| Feature | Additional requirement or limitation |
|---|---|
| GNOME panel menu and GNOME automatic page switching | The bundled Decksmith GNOME Shell extension, installed and enabled separately. See [desktop integration](desktop-integration.md). Login startup itself does not require the extension. |
| Media playback actions | An application exposing MPRIS on the session D-Bus. Available actions depend on the player. |
| Lock, Do Not Disturb, Night Light and Bluetooth | The corresponding GNOME settings/services. Bluetooth also needs an available adapter; a hardware radio block cannot be overridden. |
| Power Mode | A service providing `net.hadess.PowerProfiles`, and profiles supported by the system. Unsupported profiles are shown as unavailable. |
| Suspend, Reboot and Shutdown | systemd-logind authorization and system support. Desktop inhibitors are respected; Reboot and Shutdown require confirmation. |
| Auto-Lock | Lock-state reporting from GNOME or systemd-logind. See [Auto-Lock](auto-lock.md) for fallback behavior and desktop limitations. |
| Website artwork and URL actions | Network access for downloading website icons or opening remote sites. Core local controls need no cloud account or internet connection. |

No Elgato Windows/macOS software or OpenDeck installation is required. Missing
optional services can make individual actions unavailable even when installation
checks pass.

### Building from source

In addition to the runtime packages, install **Rust/Cargo, GCC, binutils,
`pkgconf-pkg-config` and `systemd-devel`** (libudev development headers):

```sh
sudo dnf install rust cargo gcc binutils pkgconf-pkg-config systemd-devel
python3 scripts/build-bundle.py
```

The workspace uses Rust edition 2024. Use a current toolchain compatible with the
locked dependencies. Cargo may need network access on the first build to fetch
those dependencies; no Rust toolchain is needed on machines receiving the bundle.

For installation, USB access, updates and rollback, follow the
[installation guide](installation.md). After installation, check dependencies
and bundle integrity with:

```sh
python3 ~/.local/share/decksmith/app/current/scripts/runtime-doctor.py
```

Use your configured XDG data path if it differs. The checker reports missing
dependencies; it does not install packages or change device permissions.

