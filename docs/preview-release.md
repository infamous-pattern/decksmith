# Decksmith v0.1.0-preview.2

An early reviewer preview for Fedora Workstation 44, GNOME/Wayland, x86_64 and
Stream Deck +. This release is intended for feedback, not a declaration of V1
stability. The v0.4 documentation remains the design baseline.

## Start here

Download the runtime archive, its checksum, `decksmith-install.py`,
`package_io.py`, and `INSTALL.md` from the GitHub release. Keep the installer files
together, install the listed Fedora dependencies, and follow INSTALL.md.
Installation preserves existing layouts and keeps login startup off unless you
have already enabled it. Close other Stream Deck controllers before starting
Decksmith background controls. No compilation is needed for the supplied bundle.

## Useful review paths

- Start background controls from Home; try editing a key and a dial.
- Create a page, change its theme, and switch between keys and dials.
- Try application audio, system sounds, media controls and touch-strip feedback.
- Try a custom label, artwork, and fonts; save, close and reopen the editor.
- Check keyboard navigation: Tab, Alt+1 through Alt+5, Ctrl+S and Ctrl+Z.
- Check stop/start, device unplug/reconnect, and lock/unlock if Auto-Lock is enabled.

Reboot and shutdown actions request desktop confirmation. Save other work before
trying suspend or power actions. Closing the editor leaves controls running;
Quit Decksmith stops them and clears the device.

## Known limits

- Only one Stream Deck + is controlled at a time. Other models and independent
  multi-device control are not certified by this preview.
- The runtime bundle is for Fedora 44 x86_64; other platforms need validation.
  There is no signed RPM or automatic updater yet.
- English only today; English, German, French, Spanish and Italian localization
  is planned for V2. General plugin support targets V1.5.
- Homebridge is an optional experimental companion, not a general plugin store.
  Password storage requires Secret Service/GNOME Keyring and `secret-tool`.
- Application audio controls change the application's PipeWire/PulseAudio stream;
  they do not move a website's own volume slider. System sounds are separate.
- Small-window and keyboard checks are part of development validation; full
  screen-reader certification and all GNOME fractional-scaling combinations remain
  future work. Very large text may require scrolling or a larger window.
- Physical device recovery has been exercised on the development desktop;
  testing across additional USB controllers and suspend configurations is welcome.

## Feedback

Open a GitHub issue with your Fedora/GNOME version, device model, steps to reproduce,
expected result and actual result. For visual problems, include window size,
display scaling and a screenshot. Remove passwords, tokens, private server
addresses and personal layout details from any logs or images before posting.

See [installation and recovery](installation.md) for status, rollback and removal.

## Release validation

The release preparation passed 121 Rust tests (one hardware-dependent test
ignored), strict Clippy checks, formatting, 140 Python editor tests and 13
installer/companion tests. The pinned dependency audit passed advisories, licenses,
bans and sources. Native GTK checks passed the compact 1024×768 layout in light
mode and dark mode with `GDK_DPI_SCALE=1.5`, including navigation, draft retention,
save gating, page tools and stable key/dial geometry. This environment-variable
check is not certification of compositor fractional scaling.

The reference build and GitHub checks use Rust 1.97.1. Newer linter versions may
introduce additional style diagnostics; compiler upgrades are reviewed separately.
