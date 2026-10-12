# Fresh Ubuntu installer acceptance — October 12, 2026

## Outcome and environment

The unchanged public Simplified Installer passed a fresh Ubuntu 26.04
GNOME/Wayland desktop, installing the existing experimental
`v1.0.0-ubuntu-preview.1` package as the normal user. Bundle
`1.0.0-22dc2982d0b3` and source
`e627ea4f7f660a6ff6ad4772a013989b36b717c4` were verified before installation.
Fedora V1, release archives and the workstation installation are unchanged.

This separate guest uses Canonical's Ubuntu 26.04 cloud image dated September 27,
with a repository-installed `ubuntu-desktop-minimal` environment and OS/access
tools. It is not a clone of the existing Decksmith Ubuntu VM and does not seed
Decksmith's runtime package list. Canonical's checksum signature was verified
against the full cloud-image signing-key fingerprint published in its
[image verification guide](https://ubuntu.com/docs/public-images/public-images-how-to/verify-image-checksum/).
The image SHA-256 is
`8800651811af9a85465ad1d552add729947bb16488dddb4a9b5305a3d97332b2`.
Provisioning completed without cloud-init errors. A standalone, checksum-verified
pristine disk was retained before Decksmith installation. No USB or host folders
were attached; VM autostart is disabled.

The fresh baseline lacked `pulseaudio-utils` and `python3-gi-cairo`. The helper
detected and installed both with a real APT confirmation and explicit consent
for its pinned temporary verification CLI. The fixture grants passwordless
elevation only to `apt-get`; sudo password entry is not part of this test.

## Results

| Check | Result |
|---|---|
| Experimental opt-in required | Passed |
| Fresh terminal dependency setup and signed installation | Passed |
| Runtime doctor and desktop launcher | Passed |
| Fresh install leaves startup disabled and controls stopped | Passed |
| Controlled missing `python3-gi-cairo` repair | Passed; only that package removed |
| Same-build reinstall preserves saved labels, imported artwork, brightness and enabled login preference | Passed; exact saved-data hashes unchanged, controls remain stopped |
| Refuse temporary verification CLI | Passed; app/data unchanged |
| Backup and exact saved-data restoration | Passed |
| Original-integration rollback and reinstall | Passed; saved data preserved |
| Tampered installer and wrong source identity | Both rejected before staging or installation |
| Native GNOME/Wayland editor and private-bus VirtualDeck | Passed; previews, key/dial selection, page navigation, save and validation |
| Cleanup | Startup restored disabled; guest gracefully shut down |

The controlled single cached package repair did not produce another APT question;
the fresh multi-package case separately exercised the real confirmation path.
The native editor capture was inspected. Its isolated session emitted a GVfs
mount warning; this test does not qualify the portal or artwork file picker.

## Evidence and limits

Final retained private evidence:
`20261012T184443Z-fresh-installer-ubuntu-5z8x75qo`.
It includes host/guest outcomes, dependency reports, terminal logs, saved-data
hashes, runtime doctor, native smoke output and the editor capture.

The earlier run `20261012T183645Z-fresh-installer-ubuntu-sz8vbvrd` passed fresh
installation, then stopped at its removal guard: removing `python3-pil` would
also remove `hplip`. No removal was performed. The corrected fixture uses the
standalone `python3-gi-cairo` package. The earlier outcome and disk are preserved;
the full retry started from the checksum-verified pristine baseline.

Public helper SHA-256:
`5c179475605ba7b861202e25878c739e402b7715cca32900966fdb9d83c9ee5b`.

This qualifies this cloud-image-plus-desktop environment, not all Ubuntu install
methods. Recovery covers the same published build and original integration;
older engine/schema upgrades remain unqualified. Physical USB/audio, lifecycle,
performance, portal/file-picker and human keyboard/screen-reader acceptance
remain separate gates. Ubuntu remains an experimental preview.

## Normal-session reboot follow-up

Run `20261012T192421Z-fresh-installer-reboot-6yf7t1a1` additionally passed
normal-session service start/stop before and after a confirmed reboot. The boot
ID changed, the build stayed the same, startup remained disabled and controls
stayed stopped at login. The control API responded with no USB device attached
and zero service restarts; stopping controls released its bus name. Exact
layout/artwork hashes and brightness/layout preference values were preserved.
The preferences check allows JSON formatting normalization without allowing
value changes. The guest was shut down afterward. This is not physical recovery
or enabled-at-login acceptance.

## References

- [Simplified Installer](../install-decksmith.md)
- [Ubuntu preview installation](../ubuntu-preview-installation.md)
- [Fresh Debian acceptance](2026-10-12-debian-fresh-installer.md)
