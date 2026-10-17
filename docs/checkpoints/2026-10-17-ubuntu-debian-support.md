# Ubuntu support and parallel Debian validation — October 17, 2026

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
| Fresh dependency installation and missing-package repair on current package | Pending | Pending |
| Whole-store upgrade/downgrade recovery across preview and V1.5 schema | Pending | Pending |
| GNOME indicator and foreground-app switching | Pending | Blocked on GNOME 48 compatibility review; extension metadata declares 50/51 |
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
