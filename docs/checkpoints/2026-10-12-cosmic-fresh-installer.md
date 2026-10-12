# Fresh COSMIC installer acceptance — October 12, 2026

## Outcome and environment

The unchanged public Simplified Installer passed a new Pop!_OS 24.04 COSMIC
installation from System76's official Intel/AMD build 20 ISO. It installed the
existing experimental `v1.0.0-popos-preview.1` as the normal desktop user, bundle
`1.0.0-5b198a52075d`, with accepted signed source
`be238905bb3f6aa793dcda2df063fa5a2f206cbc`.
Fedora V1, published release files and the workstation build are unchanged.

The new guest has an independent 64 GiB disk, four virtual CPUs, 8 GiB RAM and
its own UEFI variable store. It was not cloned from the retained COSMIC
compatibility guest. The ISO checksum matched System76's HTTPS release API:
`a0ef3842ab710db4f4407cf3499560b59dddbbcd59bee17beab7b0e99dc22b4c`.
The normal OS installer created the test account and installed Pop!_OS. OS
updates and SSH/guest-agent access tools were added before qualification;
Decksmith's runtime dependency list was not preinstalled. No USB or host folders
were attached. Autologin is configured only inside this test guest; VM autostart
is disabled.

The guest booted its updated `7.1.5-76070105-generic` kernel before installer
testing. The package audit was clean, and APT reported no remaining upgrades.
A standalone, checksum-verified pristine disk was retained before Decksmith
installation. Native checks used the actual COSMIC Wayland session at
1920 × 1200; its editor capture was inspected.

## OS preparation corrections

Two setup issues occurred before Decksmith installation. The test's chroot
initially omitted the installed EFI partition, so Plymouth's initramfs/kernelstub
hook failed. Mounting that partition allowed normal package configuration and
kernel installation to continue. A later firmware-service configuration prompt
needed an explicit unattended choice; the existing Pop!_OS configuration was
preserved using normal dpkg conffile options. Package authentication was never
disabled. The original error, repair and final setup logs are retained.

No installer acceptance was claimed until these OS preparation issues were
resolved, the audit was clean and the installed system booted successfully.
The baseline readiness check uses dpkg audit, not cloud-init; this is an ISO
installation.

## Results

| Check | Result |
|---|---|
| Experimental opt-in required | Passed |
| Fresh runtime dependency detection/setup in a real terminal | Passed; five missing packages installed after normal APT confirmation |
| Temporary pinned verification CLI consent | Passed |
| Signed preview admission, runtime doctor and desktop launcher | Passed |
| Fresh install leaves startup disabled and controls stopped | Passed |
| Controlled missing `python3-gi-cairo` repair | Passed; only that package removed |
| Same-build reinstall preserves saved labels, imported artwork, brightness and enabled login preference | Passed; saved-data hashes unchanged, controls remain stopped |
| Refuse temporary verification CLI | Passed; app/data unchanged |
| Backup and exact saved-data restoration | Passed |
| Original-integration rollback and reinstall | Passed; saved data preserved |
| Tampered installer and wrong source identity | Both rejected before staging or installation |
| Native COSMIC editor and private-bus VirtualDeck | Passed; previews, key/dial selection, navigation, save and validation |
| Cleanup | Startup restored disabled; guest gracefully shut down |

The five missing packages were `gir1.2-gtk-4.0`, `gir1.2-adw-1`,
`gir1.2-rsvg-2.0`, `python3-gi-cairo` and `python3-pil`. The controlled single
cached package repair did not produce another APT question; fresh installation
separately exercised that confirmation path. The fixture permits passwordless
elevation only for `apt-get`; interactive sudo password entry is not qualified.

## Evidence and limits

Final private acceptance evidence:
`20261012T191106Z-fresh-installer-cosmic-mgclaf9d`.
It includes host/guest outcomes, terminal logs, dependency reports, saved-data
hashes, runtime doctor, native smoke output and an editor capture. Separate
provisioning logs retain the official media identity and OS preparation repairs.

Public helper SHA-256:
`5c179475605ba7b861202e25878c739e402b7715cca32900966fdb9d83c9ee5b`.

Recovery covers the same published build and original integration, not upgrades
or rollback across older engine/layout schemas. This run does not qualify
physical USB/audio, lifecycle, performance, portal/file-picker or human
keyboard/screen-reader acceptance. Native test fixtures are automated rendering
checks, not a substitute for full interactive review. COSMIC remains an
experimental preview; GNOME-specific desktop integration is not COSMIC support.

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
- [COSMIC preview installation](../popos-preview-installation.md)
- [System76 downloads](https://system76.com/download-pop/)
- [Fresh Ubuntu acceptance](2026-10-12-ubuntu-fresh-installer.md)
