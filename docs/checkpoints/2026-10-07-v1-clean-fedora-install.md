# Signed V1 candidate: clean Fedora dependency installation — October 7, 2026

This checks the frozen `1.0.0-c5e4fea53afe` candidate from public source
`68c42421ceaa65bb033993b56f0babdde2a14a6e`. Archive SHA-256 remains
`4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2`.
No application code or accepted release bytes changed; V1 remains unpublished.

## Environment and isolation

A separate transient KVM guest booted the authenticated Fedora Workstation 44
1.7 live image. Its SHA-256 was rechecked as
`1620295f6a00c27c3208f0c00b8ece4eab1ec69b9002152d97488bf26a426ddf`.
Only two read-only ISO drives were attached; the OS and test installation used
a disposable RAM overlay. No retained VM disk, host directory share or physical
USB device was attached. Networking used a separate loopback SSH forwarding port
with a pinned, test-only host key. Passwordless RPM/DNF access was explicitly
configured for automation inside this disposable live guest; it is QA setup,
not an application dependency or installation recommendation.

The stock image contained all 13 documented runtime packages: Python 3.14.3,
PyGObject 3.56.2, Cairo 1.28.0, GTK 4.22.1, libadwaita 1.9.0, librsvg 2.62.0,
GLib 2.88.0, systemd 259.5, PulseAudio libraries/utilities 17.0 and WirePlumber
0.5.13, plus Pillow. The runtime reports glibc 2.43. This was a native
GNOME/Wayland session, not a desktop replacement or a retained-guest reinstall.

## Observed results

- All six download subjects passed signature/source/workflow/hosted-runner
  admission before downloaded installer code ran. Complete archive/checksum and
  standalone-file agreement passed.
- Removing `python3-pillow` only from the disposable live overlay produced a
  genuine dependency failure. The verified installer named the missing artwork
  packages and stopped. No current-release link, desktop entry, user unit or
  launcher appeared; the temporary extracted release was removed. The install
  lock and empty parent directories may remain after refusal.
- Running the documented fixed Fedora runtime package list through DNF restored
  Pillow 12.3.0. Normal Fedora package-signature checking remained enabled; the
  transaction imported the Fedora 44 key from the image and verified its package.
- Fresh per-user installation of the same signed candidate then passed. Runtime
  doctor reported ready, release integrity/architecture and VirtualDeck passed.
- The installer left login startup disabled and the service inactive. An explicit
  start produced the real user-bus API with no device connected; the daemon waited
  safely and stopped cleanly when requested.
- The private-bus VirtualDeck smoke passed preview geometry, page switching, save,
  layout validation and icon lookup. Its native editor mounted successfully after
  selecting a dial and key; the captured settings screenshot was inspected.
- Launching the actual installed application in the real GNOME session displayed
  the Home interface and the Start background controls / Not now prompt. Startup
  stayed disabled and the service stayed inactive until explicitly requested.

This closes the scoped clean Workstation dependency readiness, missing-library
refusal/provisioning/retry and native first-run installation checks. It does not
claim a fresh installed-disk reboot test: the separate October 6 real guest
per-user installation/login/reboot result remains the evidence for that case.
The preview URL bootstrap still points to preview.3 and remains checksum-only;
this test used the reviewed signed-candidate verifier and documented DNF list.
The private-bus smoke emitted portal/ScreenSaver activation warnings, so it is
not a lock, human accessibility, scaling or physical-device certification.

## Cleanup and evidence

Normal ACPI shutdown completed and the transient guest disappeared. All four
retained VMs were off afterward; the retained Fedora 44 domain XML compared
identically before/after. The desktop still ran the accepted candidate with its
same daemon PID and zero restarts. No desktop settings or packages were changed.

Private commands/results and inspected screenshots are retained under
`local/v1-final-20261007/clean-fedora/`; credentials/test keys remain ignored and
are not public documentation assets. Remaining physical lifecycle, precise
visible-response, accessibility, optional-companion and publication decisions
stay open in the [candidate checklist](2026-10-06-v1-signed-candidate.md).
