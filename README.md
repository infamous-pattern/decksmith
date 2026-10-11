# Decksmith

**Open Control, Forged for Linux.**

Decksmith is a native Linux app for your Elgato Stream Deck +. Build pages of
shortcuts, control application and device audio, and customize keys, dials and the
touch strip through a GTK4/libadwaita editor. Background controls can start at
login while the editor stays closed.

**V1 is supported on Fedora Workstation 44, x86_64, GNOME/Wayland, with one
Stream Deck +.** Debian, Ubuntu and COSMIC have separate experimental previews.

[Install](#install) · [User guide](docs/USER_GUIDE.md) ·
[Release notes](docs/v1-release-notes.md) ·
[Report an issue](https://github.com/infamous-pattern/decksmith/issues) ·
[Roadmap](docs/ROADMAP.md)

![Decksmith Home with device preview and background-control settings](docs/images/decksmith-home.png?v=914df36b4fe4)

## See it in action

[![Watch the Decksmith desktop tour and dial gauges on YouTube](docs/images/decksmith-desktop-tour-thumbnail.png?v=9a8e3926581d)](https://www.youtube.com/watch?v=wQgieOBo1VM)

**[Watch the 3-minute desktop tour](https://www.youtube.com/watch?v=wQgieOBo1VM)** ·
[English subtitles](docs/images/decksmith-desktop-tour.srt) ·
[Interface gallery](docs/interface-tour.md)

The tour shows actual Fedora, Debian, Ubuntu and COSMIC test desktops, with
VirtualDeck sample layouts. Gauge close-ups use illustrative signal values from
the shared renderer; they demonstrate appearance rather than hardware testing.
The [original V1 tour](https://www.youtube.com/watch?v=ZJG1n_8rGNY) is also available.

## What you can do

- Create and organize pages; navigate using keys or touch-strip swipes.
- Launch applications and websites, use media controls, and assign system actions
  such as Lock, Suspend, Do Not Disturb and Night Light. Reboot and Shutdown ask
  for confirmation on the desktop.
- Adjust application, speaker/headphone, microphone and system-sound levels;
  use mute, push-to-talk and live audio activity feedback.
- Customize labels, artwork, colors, themes and bundled fonts, including
  decorative Viking Runes. The editor follows the desktop's light/dark appearance.
- Set device brightness, background-only login startup and optional Auto-Lock.
  Closing the editor leaves controls running; Quit Decksmith stops them and
  blanks the entire device.

Optional actions depend on the desktop's services. The Homebridge integration is
**experimental**; general plugin support, independent multiple devices and the
15-key Stream Deck MK.2 are planned for V1.5. See the [roadmap](docs/ROADMAP.md).

## Downloads and compatibility

All current downloads require **Linux x86_64**. They are per-user runtime bundles,
not RPM or DEB packages. No Rust compiler, OpenDeck or Elgato Windows software is
needed to run them.

| System | Status | Download and installation details |
|---|---|---|
| Fedora Workstation 44, GNOME/Wayland | **Supported V1** | [v1.0.0](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0) · [Verified guide](docs/v1-verified-installation.md) |
| Debian 13, GNOME/Wayland | Experimental native preview | [Debian preview](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0-debian-preview.1) · [Guide and limits](docs/debian-preview-installation.md) |
| Ubuntu 26.04, GNOME/Wayland | Experimental native preview | [Ubuntu preview](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0-ubuntu-preview.1) · [Guide and limits](docs/ubuntu-preview-installation.md) |
| Pop!_OS 24.04, COSMIC | Experimental preview; Ubuntu 24.04 library baseline | [COSMIC preview](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0-popos-preview.1) · [Guide and limits](docs/popos-preview-installation.md) |
| Fedora Workstation 45 Beta | Test environment | [Testing scope](docs/v1-release-scope.md); no separately supported release |
| openSUSE Tumbleweed, GNOME/Wayland | Development test environment | [OS testing plan](docs/ROADMAP.md#os-testing-host-policy--october-10); no public package |

Preview VM checks do not establish physical USB/audio or lifecycle support on
those platforms. COSMIC does not inherit GNOME-only integrations. Review the
linked limits before installing. Broader GNOME/Wayland compatibility is a V1.5 goal.

### New dial gauges

![Dual-arc gauges with configured volume, independent audio activity and green microphone Live status](docs/images/decksmith-dial-gauges.png?v=0b74777f09e5)

The **inner blue arc** shows configured volume; the **outer arc** shows audio
activity. Muted controls retain a red crossed-out icon and red **Muted** text;
unmuted microphones show **Live** in green. Classic bars remain available.

**Availability:** dual-arc gauges are in the development build and published
COSMIC preview. The single-arc meter, with volume percentage on its right and
unchanged assigned actions, is development-only. The original Fedora V1,
Debian and Ubuntu downloads retain their earlier displays.
[All gauge styles and limitations](docs/audio-meters.md).

## Install

### Simplified Installer — recommended

**One setup helper handles dependencies, signed downloads and installation.**
It detects your OS, selects the correct package and offers to install missing
runtime packages. No Rust compiler is needed.

Download and run as your normal desktop user:

```sh
curl --fail --location --proto '=https' --proto-redir '=https' \
  https://raw.githubusercontent.com/infamous-pattern/decksmith/main/scripts/install.sh \
  --output decksmith-install.sh && sh decksmith-install.sh
```

On Debian 13, Ubuntu 26.04 or Pop!_OS 24.04, opt into the experimental package:

```sh
sh decksmith-install.sh --experimental
```

**[Simplified Installer: downloadable shortcut, options and first start](docs/install-decksmith.md)**

Review the linked helper before running it. Do not run it with sudo. Your saved
layouts and login preference are preserved. The helper
never starts or restarts background controls and never enables login startup.

### Manual Installation

Prefer to install dependencies and download/verify the files yourself? Follow the
matching step-by-step guide:

- **[Fedora Workstation 44 — supported V1](docs/v1-verified-installation.md)**
- [Debian 13 — experimental](docs/debian-preview-installation.md)
- [Ubuntu 26.04 — experimental](docs/ubuntu-preview-installation.md)
- [Pop!_OS 24.04 COSMIC — experimental](docs/popos-preview-installation.md)

Both methods verify signatures, checksums and archive contents before executing
release installer code. **Checksums alone do not prove authenticity.** Never skip
signature verification if it fails.

### After installing

1. Close other Stream Deck controllers and open Decksmith from your app menu.
2. Choose **Start** under Background controls. An update to a running service
   takes effect when you stop and start those controls deliberately.
3. Optionally enable **Start Decksmith at login**. New installations leave it off;
   updates preserve your saved preference.

Use a normal graphical session with D-Bus and working PipeWire audio services.
USB access may need a separate administrator-installed udev rule. The optional
GNOME panel extension is installed separately.
[Dependencies and optional services](docs/dependencies.md) ·
[USB access, update, backup and recovery](docs/installation.md).

## Understanding audio controls

Application volume changes an app's **Linux playback streams**, not sliders
inside the app or website. For example, adjusting Brave in Decksmith leaves
YouTube's volume slider unchanged and affects matching Brave streams together.

**YouTube volume → Brave app volume → speaker/headphone output volume**

**System sounds** controls desktop event sounds independently of application
and output-device levels. Choose a named output device to adjust overall listening
volume. Ordinary Volume Up/Down/Mute key actions control the default output.
The percentage shows configured volume; the live meter shows signal activity.
[Audio setup and behavior](docs/audio-controls-foundation.md) ·
[Live meters](docs/audio-meters.md) · [Media player selection](docs/media-controls.md).

## Help, testing and development

- [User guide](docs/USER_GUIDE.md) and [interface gallery](docs/interface-tour.md).
- [Installation and recovery](docs/installation.md), [desktop integration](docs/desktop-integration.md)
  and [Auto-Lock](docs/auto-lock.md).
- [V1 known limits](docs/v1-release-notes.md) and [qualification evidence](docs/v1-release-review.md).
- [Source-build prerequisites](docs/dependencies.md#building-from-source),
  [contributing](CONTRIBUTING.md) and [security reporting](SECURITY.md).
- [V1.5 feature plan](docs/ROADMAP.md#v15-feature-plan--organized-october-8),
  [product design](docs/PRODUCT_REQUIREMENTS.md) and [architecture](docs/TECHNICAL_ARCHITECTURE.md).

When reporting an issue, include your distribution/version, desktop session,
Decksmith build and device model, plus steps to reproduce. Remove credentials and
private information from logs. Do not include Homebridge passwords or two-factor codes.

The interface is currently English-only. English, German, French, Spanish and
Italian localization is planned for **V2**; see the [localization plan](docs/localization.md).

Decksmith is developed with **ChatGPT/Codex assistance**, alongside human design,
review and testing. Core controls do not require a cloud account.

## License and trademarks

Original project code is [Apache-2.0](LICENSE). Bundled fonts and icons have their
own notices: [font licenses](assets/fonts/README.md) and
[icon sources and licenses](docs/appearance-and-icons.md).

Elgato and Stream Deck are trademarks of Corsair Memory, Inc. Decksmith is an
independent project and is not affiliated with, endorsed by, or sponsored by
Elgato or Corsair.

## Support Decksmith

If you enjoy Decksmith, you can [buy me a coffee](https://buymeacoffee.com/infamouspattern).

<a href="https://buymeacoffee.com/infamouspattern"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" width="218" height="61"></a>
