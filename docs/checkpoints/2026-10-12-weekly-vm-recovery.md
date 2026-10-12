# Weekly VM maintenance recovery — October 12, 2026

## Outcome

The final six-guest maintenance batch passed. Each guest had a verified offline
recovery copy, successful OS update commands, a reboot confirmed by a changed
boot ID, installed-build smoke checks, and a confirmed powered-off state.
All seven server Linux guests, including the separate fresh-installer fixture,
were off afterward. The 2 AM Eastern nightly and Sunday 6 AM Eastern VM timers
remain enabled and unchanged. No production workstation build or release asset
was changed.

## Findings and fixes

Sunday's Fedora 44 update and reboot passed, but cleanup failed after sending
the virtual power-button event and waiting 90 seconds. The guest's GNOME
power-button setting was `suspend`; its original failure boot recorded a short
power-key press. That event is not a reliable unattended shutdown request.
The evidence establishes the policy/event mismatch, not a completed suspend
transition in that original boot.

Both maintenance and smoke runners now share a bounded graceful-cleanup helper.
It acts only for a guest started by the owning run. The preferred path uses
pinned SSH, checks the guest distribution and domain UUID, and requests normal
OS poweroff. A verified shutdown that closes SSH is given time to finish; only
the confirmed libvirt powered-off state establishes success. Explicit guest-agent
and ACPI fallbacks remain available when the preferred transport is unavailable.
Identity rejection stops fallback attempts. No forced power-off is used.
Shutdown evidence and the failure phase are saved before reporting an error.
The previous runner files are retained in private recovery backups.

The first recovery batch passed five guests and shut all six down, but COSMIC's
APT update was blocked by another process holding `/var/lib/apt/lists/lock`.
The host runner now retries that exact lock error within five minutes, retaining
the original error log. Other update failures are not retried by this rule.
The final COSMIC update passed with the lock already clear; actual lock waiting
is covered by the regression fixtures rather than claimed as live acceptance.
No locks were deleted, updater processes killed, or sudo permissions expanded.

## Final installed-build results

| Guest | Installed Decksmith build | Native editor smoke | Maintenance and shutdown |
|---|---|---|---|
| Fedora Workstation 44 | `0.1.0-257113d71d41` | Passed | Passed |
| Fedora Workstation 45 Beta | `0.1.0-257113d71d41` | Passed | Passed |
| Debian 13 | `1.0.0-ca725bc8a704` | Passed | Passed |
| Ubuntu 26.04 | `1.0.0-22dc2982d0b3` | Passed | Passed |
| Pop!_OS 24.04 COSMIC | `1.0.0-5b198a52075d` | Not run in final batch | Passed; headless VirtualDeck |
| openSUSE Tumbleweed | `1.0.0-80b6faedd10b` | Passed | Passed |

These are the guests' retained installed builds, not an automatic deployment or
qualification of current GitHub main. Native checks cover the existing bounded
panel/editor rendering, key/dial selection and shared previews. VirtualDeck checks
cover startup, previews, navigation and layout save/validation. They do not add
physical USB/audio, accessibility or lifecycle release acceptance. Existing
support and experimental-preview boundaries remain unchanged.

COSMIC used the guest-agent shutdown path. Fedora 45's final smoke cleanup
required the ACPI fallback because the test account lacked the direct command's
sudo permission and no guest-agent channel was configured. That fallback passed
this run; provisioning a preferred shutdown transport there remains an
infrastructure improvement. GNOME power-button preferences were preserved.

Twenty-three targeted regression tests pass locally and on the server, covering
busy/unowned guests, disk ownership, rejected identities, graceful fallback,
disconnect handling, shutdown timeouts, durable failures, bounded APT lock
waiting and refusal to retry unrelated package errors. Local and installed runner
hashes match.

Private evidence retains both run identifiers:

- `20261012T165916Z-weekly-vms-g4obsab5`: initial recovery; COSMIC update-lock
  failure remains recorded, with all six guests off.
- `20261012T171842Z-weekly-vms-ic_r3she`: final passing six-guest result, per-guest
  maintenance and smoke outcomes, verified backup manifests, updated OS/kernel
  context, shutdown transports and retained build identities.

The old failed systemd service status was cleared only after the passing batch;
the original failure outcomes and recovery evidence remain available. The local
notification checkpoint records the handled failure and recovery to avoid
duplicate alerts.

## Reference

[Libvirt shutdown semantics and transport modes](https://www.libvirt.org/manpages/virsh.html#shutdown)
