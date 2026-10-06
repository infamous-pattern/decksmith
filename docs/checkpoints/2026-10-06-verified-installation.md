# Verified installation path — 2026-10-06

The [verification helper](../../scripts/verify-install.py) now checks the accepted
repository, workflow, source ref, source commit and hosted-runner identity for all
six candidate files before executing downloaded installer code. It then requires
complete checksum coverage, archive integrity, matching clean-source/platform
metadata and identical standalone bootstrap files. The expected public commit is
required separately; it is never inferred from candidate metadata.

Inputs are bounded regular-file snapshots in a private temporary directory.
Missing files, links and special files are rejected. Original-path replacement
cannot alter the bytes subsequently verified or executed. Python installation
uses isolated imports from the verified snapshot. Verification is the default;
installation is explicit and never activates controls or enables startup.

## Tested artifact

- Hosted bundle: `0.1.0-0143a5db609f`.
- Public source: `1d746db7594f3d4a92d2ea4ebbcaa80d22f70928`.
- Archive SHA-256:
  `86648dca460fb3a822c7d97bc46de7334970611e769e976a2db6afdfaffec1df`.
- Archive manifest: 384 files; Fedora 44 x86_64, clean source.
- [Hosted build/signing evidence](2026-10-06-hosted-candidate-verification.md).

The helper is newly reviewed source, tested against the existing signed candidate;
it is not yet included in that archive. A later rebuilt V1 candidate must be
signed and tested separately. No release or tag was created.

## Validation

Seven security tests cover signature failure at each of the six subjects before
module import/execution, exact private-snapshot installation, Python import
isolation, input replacement during verification, missing/link/FIFO/oversized
files, incomplete/duplicate/incorrect checksums, malformed or mismatched candidate
metadata, dirty archive metadata, mismatched bootstrap files, mandatory full
source commits, verification-only behavior and retained signing-bundle handling.
All 31 installer/bootstrap/security/candidate tests passed locally.

The actual signed downloads passed the helper and installed successfully into an
isolated host directory. Controls were not activated. The same archive and helper
then passed the following checks in the registered Fedora 44 test VM:

| Check | Result |
| --- | --- |
| All six signatures, checksums, archive admission and staged install | Pass |
| Backup and restore of a custom layout/settings and imported artwork | Pass |
| Ordinary labels and emoji labels in the saved layout | Pass |
| Uninstall removes staged integration while retaining data | Pass |
| Reinstall restores staged integration and preserves saved data | Pass |
| Private-bus VirtualDeck startup, key/strip previews, navigation and save | Pass |
| Native editor smoke, key/dial selection and screenshot | Pass |

The guest used a temporary copy of the host's trusted installed GitHub CLI; no CLI
package, credentials or global integration was installed in the guest. Verification
used the retained signing bundle and fetched trusted roots normally; this was not
a disconnected-network test. The native screenshot covers the editor mounted by
the test harness, not a complete manual accessibility or main-window review.
The private native smoke disabled accessibility-bus activation and reported a
Mesa/Zink device-probe warning; the editor rendered its screenshot and both smoke
assertions passed. This does not establish GPU-driver or accessibility coverage.

Hashes of the normal guest's saved configuration, imported artwork, install record
and integration files, plus its current-release link, matched before and after.
Only the isolated stage and private test data were changed. The Fedora 44 VM was
shut down afterward; all four test VMs are off with autostart disabled. The physical
desktop still uses bundle `0.1.0-257113d71d41`; its service, saved layout and audio
routing were not changed by this work. Raw logs, machine-readable evidence and the
native screenshot remain private.

## Remaining boundary

This establishes a tested signed-candidate installation path and isolated
recovery coverage. It is not clean-system dependency installation, upgrade from a
published preview, rollback across changed engine/config formats, real user-service
activation, login/reboot acceptance or the final four-guest V1 matrix. Those checks,
physical lifecycle/latency and final security/resource/accessibility qualification
must use the frozen V1 artifact. The public preview URL installer remains unchanged.
