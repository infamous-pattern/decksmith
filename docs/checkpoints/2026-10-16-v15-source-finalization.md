# V1.5 source finalization — October 16, 2026

The focused V1.5 release is being prepared from the physically accepted paired
development build. Linked navigation is explicitly deferred beyond V1.5; general
plugins and system monitors remain V2 work. This checkpoint does not declare a
signed candidate accepted or a public release published.

The workspace application version and its four local Cargo lock entries are
1.5.0. Registry dependency versions are unchanged. The README labels this as
candidate preparation while retaining the supported V1 and experimental preview
downloads and installer pins. The candidate review and user guide distinguish
paired operation, shared controls, independent navigation, saved-profile recovery
and the supported/experimental platform boundaries.

## AI-server source checks

An immutable 656-file source snapshot, excluding private generated output, was
verified on the Fedora 44 AI server. Rust quality/security and Python checks ran
in parallel with private user directories and no physical HID, native audio,
guest, host service or schedule changes.

- Rust formatting, strict workspace/all-target/all-feature Clippy and locked
  tests passed: **342 passed, zero failed, six deliberately ignored**.
- The current cargo-deny advisory, banned-package, license and source checks passed.
- Installer/security/version checks: **114 run, one skipped**, no failures.
- Editor/model checks on a private bus/display: **235 run, one skipped**, no failures.
- Current Homebridge action/IPC/catalog/connection preference/removal checks:
  **44 passed** using the retained companion's vendor and manifest in a private
  fixture. This is mock/adapter coverage, not live accessory acceptance.

The initial broad Homebridge discovery lacked its separate websockets vendor and
historical reliability fixtures. Adding the vendor exposed the also-required
companion manifest. Both fixture failures remain retained. The corrected private
fixture passed the five current modules without changing app/adapter source.
The legacy reliability module depends on separate historical tests; it is not
claimed as rerun here. Earlier receipt-backed integration and physical Homebridge
acceptance remain separately identified in the October 15 checkpoint.

Subsequent finalization edits update documentation only; no runtime source changed
after the verified snapshot. Exact candidate checks must still run on the final
clean public source commit. The existing default/no-hardware configuration gap
is not converted into a passing result by these all-feature checks.

## Remaining steps

Review/commit and synchronize source to Gitea and the separate sanitized GitHub
history, build and verify the exact signed Fedora 44 candidate, complete candidate
admission and physical acceptance, then request release publication approval.
No stable release, installer pin or desktop installation changes in this step.

AI-server storage currently constrains full-copy VM jobs. The separate storage
review identified an existing large workspace disk as a migration candidate;
no disks, VM locations, backups, mount configuration or schedules were changed
by this source check. Storage migration requires its own verified copy/cutover.

Private raw logs, failed and corrected fixture outcomes and source hashes remain
in the enclosing workspace's `outputs/v15-finalization-20261016` evidence folder.
