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

### Production-use and passive resource sample — September 23, 2026

The user reports six days of daily production use while replacing OpenDeck with
Decksmith, across active development updates. This is valuable real-world
acceptance evidence, not six days on one frozen candidate. The user also reports
that page switching and their assigned audio/media/brightness controls worked as
expected in a recent physical check.

A separate ten-minute, read-only sample of the installed qualification build
`0.1.0-9c8de697f85b` recorded 11 samples: the service stayed active on the same
process with zero restarts and no error-level service log entries. RSS stayed at
18.2 MiB, thread count at 18 and open-file count at 16. CPU use ranged from 4.317%
to 4.717% of one core. The production workload was not controlled, so this does
not establish the idle CPU target (normally below 1%) or performance with four
active meters. The user confirmed audio was playing through one or more monitored
targets during sampling; the number of active meters was not captured.

With playback stopped, a ten-minute daemon-only sample recorded 11 samples with
1.583–1.700% of one core, 18.2 MiB RSS, 18 threads and 16 open files. A subsequent
five-minute cgroup sample included the daemon and helper processes: six samples
showed 5.921–6.255% of one core, 46.7–49.2 MiB total service memory, four to five
processes and 26–27 tasks, with no playback streams. The service remained active
on the same main PID, with zero restarts and no error-level service logs. The
daemon-only idle reading exceeds the documented below-1% target; the cgroup figure
includes helpers and is not directly comparable to that daemon-only target. Treat
this as an observed idle-resource gap requiring investigation and a repeatable
idle/load benchmark before V1, not as a production change. The full-service sample
log is `/tmp/decksmith-v1-cgroup-idle-20260923.jsonl`; the daemon-only idle log is
`/tmp/decksmith-v1-physical-idle-20260923.jsonl`.

A follow-up three-minute, read-only idle attribution sample on the same installed
build found no playback streams and four stable service processes. Total CPU ranged
from 2.963% to 3.097% of one core. Averaged across the sample, `decksmithd` used
1.792%, `audio_meter.py` 0.655%, `audio_targets.py` 0.366%, and
`control_health.py` 0.216%. The service remained active on one main PID with zero
restarts and no error-level logs. Direct cgroup memory after the sample was 44.9
MiB; summed per-process RSS was higher because RSS can count shared pages multiple
times. This sample was lower than the prior five-minute idle result, so repeatable
benchmark conditions still need to be established before attributing the difference
or changing the performance target. No production settings or code were changed.

The matching three-minute sample with one Brave playback stream active recorded
6.026–6.358% of one core service-wide. Average per-process CPU was `decksmithd`
4.794%, `audio_meter.py` 0.788%, `audio_targets.py` 0.372%, and
`control_health.py` 0.227%. Direct cgroup memory after the sample was 47.1 MiB.
The same main PID remained active with zero restarts, one playback stream remained
present, and no error-level logs were recorded. Relative to the adjacent idle sample,
service CPU rose by about three percentage points, mostly in the daemon; the meter
helper rose by only about 0.13 points.

A release-optimized isolated renderer benchmark processed 10,000 changing strip
frames in 164 ms with one meter dial and 209 ms with four. That cost is below 0.05%
of one core at 20 frames per second, so drawing the strip alone is unlikely to explain
the playback increase. A second benchmark reproduced the physical adapter's RGB
copies and upstream JPEG-quality-90 encoding for 1,000 changing 800x100 frames.
Encoding and copies took 1.526–1.633 seconds total, equivalent to about 3.05–3.27%
of one core if performed at 20 frames per second. The adapter performs this conversion
for each changed touch-strip frame. This is a strong candidate for the daemon's
playback-related CPU increase, not a confirmed stack profile: the benchmark omits
USB transfer and the full service loop. A symbolized profile or equivalent isolated
measurement of the physical update path is still needed before changing the adapter.
All benchmarks were run from a temporary source copy; no production code or settings
were changed.

A follow-up isolated comparison encoded 200 changing four-dial 800x100 strip frames
at JPEG qualities 90, 85, 80 and 75. Average encoded sizes were 23,906, 20,147,
17,904 and 16,173 bytes per frame respectively. Relative to quality 90, decoded
quality-75 frames measured 33.47 dB PSNR; a visual spot check of the generated strip
showed no obvious difference at display size, though the sample was simple UI art and
does not replace a physical-device readability check. Encoding cost was 1.656 ms per
frame at quality 90 and 1.561 ms at quality 75, estimated at 3.31% and 3.12% of one
core at 20 updates per second (1.66% and 1.56% at 10 updates per second). Thus lower
quality reduces transfer size by about 32% but saves little CPU. Halving the update
rate halves the encoder estimate, but may make the live meter feel less responsive;
neither quality nor rate was changed in production. These are isolated estimates and
need physical responsiveness and legibility review before tuning the shipping path.

The host installer created a rollback backup and preserved the existing
login-start preference. Fedora 44 and 45 VM autostart remains off;
all four test VMs were shut down after qualification. These results are local
evidence only. Repeat the required release gates against a frozen V1 candidate.

## Work order

First freeze this supported scope and create one results sheet for the four VMs
and physical host. Then close reliability and accessibility findings, run the
full installation/security gate on a release candidate, fix blockers, and repeat
affected checks before naming V1. Optional features do not replace any gate.
