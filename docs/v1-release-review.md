# V1 release review — October 7, 2026

**Maintainer accepted the documented coverage and limits on October 7, 2026.**
**V1 was published and its public downloads verified on October 7.** This review
consolidates the frozen candidate's evidence without replacing the original dated
checkpoints. The README now offers verified V1 installation; the historical URL
bootstrap remains pinned to preview.3.

## Release decision

The maintainer explicitly chose **Publish V1 with documented limits**, authorizing
an evidence commit, synchronized Gitea and sanitized GitHub publication, and
`v1.0.0` promotion of the unchanged signed candidate. The accepted scope includes
the finite resource observations, explicitly identified earlier action checks,
Fedora 44-only support, experimental Homebridge and the wider-window requirement
with enlarged text. Numeric visible-response qualification and below-1% idle CPU
remain post-V1 goals; neither target is claimed achieved.

This decision reconciles the documented testing coverage for V1. It does not
extend support, certify exhaustive hardware/action coverage or turn the resource
comparison into statistical equivalence. Fresh public downloads must pass the
unchanged authenticity verifier and retained hashes before the README points
users to V1. Preserve preview.3, the installed build and saved user data. Stop
delivery if authenticity, source identity or downloaded bytes fail verification.

## Artifact to promote

| Identity | Accepted value |
| --- | --- |
| Application / bundle | `1.0.0` / `1.0.0-c5e4fea53afe` |
| Public source commit | `68c42421ceaa65bb033993b56f0babdde2a14a6e` |
| Canonical source commit | `aa76487ec1dcfb8561dd747ecf2153fc2b1ddec3` |
| Shared source tree | `49d0ac5f41384a52c7f385e7f36dd3f8ee476df1` |
| Archive SHA-256 | `4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2` |
| Build and signing | [Candidate run 37510261430](https://github.com/infamous-pattern/decksmith/actions/runs/37510261430) |

All six candidate subjects were independently reverified on October 7 with the
retained signing bundle, exact source/workflow/repository/ref and hosted-runner
policy. Checksum coverage, archive admission, all 389 manifest files and standalone
bootstrap agreement passed. Verification did not install or activate controls.
The initial attempt could not initialize the verifier's trust cache; using a
writable isolated cache resolved it without weakening verification policy.

Later documentation commits are evidence updates, not candidate source. The
application, scripts, engine, configuration, packaging and workflows currently
match the canonical candidate source. Do not rebuild or repack accepted files,
change signed `candidate.json` from its original pending status, or suggest a
later documentation commit produced the signed archive.

## Gate ledger

| Gate | Current evidence | Remaining work or limit |
| --- | --- | --- |
| Source quality, assets, dependencies and authenticity | [Frozen-candidate checks](checkpoints/2026-10-06-v1-signed-candidate.md); six signatures reverified | Repeat admission after downloading final public assets; signatures are not a defect-free-code certificate |
| Clean Fedora installation, first run and dependencies | [Disposable clean Workstation test](checkpoints/2026-10-07-v1-clean-fedora-install.md) | V1 public download instructions need the final release URL and post-publication verification |
| Upgrade, data protection, rollback, uninstall/reinstall, startup | [Exact-candidate guest matrix and real login/reboot](checkpoints/2026-10-06-v1-signed-candidate.md) | Preserve the tested compatible-format rollback limits |
| Physical recovery and startup | [User-operated lifecycle checks](checkpoints/2026-10-07-v1-physical-lifecycle.md) | Finite reference-host evidence, not every USB controller or firmware combination |
| Keyboard, spoken controls and visual acceptance | [Human accessibility checks](checkpoints/2026-10-07-v1-accessibility.md) | No numeric contrast certification or exhaustive monitor/scaling matrix |
| Draft/error handling and layout | Supplemental exact-runtime fixtures are recorded in the [editor checkpoint](checkpoints/2026-10-07-v1-editor-release-review.md) | Mocked control writes do not establish backend action behavior |
| Experimental Homebridge | [70 fault/recovery cases plus live acceptance](checkpoints/2026-10-07-v1-homebridge-faults.md) | Separate companion binary is not covered by the core archive's signature; general plugins remain deferred |
| Sustained physical resources | [300-second quiet](checkpoints/2026-10-07-v1-quiet-resources.md), [600-second interactive](checkpoints/2026-10-07-v1-physical-resources.md), [CPU investigation](checkpoints/2026-10-07-v1-cpu-investigation.md) and [two completed quiet comparison pairs](checkpoints/2026-10-07-v1-matched-resources.md) | Pairs span a user-requested pause; finite evidence supports resource review, not statistical equivalence; one loaded observation used a Homebridge override rather than four audio targets |
| Internal dispatch | 204 physical audio dial actions; p95 21.445 ms at report-return to backend entry | Does not qualify every route, brightness timing, upstream USB delay, backend completion or pixels |
| Visible feedback | User reports correct responsive displays; numerical 150/100 ms qualification deferred beyond V1 by October 7 user decision | Neither numeric target is claimed met; synchronized exact-artifact measurement remains follow-up work |
| Action coverage | [Packaged action safety and physical push-to-talk](checkpoints/2026-10-07-v1-action-safety.md), existing exact-candidate physical use and explicitly identified historical adapter acceptance | Human overlap acceptance and directly observed sustained microphone hold passed; this is not an exhaustive new physical execution of every action |
| Distribution matrix | Fedora 44, Fedora 45 Beta, Ubuntu 26.04 and Debian 13 exercised | Fedora 44 alone is supported; Ubuntu native GUI and Debian-native runtime remain later compatibility work |
| Release decision and delivery | Maintainer accepted this coverage and its limits; [V1](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0) published, fresh downloads verified; [notes](v1-release-notes.md) and [installation guide](v1-verified-installation.md) | Retained preview bootstrap remains checksum-only; follow the verified V1 guide |

## Performance assessment for the decision

Idle daemon RSS was 17.934–17.949 MiB; loaded RSS was 27.402–27.773 MiB. Handles
remained bounded at 16–18, with no automatic restarts or reported freezes in the
sampled runs. These finite observations meet the daemon's 100 MB idle memory
target, while not excluding a long-term leak. Configurator fixture memory is a
separate observation, not full-service memory or a long-duration idle benchmark.

Quiet daemon/core CPU was 2.46%/4.42% of one logical core, versus the reference
2.11%/3.75%. Loaded CPU was 3.52%/6.53% with more dial actions than the earlier
sample. Profiling identified existing display-buffer comparisons and process-name
scans; a constrained paired VM comparison did not reproduce the larger desktop
increase. That investigation is complete as a diagnostic exercise, but the cause
of the unmatched physical increase is not established. It must not be called a
regression-free result. A matched physical comparison would require a coordinated,
recoverable switch between retained builds and identical workload/settings.
The [prepared comparison protocol](v1-matched-resource-protocol.md) identifies the
two retained builds, backup/restoration requirements, A/B/B/A phases and stop
conditions. The user authorized and prepared for this comparison on October 7.
The first two attempts stopped on transient playback; a user-requested third
attempt stopped on external capture, with GNOME Settings input monitors observed
immediately afterward. After the user closed Settings, the fourth attempt
completed one quiet pair: previous/candidate daemon CPU was 1.457%/1.533%, and
core-service CPU was 2.583%/2.766%. Memory/handles were bounded, with no automatic
restarts or unplanned actions. The user paused during the second candidate phase;
its partial data does not count as a completed repeat. The signed candidate and
saved state were independently verified restored. See the
[comparison checkpoint](checkpoints/2026-10-07-v1-matched-resources.md). This is a
single ordered pair at that point. On continuation, the remaining candidate/
previous pair completed at 2.010%/2.003% daemon and 3.717%/3.702% core CPU, with
stable memory/handles and no automatic restarts or actions. The signed candidate
and saved state were independently verified restored; all test VMs remained off.

Across the two sessions, descriptive previous/candidate means were 1.730%/1.772%
daemon and 3.143%/3.241% core CPU. The candidate differences were 0.042 and 0.098
percentage points of one core, smaller than the difference between sessions.
The earlier large different-day increase was not reproduced in these pairs.
This completes the requested measurements, with the pause and finite-sample
limits retained; it is not a continuous A/B/B/A experiment, statistical equivalence
or a universal regression-free pass. Resource/release acceptance is separate.

Below-1% daemon CPU remains post-V1 by the existing decision. On October 7 the user
also approved deferring numerical visible-response qualification beyond V1 and
documenting the limit. Functional display acceptance remains required; the
split-session CPU comparison is not a statistical regression-free guarantee. Do not request
more ordinary phone footage as if its unsynchronized timestamps could establish
an electrical-input boundary. Future numerical qualification needs a synchronized
marker or well-anchored conservative movement bound on the exact artifact.

## Promotion procedure after acceptance

1. Resolve or explicitly reconcile the open gates above and record the decision.
   Commit evidence, sanitize publication and verify identical Gitea/GitHub source
   trees through the established export process. Keep private raw evidence local.
2. Create the approved public release/tag against the **accepted public source**,
   not a later evidence commit or the private-history commit. Upload the original
   six files plus retained `attestation.json` without repacking. The six subjects
   are `decksmith-linux-x86_64.tar.gz`, `decksmith-install.py`, `package_io.py`,
   `INSTALL.md`, `SHA256SUMS` and `candidate.json`.
3. Use the reviewed draft notes as release prose after removing their draft banner.
   Clearly distinguish subsequent guidance from signed `INSTALL.md`, which still
   includes the historical preview shortcut. Core promotion does not publish the
   optional companion as an authenticated core download.
4. Download all assets from the new public release into a fresh directory. Run the
   unchanged verifier against the accepted public source and compare every file
   with retained accepted hashes. Stop promotion if anything fails; do not replace
   the signed archive with a rebuild or fall back to checksums alone.
5. Update the public README to the verified V1 path only after that result.
   Preserve preview.3 and the existing valid walkthrough URL. The existing
   `scripts/install.sh` remains a checksum-only preview path; changing its version
   variable is not an authenticity upgrade.

The recorded maintainer decision authorizes source synchronization and release
publication using this procedure. Desktop updates, service restarts, power
operations and replacement of saved user data are not part of publication.

## Completed delivery — October 7, 2026

- [Evidence quality run 37671119052](https://github.com/infamous-pattern/decksmith/actions/runs/37671119052)
  passed Rust formatting, Clippy, tests, installer tests and dependency audit.
  This documentation run did not rebuild the accepted candidate.
- [Public V1 release](https://github.com/infamous-pattern/decksmith/releases/tag/v1.0.0)
  is a stable release at the accepted public source `68c42421ceaa65bb033993b56f0babdde2a14a6e`.
  Gitea's corresponding source tag points to canonical `aa76487ec1dcfb8561dd747ecf2153fc2b1ddec3`;
  both source trees are `49d0ac5f41384a52c7f385e7f36dd3f8ee476df1`.
- The original six subjects and retained signing bundle were uploaded unchanged.
  Fresh downloads through public HTTPS release URLs matched all seven retained
  SHA-256 hashes. The unchanged verifier authenticated all six signed subjects
  against the accepted source/workflow/ref and admitted all 389 manifest files.
  Verification did not install files or activate controls.
- The archive remains SHA-256
  `4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2`.
  Signed `candidate.json` still carries its original pending/staging text; this
  separate release decision records promotion without editing signed metadata.
- README promotion followed successful public-byte verification. Preview.3 and
  the existing walkthrough remain available. Supplemental installation guidance
  distinguishes verified V1 from the historical checksum-only preview bootstrap.
  No desktop update, service restart or saved-layout change was performed.

Private verification logs and hashes remain in ignored local release-review
storage. The known support, coverage and performance limits above remain in force.
