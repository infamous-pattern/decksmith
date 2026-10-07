# V1 release scope and acceptance plan

**Decision recorded September 23, 2026.** This is the release-scope reconciliation for
the v0.4 design baseline. The original archive under `baseline/` remains unchanged.
The product requirements describe the longer-term architecture; section 6 and the
Phase 8/9 gates in `PRODUCT_REQUIREMENTS.md` point here for the agreed V1 boundary.
The current `v0.1.0-preview.3` download is a preview, not evidence that these gates
have passed.

**Performance decision updated October 5, 2026.** By user approval, the original
below-1% idle daemon CPU target is a post-V1 performance milestone rather than a
V1 release blocker. It remains unmet; this is a change to release criteria, not a
performance pass. Earlier dated qualification entries retain their original
assessment. The current requirements below supersede their CPU-gate wording.

**Visible-response decision updated October 7, 2026.** By user approval, numerical
qualification of the below-150 ms visible key update and below-100 ms local dial
feedback targets moves beyond V1, with the unmeasured boundary documented. Neither
target is claimed met. Correct responsive physical behavior, recovery, internal
dispatch, resource review and security remain required. This change does not
accept the unmatched CPU comparison or authorize V1 publication.

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
| Idle daemon CPU normally below 1% of one core | Post-V1 performance milestone; V1 still requires measured idle/loaded resources, regression review and responsive controls |
| Numerical certification of visible key updates below 150 ms and local dial feedback below 100 ms | Post-V1 measurement milestone by October 7 decision; V1 retains correct responsive feedback and documents the unmeasured boundary |

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
   restart counts against `NFR-PERF`, with the CPU scope decision below; investigate
   drift, stalls and unexpected changes. Exercise optional Homebridge with
   simulated outages, child failures,
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

### V1 resource acceptance and post-V1 CPU milestone

The accepted reference baseline is the valid 300-second quiet measurement of
source `63b9523`, installed bundle `0.1.0-430c4bd75f9a`: **2.11% daemon CPU** and
**3.75% full-service CPU**, each expressed as a percentage of one CPU core, with
**17.90 MiB daemon RSS** and **47.85 MiB cgroup memory**. The full service includes
helpers and child-process work; it is distinct from the daemon-only target.
These are observed reference-host results, not guarantees on other machines or
new universal CPU limits.

V1 must still measure idle and four-target interactive load on the final artifact,
compare results with this baseline and investigate material unexplained increases,
memory/handle growth, stalls or restarts. Existing memory targets, input-dispatch
and functional visible-feedback acceptance remain in force, as do reliability,
recovery, security and VM-matrix gates. This decision does not close those checks.
The separate October 7 decision defers numerical visible-feedback qualification;
the 150/100 ms engineering targets remain goals, not achieved results.

Post-V1 work targets normally below 1% **daemon** idle CPU through profiling and
reduction of unnecessary background work. Continue reporting full-service CPU so
work moved into helpers remains visible. Validate changes against idle/loaded
resources and input latency together, preserving display quality, polling safety,
lock detection and recovery. No new release number or calendar deadline is assigned
to this milestone.

The [October 7 CPU investigation](checkpoints/2026-10-07-v1-cpu-investigation.md)
provides concrete post-V1 leads: settled-frame dirty/revision tracking to avoid
repeated pixel-buffer comparisons, and profiling the per-second competing-process
scan. Preserve latest-wins display scheduling, failed-write/reconnect/lock/blanking
safety and competing-application protection. The constrained paired VM result
does not replace a matched physical comparison or remaining V1 acceptance.

## Current VM test matrix

V1 release-candidate testing includes all four retained VMs: Fedora Workstation
44, Fedora Workstation 45 Beta, Debian 13 and Ubuntu 26.04. Debian and Ubuntu
must be exercised and their results recorded as cross-distribution diagnostics;
their inclusion in testing does not make them supported V1 platforms.

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

The signed V1 candidate `1.0.0-c5e4fea53afe` is now frozen and has completed the
scoped four-guest diagnostic matrix, staged recovery/preview upgrade and Fedora
44 helper-failure/workload tests. These do not constitute full release
certification. See the [exact-candidate checkpoint](checkpoints/2026-10-06-v1-signed-candidate.md)
for its digest, source, findings and outstanding gates. Prior preview and
development checks remain baselines and must be repeated or explicitly carried
forward against this candidate. For each row, record pass/fail/blocked, exact build and OS versions,
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

### Follow-up local qualification build — October 2, 2026

Build `0.1.0-860b297be165` was produced from clean source commit
`1937a3546ade8540e9a49748d0c773d119de0fb4` (`source_dirty: false`); archive
SHA-256: `975e3971ac25c64a9887992aef9df345ebe761eb6c74271cc6efacaefbb79b94`.
Compared with the preceding test build, this commit changes documentation and
includes the editor screenshot; product code is unchanged. This is another local
qualification build, not a frozen V1 release candidate or public release.

| VM | Result on this exact archive |
| --- | --- |
| Fedora 44 | **Pass for install/recovery and VirtualDeck smoke:** checksum verified; install/activate, runtime doctor, VirtualDeck geometry, service start/stop, rollback to the prior release, candidate reinstall, uninstall and reinstall all passed. Layout and settings remained unchanged; login startup remained disabled. After stopping controls, launching Decksmith showed the “Start background controls?” prompt; selecting “Start controls” started the service. After reboot, the service remained stopped and login startup remained disabled. |
| Fedora 44 accessibility | **Visual scaling pass, keyboard/Orca gates still open:** at 125% light, 150% dark and 150% high contrast, the 1024×768 logical editor screenshots kept Save and Apply visible; the settings and device preview remained in the window. AT-SPI snapshots contained 34 named controls. The SSH-launched test window did not acquire keyboard focus, so its automated focus-traversal assertions are not counted. The VM has no audio backend, so audible Orca output could not be verified. |
| Fedora 45 Beta | **Pass for install/reboot smoke:** checksum, install, runtime doctor, VirtualDeck geometry and service checks passed. The pre-existing enabled login-start setting remained enabled, and after reboot the service was active. This is beta forward-compatibility evidence, not final Fedora 45 certification. |
| Ubuntu 26.04 | **Pass for install smoke:** checksum, install, runtime doctor, VirtualDeck geometry and service start passed; login startup remained disabled. This is diagnostic evidence, not Ubuntu support. |
| Debian 13 | **Expected compatibility rejection:** checksum passed, then runtime validation reported the missing `GLIBC_2.43` symbol on glibc 2.41. Installation exited before creating a current-release link or install record. This remains a diagnostic result, not Debian support. |

All VMs were shut down after testing, with autostart disabled. The checks used
VirtualDeck and could not exercise physical USB, audio routing or Stream Deck
controls. A separate clean-user graphical first-run test, audible Orca check,
human-paced keyboard/screen-reader review and physical-host qualification remain
open. GNOME display/accessibility settings were restored after the scaling run.

### Fedora 44 production-host candidate check — October 2, 2026

The same local qualification bundle, `0.1.0-860b297be165` (SHA-256
`975e3971ac25c64a9887992aef9df345ebe761eb6c74271cc6efacaefbb79b94`), was
installed on the Fedora 44 reference desktop after verifying its checksum. The
installer retained the previous release and created a recovery backup. Runtime
doctor and release-integrity checks passed; the service remained active with no
restart, the detected device reported connected and display-ready, and the saved
layout was unchanged. The user tested page switching and assigned audio, media
and brightness controls on the physical Stream Deck + and confirmed they worked
as expected.

This closes the short interactive physical-control check for this bundle only.
The user also tested USB unplug/reconnect and suspend/
resume on this bundle: the saved page and controls returned without restarting
Decksmith. These lifecycle checks pass for this bundle only. Auto-Lock passed:
Fedora's lock state appeared on the device, and unlocking restored the page and
controls. Quit-and-blank passed: quitting blanked the device, then relaunching
Decksmith restored the saved setup. Start-at-login passed on the physical host:
after signing in, background controls started while the editor stayed closed.
The user also completed a keyboard-navigation check in the editor; focus was
visible and navigation reached Save and Apply without awkward jumps. Orca speech
also passed on the physical desktop: the user enabled GNOME Screen Reader and
confirmed it worked while navigating editor controls. Exhaustive system-action
and fault-recovery coverage remain open. This local bundle is not a frozen V1
candidate; all release gates must be repeated on the final frozen artifact.

### Clean-user first run and passive host sample — October 2, 2026

The exact `0.1.0-860b297be165` bundle was installed in a disposable Fedora 44
GNOME account. Its first graphical launch showed the expected prompt to start
background controls. Choosing “Not now” left the service stopped and did not
create a layout or settings file. The screenshot is retained locally at
`local/v1-candidate-20261002-860b/accessibility/clean-user-start-controls-prompt.png`.
The temporary account was removed, the VM's original GNOME autologin was
restored, and the guest was shut down.

A separate five-minute passive sample had observed 9.20% average CPU on one core
with a 16.41% sampled peak and is retained as an earlier uncontrolled observation.

For a ten-minute interactive soak, a temporary per-page Dial 4 override targeted
Chrome, producing four distinct meter targets: Chrome, Brave, the named input,
and the named output. The user confirmed switching pages and using ordinary and
faster dial turns during the sample; they reported no freeze, blanking, missed
input, or wrong-target change. Afterward, the original four-page layout,
prior-layout backup, and control-panel settings were restored byte-for-byte; the
device remained connected and display-ready on its original page. No service
restart occurred. The raw two-second readings are retained locally at
`local/v1-resource-20261002-four-targets/resource-sample.csv`.

Across this run, cgroup CPU averaged 10.39% of one core with a 34.38% sampled
peak. Cgroup memory ranged from 57.04 to 61.14 MiB (59.28 MiB mean); main-process
RSS stayed at 30.86–30.87 MiB. Task count was 23–24, and total open file
descriptors across service processes ranged from 42 to 54. The main process ID
was unchanged and restart count remained zero. This is active-playback evidence,
not an idle CPU result. It also does not simulate helper failures under load or
measure end-to-end action latency.

A separate five-minute playback-stopped sample was taken on the same four-target
configuration. Two non-corked speech-dispatcher sink streams remained present,
but no active source-output streams were listed; treat this as a quiet-desktop
sample, not a fully quiescent audio graph. Across 149 readings, cgroup CPU
averaged 6.42% of one core (3.65% minimum, 8.43% sampled maximum), above the
below-1% idle target. Cgroup memory averaged 61.36 MiB (58.83–62.88 MiB), and
main-process RSS remained 32.73 MiB. Service file descriptors ranged from 42 to
46. The PID stayed at 282923 with zero service restarts. The raw readings are
retained locally at
`local/v1-resource-20261002-silent-idle/resource-sample.csv`; its per-sample
thread-count field was invalid and is excluded from these results. A later
60-second process breakdown attributed about 4.65% CPU to the main daemon and
0.73% to the audio-meter helper. The resource gate therefore remains open for
CPU investigation and a repeated sample after any optimization.

Source review found that the physical adapter uses a blocking HID read with a
20 ms timeout, and the CPU sample localized most daemon time to its device-worker
thread. This is correlation, not a proven root cause. No polling change was made:
increasing the timeout can add input latency, and the host has no `perf` or
`strace` profiler available for a low-level trace. Profile the worker and measure
input latency together before changing the timeout.

The current source passed formatting, strict Clippy, and all-features Rust tests
(92 passed, one microphone-hold integration test ignored because it requires a
disposable virtual microphone). The editor Python suite passed 151 tests, the
installer suite passed 18 tests, and the pinned `cargo-deny` audit passed. All
four test VMs were confirmed powered off with autostart disabled afterward.

### 2026-10-05 quiet-idle recheck and process trace

After playback was stopped and the voice session ended, a five-minute sample
found no playback sink inputs. The only uncorked source-output streams were the
two expected Decksmith peak-level captures. Across 150 two-second readings,
cgroup CPU averaged 7.60% of one core (6.73% minimum, 8.04% p95, 8.84% maximum).
The main daemon's sampled process CPU averaged 3.80%; cgroup memory averaged
49.27 MiB, main-process RSS averaged 16.88 MiB, and task count was 23–24. The
service remained active as PID 32790 with zero restarts. The raw sample is
retained locally at `local/v1-resource-20261005-quiet-idle/resource-sample.csv`.

A separate 30-second read-only process trace observed 217 short-lived `pactl`
process instances (68 sink listings, 63 source listings, 56 sink-input listings,
and 30 PulseAudio info queries). The helper's fresh per-request snapshot invoked
several category-specific commands. The local candidate now uses one combined
`pactl --format=json list` result for sinks, sources, and sink inputs while
preserving the 500 ms refresh interval. All 152 Python tests pass, including
shared/fresh snapshot and action-safety cases. A short, read-only benchmark of
eight snapshots measured 12.68 ms child CPU and 29.29 ms wall time per snapshot
for separate listings plus `info`, versus 8.86 ms child CPU and 18.18 ms wall
time for the combined listing plus `info`. This is about 30% less child CPU in
that small microbenchmark, not a service-level result. A live read-only snapshot
also returned valid states for system sounds and default devices. The installed
candidate result is recorded below; the below-1% gate remains open.

Candidate-source checks on this change: all 152 Studio Python tests and all 18
installer tests passed; the default-feature Rust workspace suite passed (92
passed, one microphone-hold test ignored), formatting passed, and the pinned
`cargo-deny` 0.20.2 audit passed after verifying its published archive checksum.
After installing `systemd-devel`, the all-features workspace suite passed (123
tests, one ignored), hardware-enabled Clippy passed with warnings denied, and
the clean qualification bundle was built and checked successfully.

The source now adds `poll_return_to_dispatch_us` to action-queued and page-request
records. It measures with a monotonic clock from the normalized physical event
returning from the adapter through capability enqueue/page routing. This is a
closer dispatch measure than the old session-relative timestamp, but it still
starts just after HID report receipt and has not been collected on the running
physical build. It is now installed in the qualification build; coordinate a
physical key-and-dial run before reporting a latency result.

### 2026-10-05 installed-candidate qualification

Clean source `f5f397b497cb791717c7d4c7721259497378df92` produced bundle
`0.1.0-6b2590ed1a9d`. This is a qualification build, not a frozen V1 release.
The full Rust check/audit, 152 Studio tests and installer suite passed. Desktop
activation preserved all five saved layout/configuration/artwork files, retained
the prior release and a configuration backup, and restored the connected display.

With playback and Bluetooth phone audio disconnected, all test VMs stopped,
and no Deck inputs during 61 readings over 300 seconds, the service cgroup
averaged **6.36% CPU** of one core (5.49% minimum, 6.79% sampled p95, 7.09%
maximum). This improves the earlier 7.60% average but **does not pass** the
below-1% idle target. Main-process CPU averaged 2.30%, main RSS 14.84 MiB,
cgroup memory 46.00 MiB and file descriptors remained at 16. The display was
ready and unlocked throughout, no playback streams were present before or
after, and the daemon PID remained unchanged with zero service restarts.
Raw local evidence is under `local/v1-checks-20261005-f5f397b/quiet/`.

Fedora 44 passed runtime doctor, VirtualDeck preview/navigation/save, native
editor and integrated compact-shell checks. Upgrade from the actual published
preview.3, rollback, uninstall with saved-data preservation, and reinstall passed.
The QA harness originally wrote one Python cache file into the immutable release;
integrity checking correctly rejected it. Removing only that test-created file
and disabling bytecode writes in the harness allowed the cycles to pass. Release
integrity policy was not weakened.

Fedora 45 Beta passed runtime doctor, VirtualDeck and native-editor smoke tests,
saved-layout preservation and a reboot with enabled background startup and no
editor window. Its SSH-launched integrated test did not acquire keyboard focus,
so automated focus traversal is not counted as a pass. Both Fedora guests were
stopped or saved afterward. Debian 13 and Ubuntu 26.04 still require checks for
this exact artifact; previous results do not qualify the new build.

A follow-up read-only inventory cache now listens for local libpulse changes
instead of launching inventory queries at every helper request. Actions continue
to discover targets freshly. The cache is disabled until subscription succeeds
and after disconnection; notifications invalidate it, with a five-second watchdog
refresh. Meter peak sampling and existing reader/health refresh intervals remain
unchanged. All 158 Studio tests pass, including invalidation, missed-notification
watchdog, unavailable-listener fallback, query failure, notification/query races,
fresh action targeting and changed default routing.

In Fedora 44, a disposable audio sink exercised discovery, volume, mute, removal,
reconnection and audio-server restart. Changes converged in 12–94 ms; listener
recovery after server restart took about 1.05 seconds. The test sink was removed
and the VM saved afterward. A separate read-only host benchmark with 20 requests
over approximately five seconds reduced inventory commands from 40 to two and
child CPU from 171.79 ms to 8.34 ms. This benchmark is helper-level evidence;
the installed service measurement follows below.

### 2026-10-05 subscription-cache desktop measurement

Clean source `778b76250ce764ba43c013b303b1275d9cccce02` produced qualification
bundle `0.1.0-a0f9aea4288c`. Rust sources and dependency files are identical to
the hardware-enabled build already checked above; its tested release binaries
were reused. The packaged Python changes passed all 158 Studio tests and 18
installer tests. Archive checksums, clean-source metadata, helper inclusion and
private-host/path metadata checks passed. Installed file hashes and the complete
release file list still matched after its helpers ran. Five native listener
start/stop cycles accumulated no threads or file descriptors.

Desktop activation retained the previous release and a configuration backup,
preserved all five saved files byte-for-byte, and restored the connected display,
page 0, brightness 55 and enabled, available, unlocked GNOME Auto-Lock state.

In a second quiet 300-second sample with all VMs stopped, no playback sink inputs
before or after, and no recorded Deck inputs, the service cgroup averaged
**3.97% CPU** of one core (3.47% minimum, 4.27% sampled p95, 6.93% maximum).
This is about 38% below the immediately preceding 6.36% measurement and 48%
below the earlier 7.60% average. Main-process CPU averaged 2.25%, main RSS
16.93 MiB and cgroup memory 46.73 MiB; file descriptors remained at 16. The
display stayed ready, connected and unlocked, the PID remained unchanged and
service restarts stayed at zero. Local evidence is retained under
`local/v1-checks-20261005-778b762/quiet/`.

**The below-1% idle CPU gate still does not pass.** A separate read-only benchmark
of 100 unchanged process-ownership scans used 670 ms process CPU, or about
0.67% of one core at the existing once-per-second scan rate. That safety check
has not been reduced or disabled. Further daemon/helper profiling is needed;
the new candidate also still requires physical interactive acceptance, loaded
meter/latency/failure testing and the complete exact-artifact VM qualification.
No frozen V1 release or public download is claimed by these measurements.

The user subsequently confirmed the installed subscription-cache build responds
correctly during physical page switching, normal and quicker audio/brightness
dial turns, and mute/unmute. This closes the requested interactive smoke check
for that build, not the remaining resource, latency, failure or final-artifact
qualification gates. The user also reported that this build feels smoother.

The next source optimization reuses the lock monitor's desktop-bus proxies,
with property caching explicitly disabled. The 100 ms check interval, 250 ms
method deadline, one-second stale-state limit and conservative lock behavior
are unchanged. This avoids repeated proxy setup while still reading current
GNOME and logind state on every check. All-feature Rust checks, warnings-denied
Clippy, formatting and the pinned dependency audit pass. An isolated D-Bus test
also passed for lock/unlock transitions with no property notifications, blocking
when the service object disappears, and recovery after it returns. Run this
additional test explicitly:

```sh
dbus-run-session -- cargo test -p decksmithd --all-features --locked session_lock::tests::reusable_proxy_reads_unsignalled_lock_changes_and_recovers -- --ignored --exact
```

It is intentionally excluded from ordinary tests so it cannot claim names or
manipulate objects on the user's real session bus.

A separate read-only benchmark of 100 three-value lock probes used 170 ms
process CPU when recreating proxies versus 60 ms when reusing uncached proxies
(10 ms accounting resolution; both use the same dependency build). Wall time
was 210 ms versus 130 ms. This is a small diagnostic comparison, not a service
CPU result. A ten-second stack-profiler attempt collected zero samples and is
not counted as evidence.

Clean source `923949f7558500ee294d980f48ac7bcacba8291a` produced bundle
`0.1.0-dcef44758197` with freshly rebuilt hardware-enabled release binaries.
Fedora 44 passed installation and runtime doctor. An isolated VirtualDeck daemon
on the guest's real desktop bus observed an actual graphical-session lock and
unlock through GNOME, rejected page changes while locked and restored page 1
after unlock. The first attempt stopped because the resumed VM's screen was
already locked; after explicitly unlocking that guest session, the test passed.
Guest screen authentication remained enabled and unchanged. The saved guest
layout hash was unchanged, test preferences were stored only in temporary XDG
directories, the guest's normal Decksmith service remained inactive, and the VM
was saved and stopped afterward.

Desktop activation retained a configuration backup and the accepted previous
build. All five saved configuration files stayed byte-identical. Initial status
confirmed a connected, ready display, page 0, brightness 55, and enabled,
available, unlocked GNOME Auto-Lock; the service had zero restarts and installed
file integrity passed. The user then confirmed the physical Deck shows its
locked state and restores the same page and working controls after unlocking.
A valid 300-second quiet measurement with all VMs stopped, no playback or external
capture before or after, and no recorded Deck inputs averaged **3.88% service
cgroup CPU** (2.93% minimum, 4.25% sampled p95, 4.44% maximum). Main-process CPU
averaged 2.09%, main RSS 18.01 MiB and cgroup memory 47.65 MiB; file descriptors
ranged from 16 to 17. The PID was unchanged, restarts stayed at zero, and all
sampled states were connected, display-ready and unlocked. Evidence is retained
under `local/v1-checks-20261005-923949f/quiet/`. **The below-1% idle CPU gate remains
open.** The preceding 3.97% result belongs to the subscription-cache artifact;
this small difference is not a repeatability study.

The next candidate resolves effective dial assignments once when parsing a layout,
then shares immutable default assignments across pages. Page overrides and the
legacy no-dial layout path remain distinct, and saving still serializes the original
configuration. This removes repeated label, binding and compressed icon copies
from device checks without duplicating shared default artwork for every page.
Regression coverage includes override dispatch, touch-strip pixels, held controls,
layout replacement, shared object identity and JSON round trips.

The peak helper now stops draining its native main loop when there is no dispatched
work, rather than making seven additional empty polls. The eight-iteration bound,
10 ms tick and 20 Hz publishing remain unchanged. Fedora 44's real PipeWire test
measured independent 0.05 and 0.40 synthetic app peaks correctly; the temporary
null sink was removed, no default routing or volume changed, and the VM was saved
and stopped afterward. Full all-feature Rust checks, warnings-denied Clippy,
formatting and the pinned dependency audit pass; all 160 Studio tests and 18
installer tests pass. These are source/native checks; no new installed CPU result
is claimed yet.

Clean source `6cff5d3b7c6f5f83d20673dc4fab1556c571558e` produced qualification
bundle `0.1.0-4e47081f8cf8` with freshly rebuilt hardware-enabled release binaries.
Its checksum, complete release integrity, clean-source metadata and packaged
private-host/path metadata checks pass. Fedora 44 passed installation, runtime
doctor and the real GNOME lock/unlock VirtualDeck check, restoring the selected
page after unlock while leaving screen authentication and its saved layout hash
unchanged. One initial post-install command named an incorrect runtime launcher;
that check stopped and was rerun using the actual runtime-doctor script. The
successful retry is the evidence used here. The VM was saved and stopped.

Desktop activation retained a backup and the prior qualification build. All five
saved files remained byte-identical; initial status was connected and display-ready
on page 0 at brightness 55 with enabled, available, unlocked GNOME Auto-Lock and
zero service restarts. Evidence is retained under `local/v1-checks-20261005-6cff5d3/`.
After the user paused playback again, a 300-second sample observed 3.54% mean
service cgroup CPU (3.19% minimum, 3.97% sampled p95, 4.57% maximum), 2.01%
main-process CPU, 17.70 MiB main RSS and 48.24 MiB cgroup memory. File descriptors
stayed at 16, the PID was unchanged and restarts stayed at zero; there was no
playback/external capture at either endpoint and no recorded Deck inputs.

**This sample is not a qualifying unlocked-idle comparison:** GNOME's configured
300-second idle timeout, with zero lock delay, locked the host at the final status
check. Status at 270 seconds was still unlocked; the first 270-second segment
averaged 3.53% cgroup CPU and 2.01% main-process CPU. These are shorter diagnostic
observations, not a replacement five-minute gate result. The host's lock and idle
preferences were left unchanged. The previous valid 3.88% result remains assigned
only to its original build, and the below-1% CPU gate stays open. Raw samples and
journal evidence are retained under `local/v1-checks-20261005-6cff5d3/quiet/`.

A separate overlapping 60-second process-accounting diagnostic attributed about
1.98% CPU to the daemon, 0.70% to the meter helper, 0.18% to audio targets and
0.22% to health checks. This excludes short-lived child CPU and cannot be summed
as the complete service result. It directs further profiling rather than claiming
a cause or a release pass. All four VMs remained stopped at completion.

The user subsequently confirmed that physical page switching, assigned audio/media
controls and brightness still respond correctly on this installed candidate. This
closes its requested interactive smoke check; the remaining CPU, latency, failure
and final-artifact qualification gates remain open.

A subsequent 300-second quiet sample on the same installed candidate remained
unlocked, connected and display-ready throughout. The user left their ordinary
applications open, paused playback, left the Deck untouched and kept the desktop
awake. No playback or external capture was present at either endpoint, no Deck
inputs were recorded, the main PID was unchanged and restart count stayed at zero.
This sample is valid under the recorded quiet-measurement criteria.

Mean service cgroup CPU was **3.83% of one core** (3.20% minimum, 4.41% sampled
p95, 6.99% maximum); mean daemon CPU was **2.09%**, mean main RSS **17.74 MiB**
and mean cgroup memory **48.11 MiB**. File descriptors stayed at 16. The below-1%
CPU target still does not pass. The result does not demonstrate a repeatable
improvement over the preceding build's valid 3.88% sample; the earlier 3.54%
locked-at-end observation remains diagnostic only. Raw evidence is retained at
`local/v1-checks-20261005-6cff5d3/quiet-unlocked/`. The installed build and all user
settings were left unchanged, and all test VMs remained stopped.

### 2026-10-05 profiling and region-update candidate

The accepted source checkpoint `1062b01` was pushed to Gitea and exported into
GitHub's separate sanitized history. Both remote branch tips were verified, and
both fetched source trees matched canonical tree
`c0f247b1e78e0138d9d06d80eb58e4d16e3aea96`. No public release or download changed.

Further read-only profiling left the installed service running without a restart.
A 15-second main-thread software-clock attempt collected zero samples and is not
counted as stack evidence. A subsequent 20-second device-worker profile collected
350 samples with six lost (1.7%). The installed and symbol-bearing binaries had
the same build ID. The flat profile attributed about 65% of sampled time to
JPEG encoding, DCT and JPEG block writing, plus 16% to `roundf`. One playback
stream was uncorked when checked afterward, so this is active-context profiling,
not a replacement quiet CPU sample. It identifies image encoding as a promising
worker optimization rather than proving that it explains all idle overhead.

A separate 100-scan ABBA process-ownership diagnostic measured 71-73 CPU ticks
for the original scan and 71 ticks for a reused-string-buffer alternative (100 Hz
accounting resolution). There was no convincing saving, so the original safety
scan and its once-per-second interval remain unchanged.

The source candidate now keeps one bounded 800x100 RGB baseline in each physical
session (240,000 bytes, about 0.23 MiB). Only changed touch-strip regions are JPEG
encoded and sent, using the existing Plus region-write API. Rectangles align to
the full-frame 8x8 JPEG grid so unchanged pixels and compression boundaries remain
consistent. The first frame, a fresh connection, an uncertain/failed write retry,
and explicit blanking send a full strip. Successful writes alone advance the
baseline; unchanged frames are skipped. The full-frame device abstraction, saved
layout, 20 ms input read and 20 Hz meter publishing remain unchanged.

Tests passed for reconstructed frames, corner/bottom-edge updates, no-op frames,
invalid inputs, failed-write recovery, fresh-session state and complete blanking.
Real JPEG decode/composition tests matched full-frame decoded pixels exactly.
All-feature workspace tests, warnings-denied Clippy, formatting and the pinned
dependency audit passed. A release-built device test binary also passed all 15
ordinary device tests in the Fedora 44 VM.

An explicitly invoked diagnostic on that otherwise idle guest compared 200
synthetic four-meter frames in full/regions/regions/full order. Full encoding took
303.64 and 330.22 ms; region encoding including difference detection and cropping
took 60.18 and 59.08 ms. Encoded pixels fell from 16,000,000 to 2,926,080 and JPEG
payload bytes from 1,452,793 to 891,378. This small synthetic comparison is about
five times faster for that frame sequence, not an installed service CPU result,
a physical USB/firmware test or a release gate. The additional diagnostic test is
ignored by ordinary test runs; invoke it explicitly:

```sh
cargo test -p decksmith-device --all-features --release --locked hardware::tests::touch_region_encoding_diagnostic -- --ignored --exact --nocapture
```

Evidence is retained under `local/v1-profile-20261005-1062b01/`. Fedora 44 was
saved and stopped afterward. The region-update source was subsequently packaged
and installed as recorded below. Complete physical blanking/recovery, resource and latency qualification
remain necessary before release readiness can be claimed.

### 2026-10-05 region-update runtime qualification

A clean source `63b9523` bundle `0.1.0-430c4bd75f9a` passed archive checksum,
all 340 packaged-file integrity checks and a private build-metadata scan. The
Fedora Workstation 44 VM installed the artifact without activating hardware
controls; its runtime dependency/integrity checks, VirtualDeck and real GNOME
lock/unlock test passed. Locked page changes were rejected and the previous page
restored after unlock. The guest saved layout hash was unchanged. The VM was saved
and stopped afterward; all four testing VMs remained stopped.

The same bundle was then activated on the desktop. The installer retained the
previous `0.1.0-4e47081f8cf8` release and made a settings/artwork backup. All five
saved JSON configuration files remained byte-identical. The Stream Deck +
reconnected, its display became ready and GNOME Auto-Lock remained available and
enabled. The user confirmed page switching, audio dials and mute, with labels,
meters and icons correct and no stale areas, flicker or missed display updates.
This confirms the requested interactive region-update smoke check, not exhaustive
blanking/recovery, latency or resource qualification. Raw evidence is retained
under `local/v1-checks-20261005-63b9523/`.

The subsequent 300-second quiet measurement on that exact installed artifact was
valid: all sampled status checks were connected, display-ready and unlocked;
playback and external capture were absent at both endpoints; no Deck inputs were
recorded, the daemon PID was unchanged and restart count stayed at zero. Mean
service cgroup CPU was **3.75% of one core** (3.38% minimum, 4.03% sampled p95,
4.66% maximum); mean daemon CPU was **2.11%**. Main RSS averaged **17.90 MiB**
and cgroup memory **47.85 MiB**, with 16-18 file descriptors. The previous valid
build measured 3.83% service CPU and 2.09% daemon CPU. This small difference does
not establish a repeatable idle CPU improvement, and the below-1% gate remains
open. The earlier synthetic encoding speedup and active-worker profile must not
be presented as a measured idle service saving. Evidence is retained under
`local/v1-checks-20261005-63b9523/quiet-unlocked/`.

After the idle measurement, the user confirmed that Quit Decksmith blanked every
key and the entire touch strip, and that relaunching/starting controls restored the
saved page cleanly. A subsequent read-only status check confirmed the physical
Deck was connected and display-ready, with all five saved configuration files
still unchanged. This closes the explicit blanking/relaunch smoke check for this
artifact; suspend/reconnect, latency, loaded-resource and remaining final release
gates are not inferred from it.

### 2026-10-05 isolated helper-failure and response checks

The installed `0.1.0-430c4bd75f9a` artifact (source `63b9523`) was exercised in
Fedora Workstation 44 / GNOME Shell 50.5 using VirtualDeck, a separate temporary
saved layout and four synthetic null-sink audio sources. Only children of that
test daemon received fault signals; the user's physical desktop controls were
left running. Two preliminary harness attempts failed setup checks (preset
selection, then an unsupported dial action in the fixture); neither is counted
as a product failure or acceptance result. The corrected fixture validated its
schema and explicitly verified that its custom layout was selected before tests.

The completed bounded run confirmed:

- A stopped meter helper lost its displayed cached readings and displayed the
  missing-measurement marker; resuming it restored live feedback.
- A killed meter helper was replaced and fresh feedback returned.
- A killed persistent volume reader was replaced and its replacement stayed
  stable through subsequent snapshots.
- A stopped volume reader was discarded after its timeout and replaced; the
  replacement was observed about 3.26 seconds after fault injection.
- Page switching remained responsive through all four cases. The daemon stayed
  alive without a restart, and guest saved-layout hashes and default audio
  routing remained unchanged.

There were 48 D-Bus page changes during these scenarios. Request-start to observed
display-ready response measured 35.73 ms median, 39.34 ms p95 and 39.76 ms maximum.
This includes bus, rendering and status polling on VirtualDeck; it does **not**
measure HID receipt-to-dispatch or physical visible-update latency. Full-strip
readback observed stale/live meter transitions after about 1.25/1.27 seconds;
those observations include D-Bus readback and polling and do not establish the
internal 400 ms expiration deadline. Meter-child replacement was observed after
the stale-display check, so its phase timing is not an end-to-end crash-recovery
measurement. This is a short failure-recovery check, not the longer four-target
physical interactive resource soak or exhaustive failure coverage.

Fixture processes, players and null-sink modules were removed; routing and saved
layout were checked again after cleanup. Fedora 44 was saved and stopped, and all
four testing VMs remained stopped. Evidence and the corrected guest-only harness
are retained under `local/v1-helper-checks-20261005/`.

The user also confirmed that physical page/key/dial controls responded correctly
during a brief follow-up check. The selected journal window contained zero
matching timing records, so no physical dispatch percentile is reported. Exact
HID receipt-to-dispatch timing and physical key/dial visible-update latency remain
open and need appropriate instrumentation; the user's successful interaction
check and VM response timings do not close those gates.

The follow-up source candidate adds monotonic report-return stamps before adapter
normalization, retains them for queued events, and records backend-entry timing
after audio-queue waits. Queue-entry timing remains distinct from dispatch and
completion. Virtual inputs and uninstrumented paths report no physical sample.
Two regression tests passed for queued report stamps and delayed snapshot/action
boundaries; the all-feature workspace suite passed (129 tests, three intentionally
ignored), with formatting and warnings-denied Clippy. The pinned dependency audit
also passed after a sandbox advisory-cache lock restriction was resolved through
reviewed access. Packaging, native runtime and physical timing qualification for
this source candidate remain pending. See [timing boundaries](input-latency.md).

### 2026-10-05 report-timing candidate runtime and physical follow-up

Source `01b9f07` was packaged from a clean checkout as
`0.1.0-c2f92c7438e6`. Its archive checksum, 341-file integrity manifest and
private-metadata checks passed. The installed Fedora 44 guest runtime passed
dependency/resource checks, the four-target helper-failure fixture and real GNOME
lock/unlock reporting with VirtualDeck. All four fixture targets were verified
live before fault injection. The daemon survived meter stall/crash and volume
reader crash/timeout without restarting; guest layout and default routing stayed
unchanged. Across 48 page requests, request-start to observed VirtualDeck
display-ready measured 6.41 ms median, 39.98 ms p95 and 40.59 ms maximum. These
are bus/render/polling observations, not physical input dispatch measurements.
The reader-timeout replacement observation was 3.26 seconds. Meter stale/live
readbacks include full-strip bus transfer and polling; replacement timing starts
after stale-readback, rather than at fault injection. The fixture was cleaned up
and Fedora 44 saved and stopped; all four test VMs remained stopped.

The same artifact was installed on the physical desktop with a retained previous
release and configuration backup. Five saved configuration hashes were unchanged.
The user confirmed that physical keys/dials, page switching, and ordinary/quicker
turns continued to work correctly. A targeted single clockwise leftmost-dial
click also changed its displayed value, according to the user. The daemon's PID
remained stable, with zero restarts, connected and display-ready.

Live capture recorded four physical right-swipe page requests. Library report
return to page-render invocation measured 16, 18, 18 and 36 microseconds (median
18 microseconds). This small, single-route sample does not qualify the overall
25 ms dispatch target or visible-update targets. The journal contained **zero
key or dial timing records**, despite the user's explicit physical-interaction
confirmation. The installed binary path and journal output were verified; no
explicit service log filter or level limit explained the gap. Its cause remains
unresolved. Successful user interaction and missing error records must not be
substituted for measured backend dispatch or completion. Key/dial recording
diagnosis, representative workloads and physical visible-update measurements
remain open. Evidence is retained under `local/v1-input-timing-20261005/`,
including `physical-follow-up.json` and the raw journal captures.

### 2026-10-05 bounded input diagnostics and audio-action reuse candidate

Clean source `b6ae486` was installed as `0.1.0-a9f64a142f5f` after 130 passing
Rust tests (three intentionally ignored), warnings-denied Clippy, formatting,
dependency audit, archive integrity and Fedora 44 installed-runtime checks.
The isolated four-target guest fixture passed meter and reader crash/stall
recovery, preserving layout and default routing. The guest was saved and stopped;
all four test VMs remained stopped. Desktop activation retained the previous
release and a configuration backup, with all five configuration hashes unchanged.

A two-minute opt-in diagnostic captured the physical Kate key and leftmost
output-volume dial. The user confirmed both worked. Aggregate snapshots showed
two key reports producing two edges, and 33 dial reports producing 33 events,
with no transport/normalization errors, lock gate or drain active at those
snapshots. Matching worker records and successful action results were captured.
The earlier missing-record cause is **not established**; this later successful
capture must not be presented as proof of a specific root cause or fix.
The diagnostic expired automatically, and its environment setting was removed
from future starts.

During the diagnostic burst, 33 dial actions all succeeded; report-return to
backend-entry measured 48.08 ms median, 111.98 ms p95 and 112.46 ms maximum.
After the diagnostic closed, 22 further dial actions all succeeded, measuring
0.041 ms median, 44.35 ms p95 and 46.96 ms maximum. One application-launch key
measured 0.022 ms to backend entry. These are short, differing workloads on one
output target, not representative V1 qualification, and do not measure visible
latency or backend completion. They identify audio-queue delay for follow-up,
rather than closing the 25 ms dispatch target. Evidence is retained in
`local/v1-input-diagnostics-20261005/physical-results.json` and raw journal records.

The subsequent source candidate reuses a separate lazy audio-action helper,
resolves each mutation freshly and preserves request order, clamping, reversal
and toggle semantics. It does not batch steps or change timing boundaries.
Transport failures discard the connection without replaying an uncertain
mutation; idle action helpers are reclaimed after 30 seconds. Source checks
cover process reuse, fresh external changes, reversal at a clamp, protocol
bounds, explicit errors, uncertain acknowledgements and idle cleanup.
See [action-helper behavior](audio-meters.md#refresh-budget).

Clean source `552c98c` was packaged as `0.1.0-12964b4edf45` after 135 passing
Rust tests (three intentionally ignored), 164 passing Studio tests,
warnings-denied Clippy, formatting, dependency checks and archive integrity
checks. Helper cleanup also kills its owned command process group, preventing
command descendants from surviving a lost connection or shutdown.

The installed Fedora 44 guest fixture passed eight recovery cases with four
independent synthetic audio targets. Successive adjustments reused the writer;
external volume changes and reversal at the upper clamp were respected without
altering other targets. A deliberately stalled writer reported a timeout after
3.037 seconds without replaying the uncertain action, and the next independent
input recovered. A killed writer likewise recovered on a subsequent input.
The writer was released after approximately 30 idle seconds while the reader
remained available, then restarted for the next adjustment. Meter and reader
crash/stall recovery also passed. The guest daemon survived, its saved layout
and default audio routing were unchanged, and the VM was saved and stopped.
These isolated VirtualDeck tests do not measure physical input latency.

The same bundle was installed on the desktop with a retained previous release
and configuration backup; all five saved configuration hashes were unchanged.
The user confirmed that physical audio dials at ordinary and quicker speeds,
plus application/media keys, worked correctly. The service remained active with
no automatic restarts. However, the collected ordinary-mode journal contained
no key/dial action timing records, so this confirms functional behavior only.
No latency improvement is inferred from that functional confirmation.

A subsequent restart of the same artifact enabled the bounded two-minute
diagnostic. Its snapshots showed normal polling without transport or
normalization errors, but no input reports during the diagnostic window.
After the diagnostic expired, 13 physical dial adjustments were captured across
an application-volume target and an output-device target. All 13 matched queued
records and succeeded, with non-null receipt stamps and no unmatched requests.
Report-return to backend-entry measured 0.031 ms median, 0.080 ms p95
(nearest-rank) and 0.080 ms maximum. These records were captured with diagnostics
off. This small sample meets the 25 ms dispatch target for its observed workload;
it does not establish representative burst performance, physical key timing,
visible feedback latency or a matched improvement over the earlier workload.
The earlier missing-record cause remains unestablished. The user confirmed the
physical dial and key worked; no key result was captured in this later sample.
The service remained active with no automatic restarts, the diagnostic expired,
and its environment setting was removed from future starts. Physical latency
and final-artifact idle/loaded resource qualification remain open. Evidence is
retained in `local/v1-audio-action-reuse-20261005/physical-diagnostic-results.json`
and the accompanying raw journal records.

### 2026-10-06 physical response follow-up on the Rune candidate

Installed clean source `7f6ce75`, bundle `0.1.0-2af1dcb9b80f`, recorded successful
physical application/media keys, touch mute, swipes and a separate audio-dial
sample. All 30 action results succeeded; 24 had physical dispatch timing, while
six experimental plugin-key results were uninstrumented. Eight swipes had
page-render invocation timing. All observed dispatch maxima were below 25 ms;
the 14 dial adjustments measured 0.035 ms p95 and maximum. The daemon did not
restart, and the saved layout was unchanged. The user confirmed functional
response and dial volume changes.

These are short samples, with sparse key coverage and no established four-active-
target workload. They do not close representative loaded/burst, brightness-dial
or physical visible-feedback timing gates. A slow-motion physical-device video
was requested for the visible-response check. The supplied original high-frame-
rate follow-up confirms visible dial/mute feedback and four page changes. One
page's sequential key repaint spans approximately 0.17 seconds from first key
change to the last destination appearance; this excludes input-to-first-pixel
delay and is not a complete latency pass. Its matching journal has 94 successful
timed built-in actions (audio-dial maximum 12.862 ms, p95 0.041 ms), plus six
experimental plugin results, of which one was busy and one failed. Page repaint
scheduling and those plugin errors need separate follow-up; precise dial visible
latency remains open. See the
[full response checkpoint](checkpoints/2026-10-06-physical-response.md).

### 2026-10-06 sustained VirtualDeck qualification

Clean source `1a0101da0f38d9c5d90c9131d5238d9ee74ef0e9` produced local bundle
`0.1.0-257113d71d41`, SHA-256
`4de46d33b1da65cd9b76cf8d136eda0c068c7c248a11ae0db44f2b3e36ac6703`.
This is a development qualification artifact, not a frozen V1 release.
Its 379 manifest files verified. Rust checks passed 140 tests with three existing
diagnostic tests ignored, formatting, warnings-denied Clippy and the dependency
audit. The unchanged Studio sources retain their 167 passing tests; installer
and security tests ran 18 cases, with one optional packaged-companion case
skipped. The bundle metadata/credential pattern scan found no matches; that
scan is not a guarantee that every security risk is absent.

The sustained guest test exposed an empty-queue spin in the live VirtualDeck
service. That adapter now waits 20 ms when idle, while ready inputs and pending
display writes remain immediate. Deterministic VirtualDeck unit tests and the
physical HID adapter retain their existing behavior. Virtual service runs accept
bounded durations up to one hour; physical probe limits remain unchanged.

Fedora 44's installed runtime passed eight reader/writer/meter recovery cases,
including stale-meter expiry, child crashes/timeouts, uncertain writes without
replay, fresh-value clamping and idle writer release. A subsequent 600-second
four-target synthetic workload completed 480 editor audio actions and 60 page
changes. All four meters were live at each of 61 sample points. The daemon
survived without restart, with the guest's saved layout and default routing
unchanged. Main RSS ranged from 12.71 to 13.54 MiB; handles stayed at 16 and
owned processes at five. Main CPU averaged 1.28% of one core, with 2.48% across
observed owned processes. Child CPU between samples can be omitted. These are
guest VirtualDeck measurements, not physical desktop resource or latency results.
Eight pre-dispatch `display_updating` admission rejections were waited out by
the harness; uncertain replies and other errors were never retried.

The same artifact was installed on the reference desktop. Integrity/dependency
checks passed, the physical display and experimental companion reconnected,
and both services had zero automatic restarts. Configuration, imported artwork,
active page, brightness and default audio routing were preserved. The previous
runtime was retained and the new backup's contents and hashes verified. This
does not constitute a restore test. All four VMs were shut off with autostart
disabled. The user then confirmed correct physical page changes, audio/media
keys, ordinary and quicker audio/brightness dial turns, labels, meters and icons
in the requested short acceptance check. The subsequent desktop idle/loaded
measurements and remaining-guest smoke results are recorded below.

A subsequent 300-second reference-desktop quiet sample had no playback/capture,
Deck inputs, lock transition or service restart. Main CPU averaged 1.63% of one
core; the core service and helpers averaged 2.91% (five-second interval p95
3.26%). Main RSS averaged 18.56 MiB and handles ranged from 16 to 18. Core cgroup
memory averaged 64.68 MiB, versus 47.85 MiB in the recorded earlier baseline;
the latest range was 62.86–72.83 MiB and ended 7.51 MiB below its starting value.
Inspection found 45.10 MiB anonymous memory, 14.84 MiB file cache and 2.15 MiB
kernel memory at the follow-up point, with 55.58 MiB summed process PSS. The
three observed helper modules were unchanged from the previous desktop bundle.
No growth trend was established, but the total-memory comparison remains open
for the loaded run; cache differences alone are not proven to explain it.

The separate experimental Homebridge unit averaged 0.53% CPU and 47.57 MiB
cgroup memory, with unchanged service identity. Core plus companion CPU therefore
averaged 3.45% of one core. The daemon's below-1% idle target remains unmet and
deferred post-V1; this observation does not close loaded resource or latency gates.

The same artifact's remaining-guest smoke results were:

| Guest | Recorded versions | Result and scope |
| --- | --- | --- |
| Fedora 45 test VM | GNOME Shell 51.0; glibc 2.44 | Bundle installation, doctor and native editor/private-bus VirtualDeck smoke passed. Saved layout unchanged. The isolated bus had accessibility-service warnings; this does not count as Orca acceptance. |
| Ubuntu 26.04 | GNOME Shell 50.1; glibc 2.43 | Bundle installation, doctor and private-bus VirtualDeck startup, previews, page navigation, save and validation passed. No graphical session or Xvfb was available, so native editor smoke was not attempted. The URL installer rejected the unsupported OS clearly. Saved layout unchanged. |
| Debian 13 | GNOME Shell 48.7; glibc 2.41 | The Fedora binary's `GLIBC_2.43` requirement prevented installation; the installer retained the saved layout. URL installer rejection was clear and bundle integrity verified. Cargo, rustc and pkg-config were absent, so a native source build remains blocked on prerequisites and was not attempted in this smoke. |

All guests were shut down after use and autostart remained disabled. These results
do not extend V1 platform support or close the exact final-artifact clean
dependency installation, upgrade/rollback/uninstall, login/reboot, accessibility
or physical USB/power-lifecycle gates.

The subsequent ten-minute physical mixed-page run completed all 148 recorded
actions successfully, including 133 dial adjustments across output, microphone,
Brave and Chrome. The user confirmed ordinary and quicker physical dial turns
and page changes worked correctly. Dial dispatch p95 was 15.745 ms at the
hardware-report-return to backend-entry boundary; visible pixels were not timed.
Main CPU averaged 2.54% of one core, with 4.66% for the core service and helpers
and 5.22% including the experimental companion. Main RSS remained 28.324–28.348
MiB, with 16–20 handles and no service restart. Core cgroup memory averaged
72.60 MiB and ranged from 70.13 to 94.99 MiB; this includes helpers/cache and
must not be equated with main RSS.

All four targets were present at 120 of 121 five-second observations; one
captured the unchanged Homebridge page's fan override. No unavailable meter was
seen in the 120 four-target frames. This is sustained physical interaction
evidence, not proof of uninterrupted four-target operation or precise visible
response. The original layout, page and all five saved JSON files were restored
and verified, including Dial 4's System sounds assignment. See the
[physical soak checkpoint](checkpoints/2026-10-06-physical-soak.md) for measurement
scope, restoration and remaining gates.

### 2026-10-06 current video and release-authenticity foundation

The current artifact's original high-frame-rate physical video confirms dial,
mute, key and page feedback. Its first inspected key repaint spans roughly
50–60 ms, compared with about 170 ms in the earlier artifact's recording. This
is visible repaint duration, excluding input-to-first-pixel delay; different page
content and recording conditions prevent a controlled speedup claim. The matching
approximate journal window has 152 successful actions, 132 dial adjustments and
six physical swipe requests. Camera/journal clocks are not independently aligned,
and some input/display areas are obscured. Precise visible-response timing remains
open. See the [current video checkpoint](checkpoints/2026-10-06-current-visible-response.md).

A manual GitHub workflow now builds Fedora 44 candidates from clean public source,
stages verified downloads, and is configured to attest and verify every candidate
file through a separate signing job. It has no release-publication step. The
workflow passed lint and its build commands passed in an isolated Fedora 44/Rust
1.97.1 fixture: Rust quality/dependency checks, installer/security/candidate tests,
Studio tests and archive staging. A host-dependent audio-reader test fixture was
corrected without changing runtime code. The authorized hosted run subsequently
built bundle `0.1.0-0143a5db609f` from public commit
`1d746db7594f3d4a92d2ea4ebbcaa80d22f70928`. All six signed subjects passed
independent verification through GitHub and the retained signing bundle, and
altered-file/wrong-source negative checks were rejected. This closes the scoped
build/signing check for that development candidate; final-artifact qualification
and a tested verified-install path remain required. Existing preview downloads
and the preview URL installer retain their previous limits. See the
[hosted verification checkpoint](checkpoints/2026-10-06-hosted-candidate-verification.md).
See the [foundation checkpoint](checkpoints/2026-10-06-release-authenticity-foundation.md)
and [verification/promotion procedure](release-authenticity.md).

## Work order

The signed candidate's previously deferred quiet measurement completed on
October 7 with bounded memory/handles and zero restarts. CPU was higher than
prior observations; this needs comparison/investigation alongside the remaining
loaded-resource work. See the [quiet resource checkpoint](checkpoints/2026-10-07-v1-quiet-resources.md).
The below-1% target remains post-V1; resource regression review remains required.

The subsequent [physical comparison](checkpoints/2026-10-07-v1-matched-resources.md)
completed two quiet pairs across a user-requested pause. Previous/candidate
descriptive means were 1.730%/1.772% daemon and 3.143%/3.241% core CPU, with
bounded memory/handles and no automatic restarts or unplanned actions. All four
complete phases were recalculated from raw counters; incomplete cancelled data
is excluded. The requested comparison measurements are complete, with the pause
and finite-sample limits retained. They do not establish statistical equivalence
or a universal regression-free result. The signed candidate and saved state were
verified restored, and all four VMs are off. Final resource/release acceptance
remains a separate decision in the [release review](v1-release-review.md).

The same signed artifact then completed a ten-minute user-confirmed physical run:
210 actions succeeded, including 204 dial adjustments across all four targets;
dial dispatch p95 was 21.445 ms. The expected four-target set was present at
120 of 121 observations, with one Homebridge page visit using its retained fan
override. Saved layout/page and all five JSON files were restored. Mean CPU was
3.52% daemon and 6.53% core service/helpers; the higher resource results still
require investigation. See the [physical resource checkpoint](checkpoints/2026-10-07-v1-physical-resources.md)
for scope, memory, dispatch limits and remaining gates.

Application version `1.0.0` is consistent across the Cargo workspace, locked
project packages, runtime bundle metadata and translation extraction. Saved-data
schemas are unchanged. The [signed V1 candidate](checkpoints/2026-10-06-v1-signed-candidate.md)
has passed source/build/signing checks, scoped staged recovery on the four-guest
matrix and Fedora 44 helper-failure/sustained workload checks. Its recoverable
desktop update and bounded physical acceptance passed, as did real fresh-profile
Fedora 44 guest installation/service and background-only reboot startup. Next
complete final resource/lifecycle, clean dependency provisioning, accessibility
and optional-companion checks, then reconcile all remaining gates. The
existing preview remains the public download until release acceptance.

The signed development candidate now passes the explicit verified-install helper
on the host's isolated filesystem and in the Fedora 44 VM. Six signature checks
precede all downloaded code, followed by checksum/archive/source admission.
Staged backup/restore, emoji labels, imported artwork, uninstall/reinstall and
private-bus VirtualDeck/native-editor smoke passed. The normal guest setup was
preserved and all VMs were powered off afterward. This closes the scoped verified
installation-path check, not final V1 installer qualification. See the
[verified installation checkpoint](checkpoints/2026-10-06-verified-installation.md).

The 2026-10-06 page-refresh candidate removes idle input waits between pending
images while retaining one-write-per-turn scheduling and input/lock fairness.
It passed local quality checks and Fedora 44 isolated validation, was installed
with recovery retained, and the user confirmed faster physical page changes with
correct controls. Seven physical request-to-write-completion journal observations
had median 14.158 ms and maximum 21.690 ms; they are not visible-pixel measurements
or a complete V1 timing pass. All four VMs are stopped. See the
[page-refresh checkpoint](checkpoints/2026-10-06-page-refresh.md).

The supported boundary and prior-build physical qualification results are
recorded above. Quiet/loaded physical resource and dispatch observations from
those earlier builds do not automatically qualify the frozen V1 archive.
Precise visible-response timing, real installation and remaining physical,
accessibility/action/companion cases must be completed or explicitly reconciled
against the identified artifact before publication. Publish only the tested
bytes with their own checksum and authenticity evidence. Optional features do
not replace any gate.
Further work to reach the below-1% daemon CPU target follows V1 under the approved
performance milestone above.

### Clean Fedora dependency and first-run follow-up — October 7, 2026

The frozen signed candidate passed a separate stock Fedora Workstation 44 live
GNOME/Wayland check using only read-only ISO drives and a RAM overlay. All 13
runtime packages were present by default. Removing Pillow in this disposable
OS caused a clear installer refusal without installed integrations; the documented
DNF package list restored it using normal signed-package verification. Fresh
signed per-user installation, runtime doctor, real waiting service, disabled
startup default, native VirtualDeck/editor smoke and the actual first-run controls
prompt passed. See the [clean installation checkpoint](checkpoints/2026-10-07-v1-clean-fedora-install.md)
for the exact artifact, package versions, limits and evidence. The retained VM
disks and desktop installation were untouched, and all test guests are off.
This complements prior installed-guest reboot evidence; the existing preview
URL installer remains preview.3/checksum-only and is not signed V1 delivery.

### Physical lifecycle and human accessibility — October 7, 2026

The frozen signed candidate has user-confirmed lock/unlock, USB unplug/reconnect,
suspend/resume, Quit/complete blanking/relaunch and background-only login startup
on the reference desktop. Subsequent read-only checks found the device ready,
Auto-Lock available and zero automatic restarts; intentional Quit/relaunch changed
the daemon process. User keyboard/focus, screen-reader and theme/enlarged-text/
display-scaling reviews also passed. The exact scaling choice and numeric contrast
were not measured. See the [lifecycle](checkpoints/2026-10-07-v1-physical-lifecycle.md)
and [accessibility](checkpoints/2026-10-07-v1-accessibility.md) checkpoints for
scope. These complete the requested reference-desktop manual checks, while precise
visible response, resource reconciliation, experimental companion cases, any
untested action/fault/draft case and final delivery/publication acceptance remain
separate. V1 is still unpublished; accepted artifact bytes are unchanged.

### Experimental companion fault/isolation follow-up — October 7, 2026

Fedora 44 VM testing passed 66 regression/authentication/recovery cases, one
loopback HTTP boundary case and three signed-core/companion isolation cases.
The 13 candidate runtime files match verified companion `01e396045147cdd7`;
all 75 companion-package files passed manifest checks. During the three fault
scenarios, ten private-bus core status/page/preview checks passed, with clean
explicit stop. Uncertain writes were not replayed. The normal guest profile,
integration, release links and startup preferences were preserved; temporary
files were removed and all VMs are off. See the [fault checkpoint](checkpoints/2026-10-07-v1-homebridge-faults.md)
for artifact correspondence, harness repairs and limits. This closes scoped
simulated companion checks. The user subsequently confirmed the agreed live
accessory-toggle/ordinary-brightness and normal-page/audio coexistence check:
everything responded correctly and stayed available. The scoped experimental
companion checks are complete; this does not certify general plugin support or
publish V1.
