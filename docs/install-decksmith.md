# Simplified Installer

The simplified helper selects an **existing signed release**, installs missing
runtime dependencies through the OS package manager, verifies the downloads and
installs Decksmith for your desktop user. It does not rebuild the app or introduce
new release binaries.

## Source checkout — optional

If you already have a reviewed source checkout, run the same helper directly:

```sh
sh scripts/install.sh
```

On Debian 13, Ubuntu 26.04 or Pop!_OS 24.04, read the corresponding
[preview limits](../README.md#downloads-and-compatibility) and add `--experimental`.

Fedora 44 selects `v1.5.0`. Each experimental OS selects its own published preview
and archive. Fedora 45 Beta, openSUSE, other versions and other architectures are
not automatically routed to a different OS's download. The helper stops and
explains the limitation.

## Downloadable shortcut

Download the script over HTTPS, [review it](../scripts/install.sh), then run it as your normal user:

```sh
curl --fail --location --proto '=https' --proto-redir '=https' \
  https://raw.githubusercontent.com/infamous-pattern/decksmith/main/scripts/install.sh \
  --output decksmith-install.sh && sh decksmith-install.sh
```

For an experimental platform, append `--experimental` to the `sh` command.
Do not run the script with sudo. Initial trust comes from the reviewed project
source obtained over GitHub HTTPS and the trusted OS GitHub CLI package. This
bootstrap does not authenticate itself; release files receive separate provenance
verification. Advanced users can pin the helper URL to a reviewed public commit
instead of the changing `main` branch.

## Manual Installation — alternative

For individual dependency, download and verification steps, use the
[Fedora](v1.5-verified-installation.md), [Debian](debian-preview-installation.md),
[Ubuntu](ubuntu-preview-installation.md) and [COSMIC](popos-preview-installation.md)
guides.

## Dependencies and verification

Python **3.11 or newer** is needed to run the verifier. It is supplied on the
listed desktop distributions. If missing, install `python3` from your trusted OS
repositories first; use `sudo dnf install python3` on Fedora or
`sudo apt-get install python3` on the listed Debian-family systems. Downloading
the shortcut also needs `curl` (install it with the same package manager if absent).

The helper checks required GTK/Python artwork, audio and system packages. If packages are missing, it shows their names and invokes DNF
or APT with sudo. You enter any password in your terminal, and the package manager
asks for confirmation. It does not automatically add repositories, run a full OS
upgrade or replace audio routing. On APT systems with stale package lists, run
`sudo apt-get update` before retrying. No Rust compiler is needed for prebuilt
packages. See [dependencies and optional services](dependencies.md).

Signature verification uses **GitHub CLI 2.102.0 or newer**, which includes
[upstream fixes for workflow/ref identity verification](https://github.com/cli/cli/releases/tag/v2.102.0).
If a suitable system CLI is available, the helper uses it. Otherwise it asks
whether to download an official, pinned **2.102.0** Linux x86_64 copy into its
private temporary directory. Its archive must match the reviewed SHA-256 stored
in the helper before the executable is read or run. The copy is removed when the
helper exits; it does not replace your installed CLI, modify PATH permanently or
add a package repository. No GitHub account is required for verification using
the retained signing bundle, although trust-root retrieval still needs network
access. If the CLI requests authentication or network verification fails, stop
and follow its guidance; never bypass signatures.

To allow that temporary copy explicitly, add `--allow-portable-gh`. To retain a
system CLI instead, update it using the [official installation guidance](https://cli.github.com/).
The helper stops if neither route is accepted or the pinned download fails checks.
The initial trust for the portable tool is the reviewed helper's upstream
version/digest pin obtained over GitHub HTTPS; it is not a checksum supplied by
the Decksmith release being verified.

All six signed subjects are verified against the configured public source commit,
repository, platform-specific signing workflow, main ref and GitHub-hosted runner
identity. Checksums and archive contents are then validated in a private snapshot
before any downloaded installer code executes. Release/source pins are reviewed
configuration, not a value supplied by the downloaded metadata.

## Useful options

```sh
sh scripts/install.sh --check
sh scripts/install.sh --verify-only --no-deps
sh scripts/install.sh --allow-portable-gh
```

`--check` prints the selected release and missing dependencies without downloading
or changing anything. `--verify-only` authenticates the downloads without
installing Decksmith; it can still offer dependency installation unless
`--no-deps` is supplied. `--no-deps` stops with a package command if dependencies
are missing. Preview platforms need `--experimental` with these options too.
For test-only isolation, `--stage-root DIRECTORY` redirects application setup and
never manages your normal user service; it is not a production installation.

## First start, updates and recovery

Save open editor changes before updating and reopen the editor afterward. Existing
saved layouts, imported artwork and startup preference are preserved. The previous
build and a configuration backup are retained by the existing per-user installer.
The helper never activates background controls or enables login startup. If
controls were already running, they continue on the previous process until you
deliberately stop/start them.

Open Decksmith from your application menu, close other Stream Deck controllers,
and choose **Start** under Background controls. Enable **Start Decksmith at login**
only if desired. A connected device is not required to install or use VirtualDeck.

The helper does not grant USB permissions or install a GNOME extension. If the
Deck is unavailable, follow the [USB access guidance](installation.md#check-and-update).
The [GNOME panel extension](desktop-integration.md) is optional; COSMIC does not
use it. Use [backup and recovery](installation.md#backup-and-recovery) for retained
builds, restore, removal and rollback. Do not replace a newer development build
with an older stable release without checking configuration compatibility.
