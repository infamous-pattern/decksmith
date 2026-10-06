# Release authenticity foundation — 2026-10-06

A manually started GitHub candidate workflow now prepares a Fedora 44 build and
signed build-provenance verification. It accepts only the sanitized public main
branch, pins actions and its Fedora base image, and separates read-only build
permissions from the signing job. It does not publish a release, make a tag or
change the desktop installation. See [release authenticity](../release-authenticity.md)
for promotion and independent verification instructions.

Candidate admission requires manifest integrity, the expected clean source commit,
Fedora 44 x86_64 metadata and bootstrap files identical to their archive copies.
The staged archive is the exact immutable byte snapshot that was verified. All
four installation downloads have checksum entries. Existing destinations are
rejected rather than overwritten.

Validation completed:

- Six admission tests cover complete checksum staging, dirty/source/platform
  mismatch, altered archive members, replaced/missing/symlinked bootstrap files,
  preservation of existing output and input replacement during verification.
- Actionlint 1.7.12 accepted the workflow, and whitespace checks passed.
- The workflow's build commands passed in an isolated Fedora 44 container with
  Rust 1.97.1: formatting, warnings-denied all-feature Clippy, 140 passing Rust
  tests with three existing diagnostics ignored, and cargo-deny 0.20.2's advisory,
  ban, license and source policies.
- Installer/security/candidate tests ran 24 cases, with the optional staged
  companion test skipped. Studio tests ran 167 cases, with one platform-dependent
  case skipped. All remaining cases passed.
- A clean local source fixture produced a bundle with 381 verified manifest files.
  Independent archive and standalone-download hashes matched after staging.

The first local container attempt lacked rustfmt/clippy components from the copied
Rust image; those were added to match the hosted toolchain step. The next attempt
exposed an existing test's accidental dependency on the host audio command. Its
fixture now supplies default-device metadata explicitly and verifies fresh values
and metadata across requests. No production audio-reader behavior changed.

The local fixture bundle is `0.1.0-ad69e43652b2`, SHA-256
`62b6376763bf5aef818dc2087477eac74a0a72def5b0749be7394cfc21f91445`.
It uses a private, clean test-fixture commit; it is not the canonical/public source
commit, not signed provenance and not a frozen V1 candidate. The disposable test
container stopped after use. No desktop installation, service, saved layout or
physical device was changed by these checks.

The hosted signing/verification jobs have not yet run. Synchronizing reviewed
source and running that workflow require publication authorization. The new
method does not retrospectively authenticate existing previews. Final-artifact
qualification and a tested verified-install path remain required before V1.
