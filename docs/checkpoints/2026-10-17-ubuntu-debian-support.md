# Ubuntu support and parallel Debian validation — October 17, 2026

## Debian GNOME 48 correction — unchanged signed candidate acceptance passed

Following explicit maintainer approval, the source extension declaration now
includes Shell 48 alongside 50/51. This matches the completed GNOME 48.7 VM trial;
no JavaScript, permissions, service behavior or version-validation policy changes.
The corrected source and integration findings were synchronized at canonical
`e34beb7cd46a1baf2fd6ff1cae454f23ebdc075f` and public
`ea29fc62a7520dd21c9df4ad0a6c11023b84e167`, identical tree
`95fe6bddcff8e961eb5811ebafbedf150fb7502e`.

- Local installer/packaging tests: 122 passed.
- Quality run: https://github.com/infamous-pattern/decksmith/actions/runs/39068287003.
- Debian build/signing: https://github.com/infamous-pattern/decksmith/actions/runs/39068293997.
- Both workflows passed on their first attempt. Corrected signed candidate:
  **1.5.2-1272d3a4d717**, archive SHA-256
  `dbbec9e918c0bfaa0b952c52428d9780a27a0e688245de57221cf844e675db2d`.

All six signed download subjects passed exact repository/workflow/source/ref and
hosted-runner provenance policy, checksum and archive verification before guest
execution. The archive declares Shell 48/50/51; both extension JavaScript hashes
match the earlier tested code.

The unchanged signed package passed normal preview upgrade, saved-file/startup
preservation, normal no-device service start/stop/restart, native waiting editor,
previous-build rollback and reinstall, paired virtual editor saves/restart, and
Homebridge assignment replacement/cross-Deck copy in both directions with undo
and redo. Native full-size and compact captures were inspected. The accessory
checks use fixtures; no real accessory or audio action was executed.

In the actual Debian GNOME 48.7 session the signed extension loaded as ENABLED
(state 1) with its original signed metadata. No guest metadata edits or version
validation bypass occurred. All seven GNOME integration checks listed below
passed, including independent foreground/default pages, indicator start/stop and
restart, editor open/Quit, source bus release/recovery, and normal no-device
source-loss detection. Final extension state was ENABLED with no extension errors.

Each suite retained a verified offline disk recovery copy and restored original
guest files, permissions, preferences and disabled/inactive startup state. Debian
shut down normally after both suites, with no cleanup recovery required. Private
evidence: `outputs/debian-gnome48-signed-20261017/completion.json`, candidate
admission, native VM outcomes and `gnome/outcomes.json`. No release was published,
Debian support was not promoted, and the Fedora desktop was unchanged. Physical
Ubuntu testing remains deferred; physical/audio/lifecycle/resource and remaining
distribution qualification gates are still open.

## Earlier GNOME integration trial — basis for the signed correction

The unchanged signed Ubuntu candidate `1.5.2-d4a17d906de7` passed the real GNOME
50.1 session checks on the retained Ubuntu VM. The Debian candidate
`1.5.2-dcc98e22be50` is rejected by GNOME 48.7 before extension enable because its
metadata declares only Shell 50/51 (`OUT_OF_DATE`, state 4). A private, guest-only
trial adding `48` to that declaration enabled the extension (state 1) without
changing its JavaScript or icon. That trial passed the same integration checks;
the shipped signed Debian package still has the original compatibility gap.

Both sessions passed:

- Actual indicator menu opening, background-control start/stop and restart, and
  opening the installed editor followed by Quit Decksmith stopping both editor
  and controls. The test waits for a visible editor before requesting Quit.
- Real GNOME foreground identities from three private GTK applications selecting
  independent Plus/MK.2 virtual pages, falling back to each layout's default for
  an unassigned application, and returning to an already-open assigned app.
- Extension disable/re-enable releasing and reacquiring its foreground bus name,
  with the virtual Decks retaining their active pages through the source cycle.
- A separate normal signed runtime, with no attached hardware, showing Waiting
  for device and reporting foreground source unavailable after extension disable,
  then recovering the actual desktop identity after re-enable. Virtual mode
  deliberately does not run the normal source-liveness monitor; that distinction
  is retained rather than weakening the normal-runtime assertion.

The Ubuntu fixture starts with user extensions globally disabled. This preference
was temporarily enabled after fresh login for the checks and restored afterward.
Shell version validation remained enabled. Both extension error lists were empty.
Native menu captures were inspected.

Earlier failed attempts are preserved. They exposed test setup/readiness mistakes:
extension API destination/path discovery, unique guest work paths, existing-window
activation, global extension enablement, asynchronous extension state transitions,
editor startup readiness, and the virtual/normal liveness distinction. A passing
corrected attempt does not erase those records or constitute a product fix.

Original guest files, permissions, GNOME preferences and disabled/inactive service
startup state were restored and verified on both guests. Debian shut down normally.
Ubuntu's first cleanup poweroff was rejected by an OS block inhibitor and its
graceful cleanup deadline expired; that original failed cleanup record remains.
Inspection then found no shutdown block inhibitor or Decksmith process. A second
identity-checked normal poweroff, under the guest lock, succeeded. Both VMs are now
off; no forced poweroff, real audio, USB passthrough or desktop change occurred.

Private evidence: `outputs/gnome-qualification-lifecycle-20261017/completion.json`,
the original `outcomes.json`, per-target evidence and the separate Ubuntu
`cleanup-recovery.json`. This earlier trial left the signed GNOME 48 correction
and unchanged-package retest outstanding; those results are now recorded above. Physical
Ubuntu acceptance remains deferred by the maintainer; broader lifecycle, audio,
resource, fresh-public-delivery and support-promotion gates remain open. No source
compatibility change, release publication or supported-platform promotion occurred.

## Earlier editor-correction candidates — native VM acceptance passed

The Homebridge assignment/cross-device clipboard corrections and accepted Fedora
desktop results were published with identical source tree `b8d048b85708316d45c05762d8129b238093378b`
at canonical `eb643c89b0c923f18a999e340ffeb4a57c160116` and public
`358001cbecf6643361c54b76c8e208a0db87b78b`. New OS candidates include the editor
corrections; the earlier installer-fix packages do not.

- Ubuntu 26.04: signed candidate **1.5.2-d4a17d906de7**, build/signing run
  https://github.com/infamous-pattern/decksmith/actions/runs/39064833468.
- Debian 13: signed candidate **1.5.2-dcc98e22be50**, build/signing run
  https://github.com/infamous-pattern/decksmith/actions/runs/39064835237.

All six download subjects for each target passed exact repository/workflow/source,
main-ref, hosted-runner, checksum and archive verification before execution. The
retained VMs ran in parallel under guest locks and the shared schedule gate, with
fresh byte-verified offline disk recovery copies and checked image integrity.

Both unchanged signed packages passed:

- Normal-user preview upgrade, saved-file/startup preservation, native no-device
  editor launch and normal service start/stop/restart.
- Previous-build rollback and same-candidate reinstall with saved data preserved.
- Native paired virtual editor: separate drafts/saves, named devices, selected
  device APIs, preview/brightness, owner-scoped history and restart persistence.
- Replacement of page/media keys with Homebridge fixture assignments, visible
  Assign to key and Save and Apply, native saves on both models, copying in both
  directions, undo/redo, independent saved layouts and compact-window capture.

These Homebridge tests use fixture accessories and virtual Decks, with native
save/readback APIs; they do not execute a real accessory or audio command. All
original user files and permissions and the disabled/inactive startup state were
restored, and both guests shut down gracefully. Native captures were inspected.
Private evidence: `outputs/ubuntu-debian-editor-20261017/evidence/`.

### Quality timing follow-up — retained, not a production correction

The notes commit's first Quality attempt failed at
`sleep_lifecycle::tests::resume_clock_gap_invalidates_awake_until_uncached_revalidation`
with `!pending.available && pending.blocked`. That assertion requires observing
the transient pending state immediately after requesting asynchronous revalidation;
a completed fresh-state reply can race the snapshot. The relevant Rust source is
unchanged from the passing Fedora candidate. No production code or dependency
policy was changed to obtain a passing result.

The unchanged regression passed 300 consecutive isolated repeats on AI_server.
The same-source Quality run's second attempt passed Rust checks, installer tests
and the security audit:
https://github.com/infamous-pattern/decksmith/actions/runs/39064814880.
The initial failure is retained separately. A deterministic fixture boundary is
still a test-stability follow-up; a green retry alone is not a claim that the
intermittent assertion has been repaired. VM execution waited for this passing
Quality gate and verified target packages.

The earlier clean-user/dependency-repair/complete-store recovery checks remain
valid evidence for their older signed packages; this run adds exact new-package
upgrade/editor/copy coverage. Pristine-OS/public installer delivery, GNOME
integration, human and physical acceptance remained open at this earlier
package-only checkpoint. Debian's GNOME 48 review was then unresolved, and the maintainer still
defers Ubuntu's local physical Deck session. No release or support promotion was
performed; the Fedora desktop remains on its previously accepted candidate.

## Follow-up Quality run 98 — corrected and verified

The documentation commit's Quality run
https://github.com/infamous-pattern/decksmith/actions/runs/39058290065 failed in
`shared_bridge_never_falls_back_and_rechecks_after_discovery`: the mock companion
unwrapped a connection reset while the stale client cancelled before granting
permission. Closing a Unix socket with unread readiness bytes can return reset
instead of EOF. A local socket-pair check reproduced that transport behavior;
2,000 repeats of the unmodified test did not reproduce the intermittent CI race.

The prepared test-only correction accepts an empty reset/EOF or explicit abort
only for stale cancellation, and a closed readiness write only for that same
case. Any apply decision still fails the stale assertion; the client must still
return `plugin_cancelled`. Successful apply and other protocol cases remain
strict. No production transport or accessory behavior changes.

The corrected test passed 1,000 consecutive runs on the AI server. All 346 Rust
tests passed (six existing ignored), with formatting and strict all-feature,
all-target Clippy passing. The maintainer approved publication of the correction
and notes to both repositories. Canonical `e2399618d09e0e53371804c4c015d438b99a29a2`
and public `9f857132b3cae3c4b46c3a796a3b2ba3af7e97ca` have identical tree
`db0d418807f90fd406d63a342746cac76610cea6`. Replacement Quality run
https://github.com/infamous-pattern/decksmith/actions/runs/39061840299 passed Rust
checks, installer tests and the unchanged dependency security audit. Historical
failed run 98 remains available as evidence. The latest Ubuntu and Debian signed
package workflows already passed (39057610984 and 39057610950); their existing
native package qualification is independent of this newer test-fixture failure.

## Additional clean-user dependency and paired-store recovery checks

The unchanged admitted Ubuntu and Debian V1.5.2 candidates were exercised in
parallel on the updated retained server VMs. Each run takes and byte-verifies a
powered-off disk recovery copy, holds the existing per-user install/configuration
aside, and restores its exact contents, permissions, package version/mark and
startup state afterward. This is a **clean per-user installation**, not a pristine
OS test or an end-to-end test of a future public installer pin.

Both initial runs passed real removal of only `python3-gi-cairo` after checking
the package-manager removal plan, detection/refusal with dependencies missing,
repair through the candidate's shared dependency helper in a real terminal, and
normal-user installation with startup disabled. APT did not need an additional
confirmation for this single-package repair; no confirmation is inferred.

The private native paired editor creates independent named profiles, pages,
brightness and history. The candidate's own backup/restore commands then recover
the saved registry, profile generations, session state and imported artwork after
controlled corruption, followed by same-candidate reinstall and normal service
startup with no attached hardware.

The first Debian recovery check stopped because its assertion required the
`waiting_for_device` fallback used before any saved catalogue can be read. The
actual reply already exposed both saved profiles, disconnected, with the selected
Plus page and brightness. Ubuntu initially passed during that fallback window.
Both original guest baselines were restored and shut down cleanly. These first
outcomes are retained separately. The corrected qualification assertion waits
for both exact saved identities and checks their models, layouts, saved pages,
brightness, owner-scoped history and disconnected status rather than accepting a
transient fallback. Confirmation runs are recorded separately.

Both confirmation runs **passed** with the unchanged signed packages. The Plus
profile recovered page 0 at 51% brightness; MK.2 recovered page 1 at 50%, with
independent custom captions and readable owner-scoped history. All backed-up
configuration and imported-icon bytes matched after restore, reinstall and service
stop. Original per-user files and permissions, dependency package version/mark,
and disabled/inactive startup state were restored; both guests shut down normally.
Private evidence is under `outputs/ubuntu-support-20261017/evidence/`:
`clean-recovery-confirmed-ubuntu2604/` and `clean-recovery-confirmed-debian13/`.
The initial outcomes are retained alongside them. This did not qualify downgrade
of a V1.5 store into an older engine, real audio, physical USB or desktop lifecycle.
The maintainer deferred the local Ubuntu hardware stage; its VM remains off.

## Scope and decision

The maintainer selected Ubuntu 26.04 LTS x86_64 GNOME/Wayland as the next supported
target, with independent Debian 13 GNOME/Wayland validation on the AI server.
This is work in progress, not a supported-release announcement. Fedora V1.5.1,
existing preview assets, the public installer pins and the daily desktop stay
unchanged. No publication or support promotion is implied by passing a build.

## Initial candidates — stopped before signing

Use the existing clean public source `f2491dfe7a6e346c5228a920af3f7031041a20e8`,
matching canonical `c1cf981c19dcb4124630fb1f851f27c7b95d3125` at tree
`2f4d54613240bbfafd349626ab6b3092826e40ae`. Candidate workflows remain named
Experimental until support acceptance is complete.

- Ubuntu build/signing: https://github.com/infamous-pattern/decksmith/actions/runs/39054102044
- Debian build/signing: https://github.com/infamous-pattern/decksmith/actions/runs/39054103892

Build execution and outcomes must be recorded separately from VM and physical
acceptance. Downloaded subjects must pass the existing exact workflow/source/ref,
hosted-runner, checksum, target and archive policy before package code executes.

## Qualification ledger

| Gate | Ubuntu 26.04 | Debian 13 |
| --- | --- | --- |
| Initial V1.5.1 candidate | Rust compile/tests passed; stopped at new maintenance advisories | Same stop |
| Signed package admission and relocation | Replacement V1.5.2 package verified, including installer fix | Same |
| Offline verified VM backup, OS update and confirmed reboot | Passed; retained-preview smoke passed; guest off | Passed; retained-preview smoke passed; guest off |
| Normal-user upgrade from retained V1.0 preview; saved files and startup state | Passed with unchanged replacement signed package; original permissions retained | Same |
| Service start/stop/restart with no hardware | Passed with unchanged replacement signed package | Same |
| Native paired virtual editor; owner-specific saves, brightness, recovery and restart | Passed with unchanged replacement signed package | Same |
| Clean per-user install and controlled missing-package repair on current package | Passed with unchanged signed candidate and actual terminal dependency helper | Same |
| Complete current paired-store/history/artwork backup, restore and reinstall | Passed; both saved identities/pages/brightness and exact bytes verified | Same |
| Pristine-OS and future public URL installer delivery | Pending | Pending |
| Whole-store upgrade/downgrade recovery across preview and V1.5 schema | Pending | Pending |
| GNOME indicator and foreground-app switching | Passed on actual GNOME 50.1 with signed candidate, virtual Decks and separate normal no-device source-liveness checks | Corrected signed candidate 1.5.2-1272d3a4d717 passed on actual GNOME 48.7, including normal no-device source liveness; no guest metadata edits |
| Human editor, file-picker, keyboard and screen-reader checks | Pending | Pending |
| Physical Plus/MK.2 USB, shared audio/media, gauges and microphone policy | Pending | Pending |
| Physical lock, suspend, hotplug, quit/relaunch and login startup | Pending | Pending |
| Idle/loaded CPU, memory and sustained use | Pending | Pending |
| Verified public delivery and explicit supported-platform decision | Pending | Not requested; validation target |

Earlier October 8/12 previews have useful install, native-editor and recovery
evidence, but it covers their older packages. The retained Ubuntu and Debian
VMs have no physical USB passthrough. Virtual tests do not certify real audio,
physical input/display behavior, bare-metal suspend or another desktop.

## Execution safeguards

The AI server is the primary OS test host. Each guest uses its existing pinned
SSH identity and per-guest lock. Refuse an already running guest or conflicting
scheduled test. Back up and byte-verify the powered-off disk before updates.
Preserve saved configuration and the previous managed release before package
installation. Restore the guest user baseline and stop owned guests gracefully;
never force power-off after a failed cleanup. Retain failed and corrected outcomes
separately. No desktop service interruption or device handoff is part of the
unattended VM stage.

Private evidence root: `outputs/ubuntu-support-20261017/` in the maintainer
workspace. Server maintenance launch: `20261017-ubuntu-debian-support-m4vxu6pt`.

## Dependency prerequisite discovered

Both initial workflows stopped before packaging/signing under the unchanged
`unmaintained = "all"` policy. RustSec published maintenance notices on October 16
for `zvariant`, `zbus_names` and `zvariant_derive` (RUSTSEC-2026-0370/0371/0372).
The notices say these crates were merged into zbus 6; no patched standalone
versions are listed. These informational maintenance notices do not, by themselves,
establish an exploitable vulnerability in the published build.

A separately reviewed source copy was migrated to exact zbus 6.0.0. Its unchanged
advisory/license/source policy passed on the AI server. Registration uses the
current async connection builder; synchronous worker boundaries explicitly await
futures through the existing executor. The selected-device Control1 adapter is
asynchronous so it yields while calling the profile endpoint on the same bus.
The first private paired-editor run caught a stalled synchronous adapter; failed
evidence is retained, and the corrected async adapter passed the retry.

The reviewed development version is now **1.5.2** to distinguish changed runtime
source from the already published V1.5.1 packages. This is not a release or
installer promotion. No public package or daily-desktop service was changed.

Validation of the isolated migration:

- Formatting and strict all-target/all-feature Clippy passed.
- 346 Rust tests passed; six existing diagnostics/fixtures remain ignored.
- AI server: 115 installer/script tests and 238 editor tests ran, with one
  environment-specific skip in each Python suite.
- Dependency advisories, bans, licenses and sources passed without new exceptions.
- Explicit private-bus lock proxy recovery passed.
- Six private native-media cases passed: success, unsupported action, owner
  replacement, uncertain completion, application-family selector and automatic selector.
- The new bounded selected-device regression exercises both model identities,
  saves/reads an emoji label, independently changes brightness and stops cleanly.
  It is added before candidate preparation/signing in all four candidate workflows.
- Native paired editor passed selected-device preview/action calls, independent
  drafts and saves, owner-scoped history, device selection, brightness, editor
  close/service continuity and persisted page/brightness after background restart.

These are source/fixture results on the AI server's Fedora environment, not native
Ubuntu or Debian acceptance. New clean public-source candidates must be built and
verified before the prepared per-OS package checks can run. User selected the
existing local Ubuntu VM for the later physical two-Deck stage; no USB handoff
has occurred. Its present audio backend is disabled, so real-audio setup must be
prepared and reviewed as part of that stage.

## Sources

- https://github.com/RustSec/advisory-db/blob/main/crates/zvariant/RUSTSEC-2026-0370.md
- https://github.com/RustSec/advisory-db/blob/main/crates/zbus_names/RUSTSEC-2026-0371.md
- https://github.com/RustSec/advisory-db/blob/main/crates/zvariant_derive/RUSTSEC-2026-0372.md
- https://docs.rs/zbus/6.0.0/zbus/

The private Xvfb captures retain the expected unavailable login-preference notice
(no normal user service manager on the private bus) and missing system GTK emoji
glyphs in text widgets; the device-rendered captions include their glyph fallback.
These captures establish the stated editor/control path, not full desktop font,
portal, accessibility or startup acceptance. The native Ubuntu/Debian gates remain
open rather than treating those private-session limits as passes.

## First V1.5.2 container attempts

Public source `cb52a0030ad140569fcd7d37dcea7b96c9a0c9aa` passed the independent
Quality workflow. Fedora, Ubuntu and Debian completed their existing quality
checks and native bundle build, then stopped before signing at the new private
paired regression. Its default layout came from the bind-mounted checkout,
owned by the outer runner rather than the container test user; the existing
layout ownership check correctly refuses that setup. The source fixture had
passed on the normal-user AI host where those owners matched.

The fixture now copies starter configuration into its own private resource tree
and explicitly points the virtual daemon there. Ownership checks and runtime code
are unchanged. It also prints the daemon log on failure. The corrected fixture
passed again on the AI host; clean container reruns remain required. Failed run
IDs: Fedora `39055589592`, Ubuntu `39055589578`, Debian `39055589570`.


## Verified V1.5.2 candidates and preview upgrade finding

The dependency migration and container fixture correction are on both hosts with
identical source tree `d3f8364cc78ed2b9bb5b1db752b7196f8bd1c39e`:
canonical `4c6dcfb08e76713073ade16d50a5e113aac910cb`, public
`5b6d052ef054ba10162bc6c9f16652f647c80112`. Quality and all three candidate
workflows passed. All six subjects per target passed the existing verifier with
exact workflow, source, main ref, hosted-runner, checksums and archive admission.

| Target | Successful build/signing run | Admitted bundle |
| --- | --- | --- |
| Fedora 44 | https://github.com/infamous-pattern/decksmith/actions/runs/39056038382 | `1.5.2-c114cafd843c` |
| Ubuntu 26.04 | https://github.com/infamous-pattern/decksmith/actions/runs/39056038529 | `1.5.2-309c7a4f9e2a` |
| Debian 13 | https://github.com/infamous-pattern/decksmith/actions/runs/39056038625 | `1.5.2-f6c7d819d5c6` |

The unmodified Ubuntu and Debian packages installed without changing saved bytes
or startup preference, but could not start the permanent service on the retained
V1.0 preview settings. Both guests had an owner-only `.config` parent, a `0775`
Decksmith folder and a `0664` preferences file. The service correctly rejected
that folder as `unsafe_layout_directory`. Both initial attempts restored the
original integration and complete user baseline, then shut down cleanly.

An additional, not-yet-published installer migration now handles that bounded
case. For an existing managed V1.0 preview, with controls stopped and the editor
closed, it verifies an unchanged self-owned regular configuration tree under an
owner-only XDG configuration parent. It refuses links, hard-linked or executable
files, special permissions, read-only directories, shared parents and oversized
or changing input. It creates and byte-verifies a private copy, preserves owner
access bits (including read-only files), retains the original folder and modes
as `.decksmith-preview-before-<timestamp>` beside the active settings, and records
that backup path. Installation failure restores the original tree and modes.
Runtime ownership checks remain unchanged.

The final installer SHA-256 tested on both guests is
`8d74705c8643e0922b50ccca2e92932f452d11eadac1bab95449af46345bfbc8`.
All **122** script/installer tests passed. Both native VM runs passed with this
reviewed source installer and the admitted signed runtime: preview upgrade,
unchanged saved bytes and startup preference, no-device service start/stop/restart,
native Wayland waiting editor, package rollback/reinstall, independent virtual
Plus/MK.2 drafts and saves, selected-device preview/action/brightness, owner-scoped
history, editor-close continuity and persisted pages/brightness after restart.
Original guest baselines and modes were restored; both guests are off. The original
preview permission trees are retained in private evidence as well.

Private final evidence directories are
`outputs/ubuntu-support-20261017/evidence/package-<ubuntu2604|debian13>-source-migration-confirmed/`.
The earlier private editor fixture also needed an explicit private umask on these
distributions; its failed result is retained separately. Four model/OS screenshots
were inspected: labels, emoji, device geometry and visible scroll affordances render
correctly. Transient disabled-action hints in captures do not substitute for human
interaction, accessibility or portal acceptance.

This qualifies the reviewed migration with the signed runtime, **not a signed
package containing the new installer**. Publishing that additional source and
rebuilding the candidates needs maintainer approval. Fresh/missing-dependency,
GNOME integration and physical Ubuntu acceptance remain open. Debian also reports
its lock source unavailable with GNOME 48 in this fixture; no lock/foreground
compatibility is claimed. Fedora V1.5.1, preview release assets, installer pins and
the daily desktop remain unchanged.


## Approved installer fix: exact signed package repeats

The maintainer approved pushing the additional installer migration and rebuilding
all three candidates, without release publication or desktop installation. Both
hosts verified identical source tree `a1882002b9865979cef6b9c5b92336b73d0cb3a3`:
canonical `c97abc6e73bfee1f4edce054da46f73e13df214d`, public source
`896a973adb59922f1fb8cdd986deab109f14888b`. The earlier paragraph's pending approval
is historical; this section records its completion.

| Target | Replacement build and signing | Admitted bundle |
| --- | --- | --- |
| Fedora 44 | https://github.com/infamous-pattern/decksmith/actions/runs/39057610936 | `1.5.2-51af13296b52` |
| Ubuntu 26.04 | https://github.com/infamous-pattern/decksmith/actions/runs/39057610984 | `1.5.2-ee1a4addd776` |
| Debian 13 | https://github.com/infamous-pattern/decksmith/actions/runs/39057610950 | `1.5.2-98b7b247f2e3` |

Quality and all three build/signing workflows passed. The unchanged verifier
admitted all six signed subjects for each target against that exact public source,
workflow and main ref, with hosted-runner restrictions, checksums and archive
validation. The complete Ubuntu and Debian packages, including their signed
installer, then passed the parallel normal-user VM repeats without a source
installer substitution. Each included the exact final installer SHA-256 recorded
above. Upgrade, no-device unit start/stop/restart, native Wayland waiting editor,
rollback/reinstall and native private paired editor/restart checks all passed.
Original saved bytes, permission modes and startup integration were explicitly
verified after restoration. Original permission trees were retained in private
evidence; both guests shut down gracefully and are off.

Evidence: `outputs/ubuntu-support-20261017/evidence/signed-fix-ubuntu2604/` and
`.../signed-fix-debian13/`. Fresh installation/missing-package repair, human/portal/
accessibility, GNOME extension integration, physical USB/audio/media/microphone and
lifecycle/resource acceptance remain open. Debian's later repeat reported a GNOME
lock source available, unlike the earlier startup sample; neither observation
certifies physical lock recovery or GNOME 48 extension compatibility.

No release, public installer pin, preview asset or daily desktop was changed.
The subsequent documentation-only receipt commit does not replace these immutable
candidate source identities or their signed files.
