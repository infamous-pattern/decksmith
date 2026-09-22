# Install, update and recover Decksmith

Decksmith now provides a relocatable **development runtime bundle** and a per-user
installer. This is not yet an RPM or a signed production release. The bundle
contains the daemon, command-line client, native editor, helpers, artwork, fonts,
icons and documentation; it does not need the source checkout after installation.
The current build targets Fedora 44 x86_64. Other distributions/architectures need
separate builds and verification. A fresh Fedora44 Workstation VM has passed
[clean runtime acceptance](clean-fedora-acceptance.md); guest physical USB access
was not part of that test.

## Dependencies

On Fedora, install runtime dependencies using the normal package manager:

```sh
sudo dnf install python3 python3-gobject python3-pillow python3-cairo gtk4 libadwaita librsvg2 glib2 systemd systemd-libs pulseaudio-libs pulseaudio-utils wireplumber
```

Use a graphical user session with its normal D-Bus and PipeWire/PulseAudio services.
The doctor checks required libraries, GTK interfaces, commands, architecture,
manifest integrity and VirtualDeck. An absent/unreadable Stream Deck is reported
separately and does not prevent installation. It does not install dependencies,
change audio routing or write to USB devices.

For a source build, additionally install Rust/Cargo, GCC, binutils, pkgconf-pkg-config
and **systemd-devel** (libudev headers). No task-specific SDK path is part of the
installed runtime. Build with `python3 scripts/build-bundle.py`; the builder uses
the locked dependency graph and hardware features. `--binaries DIR` packages already
built hardware-enabled binaries. Development bundles record the source commit and
whether uncommitted changes were present.

## One-command installation

On Fedora 44 or 45 x86_64, run as your normal desktop user:

```sh
curl -fsSL https://raw.githubusercontent.com/infamous-pattern/decksmith/v0.1.0-preview.2/scripts/install.sh | sh
```

[Review the script](../scripts/install.sh) before running it if preferred. It asks
DNF to install missing dependencies, downloads the pinned preview over HTTPS,
verifies the release checksums, and runs the per-user installer. It does not use
root for Decksmith installation, start controls, or enable login startup. Fedora
44 remains the primary tested platform. The manual steps below remain available.

## First installation

Obtain the bundle and the matching `decksmith-install.py` and `package_io.py`
from the same trusted Decksmith release output. The builder writes these two
bootstrap files and this guide beside the archive; a source checkout is not needed
on the destination machine. In the checkout, these files live under `scripts/`.
The two installer files must be in the same directory. SHA-256 checks detect
corruption; they do not authenticate an untrusted download. Only install bundles
from a source you trust, because dependency checks execute the bundled binary.

```sh
python3 decksmith-install.py install /path/to/decksmith-RELEASE.tar.gz
```

This installs under `$XDG_DATA_HOME/decksmith/app` (normally
`~/.local/share/decksmith/app`), keeps each release and updates a `current` link.
It creates a menu entry, `~/.local/bin/decksmith`, `decksmithctl`,
`decksmith-manage`, and a persistent user `decksmith.service` unit. Add
`~/.local/bin` to your PATH if necessary. Existing saved layouts, settings and
imported icons are backed up and preserved. Unknown/customized integration files
are not silently overwritten.

**Startup stays off unless you explicitly enabled it separately.** Installation
never enables login startup. It does not start or restart controls by default.
Open Decksmith and use **Start** under Background controls when ready. Close other
Stream Deck controllers first, as with the development version.

To deliberately switch running background controls to the installed version,
add `--activate`. This stops/restarts only Decksmith after validation and backup;
it does not close the editor or save drafts. Save wanted edits before reopening
the editor to load the installed version. A readiness failure restores the previous
integration and attempts to restart the previous service. Held push-to-talk controls
should be released before a deliberate restart. Nothing changes system audio routing.

The first migration may replace the development menu entry. `rollback --original`
restores that entry; keep the old checkout until migration is accepted. Existing
USB access rules, including OpenDeck rules, are left alone.

## Check and update

```sh
decksmith-manage status
python3 ~/.local/share/decksmith/app/current/scripts/runtime-doctor.py
decksmith-manage install /path/to/new-decksmith-RELEASE.tar.gz
```

Use the corresponding XDG data path if customized. Updates retain the previous
release and take a fresh configuration/icon backup. To activate immediately, use
`install … --activate`; otherwise stop and start Background controls when ready.
The unit restarts after an unexpected process failure with bounded retries; normal
stops remain stopped. Device reconnect handling is retained in the daemon. Inspect
failures with `journalctl --user -u decksmith.service`; after correcting a repeated
failure, use `systemctl --user reset-failed decksmith.service` and start again.

For a new Fedora installation with inaccessible USB, inspect the optional
`packaging/udev/70-decksmith-plus.rules` in the bundle. It grants active-session
access only for the Stream Deck +. An administrator can install an appropriate
rule and reload udev, then reconnect the device. The per-user installer does not
alter `/etc`, replace existing rules or claim access was tested when no device is
available. The main panel now offers an explicit login-startup preference; the
installer preserves the existing setting. The optional [GNOME panel menu](desktop-integration.md)
is installed separately. [Auto-Lock](auto-lock.md) is available as an explicit preference.

## Backup and recovery

```sh
decksmith-manage backup
```

Backups go to `$XDG_STATE_HOME/decksmith/backups` (normally
`~/.local/state/decksmith/backups`). They include all Decksmith configuration files
and imported icons, use private file permissions and include integrity hashes.
Keep an additional copy outside this disk if you need protection against disk loss.
They do not include unsaved editor drafts or arbitrary files outside managed data.

Save wanted edits, close Decksmith and stop Background controls before restoring,
rolling back or removing integration. The tools refuse these operations while the
normal app or service is active.

```sh
decksmith-manage restore /path/to/backup.tar.gz
decksmith-manage rollback
```

Restore validates the archive and layouts before writing, takes a pre-restore backup,
and merges backed-up files into managed data. Newer extra files are preserved, so
this is not an exact deletion-based snapshot restore. Writes are individually atomic;
a reported write failure restores overwritten files. A power loss across multiple
files may require another restore from the retained backup.

Rollback selects the previous installed release after checking dependencies and
saved-layout compatibility. Configuration is preserved; if an older release cannot
read it, the operation stops so you can choose a suitable backup. Start controls
when ready. For the first migration, use `decksmith-manage rollback --original` to
restore pre-install launchers/service integration instead. Release files and data
remain available. Keep a copy of the standalone installer for recovery if a launcher
is damaged or removed.

## Remove integration without losing data

```sh
decksmith-manage uninstall
```

After the app and controls are stopped, this disables/removes only unchanged managed
launchers and the service unit. Configuration, imported icons, all releases and
backups are deliberately retained. It is not a disk-space purge and does not remove
USB rules or shared dependencies. Reinstall a trusted bundle to restore integration.

## Isolated validation

`python3 scripts/decksmith-install.py --stage-root /tmp/decksmith-test install BUNDLE`
exercises real extraction, dependency checks, backup and integration generation in
an isolated tree. Every manager command accepts that prefix; it never calls the
host service manager. Test files use their own config/data/state/bin directories.
Do not launch its panel without isolated XDG variables and a private D-Bus session.

`tests/runtime_bundle_smoke.py ROOT` runs under `dbus-run-session` with
`DECKSMITH_ISOLATED_TEST=1` and fresh XDG config/data/state paths. It opens only
VirtualDeck and tests previews, navigation and saving. `--native` also checks an
isolated GTK panel/editor and captures its preview. It must not run on the real
session bus. The internal `--virtual-service` daemon mode is bounded to 120 seconds
and exists for this test; it is not the installed physical-device service command.

September 14 live acceptance covers the approved installation migration, one physical
unplug/reconnect with automatic recovery, and a prior-release rollback followed by
return to the current release. Configuration/audio were preserved and startup stayed
disabled. The two rollback versions share an identical daemon; this does not validate
older engine or storage-schema migrations. A fresh Fedora44 Workstation VM also passed
[clean runtime acceptance](clean-fedora-acceptance.md), including reboot persistence
and retained-data removal/reinstallation. Guest physical USB access remains outside
that result. Native RPM packaging and signed release distribution remain future work.
