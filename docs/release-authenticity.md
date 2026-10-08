# Release authenticity and candidate preparation

The manually started **Verified release candidate** workflow builds a new Fedora
44 x86_64 bundle from the sanitized GitHub `main` source. It runs Rust quality and
dependency checks, installer/security tests and Studio model tests before staging
the downloads. It does not create a tag, publish a release or install on anyone's
desktop. A successful build is a candidate that still needs release qualification.

The build uses Rust 1.97.1 and a Fedora 44 container pinned by image digest. Its
dependency packages come from Fedora's repositories at run time; this is not a
claim of bit-for-bit reproducibility. Actions are pinned by full commit SHA and
checkout credentials are not retained. Only the separate attestation job receives
OIDC and attestation-write permissions; it does not check out or execute project
source. Both jobs run on GitHub-hosted runners. The workflow accepts no source,
archive or shell-command input, and only runs for the expected repository's main
branch.

Candidate staging verifies every archive member against the bundle manifest,
requires a clean source tree and the expected full source commit, and checks that
the standalone installer, package library and installation guide match their
verified archive copies. It preserves the archive's original bytes and refuses an
existing output directory. The generated checksum manifest covers all four
installation downloads. Local staging alone does not create a signature.

The signing job uses GitHub's `actions/attest` to generate Sigstore build provenance
for all staged files, then verifies their repository, workflow, source commit,
source ref and GitHub-hosted runner identity. This follows
[GitHub's artifact attestation guide](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).
Attestations and verification bind the workflow and source identity to each
download's digest. They do not prove the source is defect-free, replace security
review or certify the device's behavior.

## Maintainer steps

1. Commit reviewed source, check for private information, and synchronize Gitea
   with the sanitized GitHub source tree through the established export process.
2. Manually start **Verified release candidate** on GitHub's `main` branch. Retain
   its public commit SHA, workflow run URL, logs, candidate downloads and signing
   bundle. Do not substitute a local archive for the CI-built archive.
3. Independently verify the downloaded candidate using the commands below and
   record its exact archive SHA-256 and source-tree correspondence with Gitea.
4. Run the final VM, physical-device, installer/migration/rollback/uninstall,
   accessibility, resource, lifecycle and security gates on this exact candidate.
   Any rebuilt or modified artifact needs new verification and qualification.
5. Publish only after the release decision. Upload the exact accepted files and
   signing bundle without repacking them. Repeat verification after downloading
   them from the public release. Workflow artifacts expire after 14 days, so retain
   accepted evidence durably.

No V1 artifact is signed merely because this workflow is present. Existing preview
and local development bundles retain their original provenance limits.

The first hosted development candidate passed both workflow verification and
independent verification of all six subjects, including checks against a retained
signing bundle. Its exact artifact, source and outstanding qualification are
recorded in the [2026-10-06 checkpoint](checkpoints/2026-10-06-hosted-candidate-verification.md).

## Verify downloads before executing them

Install a recent GitHub CLI through a trusted package source. Obtain the expected
full **public** source commit from the accepted release evidence. Its SHA differs
from Gitea's private-history commit even when their source trees match.

From the directory containing the downloaded files, set the expected source SHA:

```sh
expected_source=FULL_PUBLIC_COMMIT_SHA_FROM_RELEASE_EVIDENCE
for file in decksmith-linux-x86_64.tar.gz decksmith-install.py package_io.py INSTALL.md SHA256SUMS candidate.json; do
    gh attestation verify "$file" --repo infamous-pattern/decksmith \
        --signer-workflow infamous-pattern/decksmith/.github/workflows/release-candidate.yml \
        --source-ref refs/heads/main --source-digest "$expected_source" \
        --deny-self-hosted-runners || exit 1
done
sha256sum -c SHA256SUMS
```

Proceed with the installation guide only after verification succeeds. Verification
may require GitHub CLI authentication and network access; never treat an unavailable
attestation, an old CLI or a failed check as a successful verification. To verify
against a separately retained Sigstore signing bundle, add `--bundle PATH` to
each GitHub CLI verification command. Follow
[GitHub's offline verification guidance](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/verify-attestations-offline)
when offline trust-root handling is needed.

## Verified installation helper

From a reviewed, trusted source checkout, the standard-library-only
[verification helper](../scripts/verify-install.py) combines the checks above
with candidate admission and an optional per-user installation:

```sh
python3 -I scripts/verify-install.py /path/to/downloads \
    --source-sha FULL_ACCEPTED_PUBLIC_COMMIT_SHA
```

Its default is verification only. Add `--install` to install after verification,
or `--install --stage-root /path/to/isolated-test` to exercise installation without
changing normal integration or managing host services. It never passes
`--activate` or enables login startup. Runtime dependencies must already be
installed using the normal trusted package manager.

Obtain the expected commit from independently accepted release evidence, not from
the downloaded candidate metadata. The helper itself is part of the initial
trusted checkout; it does not authenticate its own origin. It is not a replacement
for trusting or inspecting a downloaded bootstrap. Existing preview downloads
lack these attestations and cannot be installed through this method.

The helper snapshots bounded regular files into a private temporary directory,
rejecting links, special files and missing subjects. It verifies all six signed
subjects before importing any downloaded module, then checks complete checksums,
archive integrity, clean source/platform metadata and bootstrap byte identity.
Installation runs only from the verified snapshot with isolated Python imports.
Replacing the original download directory during verification cannot change the
executed bytes. Any failed check stops before installation.

Add `--bundle /path/to/attestation.json` for a retained signing bundle; trusted
roots may still require network access. A failed or unavailable verifier is an
error, never a reason to fall back to checksum-only installation. See the
[verified installation checkpoint](checkpoints/2026-10-06-verified-installation.md)
for the exact tested development candidate and remaining release gates.

The existing preview URL bootstrap checks download integrity, but does not perform
these authenticity checks. It remains pinned to preview.3. V1 publication must
include a tested verified-install path and accurate instructions; this foundation
does not silently promote the preview installer to V1.


## Experimental Debian candidate

Debian 13 builds use a separate pinned workflow and archive identity. See the
[Debian native-build checkpoint](debian-native-build.md#separate-signed-debian-candidate).
The verification helper requires explicit `--target debian13`; omitting it keeps
the existing Fedora verification policy. Candidate signing alone does not grant
supported-platform status or authorize release publication.
