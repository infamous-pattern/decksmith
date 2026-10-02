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

No frozen V1 release candidate exists yet. A local qualification bundle has now
been exercised on these systems; its results are recorded below, but they are not
V1 release certification. Prior preview and development checks remain baselines
and must be repeated or explicitly carried forward against the eventual frozen
V1 candidate. For each row, record pass/fail/blocked, exact build and OS versions,
evidence path, known limitations and whether a failure is shared with Fedora.
Record the same details for the physical reference-host run.

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
idle/load benchmark before V1, not as a production change. The raw sample logs
are retained with local qualification evidence and are not included in public notes.

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

### Automated source quality checkpoint — October 1, 2026

The automated checks were rerun on Fedora Workstation 44 from source commit
`9ae11212c2837be460dbb4a8403a34455bb49696` using Rust 1.97.1. Formatting and
strict Clippy checks passed; the Rust workspace reported 121 passing tests and
one intentionally ignored virtual-microphone test because no disposable virtual
input was configured. The GTK/editor suite passed all 147 tests, and the
installer/security Python suite passed all 18 tests. Installer shell syntax and
Python bytecode compilation passed. `cargo-deny` 0.20.2 passed advisory, ban,
license and source checks after its archive checksum was verified.

A release-mode development bundle, `0.1.0-500f2477accf`, was also built from
this source tree. Its archive checksum and all manifest file digests verified;
isolated staging-mode install, status and uninstall completed, retaining the
generated backup and never activating startup. The bundle is marked dirty
because this checkpoint itself is an uncommitted documentation update; it is
not a release candidate and was not installed on the desktop or a VM.

This is source-level automated evidence, not a release-candidate qualification.
It does not close the physical-device and four-active-meter reliability runs,
the human-paced Orca/display-scaling review, or clean-user release-artifact
install, migration and rollback tests. The optional virtual-microphone path
also remains untested by this run.

### Local qualification bundle — October 1, 2026

The release-mode qualification bundle `0.1.0-db5d598eefe4` was built from clean
source commit `d43e3484b68d5e0cf02f82f338c0e8c2b7d2f29f`. Its rebuilt archive SHA-256
is `a5aac22345fb1499cbe8f902b332a3ae71ee1e94b3ab4fc89b093f2e35c6b56a`; the
archive checksum and manifest file digests verified. The isolated installer
install/status/uninstall path passed and generated a backup without activating
startup. This is a local qualification build, not a frozen V1 release candidate.

The original temporary archive was lost when the host rebooted. The rebuilt
bundle has the same release ID and source commit as the archive used for the VM
checks immediately before that reboot, so the verified payload manifest is the
same; the new compressed archive has a different outer checksum. The exact final
V1 distribution archive must still be tested and its own digest published.

| Target | Qualification result |
| --- | --- |
| Fedora 44 VM | **Pass for recorded smoke:** candidate install/upgrade, runtime doctor, VirtualDeck geometry, service start/stop, rollback to the previous build, candidate reinstall, and persistence across guest reboot. Saved layout hash remained unchanged. A separate native-editor screenshot attempt timed out under software rendering; it does not count as an editor/accessibility pass. |
| Fedora 45 Beta VM | **Pass for recorded smoke:** install, doctor, VirtualDeck geometry, service start/stop. The VM is diagnostic only and does not certify final Fedora 45. |
| Ubuntu 26.04 VM | **Pass for recorded smoke after adding `pulseaudio-utils` to the disposable guest:** install, doctor, VirtualDeck geometry, service start/stop. This does not make Ubuntu a supported V1 platform. |
| Debian 13 VM | **Expected compatibility block:** the Fedora-built daemon needs glibc 2.43; the guest has glibc 2.41. The installer rejected it before creating install state or replacing the prior release. A Debian-native build remains untested. |

All four VMs were shut off after the qualification run, and autostart remains
disabled. A later attempt to re-enter the Fedora 44 guest over its forwarded SSH
port timed out before the SSH banner; a console screenshot showed the GNOME
session, so the VM was shut down without changes. This access issue did not alter
the completed candidate smoke results. A focused keyboard, text-scaling and
AT-SPI review was completed in a later Fedora 44 VM session; its evidence and
remaining accessibility limits are recorded below.

On the physical Fedora Workstation 44 host, the same release ID was installed and
activated. The installer created a local rollback backup and retained
`0.1.0-9c8de697f85b` for rollback. The saved layout in the backup matched the live
file; its local path and hash are omitted from the public notes.
The runtime doctor passed all checks, found the physical Stream Deck + with read
access, and the service connected on the new process with zero restarts. The user
then completed a five-minute physical check of display, page switching,
audio/media controls and the brightness dial, and reported that everything
worked. During a later natural GNOME idle lock, the read-only control API reported
Auto-Lock available/enabled/locked and the device connected with its display
ready on the saved page. The user visually confirmed that the physical display
showed “Locked” after returning to the workstation. A 30-minute passive idle
sample is recorded below.

### Physical-host interactive control check — October 1, 2026

The user reported a five-minute normal-use pass with audio controls and page
switching, followed by a faster app-volume dial rotation while switching pages.
Both checks worked as expected; no freeze, missed input, disconnection or
unexpected value was observed. The dial was returned to its starting level.
The exact running build could not be reverified during this pass because the
local session could not access the user's systemd service bus. These user-observed
checks do not cover suspend/resume recovery.

The user also confirmed that unplugging and reconnecting the physical Stream
Deck restored its display and controls. The exact running build was not
independently verified during this check; suspend/resume remains outstanding.

The user confirmed Auto-Lock worked across multiple desktop lock/unlock cycles
today. The exact running build was not independently verified during this check.

The user then suspended and resumed the Fedora desktop and confirmed Decksmith's
display and controls recovered perfectly. This Codex session was interrupted
during the host suspend and resumed afterward; that interruption was separate
from Decksmith's recovery. The exact running build was not independently verified.

The user confirmed that Quit blanked the physical Stream Deck, and reopening
Decksmith allowed Background Controls to be started with the saved page restored.
The exact running build was not independently verified.

The user confirmed that start-at-login works as expected on the physical Fedora
host. The exact running build was not independently verified.

### Physical-host monitored-target sample — October 1, 2026

A ten-minute passive sample ran on Fedora Workstation 44 against the already
installed qualification build `0.1.0-9c8de697f85b`. Four configured audio-meter
targets remained present for all 40 samples. Average CPU was 9.76% of one core
(range 9.32–10.38%); cgroup memory ranged from 54.9 to 57.9 MiB. The observed
process count peaked at five, with 24 threads and 47 open file descriptors.
The systemd restart count stayed at zero and no error-level service log lines
were recorded.

This was a passive resource/stability observation: no keys or dials were used,
playback state was not recorded, and target names were intentionally excluded
from the sample. It is not the required sustained four-active-meter reliability
run or a performance acceptance result; its CPU figure is workload-specific and
should not be compared with the idle target. Raw process-statistics data is retained
with local qualification evidence and is not included in public notes. No service,
device setting or production file was changed.

### Candidate physical mixed-playback/idle sample — October 1, 2026

After activating `0.1.0-db5d598eefe4`, a ten-minute passive sample recorded 41
observations while four configured audio-meter targets were present. Average
service-cgroup CPU was 8.10% of one core (range 6.07–9.78%); cgroup memory ranged
from 44.3 to 47.7 MiB, and `pids.current` ranged from 23 to 24 tasks. One playback
stream was present for the first six minutes, briefly two streams were present,
then no playback stream was observed for the remaining four minutes. The systemd
main PID remained `58865`, the restart count stayed at zero, and no error-level
service log entries appeared. These varying playback conditions make this a
mixed-load observation, not a ten-minute continuous-playback soak or a performance
acceptance result. No audio target, service or device setting was changed by the
sampler. Raw data is retained locally and is not included in public notes.

### Candidate physical idle soak — October 1, 2026

A 30-minute passive idle soak ran on Fedora Workstation 44 with candidate
`0.1.0-db5d598eefe4`; the desktop naturally entered GNOME's locked state during
the sample. Across 61 observations, the service stayed active with zero systemd
restarts and no error-level journal entries. CPU
averaged 3.629% of one core (range 3.546–3.792% across 60 valid readings).
Cgroup memory averaged 45.82 MiB (range 44.67–47.49 MiB), task count was 23–24,
and no audio streams were present. This idle sample was followed by the controlled
continuous-playback four-meter soak recorded below. The user later
confirmed that the physical display showed “Locked” while the desktop was locked.
The sampler did not change power state, service settings or device settings. Raw
data is retained locally and is not included in public notes.

### Candidate physical four-meter playback soak — October 1, 2026

A controlled 30-minute passive soak ran on Fedora Workstation 44 with candidate
`0.1.0-db5d598eefe4` and the first page selected. All four distinct audio-meter
targets were available at each of 60 recorded samples. A playback stream on the
configured application target remained continuously present during five-second
continuity checks. The service stayed active with zero restarts; no error-level
journal entries appeared. The device remained connected
and the display ready at the final status check. Service-cgroup CPU averaged
9.604% of one core (range 9.235–10.165%), memory averaged 46.21 MiB (range
44.56–47.68 MiB), and task count was 23–24. This is a resource and continuity
soak; no keys or dials were exercised during it. Raw data is retained locally and
is not included in public notes.

### Fedora 44 editor accessibility review — October 1, 2026

The installed GTK editor was reviewed in the Fedora 44 VM with Orca enabled and
GNOME text scaling temporarily set to 125% and 150%. The accessibility tree
exposed 39 interactive controls, all with names; 36 were focusable. Orca
enumerated Decksmith, and keyboard Tab focus plus the Alt+3 Keys & Dials
shortcut worked. At both tested text scales the fixed Save and Apply bar stayed
visible; the device preview remained in the window, while lower editor settings
require scrolling. Tab reached the Appearance section and device-preview
controls. The GNOME high-contrast accessibility setting was also enabled; the
editor retained readable text and visible keyboard focus outlines, while the
custom device artwork remained unchanged.

An actual 125% monitor scale was applied through Mutter. The original
`0.1.0-db5d598eefe4` window overflowed to the right and bottom, clipping part of
Save and Apply. The fix removes the workspace shell's fixed 1280x860 sizing and
reduces the window's minimum size to fit smaller logical work areas. A VM-only
verification build, `0.1.0-c79af4afe46a` (archive SHA-256
`c44cee6410a7158872d67b342bcd3a70ee9ce9b4d54c464dfb07a0ccedd5f223`), was
launched after setting the monitor scale to 125%. Home and Keys & Dials fit
horizontally, Save and Apply remained visible, keyboard focus reached Appearance
and the dials, and focusing a dial scrolled the preview into view. The high-
contrast screenshot retained readable labels and visible focus. This verifies
the correction on the local build; it must be repeated on the frozen V1 artifact.
The edited source also passed the native `check_workspace_gtk.py --compact`
regression in the Fedora guest at 1024x768, covering workspace geometry,
key/dial focus traversal, page changes, draft/history behavior, status gating and
the five sections.

The VM's audio backend is explicitly `type='none'`, so although Orca enumerated
Decksmith, spoken output could not be heard or certified. Orca also logged
display/AT-SPI warnings during startup. Text scale, monitor scale, high-contrast
and screen-reader settings were restored to their original defaults, Orca was
stopped, and the VM was shut down with autostart disabled. No saved layout edits
were made.

### Clean-user installer qualification — October 1, 2026

A disposable `v1test` account in the Fedora 44 VM installed verification bundle
`0.1.0-c79af4afe46a` from a fresh profile. Installer status and runtime doctor
passed (`ready: true`, including release integrity and VirtualDeck); startup
remained disabled and the background service stayed inactive. Installing prior
local bundle `0.1.0-9c8de697f85b` and upgrading back to `0.1.0-c79af4afe46a`
created configuration backups. The backup contained `config/layout.json`, and
its SHA-256 (`544c7b44d0d78159cc11b09b64dc1ac3e595f0f23efa408e24864efe14459159`)
was unchanged through upgrade and rollback. Rollback selected the prior release;
reinstall returned to the updated build. Uninstall removed managed integration
while retaining the saved layout and release directories; reinstall succeeded.
The disposable account and its files were removed after testing.

The clean account's native first-run window was not verified. This Codex session
could not access the VM console to perform the separate graphical login, so the
test stopped before opening the window. No graphical-session credentials were
copied. A console-accessible graphical login for the clean test account is still
needed. This local verification build is not a frozen V1 release candidate and
does not close final artifact qualification.

### Local candidate VM smoke — October 2, 2026

Release bundle `0.1.0-377d2fdf128b` was built from clean source commit
`c9744bd03ea105fbec0e96266c41391ce6992ea6` (`source_dirty: false`). Its archive
SHA-256 is
`99b137598798f3b98548ca9c781a74e094ae4efc2acd802ff23001cc01c3f3f3`. The same
archive digest was verified in each guest before installation. This is a local
qualification artifact, not a frozen V1 release candidate or public release.

| VM | October 2 result |
| --- | --- |
| Fedora 44 | **Pass for this smoke:** candidate installed over `0.1.0-c79af4afe46a`; runtime doctor and release integrity passed; VirtualDeck reported 4×2 keys, four dials, 120×120 key images and an 800×100 touch area; the background service started and stopped. The private-bus smoke passed previews, navigation, save and layout validation. The native Wayland editor smoke passed key/dial selection and produced a rendered preview. The existing saved-layout checksum was unchanged. |
| Fedora 45 Beta | **Pass for this smoke:** candidate installed over `0.1.0-db5d598eefe4`; runtime doctor, VirtualDeck geometry, service start/stop and private-bus preview/navigation/save/layout-validation smoke passed. The existing layout hash was unchanged. This remains a beta diagnostic result, not final Fedora 45 certification. |
| Ubuntu 26.04 | **Pass for this smoke:** candidate installed over `0.1.0-db5d598eefe4`; runtime doctor, VirtualDeck geometry, service start/stop and private-bus preview/navigation/save/layout-validation smoke passed. No saved layout existed before installation, so there was no prior layout to compare. This is diagnostic evidence, not Ubuntu support. |
| Debian 13 | **Expected unsupported-runtime rejection:** runtime validation reported that the Fedora-built daemon requires `GLIBC_2.43`, unavailable on Debian's glibc 2.41. The installer exited before creating an installed release or current-release link. |

All four guests were shut down after testing; libvirt autostart remains disabled.
The Fedora 44 native editor capture is retained at
`docs/images/decksmith-v1-candidate-20261002.png`.
The VMs have no physical Stream Deck USB passthrough or host audio backend, so
this run does not cover physical hardware, audio routing, accessibility, upgrade
rollback, reboot persistence or the required reference-host gates. The Fedora 44
native GUI check used a private test bus and temporary XDG directories; it did
not change the guest's saved layout. These results apply only to this artifact
and do not by themselves certify V1.

## Work order

The supported boundary and initial results are recorded above. Remaining work is
to repeat the corrected scaled-display layout on the frozen V1 artifact, verify
audible Orca output with a suitable VM audio path, verify first-run UI in a
separately authorized clean-user graphical session, and repeat install,
migration, rollback and uninstall on the frozen V1 artifact. Then resolve any
critical reliability, security, device-recovery or core-control findings; repeat
affected checks on the exact V1 candidate and publish that tested artifact with
its own checksum. Optional features do not replace any gate.
