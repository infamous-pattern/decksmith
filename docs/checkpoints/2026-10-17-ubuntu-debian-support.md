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
| Signed package admission and relocation | Pending | Pending |
| Offline verified VM backup, OS update and confirmed reboot | Passed; retained-preview smoke passed; guest off | Passed; retained-preview smoke passed; guest off |
| Normal-user upgrade from retained V1.0 preview; saved files and startup state | Pending | Pending |
| Service start/stop/restart with no hardware | Pending | Pending |
| Native paired virtual editor; owner-specific saves, brightness, recovery and restart | Pending | Pending |
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
