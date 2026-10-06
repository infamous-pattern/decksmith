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
also passed. Resource/lifecycle/accessibility and other final release gates remain
open. See the [exact-candidate checkpoint](checkpoints/2026-10-06-v1-signed-candidate.md)
for the archive digest, test scope and remaining work.

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
newly compiled candidate automatically. The below-1% daemon CPU milestone remains
post-V1 by the recorded scope decision. Precise input-to-visible-pixel timing and
any untested required cases must remain open rather than being inferred from
successful interaction or a provenance signature.
