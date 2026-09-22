# Fedora 45 Beta compatibility check — September 15, 2026

A separate Fedora Workstation 45 Beta VM was installed from authenticated release
media to test the existing Decksmith runtime without changing the Fedora 44 host
or its retained acceptance VM. This is a beta compatibility sample, not final
Fedora 45 certification or physical Stream Deck certification.

## Environment and provenance

- Official [Fedora Beta media](https://dl.fedoraproject.org/pub/fedora/linux/releases/test/45_Beta/Workstation/x86_64/iso/):
  `Fedora-Workstation-Live-45_Beta-1.3.x86_64.iso`, 2,981,865,472 bytes.
- ISO SHA-256: `71367760b4cfda9cb5a1dbe9875a2c5fd89ef8333e6fac7508c366e650207905`.
- Signed CHECKSUM verified with Fedora 45 key
  `4F50A6114CD5C6976A7F1179655A4B02F577861E`, matched against
  [Fedora's published fingerprint](https://fedoraproject.org/security/).
  The mirror redirect returned HTTP 403; the official direct download succeeded.
- Fresh 40 GiB qcow2, 4 vCPU, 8 GiB RAM, BIOS/q35, virtio devices,
  GNOME/Wayland, user-session libvirt `qemu:///session`.
- Persistent domain `decksmith-fedora45-beta` (title **Decksmith Fedora 45 Beta**),
  UUID `572c5233-66f7-4908-868a-a11d28805616`. ISO detached after installation.
- No host shares, USB passthrough or host audio backend; localhost-only guest SSH,
  SPICE console and user-mode networking. The Fedora 44 VM remains separate.
- ISO package baseline, without a general update: kernel `7.2.0-61.fc45`,
  GNOME Shell `51~beta-4`, Python `3.15.0~rc1-1`, GTK `4.23.3-1`,
  libadwaita `1.10~beta.1-2`, PyGObject `3.57.1-6`, Pillow `12.3.0-3`,
  GLib `2.89.4-1`, systemd `261.2-1`, PipeWire `1.6.8-3`,
  WirePlumber `0.5.14-2`. Versions are from the installed guest.
- Decksmith current runtime `0.1.0-3785fa29030e`, source `54d5eb2`,
  archive SHA-256 `c82c5fa84059ffdb925ca4e888a505bc9f35597f3dc36d9c0d2f669e398db7a5`.
  This is the same release installed on the host, not a guest-specific rebuild.

## Completed application checks

- Standalone user installation succeeded with all runtime dependencies already
  present. Runtime doctor reported ready with no physical devices. No code changes
  or additional guest packages were needed.
- Native panel/editor, no-device status, icon-picker Cancel and unchanged saved
  layout passed. The native editor screenshot was visually inspected.
- Private-bus VirtualDeck startup, all eight key previews, 800×100 touch preview,
  page switching, saving and layout validation passed, with graceful daemon exit.
- Backups/restoration with imported artwork, prior-release update, rollback/return,
  integration removal/reinstall and preservation of configuration passed.
- The actual installed desktop launcher registered the application, and visible
  Start/Stop buttons controlled the guest service. Running correctly displayed
  “Waiting for device”; the service had zero automatic restarts and startup disabled.
- Guest-only temporary null output and playback stream: output volume/mute,
  independently controlled app volume/mute, and measured output/app signal meters
  passed. The temporary audio module and playback/meter processes were removed
  at test completion. This does not certify physical audio devices or microphone capture.

## Reboot and observations

The installed guest rebooted successfully with a new boot ID. Saved layout/artwork
hashes and parsed configuration values matched the recovery fixture; the daemon
normalized settings JSON formatting without changing values. Decksmith remained
inactive with startup disabled and zero automatic restarts, and its menu launcher
worked again after reboot. Guest HTTPS access also passed.

During shutdown Fedora's `dnf5daemon-server.service` delayed reboot for 134 seconds
(12:02:15–12:04:29 local time), after the user manager had stopped. Its log reported
`NameOwnerChanged: map::at` before it deactivated successfully. No forced reset or
package-service modification was needed. This is separate from the Decksmith service
checks; the underlying cause is not established.

GTK emitted deprecation and virtual graphics fallback warnings; native rendering
completed successfully. The private D-Bus test also logged an accessibility-bus
activation failure, so that isolated check does not certify accessibility support.
These results do not certify GPU acceleration, physical USB permissions/hotplug,
real application/media-player behavior, or every future Beta update. The two tested
Decksmith releases have identical daemon binaries, limiting rollback coverage to
installation/UI compatibility rather than older engine/schema migrations.

## Retained VM

Open **Virtual Machine Manager → QEMU/KVM User session → decksmith-fedora45-beta**.
The normal administrative guest account is `decktest`; its password is retained
privately under `local/fedora45-vm/guest-credentials.json` and provided directly to
the user. Graphical automatic login is enabled; sudo requires the guest password.
Host-login VM autostart and Decksmith automatic startup remain disabled. The VM
is left running with its virt-manager console and installed Decksmith panel open.
The host Decksmith service retained the same PID, invocation and active state,
and the Fedora 44 VM remained powered off throughout this test.

VM disk, ISO, validated domain XML, guest-only keys, test fixtures, package inventory,
logs and screenshots are retained under `local/fedora45-vm/` (ignored by Git).
No host personal configuration or artwork was copied into the guest.
