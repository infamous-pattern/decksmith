# Fresh Fedora installer acceptance — October 12, 2026

## Outcome and scope

The reviewed terminal fix passes a new Fedora Workstation 44 GNOME/Wayland guest
installation and a controlled missing-dependency case. Both use the existing
public, signed V1 release and install build `1.0.0-c5e4fea53afe` as the regular
desktop user. The corrected helper was supplied as a reviewed local candidate.
After publication to GitHub, a fresh download of the public `main` shortcut
matched that tested helper byte for byte and matched the digest recorded below.
No release archive, signing identity, source pin or production desktop build was
changed. Gitea synchronization remains pending because its pinned SSH endpoint
was unavailable during publication; the public GitHub installer is updated.

## What the fresh test found

The scheduled October 11 run failed before boot: the VM requested automatic BIOS
selection, but that host's firmware-descriptor catalog advertised UEFI only.
Selecting its installed SeaBIOS ROM directly resolved the launch failure and
passed an actual Anaconda installation and installed-system boot. Existing test
VMs and their disks were preserved.

Fedora's confined guest agent could read mount information but could not read
SSH host keys. The public host key was obtained from a read-only filesystem view
of the powered-off new guest, then pinned before SSH access. SELinux remained
enforcing. An independent pristine qcow2 copy was retained and its digest checked
before subsequent tests. Guests were shut down gracefully; no force-off was used.

The public helper's interactive prompt then failed on a real controlling
terminal. Opening `/dev/tty` with buffered `r+` creates a seek-dependent stream;
the terminal is not seekable. The helper caught that error and incorrectly
suggested that explicit portable-CLI consent was needed without showing a prompt.
The same open mode also affected dependency confirmation.

The fix uses separate read/write text streams for CLI consent and a read-only
terminal stream for the package manager's input. Explicit consent, refusal,
normal package-manager confirmation, signature verification and cleanup behavior
are preserved. The generated shell helper contains the exact reviewed Python
source. Tests now use real pseudo-terminals instead of only regular-file mocks.

## Results

The guest was installed anew from Fedora 44 repositories with the Fedora
Workstation environment, SSH/guest-agent access and desktop autologin. It was not
a copy of a previously prepared Decksmith guest. This repository-installed
Workstation environment already contained every listed runtime package; it is
not a claim that every live-ISO or minimal installation has the same package set.

| Check | Result |
|---|---|
| Fresh desktop, no previous Decksmith installation/layout | Passed |
| Real terminal consent for temporary verification CLI | Passed |
| Signed public V1 download, verification and regular-user installation | Passed |
| Runtime doctor and desktop launcher | Passed |
| Background controls remain stopped and startup remains disabled | Passed |
| Controlled missing `pulseaudio-utils` dependency | Detected and installed after normal DNF confirmation |
| Second signed installation and runtime doctor | Passed |
| Native GNOME/Wayland panel/editor, key/dial selection and rendered capture | Passed |
| Private-bus VirtualDeck previews, page navigation, layout save/validation | Passed |
| Fixture restoration and final shutdown | Passed; all seven server guests off |

The missing-package fixture removed `pulseaudio-utils` and its small dependent
`speech-dispatcher-utils` package after reviewing DNF's proposed transaction.
Both were restored afterward. This was a deliberate test in the disposable guest,
not a package removal on the workstation or an existing compatibility VM.

`dbus-daemon` was added only after the two installer cases, to provide
`dbus-run-session` for isolated test buses. It is a test-tool requirement, not a
new Decksmith runtime dependency. The native smoke driver also now waits for
confirmed page/display state: a page request can be acknowledged before the
updated status is published. The bounded wait does not establish numeric visible
response timing. The final native run passed with the retained signed V1 binary.

96 relevant installer/release tests pass, including real-terminal acceptance and
refusal, dependency input, generated-source parity and signed-install admission
checks. Shell syntax, bootstrap freshness and documentation whitespace checks pass.

## Evidence and limits

Reviewed candidate helper SHA-256:
`5c179475605ba7b861202e25878c739e402b7715cca32900966fdb9d83c9ee5b`.

Private AI-server evidence is retained under these run identifiers:

- `20261012T143019Z-fresh-installer-candidate-em6by2g4`: both installer cases,
  before/after dependencies, normal terminal confirmations, signature logs,
  runtime reports and restored fixture packages. Its initial GUI-driver failure
  remains recorded rather than overwritten.
- `20261012T143955Z-fresh-installer-native-bie579fj`: successful native continuation,
  combined acceptance results, editor capture and final powered-off state.
- Earlier provisioning, access and harness failures remain available for diagnosis.

This qualifies the reviewed helper on this fresh Fedora 44 environment. It does
not add fresh dependency qualification for Debian, Ubuntu or COSMIC, nor physical
USB/audio, screen-reader, lifecycle or performance acceptance. Existing limits
and release support status remain unchanged.

## References

- [Simplified Installer](../install-decksmith.md)
- [Initial installer qualification](2026-10-11-simplified-installer.md)
- [Libvirt guest firmware configuration](https://libvirt.org/formatdomain.html#bios-bootloader)
