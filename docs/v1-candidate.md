# V1 candidate preparation

Application version `1.0.0` is prepared for qualification. It is not a release
announcement, tag or public download. The published `v0.1.0-preview.3` remains the
preview offered by the README and preview installer until a release decision.

The signed candidate is now frozen as `1.0.0-c5e4fea53afe`, from public source
`68c42421ceaa65bb033993b56f0babdde2a14a6e`. Hosted and independent verification,
the four-VM diagnostic matrix, staged recovery/preview upgrade and Fedora 44
helper-failure/ten-minute workload checks are complete. It is now installed on
the reference desktop with saved state preserved and passing bounded physical
acceptance; real Fedora 44 guest installation and background-only reboot startup
also passed. Subsequent lifecycle/accessibility checks are recorded below;
resource reconciliation and final delivery/acceptance
remain open. See the [exact-candidate checkpoint](checkpoints/2026-10-06-v1-signed-candidate.md)
for the archive digest, test scope and remaining work.

The previously deferred five-minute physical quiet observation is now recorded
in the [October 7 resource checkpoint](checkpoints/2026-10-07-v1-quiet-resources.md).
Memory and handles were bounded with zero restarts, but CPU was higher than prior
observations; the comparison remains under investigation. A completed quiet
measurement does not close the overall resource/reliability gate.

The [October 7 physical resource checkpoint](checkpoints/2026-10-07-v1-physical-resources.md)
records ten minutes of user-confirmed ordinary/quicker dial use and page switching,
210 successful actions and complete saved-layout restoration. Four target meters
were observed at 120 of 121 points; a brief Homebridge page visit retained its
fan override. Dial dispatch p95 was 21.445 ms. CPU comparison remains open;
these measurements do not constitute full V1 acceptance. Numerical visible-response
qualification was subsequently deferred beyond V1 by the October 7 user decision,
with no claim that the 150/100 ms targets passed.

The [CPU investigation](checkpoints/2026-10-07-v1-cpu-investigation.md) identifies
existing display-buffer comparisons and competing-process scans as profiling
leads. A constrained old/signed/signed/old VirtualDeck comparison averaged 1.358%
versus 1.375% daemon CPU, without reproducing the larger desktop increase.
The physical comparison remains unmatched; installed code and saved data were
preserved, and all test VMs are off. This is investigative evidence, not a full
physical regression or release pass.

The authorized [matched physical comparison](checkpoints/2026-10-07-v1-matched-resources.md)
initially stopped on transient playback and external capture with GNOME Settings
input monitors observed. After Settings closed, a retry completed one quiet pair:
previous/candidate daemon CPU was 1.457%/1.533%, core-service CPU 2.583%/2.766%,
with bounded memory/handles and no restarts or actions. The user paused during the
second candidate run. Its partial data was excluded. On continuation the second
candidate/previous pair completed at 2.010%/2.003% daemon and 3.717%/3.702% core
CPU. Across the two sessions, descriptive previous/candidate means are
1.730%/1.772% daemon and 3.143%/3.241% core CPU. The requested measurements are
complete across an explicit pause, with bounded resources and no automatic
restarts or unplanned actions. They do not establish statistical equivalence or
a continuous A/B/B/A result. Resource/release acceptance remains separate.
Independent checks confirmed the signed candidate and saved state restored;
all four test VMs are off.

The [clean Fedora installation checkpoint](checkpoints/2026-10-07-v1-clean-fedora-install.md)
now records stock Workstation dependency readiness, genuine missing-Pillow
refusal, signed-package provisioning and a successful fresh signed-candidate
installation in a disposable native GNOME/Wayland live session. The actual
first-run controls prompt and disabled startup default were verified. This
complements the prior installed-guest reboot test; it does not qualify the
preview URL installer as signed V1 delivery. All test VMs are off.

The [physical recovery follow-up](checkpoints/2026-10-07-v1-physical-lifecycle.md)
records user-confirmed lock/unlock, USB unplug/reconnect, suspend/resume and
Quit/blanking/relaunch and background-only login startup on this exact candidate,
with page/controls restored and zero automatic restarts. Quit/relaunch intentionally
restarted the service. Other untested action/fault cases remain open.

The [human accessibility follow-up](checkpoints/2026-10-07-v1-accessibility.md)
now records successful user-operated keyboard traversal, visible focus and
screen-reader acceptance, plus user-confirmed theme, enlarged-text and display-
scaling review. The exact selected scaling and numeric contrast were not measured.

The [Homebridge fault checkpoint](checkpoints/2026-10-07-v1-homebridge-faults.md)
records 70 passing VM checks, including authentication expiry, uncertain-write
protection, local HTTP access controls and responsiveness of the signed core during
companion faults. The normal guest setup was preserved and all VMs are off. The
separate companion remains experimental. The user subsequently confirmed the
agreed live accessory toggle/brightness and built-in audio coexistence check
works correctly and stays available.

The [editor/delivery review](checkpoints/2026-10-07-v1-editor-release-review.md)
records 31 focused draft/status/error tests and native light, dark, compact and
default-width enlarged-text passes. Compact width combined with enlarged text
clips content; use the default wider window. The normal guest was preserved and
all VMs are off. The [release ledger](v1-release-review.md) records prepared notes
and verified-install guidance; no public release was created.

Cargo workspace metadata is the application-version source for compiled binaries,
runtime archive identity/metadata and translation extraction. Layout, package,
backup, candidate and install-record schema versions remain at their existing
values; a product-version change does not imply a data migration.

The candidate workflow must build from a clean, immutable public source commit,
sign its exact downloads and pass independent verification. The accepted commit,
bundle ID and archive digest are recorded separately from subsequent evidence-only
documentation commits. Do not substitute rebuilt files for the accepted archive.

## Qualification sequence

1. Build and independently verify the hosted candidate, including the verified
   installation helper bundled with it. Run full source and asset/security checks.
2. On Fedora 44, test staged and real per-user installation, dependency readiness,
   upgrade from published preview.3, saved-data backup/restore, compatible rollback,
   safe rejection of unsupported saved formats, uninstall/reinstall and login/reboot.
   Preserve the normal guest's setup or use a disposable staged installation.
3. Exercise the exact artifact in all four retained VMs, recording Ubuntu/Debian
   compatibility findings without advertising them as supported V1 platforms.
   Shut each VM down before starting the next and leave autostart disabled.
4. Qualify the physical Fedora desktop only after saved edits and a recoverable
   update are confirmed. Complete action/lifecycle, resource, visible-response,
   accessibility and optional-companion gates with their actual measurement limits.
5. Make the release decision only when the required gates in the
   [V1 scope](v1-release-scope.md) are satisfied. Then publish exact accepted files,
   verify their downloaded signatures/hashes and update public installation guidance.

Current prior-build evidence is useful for comparison; it does not qualify a
newly compiled candidate automatically. The below-1% daemon CPU milestone and,
by the October 7 user decision, numerical input-to-visible-pixel qualification
remain post-V1 goals. Their targets are not claimed met. Any untested required
cases remain open rather than being inferred from successful interaction or a
provenance signature. See the [consolidated release review](v1-release-review.md)
for the current ledger, prepared delivery guidance and remaining decisions.
