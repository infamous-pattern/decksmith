# Simplified Installer and README cleanup — October 11, 2026

## Change and scope

The source helper replaces the historical preview.3 bootstrap with reviewed,
explicit release routes: Fedora 44 V1 and separate opt-in Debian 13, Ubuntu 26.04
and Pop!_OS 24.04 COSMIC previews. Existing release archives, signed subjects,
source/workflow pins and support status are unchanged. Fedora 45 and openSUSE
remain test environments, not automatic installation routes.

The downloadable shell helper is generated from the source orchestrator, shared
runtime package catalog and signature verifier. A regression check requires exact
embedded/source parity. Missing runtime packages are offered through the trusted
OS package manager with its normal confirmation; application installation stays
under the normal user. Python 3.11+ is required before the helper can run.

Verification requires GitHub CLI 2.102.0+, including upstream workflow/ref
identity fixes. An older or absent CLI can be replaced for this invocation only
by an explicitly accepted temporary upstream copy. Its reviewed SHA-256 pin is
checked before reading/running the executable; no system CLI or repository is
replaced. All six Decksmith subjects then require provenance verification,
complete checksums and archive admission before downloaded installer code runs.
Temporary downloads and the verification tool are cleaned up on exit.

The README now separates **Simplified Installer**, **Manual Installation** and
**After Installing**. It leads with the product, tour, features and a compact
support table. Full screenshots and dependency/build guidance have moved to
linked pages. Released/experimental/development distinctions, audio semantics,
AI-assistance disclosure, trademarks, licenses and the support link remain visible.

## Validation

- 108 script tests pass, including existing signed-install admission and recovery
  tests. New tests cover exact bootstrap/source parity, platform/version selection,
  explicit preview opt-in, installed-versus-removed APT package records, package
  confirmation through the terminal, no-change check mode, HTTPS redirects,
  bounded downloads, obsolete CLI rejection and a wrong portable-tool digest
  preventing executable extraction/run.
- Shell syntax, generated-bootstrap freshness and whitespace checks pass. Local
  links and image targets resolve in the edited documentation.
- AI-server VMs each ran the final generated helper with a real public signed
  download, the pinned temporary GitHub CLI, `--no-deps` and `--stage-root`:

| VM | Package admitted | Result |
|---|---|---|
| Fedora Workstation 44 | `1.0.0-c5e4fea53afe` | Signed isolated installation passed |
| Debian 13 | `1.0.0-ca725bc8a704` | Signed isolated installation passed |
| Ubuntu 26.04 | Existing signed Ubuntu preview | Signed isolated installation passed |
| Pop!_OS 24.04 COSMIC | Existing signed COSMIC preview | Signed isolated installation passed |

All four guests were started separately and shut down gracefully afterward.
No production desktop installation, running controls, audio routing or saved
layout was changed. These isolated checks do not add physical hardware/audio,
lifecycle or accessibility qualification to the experimental previews.

## Test limitations and follow-up

VMs already had application runtime packages. Full dependency installation on a
bare desktop and the normal user-confirmation interaction remain separate
first-install acceptance checks; fixtures cover the package-manager commands and
terminal behavior. Missing/old GitHub CLI failures were observed before the final
portable path passed. Initial Fedora test preparation installed the OS `gh`
package; DNF reported invalidating a pending guest offline-update transaction.
This does not affect the installer test result, but guest update scheduling should
be checked on its next maintenance run. No force power-off or OS upgrade was used.

The user approved source publication after this local qualification. The helper
is reviewed source obtained over GitHub HTTPS, not a self-authenticating signed
bootstrap. A pinned public source URL can be used instead of `main`. Public release
assets must not be relabeled or rewritten when publishing the source helper.

## Related guidance

- [Simplified Installer](../install-decksmith.md)
- [Manual Fedora installation](../v1-verified-installation.md)
- [Dependencies](../dependencies.md)
- [Upstream GitHub CLI fixes](https://github.com/cli/cli/releases/tag/v2.102.0)
