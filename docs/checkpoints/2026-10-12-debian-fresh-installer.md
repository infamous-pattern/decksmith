# Fresh Debian installer acceptance — October 12, 2026

## Outcome and environment

The published Simplified Installer passed fresh Debian 13 GNOME/Wayland
installation, missing-package repair, saved-data preservation, signed admission
and recovery checks. It installed the existing experimental
`v1.0.0-debian-preview.1` package, build `1.0.0-ca725bc8a704`, as the normal desktop
user. No product code, release archive or workstation installation was changed.

The separate guest was provisioned from Debian's official generic AMD64 cloud
image and updated signed Debian repositories, with a repository-installed GNOME
desktop. It was not cloned from the existing Decksmith Debian compatibility VM.
The base image was fetched over HTTPS and checked against its pinned official
SHA-512 checksum; this is not a claim of an independently signed base image.
No Stream Deck or host directories were attached. The guest has four virtual
CPUs, 6 GiB RAM and a 40 GiB virtual disk. Its completed cloud-init baseline
reported no errors. A standalone, checksum-verified pristine disk was retained
before Decksmith installation.

Only OS/desktop/access tools were provisioned in advance. The installer found
three missing runtime packages: `pulseaudio-utils`, `python3-gi-cairo` and
`python3-pil`. It installed them through the ordinary APT path, including a real
package-manager confirmation in the controlling terminal. The fixture permits
the normal user to elevate `apt-get` without a password; interactive sudo
password entry is not qualified by this unattended run.

## Results

| Check | Result |
|---|---|
| Experimental opt-in required | Passed; installation refused without the flag |
| Fresh public helper in a real normal-user terminal | Passed; APT and temporary-verifier consent observed |
| Signed release selection, provenance and runtime doctor | Passed |
| Desktop launcher; controls stopped and startup disabled after fresh install | Passed |
| Controlled removal and repair of only `python3-pil` | Passed |
| Same-build reinstall preserves page/key labels, imported artwork, brightness and enabled login preference | Passed; saved-data hashes unchanged, controls remain stopped |
| Refuse temporary verification CLI | Passed; installation and saved data unchanged |
| Backup and exact saved-data restoration | Passed |
| Original-integration rollback followed by reinstall | Passed; saved data preserved |
| Tampered standalone installer | Rejected before installation or staging |
| Wrong accepted source identity | Rejected before installation or staging |
| Native Wayland panel/editor and private-bus VirtualDeck | Passed; key/dial selection, previews, page navigation, save and validation |
| Final cleanup | Startup disabled, test guest gracefully shut down, all eight server guests off |

APT repaired the single cached package without another confirmation prompt.
That is normal APT behavior; the fresh multi-package installation separately
proved the terminal confirmation path. Signature and checksum verification
remained enabled throughout.

The native editor capture was inspected. These automated checks exercise the
published package's panel/editor path and isolated fixture layout; they do not
claim human acceptance of every sidebar, keyboard or screen-reader interaction.
The isolated session emitted GVfs/document-portal mount warnings. File-picker
and document-portal integration were not acceptance targets of this run.

## Evidence and limits

Public helper SHA-256:
`5c179475605ba7b861202e25878c739e402b7715cca32900966fdb9d83c9ee5b`.

Accepted signed source:
`dee2f2c285041dc19388ed579b90886e4b58341a`.

Final private evidence run:
`20261012T181833Z-fresh-installer-debian-55trfxok`.
Retained evidence includes host and guest outcomes, before/after dependency
reports, real-terminal installation logs, saved-data hashes, runtime doctor,
native smoke output and an editor capture.

Two earlier harness failures are retained:

- `20261012T180309Z-fresh-installer-debian-p7lvsxon`: the test used an invalid
  artwork enum in its saved-layout fixture after the public installation passed.
  The fixture was corrected to use the editor's normalized image/source fields.
- `20261012T180943Z-fresh-installer-debian-9e1o1hti`: installation and repair
  succeeded, but the harness incorrectly required APT to ask a second question
  for a single cached package. That assertion was corrected.

Both failed test disks and their logs remain available. Each retry started from
the verified pristine pre-Decksmith baseline. No failed outcome was overwritten.

This qualifies this fresh cloud-image-plus-GNOME environment, not every Debian
installation method. Reinstall and integration recovery used the same published
build; upgrades or rollbacks across older engine/layout schema versions remain
unqualified. Physical USB, real audio, lock/suspend/hotplug recovery, performance,
file-picker and screen-reader checks retain separate gates. Debian remains an
experimental preview; Fedora V1 support is unchanged.

## References

- [Simplified Installer](../install-decksmith.md)
- [Debian preview installation](../debian-preview-installation.md)
- [Fresh Fedora acceptance](2026-10-12-fresh-installer-acceptance.md)
