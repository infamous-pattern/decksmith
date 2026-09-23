# V1 release scope and acceptance plan

**Decision recorded September 23, 2026.** This is the release-scope reconciliation for
the v0.4 design baseline. The original archive under `baseline/` remains unchanged.
The product requirements describe the longer-term architecture; section 6 and the
Phase 8/9 gates in `PRODUCT_REQUIREMENTS.md` point here for the agreed V1 boundary.
The current `v0.1.0-preview.3` download is a preview, not evidence that these gates
have passed.

## What V1 promises

The supported V1 product is a local, English-language Decksmith installation on
**Fedora Workstation 44, x86_64, GNOME/Wayland, with one Stream Deck +**. A Fedora
45 final-release build may be listed only after final Fedora 45 validation; the
retained Fedora 45 Beta VM cannot certify the final release. One attached device is
controlled at a time. The supported installation path may be the current verified
per-user bundle and URL installer; an RPM is not a V1 prerequisite.

The V1 feature set is the integrated Home / Pages / Keys & Dials / About editor;
saved pages and application-based page switching; key, dial and touch-strip
assignments and feedback; brightness; customizable labels, artwork, typography
and themes; layout import/export and recovery; built-in application and website
launch, audio, media and system actions; background controls, login preference,
GNOME indicator and Auto-Lock. Shared dial defaults and page overrides remain part
of this set. Save and Apply continues to be explicit. Every advertised action
must work on the reference desktop or be clearly reported unavailable when its
system capability is absent. Existing custom labels, artwork, pages and
assignments must survive upgrades, rollback and ordinary device recovery.

The optional Homebridge/OpenHomeB integration remains **experimental** in V1.
Its general plugin compatibility, permission sandboxing and marketplace support
are not V1 claims. If included in a V1 bundle, failures, authentication expiry or
removal must not disable built-in controls or lose assignments; its own status and
access limitations must be visible. A release may omit this experimental component
without failing the core V1 gate.

## Reconciled deferrals from the original V1 list

These remain product goals but are not release blockers for this focused V1:

| Originally listed in V1 | Reconciled destination |
| --- | --- |
| Multiple profiles, nested workspaces/folders, global/profile/workspace binding scopes and sticky keys | Later profile/workspace milestone; current pages and shared dial defaults remain V1 |
| Double/long/hold trigger variants beyond the verified key/dial/touch behavior; multi-step actions, multi-state controls, Context Layers, Control Views, dial stacks and action wheels | Later workflow/control milestone; retain safe release/hold cancellation where currently used |
| General file/folder opening, shell commands, keyboard injection, generic HTTP/MQTT and dynamic capability providers | Later capability milestones; do not advertise unimplemented actions |
| SQLite storage and its database migrations | Later storage migration; V1 must protect and migrate its **actual versioned saved format** and provide tested backup/rollback |
| Panoramic key-grid and full-width touch-strip backgrounds, configurable per-state imagery, idle/sleep and configurable locked appearance | Later appearance/device milestones; existing themes, per-control artwork and safe Auto-Lock remain V1 |
| Standalone profile/theme/workflow packages | Later sharing milestone; existing versioned layout export/import remains V1 |
| Native RPM/COPR packaging | Later packaging milestone; the V1 per-user installer must meet the release gates below |
| Simultaneous independent devices and other Stream Deck models | Later hardware milestones, with explicit model-specific certification |

General plugin delivery and **broad GNOME/Wayland distribution compatibility**
both target V1.5. The V1.5 distribution goal starts with supported installation,
editor, background service and recovery on Debian 13 and Ubuntu 26.04, then an
explicitly tested matrix of additional GNOME/Wayland distributions and versions.
The V1 diagnostic VM results are the starting evidence, not support certification.
Full English/German/French/Spanish/Italian localization targets V2. Debian,
Ubuntu, other desktops and architectures are not advertised as supported V1
installations. This deferral is a scope decision, not a claim that the larger v0.4
features have been implemented.

## Required release gates

1. **Core behavior and physical device.** On the reference desktop, check every
   advertised key/dial/touch action, image and preview parity, page changes,
   brightness, audio/media/system feedback, Auto-Lock, login start, Quit/blanking,
   unplug/reconnect and suspend/resume. Repeat the disruptive lifecycle checks;
   verify the saved page and assignments restore without replaying held inputs.
2. **Sustained reliability and resources.** Record a reproducible longer run with
   four active meter targets, ordinary and rapid dial use, page switching and
   background-helper failures. Measure CPU, memory, handles, response latency and
   restart counts against `NFR-PERF`; investigate drift, stalls and unexpected
   changes. Exercise optional Homebridge with simulated outages, child failures,
   authentication expiry and recovery, plus one agreed live check if it ships.
3. **Usability and accessibility.** Complete a human-paced keyboard and Orca
   review, focus/error feedback, light/dark/high-contrast, enlarged text and actual
   display scaling. Verify no draft loss, focus theft or key/dial layout shift
   during refresh; document and fix release-blocking findings.
4. **Installation, data and security.** From the exact release candidate, test
   dependency installation on a clean Fedora user/system, first run, update from a
   prior published preview, backup, rollback across changed engine/config formats,
   uninstall/reinstall and reboot. Preserve user data. Run the complete quality
   gate, dependency and credential/metadata audits, validate bundled assets and
   provide a release-authenticity method beyond checksums alone. Ship accurate
   installation, recovery, privacy, known-limit and feedback guidance.
5. **VM matrix and physical host.** Run the matrix below for each release candidate,
   record the exact build, guest OS/GNOME/architecture, cases, results and limits.
   A VM result never substitutes for physical USB/audio/suspend acceptance on the
   Fedora desktop. Keep test VMs powered off with autostart disabled after use.
6. **Release decision.** Resolve all critical data-loss, security, device-recovery
   and core-control defects. Document lesser known issues. Publish only the exact
   tested artifacts, hashes/authenticity evidence, support matrix and release
   notes; verify the Gitea and sanitized GitHub source trees correspond.

## Current VM test matrix

All four retained machines are registered in virt-manager's **QEMU/KVM User
session**. Their current state is powered off, with VM autostart disabled. They
have no physical Stream Deck USB passthrough or host audio backend.

| VM | V1 role | Minimum release-candidate checks | Passing interpretation |
| --- | --- | --- | --- |
| `decksmith-fedora44` | Supported-platform gate | URL and missing-dependency installation, doctor, native editor/VirtualDeck, service/login lifecycle, audio fixture, upgrade/rollback, data retention, accessibility and reboot | Must pass before V1 |
| `decksmith-fedora45-beta` | Forward-compatibility gate | Same feasible installed-app smoke, editor/VirtualDeck, service/recovery and packaging checks; record beta package versions | Must be run and defects triaged; does **not** certify final Fedora 45 |
| `decksmith-debian13` | Cross-distribution diagnostic | Verify installer rejects unsupported OS clearly; inspect package/ABI gaps, attempt a native source build and isolated editor/VirtualDeck smoke where feasible | Report results and blockers; a failure does not claim Debian support or block Fedora V1 unless it reveals a shared defect |
| `decksmith-ubuntu2604` | Cross-distribution diagnostic | Verify installer rejects unsupported OS clearly; inspect dependencies, attempt an isolated native app/VirtualDeck smoke where feasible | Report results and blockers; no Ubuntu support claim for V1 |

The current Fedora-built daemon requires glibc 2.43, while Debian 13 provides
glibc 2.41, so copying that binary is not a valid Debian compatibility test.
Use a Debian-native build or record the build/ABI blocker. Current Fedora-only
installation remains an expected documented limitation on Debian and Ubuntu;
these guests must still be exercised and their findings recorded.

No V1 release candidate exists yet, so this matrix is **planned**, not a new test
result. Prior preview and development checks are useful baselines but must be
repeated or explicitly carried forward against an identified candidate build.
For each row, record pass/fail/blocked, exact build and OS versions, evidence path,
known limitations and whether a failure is shared with Fedora. Record the same
details for the physical reference-host run.

### Development baseline run — September 23, 2026

This first pass used local bundle `0.1.0-b5c38d299da5` (SHA-256
`2d8c2795df30ef8dda84be406ec6887306ed09af919d03c1ea437f8bd4d428b2`). It was
built from the current development tree to establish the matrix, not from a frozen
V1 candidate. It does **not** close any release gate above.

| VM | Environment observed | Baseline result |
| --- | --- | --- |
| Fedora 44 | GNOME Shell 50.4; glibc 2.43 | **Pass for this smoke only:** update preserved the existing saved layout; installer kept login startup unchanged; VirtualDeck reported 4×2 keys, four dials and 800×100 touch area; background service started, entered `waiting` with no USB device, and stopped cleanly. |
| Fedora 45 Beta | GNOME Shell 51.beta; glibc 2.44 | **Pass for this smoke only:** bundle installed over the existing test install; VirtualDeck geometry matched Fedora 44; service started and stopped cleanly. Beta is not a supported-release certification. |
| Debian 13 | GNOME Shell 48.7; glibc 2.41 | **Expected compatibility block:** after adding `pulseaudio-utils` in the test VM, the Fedora binary failed its runtime check because it needs glibc 2.43. Installer exited before creating install state. A Debian-native build remains untested. |
| Ubuntu 26.04 | GNOME Shell 50.1; glibc 2.43 | **Pass for this smoke only after installing `pulseaudio-utils` in the test VM:** bundle install, VirtualDeck geometry and service start/stop passed. This is diagnostic evidence, not V1 support. |

The Rust core test run reported 90 passed and one intentionally ignored; Clippy,
installer tests, dependency/security checks, and cargo-deny advisory, ban, license
and source checks passed. All guests lacked physical USB passthrough and host audio,
so hardware, audio routing, unplug/reconnect and device recovery were not tested.
The test VMs were powered off after the run, with VM autostart still disabled.
Repeat applicable checks against the identified release candidate and complete
the physical Fedora 44 host gate before V1.

### Installer and qualification follow-up — September 23, 2026

Follow-up used local bundle `0.1.0-9c8de697f85b` (SHA-256
`1271f41d55eea3e8fe6fbdbfaab5494c8ce784093776ba0004d4d3e649e68adc`), built
from source commit `f68933a`. This remains a local development qualification
build, not a V1 release candidate and not a public release.

The follow-up fixed two installer defects found during the first pass: rejected
runtime validation could leave an orphan release directory, and a safe custom
desktop launcher could block updates. Regression tests now verify failure
cleanup, preservation of a safe customized launcher through update/rollback,
rejection of an unsafe launcher, and migration to the canonical bundled icon
when the existing launcher points to a matching older icon file. The installer
suite passed (18 passed, one optional test skipped).

The full automated quality gate also passed for source commit `f68933a`:
formatting, strict Clippy, 121 Rust tests (one environment-dependent test
intentionally ignored), 147 editor tests, installer shell syntax, and fresh
cargo-deny advisory, ban, license and source checks. The build used existing
task-local libudev development metadata; no system packages were installed.

| Target | Follow-up result |
| --- | --- |
| Fedora 44 VM | **Pass for this smoke:** install/activation, matching runtime doctor, VirtualDeck geometry, and service start/stop. VM shut down afterward. |
| Fedora 45 Beta VM | **Pass for this smoke:** install, runtime doctor, VirtualDeck geometry, and service start/stop. VM shut down afterward. |
| Ubuntu 26.04 VM | **Pass for this smoke:** install, doctor, VirtualDeck and service start/stop after adding `pulseaudio-utils` to the disposable guest. VM shut down afterward. |
| Debian 13 VM | **Expected unsupported-runtime rejection:** glibc 2.43 requirement is unmet on glibc 2.41; after the rejection there was no install state, current link, temporary extraction or orphaned candidate release. |
| Physical Fedora 44 host | **Pass for short physical smoke:** installed and activated this bundle; runtime doctor passed all checks and discovered the attached Stream Deck + with read access. The prior layout hash remained unchanged, the custom launcher was preserved and its icon path canonicalized. The service reconnected; logs recorded touch-page navigation and successful key actions. The user confirmed the display/layout looked normal, then exercised page switching and their assigned audio/media/brightness controls for a few minutes and reported that everything worked as expected. The broader gate remains open for exhaustive action coverage, sustained-use/resource measurements, suspend/resume, unplug/reconnect, Auto-Lock, login, blanking/Quit and fault recovery. |

A separate Fedora 44 VM lifecycle follow-up was not completed: the guest was
started, but this test environment had no noninteractive guest login available.
It was shut down without changing guest state; autostart remains disabled.

The host installer created a rollback backup and preserved the existing
login-start preference. Fedora 44 and 45 VM autostart remains off;
all four test VMs were shut down after qualification. These results are local
evidence only. Repeat the required release gates against a frozen V1 candidate.

## Work order

First freeze this supported scope and create one results sheet for the four VMs
and physical host. Then close reliability and accessibility findings, run the
full installation/security gate on a release candidate, fix blockers, and repeat
affected checks before naming V1. Optional features do not replace any gate.
