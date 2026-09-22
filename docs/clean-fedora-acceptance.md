# Clean Fedora 44 runtime acceptance — September 14, 2026

**Passed** on a newly installed Fedora Workstation 44 x86_64 VM, using the published
runtime bundles and standalone installer without a source checkout or copied user
settings. This completes the bounded clean-Fedora runtime criterion in the installation
milestone, alongside separately recorded host migration/reconnect/rollback checks.

## Environment and provenance

Created a disposable local QEMU/KVM guest with 4 virtual CPUs, 8 GiB RAM and a new
40 GiB sparse disk. Used existing host tools without installing host packages or
changing host services/groups. No shared home folder, USB passthrough or host audio/
microphone backend was attached. Networking used QEMU user networking; management
ports bound only to loopback. QEMU was used directly rather than registering a Boxes
or libvirt machine. The installed guest booted from its own Btrfs virtual disk with
a fresh test account and a GNOME/Wayland graphical session. Guest-only SSH, automatic
test login and passwordless sudo were configured for VM automation; they are not
Decksmith prerequisites. The application installer and user service ran unprivileged.

The official [Fedora Workstation 44 image](https://www.fedoraproject.org/workstation/download/)
was authenticated with its signed checksum. The valid signing fingerprint matched
[Fedora's published key](https://fedoraproject.org/security/):
`36F612DCF27F7D1A48A835E4DBFCF71C6D9F90A6`.
`Fedora-Workstation-Live-44-1.7.x86_64.iso` SHA-256:
`1620295f6a00c27c3208f0c00b8ece4eab1ec69b9002152d97488bf26a426ddf`.

Installed bundle `0.1.0-3785fa29030e` from source `54d5eb2`, using retained prior
`0.1.0-1d1e92a45c88` for update/rollback tests. The current archive matched SHA-256
`c82c5fa84059ffdb925ca4e888a505bc9f35597f3dc36d9c0d2f669e398db7a5`.
All runtime dependencies were already present on this Workstation image; no additional
host or guest packages were needed for acceptance. This was the installed ISO baseline,
not a claim of testing every later Fedora update or minimal/server spin. Examples:
GTK 4.22.1, libadwaita 1.9.0, Python GObject 3.56.2, Pillow 12.1.0, systemd 259.5,
PulseAudio libraries/utilities 17.0 and WirePlumber 0.5.13.

## Verified behavior

- Standalone bundle installation, integrity/dependency doctor, stable installed paths,
  menu launcher and its actual application process passed. Hardware inventory was
  empty as intended. Missing-dependency failure was not induced because dependencies
  were present.
- Actual native menu Start/Stop buttons controlled the guest user service. It reported
  Running / Waiting for device without pretending physical hardware was connected.
  The service stopped normally and had no automatic restart failures. Startup stayed
  disabled, including after a guest reboot.
- Native panel/editor/icon-picker Cancel checks passed with the normal no-device
  service; the saved layout was unchanged. A separate private-bus VirtualDeck session
  passed shared key/touch rendering, navigation, saving and layout validation, native
  editor/icon checks and graceful termination. Its screenshot was visually inspected.
- Guest-only test layout and imported artwork survived backup/restore, update, rollback
  and return, integration removal and reinstall. Removal preserved data, backups and
  retained versions as documented. No host configuration/artwork was used as a fixture.
- After reboot and normal service startup, layout and artwork hashes matched. The daemon
  rewrote the compact fixture settings JSON with pretty formatting; parsed layout/
  brightness values matched exactly. That benign serialization change was distinguished
  from data loss rather than claiming every file remained byte-identical.

Software-rendered GTK emitted EGL/Zink probe warnings in this non-accelerated guest;
interfaces and render checks nevertheless passed. GPU acceleration was not certified.
The two rollback bundles share an identical daemon binary, so this does not establish
older engine/schema migration compatibility. Physical USB permissions, hotplug, latency
and real audio-device behavior are not covered by VM-only results.

## Retention and host state

The guest was cleanly powered off; its disk, ISO, scripts, disposable guest credentials
and evidence remain under ignored `local/fedora-vm/` with a restart/readme file. The
virtual disk occupied about 6.2 GiB at completion. The localhost transfer helper stopped;
no artifact deletion was performed.

The host's current Decksmith release remained `0.1.0-3785fa29030e`, PID 948984,
invocation `7708ffa6b05a4224849f38e69a7f6edf`, active with zero automatic restarts and
login startup disabled. No host audio routing, device attachment or Decksmith runtime
change was part of this work.
