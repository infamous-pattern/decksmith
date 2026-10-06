# Hosted candidate build and independent verification — 2026-10-06

The authorized [GitHub candidate run](https://github.com/infamous-pattern/decksmith/actions/runs/37506726695)
completed successfully. Its build and separate signing jobs passed. This is a
development candidate, not a V1 release: no release or tag was created and the
desktop installation, service and saved layout were not changed.

## Exact artifact and source

| Item | Verified value |
| --- | --- |
| Public source commit | `1d746db7594f3d4a92d2ea4ebbcaa80d22f70928` |
| Corresponding canonical commit | `06cdd3b33817516dd514b7d8eb3f5ffa0d2a7064` |
| Shared source tree | `d0656a865606154e5d22b023de1422f98dc7b7e3` |
| Bundle | `0.1.0-0143a5db609f` |
| Archive SHA-256 | `86648dca460fb3a822c7d97bc46de7334970611e769e976a2db6afdfaffec1df` |
| Build target | Fedora 44, Linux x86_64; Rust 1.97.1 |
| Manifest coverage | 384 files; clean source metadata and expected public commit |

The two repositories retain separate histories with matching source trees. This
report may subsequently appear in a newer documentation commit; the candidate's
source remains the exact public commit above, not the latest branch head.

## Results

- Hosted formatting, warnings-denied all-feature Clippy, Rust tests and cargo-deny
  advisory, ban, license and source policies passed. Rust tests passed 140 cases;
  three existing diagnostic cases were ignored.
- Installer/security/candidate tests ran 24 cases with one optional companion case
  skipped. Studio tests ran 167 cases with one platform-dependent case skipped.
  All remaining cases passed, followed by release compilation and staging.
- GitHub generated signed build provenance covering six subjects: the archive,
  standalone installer, package library, installation guide, checksum manifest
  and candidate metadata. The signing job's verification passed.
- After downloading the artifacts, independent local verification passed for
  every subject both through GitHub's attestation records and using the retained
  signing bundle. Each verification required the expected repository, workflow,
  main ref, exact public source commit and GitHub-hosted runner identity.
- Checksums passed for all four installation downloads. Trusted local archive
  validation checked all 384 manifest entries and rejected unlisted members.
  Standalone bootstrap files matched their archive copies byte for byte, and
  candidate metadata matched the verified bundle and expected source/platform.
- Negative verification checks rejected an altered installation guide and the
  correct guide paired with an incorrect expected source commit.

The first local verification attempt could not initialize the verifier because
the normal cache was outside the workspace's writable paths. Retrying with a
dedicated writable cache succeeded; no trust policy was relaxed. Verification
using the retained signing bundle still refreshed trusted roots over the network,
so this is not a fully disconnected verification test.

Downloaded artifacts, signing evidence, logs and machine-readable results are
retained privately. Workflow artifacts expire after 14 days; do not rely on the
workflow download alone for durable release evidence. The original physical video
and personal camera metadata were not published.

## Release boundary

This closes the hosted build/signing and independent-verification check for this
exact development candidate. It does not retroactively sign existing previews or
qualify this new archive on a physical device or VM. Attestation confirms artifact
identity and provenance, not the absence of defects or bit-for-bit reproducibility.

V1 still needs a frozen version/artifact, a tested verified-install path, the
remaining visible-response and action/failure coverage, and final installer,
migration, rollback, uninstall, VM, accessibility, lifecycle, resource and security
checks against that exact artifact. See the [release procedure](../release-authenticity.md)
and [V1 scope](../v1-release-scope.md).
